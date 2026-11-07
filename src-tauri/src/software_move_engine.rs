use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use winreg::enums::*;
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const MOVE_ROOT: &str = "MovedApps";
const MANIFEST_SUFFIX: &str = ".wc-move.json";
const MIN_FREE_SPACE: u64 = 100 * 1024 * 1024;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareInfo {
    pub key: String,
    pub display_name: String,
    pub display_path: String,
    pub source_path: String,
    pub version: Option<String>,
    pub size_bytes: u64,
    pub file_count: u64,
    pub exists: bool,
    pub is_moved: bool,
    pub moved_to: Option<String>,
    pub recovery_state: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub letter: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MoveResult {
    pub skipped_files: Vec<String>,
    pub junction_created: bool,
    pub recovery_required: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveManifest {
    key: String,
    source_path: String,
    target_path: String,
    display_name: String,
    version: Option<String>,
}

#[derive(Clone)]
struct Candidate {
    path: PathBuf,
    display_name: String,
    version: Option<String>,
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

fn path_key(path: &Path) -> String {
    let normalized = normalized_path(path);
    let mut hash = 0xcbf29ce484222325u64;
    for byte in normalized.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("app-{hash:016x}")
}

fn display_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Application")
        .to_string()
}

fn expand_environment(value: &str) -> String {
    let vars: HashMap<String, String> = std::env::vars()
        .map(|(key, value)| (key.to_lowercase(), value))
        .collect();
    let mut expanded = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find('%') {
        expanded.push_str(&remaining[..start]);
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('%') else {
            expanded.push_str(&remaining[start..]);
            return expanded;
        };
        let name = &after_start[..end];
        if let Some(replacement) = vars.get(&name.to_lowercase()) {
            expanded.push_str(replacement);
        } else {
            expanded.push('%');
            expanded.push_str(name);
            expanded.push('%');
        }
        remaining = &after_start[end + 1..];
    }
    expanded.push_str(remaining);
    expanded
}

fn path_from_executable(value: &str) -> Option<PathBuf> {
    let expanded = expand_environment(value.trim());
    let trimmed = expanded.trim();
    let executable = if let Some(quoted) = trimmed.strip_prefix('"') {
        let end = quoted.find('"')?;
        PathBuf::from(&quoted[..end])
    } else {
        let lower = trimmed.to_lowercase();
        let exe_end = lower.find(".exe").map(|index| index + 4)?;
        PathBuf::from(&trimmed[..exe_end])
    };
    executable
        .parent()
        .filter(|parent| parent.is_dir())
        .map(Path::to_path_buf)
}

fn path_from_icon(value: &str) -> Option<PathBuf> {
    let expanded = expand_environment(value.trim());
    let unquoted = expanded.trim_matches('"');
    let candidate = unquoted
        .find(".exe")
        .map(|index| &unquoted[..index + 4])
        .unwrap_or(unquoted);
    let path = PathBuf::from(candidate.trim_matches('"'));
    if path.is_file() {
        path.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

fn protected_install_path(path: &Path) -> bool {
    let value = normalized_path(path);
    if value.is_empty() {
        return true;
    }
    let windows_dir = std::env::var("WINDIR")
        .or_else(|_| std::env::var("SystemRoot"))
        .unwrap_or_else(|_| String::from(r"C:\Windows"));
    let windows_dir = normalized_path(Path::new(&windows_dir));
    value == windows_dir
        || value.starts_with(&format!("{windows_dir}\\"))
        || value.contains(r"\windowsapps")
        || value.contains(r"\windows\installer")
        || value.contains(r"\windows\servicing")
        || value.ends_with(r"\common files")
}

fn add_candidate(
    candidates: &mut HashMap<String, Candidate>,
    path: PathBuf,
    display_name: Option<String>,
    version: Option<String>,
) {
    let path = PathBuf::from(expand_environment(&path.to_string_lossy()));
    if protected_install_path(&path) {
        return;
    }
    let identity = normalized_path(&path);
    let name = display_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| display_name_from_path(&path));
    candidates
        .entry(identity)
        .and_modify(|candidate| {
            if candidate.display_name == display_name_from_path(&candidate.path) {
                candidate.display_name = name.clone();
            }
            if candidate.version.is_none() {
                candidate.version = version.clone();
            }
        })
        .or_insert(Candidate {
            path,
            display_name: name,
            version,
        });
}

fn registry_string(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn scan_uninstall_registry(candidates: &mut HashMap<String, Candidate>) {
    let uninstall_paths = [
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for uninstall_path in uninstall_paths {
            for view in [0, KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
                let root = RegKey::predef(hive);
                let Ok(uninstall) = root.open_subkey_with_flags(uninstall_path, KEY_READ | view)
                else {
                    continue;
                };
                for subkey_name in uninstall.enum_keys().flatten() {
                    let Ok(entry) = uninstall.open_subkey_with_flags(&subkey_name, KEY_READ) else {
                        continue;
                    };
                    if entry.get_value::<u32, _>("SystemComponent").unwrap_or(0) == 1
                        || registry_string(&entry, "ParentKeyName").is_some()
                    {
                        continue;
                    }
                    let display_name = registry_string(&entry, "DisplayName").unwrap_or_default();
                    if display_name.is_empty() {
                        continue;
                    }
                    let display_lower = display_name.to_lowercase();
                    if ["security update", "hotfix", "update for", "更新程序"]
                        .iter()
                        .any(|marker| display_lower.contains(marker))
                    {
                        continue;
                    }
                    let install_location = registry_string(&entry, "InstallLocation")
                        .map(|value| PathBuf::from(expand_environment(&value)))
                        .filter(|path| path.is_dir());
                    let icon_location = registry_string(&entry, "DisplayIcon")
                        .and_then(|value| path_from_icon(&value));
                    let uninstall_location = registry_string(&entry, "UninstallString")
                        .and_then(|value| path_from_executable(&value));
                    let path = install_location.or(icon_location).or(uninstall_location);
                    if let Some(path) = path {
                        add_candidate(
                            candidates,
                            path,
                            Some(display_name),
                            registry_string(&entry, "DisplayVersion"),
                        );
                    }
                }
            }
        }
    }
}

fn scan_app_paths_registry(candidates: &mut HashMap<String, Candidate>) {
    let app_paths = [
        r"Software\Microsoft\Windows\CurrentVersion\App Paths",
        r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths",
    ];
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for app_path in app_paths {
            for view in [0, KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
                let root = RegKey::predef(hive);
                let Ok(apps) = root.open_subkey_with_flags(app_path, KEY_READ | view) else {
                    continue;
                };
                for subkey_name in apps.enum_keys().flatten() {
                    let Ok(entry) = apps.open_subkey_with_flags(&subkey_name, KEY_READ) else {
                        continue;
                    };
                    let Some(executable) = entry
                        .get_value::<String, _>("")
                        .ok()
                        .filter(|value| !value.trim().is_empty())
                    else {
                        continue;
                    };
                    let Some(path) = path_from_executable(&executable) else {
                        continue;
                    };
                    add_candidate(candidates, path, Some(subkey_name), None);
                }
            }
        }
    }
}

fn directory_has_executable(path: &Path, depth: u8) -> bool {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                return false;
            };
            if kind.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
            {
                return true;
            }
            depth > 0
                && kind.is_dir()
                && !kind.is_symlink()
                && directory_has_executable(&path, depth - 1)
        })
}

