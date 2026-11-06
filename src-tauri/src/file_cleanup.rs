use std::collections::VecDeque;
use std::fs;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};

use crate::clean::CleanResult;

const MAX_WORKERS: usize = 4;

#[derive(Clone)]
pub(crate) enum FileFilter {
    Pattern(String),
    Extensions(Vec<String>),
    WpsBackups,
}

impl FileFilter {
    fn matches(&self, relative: &Path) -> bool {
        let name = relative
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        match self {
            Self::Pattern(pattern) => crate::winapp2::wildmatch(name, pattern),
            Self::Extensions(extensions) => relative
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    extensions
                        .iter()
                        .any(|wanted| ext.eq_ignore_ascii_case(wanted))
                }),
            Self::WpsBackups => {
                relative.parent().is_some_and(has_backup_component)
                    || relative
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| {
                            ext.eq_ignore_ascii_case("bak") || ext.eq_ignore_ascii_case("wbk")
                        })
            }
        }
    }
}

fn has_backup_component(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("backup")
    })
}

fn path_key(path: &Path) -> PathBuf {
    let units: Vec<_> = path
        .as_os_str()
        .encode_wide()
        .map(|unit| {
            if (b'A' as u16..=b'Z' as u16).contains(&unit) {
                unit + (b'a' - b'A') as u16
            } else {
                unit
            }
        })
        .collect();
    PathBuf::from(std::ffi::OsString::from_wide(&units))
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    // Includes directory junctions as well as symbolic links.
    metadata.file_attributes() & 0x400 != 0
}

struct Rule {
    root: PathBuf,
    filter: FileFilter,
    recursive: bool,
    remove_contents: bool,
    remove_self: bool,
}

impl Rule {
    fn matches(&self, file: &Path) -> bool {
        let Ok(relative) = file.strip_prefix(&self.root) else {
            return false;
        };
        (self.recursive || relative.components().count() == 1) && self.filter.matches(relative)
    }

    fn needs_directory(&self, dir: &Path) -> bool {
        self.root.starts_with(dir) || (self.recursive && dir.starts_with(&self.root))
    }

    fn removes_directory(&self, dir: &Path) -> bool {
        let Ok(relative) = dir.strip_prefix(&self.root) else {
            return false;
        };
        if relative.as_os_str().is_empty() {
            return self.remove_self;
        }
        self.remove_contents
            || (matches!(self.filter, FileFilter::WpsBackups)
                && relative.parent().is_some_and(has_backup_component))
    }
}

/// All file rules share one traversal. Ancestor roots subsume descendant roots;
/// matching rules are combined before a file is scheduled for deletion.
#[derive(Default)]
pub(crate) struct CleanupPlan {
    rules: Vec<Rule>,
}

impl CleanupPlan {
    pub(crate) fn add_folder(&mut self, root: &Path) {
        self.add_rule(root, FileFilter::Pattern("*".into()), true, true, false);
    }

    pub(crate) fn add_rule(
        &mut self,
        root: &Path,
        filter: FileFilter,
        recursive: bool,
        remove_contents: bool,
        remove_self: bool,
    ) {
        if root.as_os_str().is_empty() || !root.is_absolute() {
            return;
        }
        let Ok(metadata) = fs::symlink_metadata(root) else {
            return;
        };
        if !metadata.is_dir() || is_reparse_point(&metadata) {
            return;
        }
        // Resolves aliases and normalizes . / .. before overlapping roots are merged.
        let Ok(root) = fs::canonicalize(root) else {
            return;
        };
        self.rules.push(Rule {
            root: path_key(&root),
            filter,
            recursive,
            remove_contents,
            remove_self,
        });
    }

    fn roots(&self) -> Vec<PathBuf> {
        let mut roots: Vec<_> = self.rules.iter().map(|rule| rule.root.clone()).collect();
        roots.sort_by_key(|path| path.components().count());
        let mut unique: Vec<PathBuf> = Vec::new();
        for root in roots {
            if !unique.iter().any(|ancestor| root.starts_with(ancestor)) {
                unique.push(root);
            }
        }
        unique
    }

    pub(crate) fn run(&self) -> CleanResult {
        self.run_observed(&|_| {}, &|_| {})
    }

