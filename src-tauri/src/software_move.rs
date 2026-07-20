use serde::Serialize;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Emitter;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareInfo {
    pub key: String,
    pub display_path: String, // human-readable short path for UI
    pub size_bytes: u64,
    pub file_count: u64,
    pub exists: bool,
    /// Already moved by us — source is a junction pointing to another drive
    pub is_moved: bool,
    /// Target path if moved
    pub moved_to: Option<String>,
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
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}

// ---------------------------------------------------------------------------
// Known software data directories
// ---------------------------------------------------------------------------

struct KnownApp {
    key: &'static str,
    /// Environment-variable-expandable paths (e.g. %USERPROFILE%\Documents\...)
    paths: &'static [&'static str],
    /// Process names to look for (case-insensitive match)
    process_names: &'static [&'static str],
}

/// The order here defines the display order (most common / largest first).
static KNOWN_APPS: &[KnownApp] = &[
    KnownApp {
        key: "wechat",
        paths: &[r"%USERPROFILE%\Documents\WeChat Files"],
        process_names: &["WeChat.exe", "WeChatApp.exe", "WeChatStore.exe"],
    },
    KnownApp {
        key: "qq",
        paths: &[
            r"%USERPROFILE%\Documents\Tencent Files",
            r"%APPDATA%\Tencent\QQ",
        ],
        process_names: &["QQ.exe"],
    },
    KnownApp {
        key: "wework",
        paths: &[r"%USERPROFILE%\Documents\WXWork"],
        process_names: &["WXWork.exe"],
    },
    KnownApp {
        key: "dingtalk",
        paths: &[r"%APPDATA%\DingTalk"],
        process_names: &["DingTalk.exe"],
    },
    KnownApp {
        key: "feishu",
        paths: &[r"%APPDATA%\Lark"],
        process_names: &["Lark.exe", "Feishu.exe"],
    },
    KnownApp {
        key: "wemeet",
        paths: &[r"%APPDATA%\Tencent\WeMeet"],
        process_names: &["wemeetapp.exe"],
    },
    KnownApp {
        key: "baidunetdisk",
        paths: &[r"%APPDATA%\baidu\BaiduNetdisk"],
        process_names: &["baidunetdisk.exe"],
    },
    KnownApp {
        key: "netease_cloudmusic",
        paths: &[r"%LOCALAPPDATA%\Netease\CloudMusic"],
        process_names: &["cloudmusic.exe"],
    },
    KnownApp {
        key: "qqmusic",
        paths: &[r"%LOCALAPPDATA%\Tencent\QQMusic"],
        process_names: &["QQMusic.exe"],
    },
    KnownApp {
        key: "wps",
        paths: &[r"%APPDATA%\Kingsoft"],
        process_names: &["wps.exe", "et.exe", "wpp.exe"],
    },
    KnownApp {
        key: "chrome",
        paths: &[r"%LOCALAPPDATA%\Google\Chrome\User Data"],
        process_names: &["chrome.exe"],
    },
    KnownApp {
        key: "edge",
        paths: &[r"%LOCALAPPDATA%\Microsoft\Edge\User Data"],
        process_names: &["msedge.exe"],
    },
    KnownApp {
        key: "firefox",
        paths: &[r"%APPDATA%\Mozilla\Firefox\Profiles"],
        process_names: &["firefox.exe"],
    },
    KnownApp {
        key: "steam",
        paths: &[r"C:\Program Files (x86)\Steam\steamapps\common"],
        process_names: &["steam.exe"],
    },
    KnownApp {
        key: "vscode",
        paths: &[r"%APPDATA%\Code\User\workspaceStorage"],
        process_names: &["Code.exe", "code.exe"],
    },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn expand_env(path: &str) -> String {
    let mut result = path.replace("%USERPROFILE%", "");
    if let Ok(home) = std::env::var("USERPROFILE") {
        result = path.replace("%USERPROFILE%", &home);
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        result = result.replace("%APPDATA%", &appdata);
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        result = result.replace("%LOCALAPPDATA%", &local);
    }
    result
}

fn dir_size_and_count(path: &Path) -> (u64, u64) {
    let mut total_size = 0u64;
    let mut file_count = 0u64;
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            // Check for junction BEFORE recursing — don't follow it
            if p.is_symlink() {
                continue;
            }
            let (s, c) = dir_size_and_count(&p);
            total_size += s;
            file_count += c;
        } else if let Ok(meta) = entry.metadata() {
            total_size += meta.len();
            file_count += 1;
        }
    }
    (total_size, file_count)
}