fn scan_common_install_roots(candidates: &mut HashMap<String, Candidate>) {
    let mut roots = Vec::new();
    for variable in ["PROGRAMFILES", "PROGRAMFILES(X86)", "ProgramW6432"] {
        if let Ok(root) = std::env::var(variable) {
            roots.push(PathBuf::from(root));
        }
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        roots.push(PathBuf::from(local_app_data).join("Programs"));
    }

    for root in roots {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !entry.file_type().is_ok_and(|kind| kind.is_dir())
                || protected_install_path(&path)
                || !directory_has_executable(&path, 2)
            {
                continue;
            }
            add_candidate(candidates, path, None, None);
        }
    }
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn read_link_target(path: &Path) -> Option<PathBuf> {
    let target = fs::read_link(path).ok()?;
    if target.is_absolute() {
        Some(target)
    } else {
        path.parent().map(|parent| parent.join(target))
    }
}

fn manifest_path(target: &Path, key: &str) -> PathBuf {
    target
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{key}{MANIFEST_SUFFIX}"))
}

fn read_manifest(path: &Path) -> Option<MoveManifest> {
    let content = fs::read(path).ok()?;
    serde_json::from_slice(&content).ok()
}

fn manifests_for_installed_drives() -> HashMap<String, MoveManifest> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut manifests = HashMap::new();
    let mut visited = Vec::new();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy();
        let Some(letter) = mount.chars().next().filter(|_| mount.contains(':')) else {
            continue;
        };
        let root = PathBuf::from(format!("{}:\\{MOVE_ROOT}", letter.to_ascii_uppercase()));
        if visited
            .iter()
            .any(|path: &PathBuf| normalized_path(path) == normalized_path(&root))
        {
            continue;
        }
        visited.push(root.clone());
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                if let Some(manifest) = read_manifest(&entry.path()) {
                    manifests.insert(normalized_path(Path::new(&manifest.source_path)), manifest);
                }
            }
        }
    }
    manifests
}