    fn run_observed(
        &self,
        visit: &(impl Fn(&Path) + Sync),
        deleting: &(impl Fn(&Path) + Sync),
    ) -> CleanResult {
        let roots = self.roots();
        if roots.is_empty() {
            return CleanResult::new();
        }
        let queue = WorkQueue {
            state: Mutex::new(QueueState {
                pending: roots.into_iter().map(Job::Directory).collect(),
                active: 0,
            }),
            ready: Condvar::new(),
        };
        let workers = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(2)
            .clamp(2, MAX_WORKERS);
        let mut result = CleanResult::new();
        let mut directories = Vec::new();
        std::thread::scope(|scope| {
            let mut tasks = Vec::new();
            for _ in 0..workers {
                let queue = &queue;
                tasks.push(scope.spawn(move || {
                    let mut partial = CleanResult::new();
                    let mut directories = Vec::new();
                    while let Some(job) = queue.next() {
                        // Always releases the active job, including during a worker panic.
                        let _active = ActiveJob(queue);
                        match job {
                            Job::Directory(dir) => {
                                self.scan_directory(&dir, queue, visit, &mut partial);
                                let key = path_key(&dir);
                                if self.rules.iter().any(|rule| rule.removes_directory(&key)) {
                                    directories.push(dir);
                                }
                            }
                            Job::File(path, size) => {
                                deleting(&path);
                                match fs::remove_file(&path) {
                                    Ok(()) => partial.add_freed(size),
                                    Err(error) => partial.add_error(format!(
                                        "Skip {}: {}",
                                        path.display(),
                                        error
                                    )),
                                }
                            }
                        }
                    }
                    (partial, directories)
                }));
            }
            for task in tasks {
                match task.join() {
                    Ok((partial, mut dirs)) => {
                        result.merge(partial);
                        directories.append(&mut dirs);
                    }
                    Err(_) => result.add_error("Clean worker panicked".into()),
                }
            }
        });
        // remove_dir does not scan contents. Parents are removed only after children
        // and all file jobs have finished, so locked files remain safely in place.
        directories.sort_by_key(|dir| std::cmp::Reverse(dir.components().count()));
        for dir in directories {
            if let Err(error) = fs::remove_dir(&dir) {
                result.add_error(format!("Cannot delete {}: {}", dir.display(), error));
            }
        }
        result
    }

    fn scan_directory(
        &self,
        dir: &Path,
        queue: &WorkQueue,
        visit: &(impl Fn(&Path) + Sync),
        result: &mut CleanResult,
    ) {
        visit(dir);
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) => {
                result.add_error(format!("Cannot read {}: {}", dir.display(), error));
                return;
            }
        };
        let mut batch = Vec::with_capacity(128);
        let dir_key = path_key(dir);
        let active_rules: Vec<_> = self
            .rules
            .iter()
            .filter(|rule| {
                rule.root == dir_key || (rule.recursive && dir_key.starts_with(&rule.root))
            })
            .collect();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    result.add_error(format!("Cannot read entry in {}: {}", dir.display(), error));
                    continue;
                }
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    result.add_error(format!("Cannot inspect {}: {}", path.display(), error));
                    continue;
                }
            };
            if is_reparse_point(&metadata) {
                continue;
            }
            let key = path_key(&path);
            if metadata.is_dir() {
                if self.rules.iter().any(|rule| rule.needs_directory(&key)) {
                    batch.push(Job::Directory(path));
                }
            } else {
                visit(&path);
                if active_rules.iter().any(|rule| rule.matches(&key)) {
                    batch.push(Job::File(path, metadata.len()));
                }
            }
            if batch.len() >= 128 {
                queue.push(&mut batch);
            }
        }
        queue.push(&mut batch);
    }
}

enum Job {
    Directory(PathBuf),
    File(PathBuf, u64),
}

struct QueueState {
    pending: VecDeque<Job>,
    active: usize,
}

struct WorkQueue {
    state: Mutex<QueueState>,
    ready: Condvar,
}

impl WorkQueue {
    fn next(&self) -> Option<Job> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        loop {
            if let Some(job) = state.pending.pop_front() {
                state.active += 1;
                return Some(job);
            }
            if state.active == 0 {
                return None;
            }
            state = self
                .ready
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
    }

    fn push(&self, jobs: &mut Vec<Job>) {
        if jobs.is_empty() {
            return;
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.pending.extend(jobs.drain(..));
        self.ready.notify_all();
    }
}

struct ActiveJob<'a>(&'a WorkQueue);