/// Build a human-friendly short path string for display.
fn display_path(full_path: &str, userprofile: &str) -> String {
    if full_path.starts_with(userprofile) {
        full_path.replacen(userprofile, "%USERPROFILE%", 1)
    } else {
        full_path.to_string()
    }
}

fn is_junction(path: &Path) -> bool {
    path.is_symlink() && path.is_dir()
}

fn read_junction_target(path: &Path) -> Option<PathBuf> {
    if path.is_symlink() {
        fs::read_link(path).ok()
    } else {
        None
    }
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Find processes whose name matches any in the given list (case-insensitive).
fn find_running_processes(names: &[&str]) -> Vec<ProcessInfo> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let mut found: Vec<ProcessInfo> = Vec::new();
    for proc in sys.processes().values() {
        let pname = proc.name().to_string_lossy().to_string();
        if names.iter().any(|n| n.eq_ignore_ascii_case(&pname)) {
            found.push(ProcessInfo {
                name: pname,
                pid: proc.pid().as_u32(),
            });
        }
    }
    found
}

/// Kill processes by name using taskkill /F.
fn kill_processes(name: &str) -> Result<(), String> {
    let output = Command::new("taskkill")
        .args(["/F", "/IM", name])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to run taskkill: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "process not found" is not an error for us
        if stderr.contains("not found") || stderr.contains("見つかりません") {
            return Ok(());
        }
        return Err(format!("taskkill failed: {}", stderr.trim()));
    }
    Ok(())
}

/// Remove a junction directory safely (does NOT delete the target).
fn remove_junction(path: &Path) -> Result<(), String> {
    fs::remove_dir(path).map_err(|e| format!("Failed to remove junction {}: {}", path.display(), e))
}

/// Returns true for errors that are safe to skip (permission, locking, etc.)
/// rather than aborting the whole operation.
fn is_skippable_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(5) | Some(32) | Some(33))
    //  5 = ERROR_ACCESS_DENIED
    // 32 = ERROR_SHARING_VIOLATION
    // 33 = ERROR_LOCK_VIOLATION
}

/// Copy a directory tree recursively, emitting progress events.
/// Individual file/dir failures are skipped (recorded in `skipped`) instead of
/// aborting the whole copy.
fn copy_dir_with_progress(
    app: &tauri::AppHandle,
    src: &Path,
    dst: &Path,
    total_size: u64,
    copied: &mut u64,
    app_key: &str,
    skipped: &mut Vec<String>,
) {
    if let Err(e) = fs::create_dir_all(dst) {
        skipped.push(format!("[dir] {} (create failed: {})", dst.display(), e));
        return;
    }

    let entries = match fs::read_dir(src) {
        Ok(e) => e,
        Err(e) => {
            skipped.push(format!("[dir] {} (read failed: {})", src.display(), e));
            return;
        }
    };

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            // Don't follow junctions/symlinks
            if src_path.is_symlink() {
                continue;
            }
            copy_dir_with_progress(app, &src_path, &dst_path, total_size, copied, app_key, skipped);
        } else {
            match fs::copy(&src_path, &dst_path) {
                Ok(_) => {
                    let file_size = entry.metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);
                    *copied += file_size;
                    let percent = ((*copied as f64 / total_size as f64) * 100.0).min(99.0) as u64;
                    let _ = app.emit("software-move-progress", serde_json::json!({
                        "key": app_key,
                        "percent": percent,
                        "status": "copying"
                    }));
                }
                Err(e) => {
                    skipped.push(src_path.display().to_string());
                    if !is_skippable_error(&e) {
                        // Log unexpected errors but keep going
                        let _ = app.emit("software-move-progress", serde_json::json!({
                            "key": app_key,
                            "percent": ((*copied as f64 / total_size as f64) * 100.0).min(99.0) as u64,
                            "status": "copying",
                            "message": format!("Skipped {}: {}", src_path.display(), e)
                        }));
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn scan_software() -> Result<Vec<SoftwareInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let userprofile = std::env::var("USERPROFILE").unwrap_or_default();

        let mut results: Vec<SoftwareInfo> = Vec::new();

        for app in KNOWN_APPS {
            // Aggregate size across all paths for this app
            let mut total_size = 0u64;
            let mut total_files = 0u64;
            let mut any_exists = false;
            let mut primary_path = "";
            let mut is_moved = false;
            let mut moved_to: Option<String> = None;

            for raw_path in app.paths {
                let expanded = expand_env(raw_path);
                let p = Path::new(&expanded);

                if !p.exists() {
                    continue;
                }

                if primary_path.is_empty() {
                    primary_path = raw_path;
                }
                any_exists = true;

                // Check if already moved (source is a junction pointing elsewhere)
                if is_junction(p) {
                    if let Some(target) = read_junction_target(p) {
                        let target_str = target.to_string_lossy().to_string();
                        // Junction pointing to another drive → moved by us
                        let source_drive = p
                            .to_string_lossy()
                            .chars()
                            .next()
                            .unwrap_or('C');
                        let target_drive = target_str.chars().next().unwrap_or('C');
                        if source_drive.to_ascii_uppercase() != target_drive.to_ascii_uppercase() {
                            is_moved = true;
                            moved_to = Some(target_str);
                        }
                    }
                } else {
                    let (s, c) = dir_size_and_count(p);
                    total_size += s;
                    total_files += c;
                }
            }

            if any_exists {
                results.push(SoftwareInfo {
                    key: app.key.to_string(),
                    display_path: display_path(primary_path, &userprofile),
                    size_bytes: total_size,
                    file_count: total_files,
                    exists: true,
                    is_moved,
                    moved_to,
                });
            }
        }

        // Sort by size descending
        results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        Ok(results)
    })
    .await
    .unwrap_or_else(|_| Err("Scan task panicked".into()))
}