fn directory_size_and_count(path: &Path) -> (u64, u64) {
    let mut size = 0u64;
    let mut count = 0u64;
    let Ok(entries) = fs::read_dir(path) else {
        return (0, 0);
    };
    for entry in entries.flatten() {
        let child = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            let (child_size, child_count) = directory_size_and_count(&child);
            size = size.saturating_add(child_size);
            count = count.saturating_add(child_count);
        } else if kind.is_file() {
            if let Ok(metadata) = entry.metadata() {
                size = size.saturating_add(metadata.len());
                count += 1;
            }
        }
    }
    (size, count)
}

fn file_contents_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let left_metadata = fs::metadata(left).map_err(|error| error.to_string())?;
    let right_metadata = fs::metadata(right).map_err(|error| error.to_string())?;
    if left_metadata.len() != right_metadata.len() {
        return Ok(false);
    }
    let mut left_file = fs::File::open(left).map_err(|error| error.to_string())?;
    let mut right_file = fs::File::open(right).map_err(|error| error.to_string())?;
    let mut left_buffer = [0u8; 64 * 1024];
    let mut right_buffer = [0u8; 64 * 1024];
    loop {
        let left_read = left_file
            .read(&mut left_buffer)
            .map_err(|error| error.to_string())?;
        let right_read = right_file
            .read(&mut right_buffer)
            .map_err(|error| error.to_string())?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn directory_contents_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let mut left_entries = HashMap::new();
    let mut right_entries = HashMap::new();
    for (root, entries) in [(left, &mut left_entries), (right, &mut right_entries)] {
        for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            entries.insert(
                entry.file_name().to_string_lossy().to_lowercase(),
                entry.path(),
            );
        }
    }
    if left_entries.len() != right_entries.len()
        || left_entries
            .keys()
            .any(|name| !right_entries.contains_key(name))
    {
        return Ok(false);
    }
    for (name, left_path) in left_entries {
        let right_path = &right_entries[&name];
        let left_kind = fs::symlink_metadata(&left_path).map_err(|error| error.to_string())?;
        let right_kind = fs::symlink_metadata(right_path).map_err(|error| error.to_string())?;
        if left_kind.file_type().is_symlink() || right_kind.file_type().is_symlink() {
            return Ok(false);
        }
        if left_kind.is_dir() != right_kind.is_dir() || left_kind.is_file() != right_kind.is_file()
        {
            return Ok(false);
        }
        if left_kind.is_dir() {
            if !directory_contents_equal(&left_path, right_path)? {
                return Ok(false);
            }
        } else if !file_contents_equal(&left_path, right_path)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn copy_directory(
    app: &tauri::AppHandle,
    source: &Path,
    target: &Path,
    total_size: u64,
    copied: &mut u64,
    key: &str,
) -> Result<(), String> {
    let source_meta = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if source_meta.file_type().is_symlink() || !source_meta.is_dir() {
        return Err(format!("不支持迁移重解析点或非目录: {}", source.display()));
    }
    fs::create_dir(target)
        .map_err(|error| format!("无法创建目录 {}: {error}", target.display()))?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        let metadata = fs::symlink_metadata(&from).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "发现联接或符号链接，已停止以保留完整数据: {}",
                from.display()
            ));
        }
        if metadata.is_dir() {
            copy_directory(app, &from, &to, total_size, copied, key)?;
        } else if metadata.is_file() {
            fs::copy(&from, &to)
                .map_err(|error| format!("复制失败 {}: {error}", from.display()))?;
            if !file_contents_equal(&from, &to)? {
                return Err(format!("文件校验失败: {}", from.display()));
            }
            *copied = copied.saturating_add(metadata.len());
            let percent = if total_size == 0 {
                99
            } else {
                ((*copied as f64 / total_size as f64) * 99.0).min(99.0) as u64
            };
            let _ = app.emit(
                "software-move-progress",
                serde_json::json!({ "key": key, "percent": percent, "status": "copying" }),
            );
        } else {
            return Err(format!("不支持的文件类型: {}", from.display()));
        }
    }
    if let Ok(metadata) = fs::metadata(source) {
        let _ = fs::set_permissions(target, metadata.permissions());
    }
    Ok(())
}