impl Drop for ActiveJob<'_> {
    fn drop(&mut self) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.active -= 1;
        self.0.ready.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::windows::fs::OpenOptionsExt;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "windows-cleaner-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(fs::canonicalize(path).unwrap())
        }
        fn file(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"data").unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn overlapping_rules_visit_every_file_and_directory_once() {
        let fixture = Fixture::new();
        let temp = fixture.file("cache/nested/one.tmp");
        let log = fixture.file("logs/two.LOG");
        let msp = fixture.file("patch/three.MSP");
        let keep = fixture.file("keep.txt");
        let mut plan = CleanupPlan::default();
        plan.add_rule(
            &fixture.0,
            FileFilter::Extensions(vec!["tmp".into(), "log".into(), "msp".into()]),
            true,
            false,
            false,
        );
        plan.add_folder(&fixture.0.join("cache"));
        plan.add_folder(&fixture.0.join("CACHE").join("."));
        plan.add_rule(
            &fixture.0.join("cache/nested"),
            FileFilter::Pattern("*.tmp;*.log".into()),
            true,
            false,
            false,
        );
        let visits = Mutex::new(HashMap::<PathBuf, usize>::new());
        let result = plan.run_observed(
            &|path| {
                *visits.lock().unwrap().entry(path_key(path)).or_default() += 1;
            },
            &|_| {},
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.bytes_freed, 12);
        assert!(!temp.exists() && !log.exists() && !msp.exists());
        assert!(keep.exists());
        assert!(fixture.0.join("cache").exists());
        assert!(!fixture.0.join("cache/nested").exists());
        let visits = visits.into_inner().unwrap();
        assert_eq!(visits.len(), 9);
        assert!(visits.values().all(|count| *count == 1), "{visits:?}");
    }

    #[test]
    fn shallow_ancestor_preserves_unmatched_files_and_reaches_nested_rules() {
        let fixture = Fixture::new();
        fixture.file("top.tmp");
        let keep = fixture.file("branch/keep.tmp");
        fixture.file("branch/target/delete.log");
        let mut plan = CleanupPlan::default();
        plan.add_rule(
            &fixture.0,
            FileFilter::Pattern("*.tmp".into()),
            false,
            false,
            false,
        );
        plan.add_folder(&fixture.0.join("branch/target"));
        let result = plan.run();
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.bytes_freed, 8);
        assert!(keep.exists());
        assert!(fixture.0.join("branch/target").exists());
    }

    #[test]
    fn locked_file_is_attempted_once_across_overlapping_rules() {
        let fixture = Fixture::new();
        let path = fixture.file("locked.tmp");
        let _locked = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        let mut plan = CleanupPlan::default();
        plan.add_folder(&fixture.0);
        plan.add_rule(
            &fixture.0,
            FileFilter::Extensions(vec!["tmp".into()]),
            true,
            false,
            false,
        );
        let result = plan.run();
        assert_eq!(result.bytes_freed, 0);
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(path.exists());
    }

    #[test]
    fn wps_backup_contents_and_remove_self_keep_their_scope() {
        let fixture = Fixture::new();
        fixture.file("kingsoft/office6/backup/Nested/document.txt");
        fixture.file("kingsoft/office6/recover.BAK");
        let keep = fixture.file("kingsoft/office6/document.txt");
        fixture.file("Remove/cache.tmp");
        let mut plan = CleanupPlan::default();
        plan.add_rule(
            &fixture.0.join("kingsoft"),
            FileFilter::WpsBackups,
            true,
            false,
            false,
        );
        plan.add_rule(
            &fixture.0.join("remove"),
            FileFilter::Pattern("*".into()),
            true,
            true,
            true,
        );
        let result = plan.run();
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.bytes_freed, 12);
        assert!(keep.exists());
        assert!(fixture.0.join("kingsoft/office6/backup").exists());
        assert!(!fixture.0.join("kingsoft/office6/backup/nested").exists());
        assert!(!fixture.0.join("remove").exists());
    }

    #[test]
    fn flat_directory_deletes_files_on_multiple_workers() {
        let fixture = Fixture::new();
        for index in 0..512 {
            fixture.file(&format!("{index}.tmp"));
        }
        let mut plan = CleanupPlan::default();
        plan.add_folder(&fixture.0);
        let deleting = Mutex::new(std::collections::HashSet::new());
        let first_jobs = std::sync::atomic::AtomicUsize::new(0);
        let rendezvous = std::sync::Barrier::new(2);
        let result = plan.run_observed(&|_| {}, &|_| {
            deleting.lock().unwrap().insert(std::thread::current().id());
            if first_jobs.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 2 {
                rendezvous.wait();
            }
        });
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.bytes_freed, 2048);
        assert!(deleting.into_inner().unwrap().len() >= 2);
        assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
    }

    #[test]
    fn junction_loop_is_not_followed_or_cleaned() {
        let fixture = Fixture::new();
        fixture.file("cache/delete.tmp");
        let outside = fixture.file("outside/keep.tmp");
        let junction = fixture.0.join("cache/link");
        let target = fixture.0.join("outside");
        // mklink does not accept the verbatim prefix returned by canonicalize.
        let junction_arg = junction
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string();
        let target_arg = target
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string();
        let output = std::process::Command::new("cmd")
            .args(["/d", "/c", "mklink", "/J", &junction_arg, &target_arg])
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output);
        let mut plan = CleanupPlan::default();
        plan.add_folder(&fixture.0.join("cache"));
        plan.add_folder(&junction);
        let visits = Mutex::new(Vec::new());
        let result = plan.run_observed(
            &|path| visits.lock().unwrap().push(path.to_path_buf()),
            &|_| {},
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.bytes_freed, 4);
        assert!(outside.exists());
        assert_eq!(visits.into_inner().unwrap().len(), 2);
        fs::remove_dir(junction).unwrap();
    }
}