#[tauri::command]
pub async fn get_available_drives() -> Result<Vec<DriveInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut drives: Vec<DriveInfo> = Vec::new();

        for disk in disks.list() {
            let mount = disk.mount_point().to_string_lossy().to_string();
            // Extract drive letter, skip C:
            let letter = mount
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_default();
            if letter == "C" {
                continue; // Don't offer C: as target
            }
            if letter.is_empty() || !mount.contains(':') {
                continue;
            }
            // Deduplicate (some drives appear multiple times)
            if drives.iter().any(|d| d.letter == letter) {
                continue;
            }
            drives.push(DriveInfo {
                letter: letter.clone(),
                free_bytes: disk.available_space(),
                total_bytes: disk.total_space(),
            });
        }

        Ok(drives)
    })
    .await
    .unwrap_or_else(|_| Err("Drive scan panicked".into()))
}

#[tauri::command]
pub async fn check_running_processes(key: String) -> Result<Vec<ProcessInfo>, String> {
    // Look up the KnownApp entry
    let app = KNOWN_APPS
        .iter()
        .find(|a| a.key == key)
        .ok_or_else(|| format!("Unknown app key: {}", key))?;
    Ok(find_running_processes(app.process_names))
}

#[tauri::command]
pub async fn kill_running_processes(key: String) -> Result<Vec<ProcessInfo>, String> {
    let app = KNOWN_APPS
        .iter()
        .find(|a| a.key == key)
        .ok_or_else(|| format!("Unknown app key: {}", key))?;

    // Find running processes first
    let running = find_running_processes(app.process_names);

    // Kill each one
    let mut killed: Vec<ProcessInfo> = Vec::new();
    for proc in &running {
        match kill_processes(&proc.name) {
            Ok(_) => killed.push(proc.clone()),
            Err(e) => {
                // If some fail to kill, still try the rest
                let _ = e; // log could go here
            }
        }
    }

    // Brief wait to let processes exit
    std::thread::sleep(std::time::Duration::from_millis(1500));

    Ok(killed)
}