fn unique_sibling(parent: &Path, prefix: &str, key: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    parent.join(format!(".wc-{prefix}-{key}-{}-{stamp}", std::process::id()))
}

fn write_manifest(target: &Path, manifest: &MoveManifest) -> Result<PathBuf, String> {
    let destination = manifest_path(target, &manifest.key);
    let temporary = unique_sibling(
        destination.parent().unwrap_or_else(|| Path::new(".")),
        "manifest",
        &manifest.key,
    );
    let bytes = serde_json::to_vec_pretty(manifest).map_err(|error| error.to_string())?;
    let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&temporary, &destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })?;
    Ok(destination)
}

fn create_directory_junction(source: &Path, target: &Path) -> Result<(), String> {
    let source_arg = source.to_string_lossy().into_owned();
    let target_arg = target.to_string_lossy().into_owned();
    let output = Command::new("cmd")
        .args(["/D", "/C", "mklink", "/J", &source_arg, &target_arg])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| {
            crate::logger::error(
                "move",
                "目录联接命令启动失败",
                &format!("source={} target={} error={error}", source.display(), target.display()),
            );
            format!("无法创建目录联接: {error}")
        })?;
    if output.status.success() && is_link(source) {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\0', "");
    let stdout = String::from_utf8_lossy(&output.stdout).replace('\0', "");
    let detail = format!(
        "source={} target={} status={} stdout={} stderr={}",
        source.display(),
        target.display(),
        output.status,
        stdout.trim(),
        stderr.trim()
    );
    crate::logger::error("move", "目录联接创建失败", &detail);
    Err(format!(
        "无法创建目录联接: {}",
        if !stderr.trim().is_empty() {
            stderr.trim()
        } else if !stdout.trim().is_empty() {
            stdout.trim()
        } else {
            "Windows 未返回详细错误"
        }
    ))
}

fn emit_progress(app: &tauri::AppHandle, key: &str, percent: u64, status: &str) {
    let _ = app.emit(
        "software-move-progress",
        serde_json::json!({ "key": key, "percent": percent, "status": status }),
    );
}

fn drive_letter(path: &Path) -> Option<char> {
    let value = path.to_string_lossy();
    let mut chars = value.chars();
    let letter = chars.next()?;
    (chars.next() == Some(':')).then_some(letter.to_ascii_uppercase())
}

fn target_free_space(letter: char) -> Option<u64> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    disks.list().iter().find_map(|disk| {
        let mount = disk.mount_point().to_string_lossy();
        (drive_letter(Path::new(mount.as_ref())) == Some(letter)).then_some(disk.available_space())
    })
}

fn validate_drive(letter: &str) -> Result<char, String> {
    let mut chars = letter.chars();
    let Some(drive) = chars.next() else {
        return Err("目标磁盘无效".into());
    };
    if chars.next().is_some() || !drive.is_ascii_alphabetic() {
        return Err("目标磁盘无效".into());
    }
    Ok(drive.to_ascii_uppercase())
}

fn load_all_candidates() -> Vec<SoftwareInfo> {
    let mut candidates = HashMap::new();
    scan_uninstall_registry(&mut candidates);
    scan_app_paths_registry(&mut candidates);
    scan_common_install_roots(&mut candidates);
    let manifests = manifests_for_installed_drives();
    for manifest in manifests.values() {
        let source = PathBuf::from(&manifest.source_path);
        add_candidate(
            &mut candidates,
            source,
            Some(manifest.display_name.clone()),
            manifest.version.clone(),
        );
    }

    let mut results = Vec::new();
    for candidate in candidates.into_values() {
        let source = candidate.path;
        let key = path_key(&source);
        let identity = normalized_path(&source);
        let manifest = manifests.get(&identity);
        let manifest_target = manifest.map(|item| PathBuf::from(&item.target_path));
        let source_is_link = is_link(&source);
        let link_target = source_is_link.then(|| read_link_target(&source)).flatten();
        let target = link_target.or_else(|| manifest_target.clone());
        let source_metadata_exists = fs::symlink_metadata(&source).is_ok();
        let source_exists = source.exists();
        let target_exists = target.as_ref().is_some_and(|path| path.exists());

        let (is_moved, recovery_state, size_path) = if source_is_link {
            if target_exists {
                (true, "moved", target.clone().unwrap())
            } else {
                (false, "linkTargetMissing", source.clone())
            }
        } else if source_exists && target_exists && manifest.is_some() {
            (true, "sourceAndTarget", source.clone())
        } else if !source_exists && target_exists && manifest.is_some() {
            (true, "sourceMissing", target.clone().unwrap())
        } else if source_exists && manifest.is_some() && !target_exists {
            (false, "targetMissing", source.clone())
        } else if source_exists {
            (false, "normal", source.clone())
        } else {
            continue;
        };
        let (size_bytes, file_count) = directory_size_and_count(&size_path);
        results.push(SoftwareInfo {
            key,
            display_name: candidate.display_name,
            display_path: source.to_string_lossy().into_owned(),
            source_path: source.to_string_lossy().into_owned(),
            version: candidate.version,
            size_bytes,
            file_count,
            exists: source_metadata_exists || target_exists,
            is_moved,
            moved_to: target.map(|path| path.to_string_lossy().into_owned()),
            recovery_state: recovery_state.to_string(),
        });
    }
    results.sort_by(|left, right| right.size_bytes.cmp(&left.size_bytes));
    results
}

fn path_matches(executable: &Path, root: &Path) -> bool {
    let executable = normalized_path(executable);
    let root = normalized_path(root);
    executable == root || executable.starts_with(&format!("{root}\\"))
}

fn find_running_processes(source: &Path) -> Vec<ProcessInfo> {
    let mut roots = vec![source.to_path_buf()];
    if is_link(source) {
        if let Some(target) = read_link_target(source) {
            roots.push(target);
        }
    } else if let Some(manifest) = read_manifest(&manifest_path(
        &PathBuf::from(format!(
            "{}:\\{MOVE_ROOT}\\placeholder",
            drive_letter(source).unwrap_or('C')
        )),
        &path_key(source),
    )) {
        roots.push(PathBuf::from(manifest.target_path));
    } else {
        for disk in sysinfo::Disks::new_with_refreshed_list().list() {
            let mount = disk.mount_point().to_string_lossy();
            let Some(letter) = drive_letter(Path::new(mount.as_ref())) else {
                continue;
            };
            let root = PathBuf::from(format!("{}:\\{MOVE_ROOT}", letter));
            let Ok(entries) = fs::read_dir(root) else {
                continue;
            };
            for entry in entries.flatten() {
                if let Some(manifest) = read_manifest(&entry.path()) {
                    if normalized_path(Path::new(&manifest.source_path)) == normalized_path(source)
                    {
                        roots.push(PathBuf::from(manifest.target_path));
                    }
                }
            }
        }
    }
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system
        .processes()
        .values()
        .filter_map(|process| {
            let executable = process.exe()?;
            roots
                .iter()
                .any(|root| path_matches(executable, root))
                .then(|| ProcessInfo {
                    name: process.name().to_string_lossy().into_owned(),
                    pid: process.pid().as_u32(),
                })
        })
        .collect()
}