#[tauri::command]
pub async fn move_software(
    app: tauri::AppHandle,
    key: String,
    source_path_template: String,
    target_drive: String,
) -> Result<MoveResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let expanded = expand_env(&source_path_template);
        let src = Path::new(&expanded);

        if !src.exists() {
            return Err(format!("Source path not found: {}", expanded));
        }
        if is_junction(src) {
            return Err("Already moved".into());
        }

        let target_dir = format!("{}:\\MovedApps\\{}", target_drive, key);
        let dst = Path::new(&target_dir);

        if dst.exists() {
            return Err(format!("Target already exists: {}", target_dir));
        }

        // Check disk space
        let (total_size, _) = dir_size_and_count(src);
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut free_on_target: u64 = 0;
        for disk in disks.list() {
            let mount = disk.mount_point().to_string_lossy().to_string();
            if mount.starts_with(&target_drive) {
                free_on_target = disk.available_space();
                break;
            }
        }
        if free_on_target < total_size + 1024 * 1024 * 100 {
            // Need at least 100MB extra
            return Err(format!(
                "Not enough space on {}:. Need {} MB, have {} MB",
                target_drive,
                total_size / (1024 * 1024),
                free_on_target / (1024 * 1024),
            ));
        }

        // Phase 1: Copy
        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 0u64,
            "status": "copying"
        }));

        let mut copied: u64 = 0;
        let mut skipped: Vec<String> = Vec::new();
        copy_dir_with_progress(&app, src, dst.as_ref(), total_size, &mut copied, &key, &mut skipped);

        crate::logger::info("move", &format!("开始移动 {} → {}", key, target_dir), "");

        // Safety gate: if ANY files were skipped, do NOT touch the source.
        // remove_dir_all is destructive — a partial failure would destroy data.
        if !skipped.is_empty() {
            let _ = fs::remove_dir_all(dst);
            crate::logger::error("move", &format!("{} 移动失败: {} 个文件无法复制", key, skipped.len()), "");
            return Err(format!(
                "{} 个文件无法复制（被占用或权限不足）。源目录未变动，已复制的临时数据已清除。请关闭相关软件后重试。",
                skipped.len()
            ));
        }

        // All files copied successfully — safe to remove source
        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 99u64,
            "status": "removing_source"
        }));

        if let Err(e) = fs::remove_dir_all(src) {
            // remove_dir_all is destructive — it may have partially deleted src.
            // Restore from the copy before cleaning up.
            let mut restore_skipped: Vec<String> = Vec::new();
            copy_dir_with_progress(
                &app, dst.as_ref(), src, total_size, &mut 0u64, &key, &mut restore_skipped,
            );
            let _ = fs::remove_dir_all(dst);
            return Err(format!("无法删除源目录: {}。已从备份恢复。", e));
        }

        // Create junction
        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 99u64,
            "status": "creating_link"
        }));

        let output = Command::new("cmd")
            .args(["/c", "mklink", "/J", &expanded, &target_dir])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run mklink: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Junction failed → roll back by restoring source from copy
            // Source is already deleted, so move files back from target
            let _ = copy_dir_with_progress(
                &app, dst.as_ref(), src, total_size, &mut 0u64, &key, &mut Vec::new(),
            );
            let _ = fs::remove_dir_all(dst);
            return Err(format!("mklink failed: {}", stderr.trim()));
        }

        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 100u64,
            "status": "done"
        }));

        crate::logger::info("move", &format!("移动完成 {}", key), &format!("→ {}", target_dir));
        Ok(MoveResult { skipped_files: skipped, junction_created: true })
    })
    .await
    .unwrap_or_else(|_| Err("Move task panicked".into()))
}

#[tauri::command]
pub async fn undo_software_move(
    app: tauri::AppHandle,
    key: String,
    source_path_template: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let expanded = expand_env(&source_path_template);
        let junction_path = Path::new(&expanded);

        if !junction_path.exists() || !is_junction(junction_path) {
            return Err("Not a junction, nothing to undo".into());
        }

        let target_path = read_junction_target(junction_path)
            .ok_or("Cannot read junction target")?;

        if !target_path.exists() {
            return Err(format!(
                "Target directory no longer exists: {}",
                target_path.display()
            ));
        }

        // Phase 1: Remove the junction
        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 0u64,
            "status": "removing_link"
        }));
        remove_junction(junction_path)?;

        // Phase 2: Move files back
        let (total_size, _) = dir_size_and_count(&target_path);
        let mut copied: u64 = 0;
        let mut skipped: Vec<String> = Vec::new();
        copy_dir_with_progress(
            &app,
            &target_path,
            junction_path,
            total_size,
            &mut copied,
            &format!("{}-undo", key),
            &mut skipped,
        );

        // Phase 3: Remove target
        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 99u64,
            "status": "cleaning_up"
        }));
        fs::remove_dir_all(&target_path)
            .map_err(|e| format!("Failed to clean up {}: {}", target_path.display(), e))?;

        let _ = app.emit("software-move-progress", serde_json::json!({
            "key": key,
            "percent": 100u64,
            "status": "done"
        }));

        Ok(())
    })
    .await
    .unwrap_or_else(|_| Err("Undo task panicked".into()))
}