fn kill_process(pid: u32) -> Result<(), String> {
    let output = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("无法运行 taskkill: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let message = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if message.contains("not found")
        || message.contains("no running instance")
        || message.contains("不存在")
    {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

#[tauri::command]
pub async fn scan_software() -> Result<Vec<SoftwareInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(load_all_candidates()))
        .await
        .unwrap_or_else(|_| Err("软件扫描任务失败".into()))
}

#[tauri::command]
pub async fn get_available_drives() -> Result<Vec<DriveInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut drives: Vec<DriveInfo> = Vec::new();
        for disk in disks.list() {
            let mount = disk.mount_point().to_string_lossy().to_string();
            let Some(letter) = drive_letter(Path::new(&mount)) else {
                continue;
            };
            if letter == 'C'
                || drives
                    .iter()
                    .any(|drive| drive.letter == letter.to_string())
            {
                continue;
            }
            drives.push(DriveInfo {
                letter: letter.to_string(),
                free_bytes: disk.available_space(),
                total_bytes: disk.total_space(),
            });
        }
        Ok(drives)
    })
    .await
    .unwrap_or_else(|_| Err("磁盘扫描任务失败".into()))
}

#[tauri::command]
pub async fn check_running_processes(source_path: String) -> Result<Vec<ProcessInfo>, String> {
    let source = PathBuf::from(source_path);
    tauri::async_runtime::spawn_blocking(move || Ok(find_running_processes(&source)))
        .await
        .unwrap_or_else(|_| Err("进程检查任务失败".into()))
}

#[tauri::command]
pub async fn kill_running_processes(source_path: String) -> Result<Vec<ProcessInfo>, String> {
    let source = PathBuf::from(source_path);
    tauri::async_runtime::spawn_blocking(move || {
        let running = find_running_processes(&source);
        let mut killed = Vec::new();
        for process in running {
            if kill_process(process.pid).is_ok() {
                killed.push(process);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(800));
        Ok(killed)
    })
    .await
    .unwrap_or_else(|_| Err("关闭软件进程失败".into()))
}

#[tauri::command]
pub async fn move_software(
    app: tauri::AppHandle,
    key: String,
    source_path: String,
    target_drive: String,
    display_name: String,
    version: Option<String>,
) -> Result<MoveResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = PathBuf::from(expand_environment(&source_path));
        let drive = validate_drive(&target_drive)?;
        if key != path_key(&source) {
            return Err("软件路径标识已过期，请重新扫描后重试".into());
        }
        if !source.is_dir() || is_link(&source) {
            return Err(format!("源目录不存在或已经是联接: {}", source.display()));
        }
        if protected_install_path(&source) {
            return Err("系统目录或受保护目录不能迁移".into());
        }
        if drive_letter(&source) == Some(drive) {
            return Err("目标磁盘必须与软件当前所在磁盘不同".into());
        }
        let (total_size, _) = directory_size_and_count(&source);
        let free = target_free_space(drive).ok_or_else(|| format!("无法读取 {drive}: 磁盘空间"))?;
        if free < total_size.saturating_add(MIN_FREE_SPACE) {
            return Err(format!(
                "{drive}: 磁盘空间不足，需要至少 {} MB",
                total_size.saturating_add(MIN_FREE_SPACE) / (1024 * 1024)
            ));
        }
        let root = PathBuf::from(format!("{drive}:\\{MOVE_ROOT}"));
        fs::create_dir_all(&root).map_err(|error| format!("无法创建目标目录: {error}"))?;
        let target = root.join(&key);
        let manifest_file = manifest_path(&target, &key);
        if target.exists() {
            return Err(format!("目标位置已存在，未覆盖: {}", target.display()));
        }
        if manifest_file.exists() {
            if !manifest_file.is_file() {
                return Err(format!(
                    "目标位置已被占用，未覆盖: {}",
                    manifest_file.display()
                ));
            }
            if let Some(existing) = read_manifest(&manifest_file) {
                let existing_target = PathBuf::from(&existing.target_path);
                if existing_target.is_dir() {
                    return Err(format!(
                        "已存在可恢复的迁移副本: {}",
                        existing_target.display()
                    ));
                }
            }
            fs::remove_file(&manifest_file)
                .map_err(|error| format!("无法清理旧迁移记录: {error}"))?;
        }

        // Remove only stale records for this source. A live target means the
        // previous migration is recoverable and must not be overwritten.
        for disk in sysinfo::Disks::new_with_refreshed_list().list() {
            let mount = disk.mount_point().to_string_lossy();
            let Some(letter) = drive_letter(Path::new(mount.as_ref())) else {
                continue;
            };
            let manifest_root = PathBuf::from(format!("{}:\\{MOVE_ROOT}", letter));
            let Ok(entries) = fs::read_dir(&manifest_root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
                {
                    continue;
                }
                let Some(existing) = read_manifest(&path) else {
                    continue;
                };
                if normalized_path(Path::new(&existing.source_path)) != normalized_path(&source) {
                    continue;
                }
                let existing_target = PathBuf::from(&existing.target_path);
                if existing_target.is_dir() {
                    return Err(format!(
                        "已存在可恢复的迁移副本: {}",
                        existing_target.display()
                    ));
                }
                let _ = fs::remove_file(path);
            }
        }
        let staging = unique_sibling(&root, "staging", &key);
        let manifest = MoveManifest {
            key: key.clone(),
            source_path: source.to_string_lossy().into_owned(),
            target_path: target.to_string_lossy().into_owned(),
            display_name,
            version,
        };

        emit_progress(&app, &key, 0, "copying");
        let mut copied = 0;
        if let Err(error) = copy_directory(&app, &source, &staging, total_size, &mut copied, &key) {
            let _ = fs::remove_dir_all(&staging);
            return Err(format!("复制未完成，源目录保持原样: {error}"));
        }
        if !directory_contents_equal(&source, &staging)? {
            let _ = fs::remove_dir_all(&staging);
            return Err("复制后的内容校验失败，源目录保持原样".into());
        }
        fs::rename(&staging, &target).map_err(|error| {
            format!(
                "已校验的副本仍保留在 {}，无法完成目标目录切换: {error}",
                staging.display()
            )
        })?;
        if let Err(error) = write_manifest(&target, &manifest) {
            return Err(format!(
                "源目录未修改；完整副本已保留在 {}，但迁移记录写入失败: {error}",
                target.display()
            ));
        }

        emit_progress(&app, &key, 99, "switching");
        let backup = unique_sibling(
            source.parent().unwrap_or_else(|| Path::new(".")),
            "source-backup",
            &key,
        );
        fs::rename(&source, &backup).map_err(|error| {
            format!(
                "源目录未改变，已校验副本保留在 {}: {error}",
                target.display()
            )
        })?;
        if let Err(error) = create_directory_junction(&source, &target) {
            let restore_result = fs::rename(&backup, &source);
            return Err(match restore_result {
                Ok(()) => format!(
                    "{error}；源目录已恢复，校验副本保留在 {} 供恢复使用",
                    target.display()
                ),
                Err(_restore_error) => format!(
                    "{error}；源数据仍保留在 {}，请从该目录恢复。校验副本位于 {}",
                    backup.display(),
                    target.display()
                ),
            });
        }
        let backup_cleanup_failed = fs::remove_dir_all(&backup).is_err();
        emit_progress(&app, &key, 100, "done");
        crate::logger::info(
            "move",
            &format!("软件迁移完成: {}", source.display()),
            &format!("→ {}", target.display()),
        );
        Ok(MoveResult {
            skipped_files: Vec::new(),
            junction_created: true,
            recovery_required: backup_cleanup_failed,
        })
    })
    .await
    .unwrap_or_else(|_| Err("迁移任务异常终止；源目录和目标副本均保留".into()))
}

#[tauri::command]
pub async fn undo_software_move(
    app: tauri::AppHandle,
    key: String,
    source_path: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = PathBuf::from(expand_environment(&source_path));
        let link_target = is_link(&source)
            .then(|| read_link_target(&source))
            .flatten();
        let manifest_file = if let Some(target) = &link_target {
            manifest_path(target, &key)
        } else {
            let mut found = None;
            for disk in sysinfo::Disks::new_with_refreshed_list().list() {
                let mount = disk.mount_point().to_string_lossy();
                let Some(letter) = drive_letter(Path::new(mount.as_ref())) else {
                    continue;
                };
                let candidate =
                    PathBuf::from(format!("{}:\\{MOVE_ROOT}\\{key}{MANIFEST_SUFFIX}", letter));
                if candidate.is_file() {
                    found = Some(candidate);
                    break;
                }
            }
            found.ok_or_else(|| "没有找到迁移记录，无法安全恢复".to_string())?
        };
        let manifest = read_manifest(&manifest_file);
        let target = link_target.or_else(|| {
            manifest
                .as_ref()
                .map(|item| PathBuf::from(&item.target_path))
        });
        let target = target.ok_or_else(|| "迁移记录无效，无法定位目标副本".to_string())?;
        if !target.is_dir() {
            return Err(format!("目标副本不存在: {}", target.display()));
        }

        if source.exists() && !is_link(&source) {
            let equal = directory_contents_equal(&source, &target)?;
            if !equal {
                return Err(format!(
                    "源目录和目标副本内容不同，均已保留。请先备份后再处理: {}",
                    source.display()
                ));
            }
            fs::remove_dir_all(&target)
                .map_err(|error| format!("源目录完整保留，但目标副本清理失败: {error}"))?;
            let _ = fs::remove_file(&manifest_file);
            return Ok(());
        }

        emit_progress(&app, &key, 0, "restoring");
        let parent = source.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|error| format!("无法创建源目录父级: {error}"))?;
        let staging = unique_sibling(parent, "restore", &key);
        let (total_size, _) = directory_size_and_count(&target);
        let mut copied = 0;
        copy_directory(
            &app,
            &target,
            &staging,
            total_size,
            &mut copied,
            &format!("{key}-undo"),
        )?;
        if !directory_contents_equal(&target, &staging)? {
            let _ = fs::remove_dir_all(&staging);
            return Err("恢复副本校验失败，迁移目标保持不变".into());
        }

        let link_backup = if is_link(&source) {
            let backup = unique_sibling(parent, "link-backup", &key);
            fs::rename(&source, &backup)
                .map_err(|error| format!("无法暂存原联接，目标副本保持不变: {error}"))?;
            Some(backup)
        } else {
            None
        };
        if let Err(error) = fs::rename(&staging, &source) {
            if let Some(backup) = &link_backup {
                let _ = fs::rename(backup, &source);
            }
            let _ = fs::remove_dir_all(&staging);
            return Err(format!("恢复切换失败，迁移目标仍保留: {error}"));
        }
        if let Some(backup) = link_backup {
            let _ = fs::remove_dir(&backup);
        }
        emit_progress(&app, &key, 99, "cleaning_up");
        fs::remove_dir_all(&target).map_err(|error| {
            format!(
                "源目录已恢复且通过校验；目标副本仍保留在 {}: {error}",
                target.display()
            )
        })?;
        let _ = fs::remove_file(&manifest_file);
        emit_progress(&app, &key, 100, "done");
        Ok(())
    })
    .await
    .unwrap_or_else(|_| Err("恢复任务异常终止；迁移目标副本保留".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("wc-software-move-test-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn key_is_stable_and_case_insensitive() {
        assert_eq!(
            path_key(Path::new(r"C:\Apps\Demo")),
            path_key(Path::new(r"c:\apps\demo"))
        );
    }

    #[test]
    fn content_verification_detects_changed_bytes() {
        let root = fixture();
        let left = root.join("left.bin");
        let right = root.join("right.bin");
        fs::write(&left, b"before").unwrap();
        fs::write(&right, b"before").unwrap();
        assert!(file_contents_equal(&left, &right).unwrap());
        fs::write(&right, b"after!").unwrap();
        assert!(!file_contents_equal(&left, &right).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn directory_verification_requires_matching_tree_and_content() {
        let root = fixture();
        let left = root.join("left");
        let right = root.join("right");
        fs::create_dir_all(left.join("nested")).unwrap();
        fs::create_dir_all(right.join("nested")).unwrap();
        fs::write(left.join("nested/data.bin"), b"payload").unwrap();
        fs::write(right.join("nested/data.bin"), b"payload").unwrap();
        assert!(directory_contents_equal(&left, &right).unwrap());
        fs::write(right.join("nested/data.bin"), b"changed").unwrap();
        assert!(!directory_contents_equal(&left, &right).unwrap());
        fs::remove_dir_all(root).unwrap();
    }
}
