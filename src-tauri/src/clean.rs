use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone)]
pub struct CleanResult {
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CleanOption {
    Prefetch,
    UserTemp,
    WindowsTemp,
    SystemLogs,
    SoftwareDistribution,
    BrowserCache,
    TempFilesC,
    RestorePoints,
    Hibernation,
    DeliveryOptimization,
    Thumbnails,
    Defender,
    InetCache,
    WerReports,
    ShaderCache,
    NvidiaDebugLogs,
    WpsBackups,
    Winapp2,
    MspFiles,
}

#[derive(Deserialize)]
pub struct CleanRequest {
    pub options: Vec<CleanOption>,
}

impl CleanResult {
    pub(crate) fn new() -> Self {
        CleanResult {
            bytes_freed: 0,
            errors: Vec::new(),
        }
    }

    pub(crate) fn add_freed(&mut self, bytes: u64) {
        self.bytes_freed += bytes;
    }

    pub(crate) fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub(crate) fn merge(&mut self, other: CleanResult) {
        self.bytes_freed += other.bytes_freed;
        self.errors.extend(other.errors);
    }
}

/// Delete all files and subdirectories inside a folder.
/// Each file that is locked by another process will be skipped (error logged, not fatal).
fn clean_folder_contents(folder: &Path) -> CleanResult {
    let mut result = CleanResult::new();

    let entries = match fs::read_dir(folder) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", folder.display(), e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            match fs::remove_dir_all(&path) {
                Ok(_) => {}
                Err(e) => result.add_error(format!("Cannot delete {}: {}", path.display(), e)),
            }
        } else {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            match fs::remove_file(&path) {
                Ok(_) => result.add_freed(size),
                Err(e) => {
                    result.add_error(format!("Skip {}: {}", path.display(), e))
                }
            }
        }
    }

    result
}

fn windows_dir() -> PathBuf {
    PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into()))
}

fn local_appdata() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
}

fn clean_prefetch() -> CleanResult {
    let mut result = CleanResult::new();
    let path = windows_dir().join("Prefetch");
    if !path.exists() {
        return result;
    }

    let entries = match fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", path.display(), e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            continue;
        }
        let is_pf = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pf"))
            .unwrap_or(false);
        if !is_pf {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        match fs::remove_file(&p) {
            Ok(_) => result.add_freed(size),
            Err(e) => result.add_error(format!("Skip {}: {}", p.display(), e)),
        }
    }

    result
}

fn clean_user_temp() -> CleanResult {
    let path = PathBuf::from(std::env::var("TEMP").unwrap_or_default());
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

fn clean_windows_temp() -> CleanResult {
    let path = windows_dir().join("Temp");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

fn clean_system_logs() -> CleanResult {
    let path = windows_dir().join("Logs");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

fn is_windows_update_active() -> bool {
    std::process::Command::new("sc")
        .args(["query", "wuauserv"])
        .output()
        .map(|out| {
            let text = String::from_utf8_lossy(&out.stdout);
            text.contains("RUNNING")
        })
        .unwrap_or(false)
}

fn clean_software_distribution() -> CleanResult {
    let mut result = CleanResult::new();

    if is_windows_update_active() {
        result.add_error("Windows Update is active, skipping SoftwareDistribution cleanup".into());
        return result;
    }

    let path = windows_dir().join("SoftwareDistribution").join("Download");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        result
    }
}

fn clean_restore_points() -> CleanResult {
    let mut result = CleanResult::new();

    let out = std::process::Command::new("vssadmin")
        .args(["delete", "shadows", "/for=C:", "/all", "/quiet"])
        .output();

    match out {
        Ok(o) if !o.status.success() => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            result.add_error(format!("vssadmin: {}", stderr.trim()));
        }
        Err(e) => {
            result.add_error(format!("Cannot run vssadmin: {}", e));
        }
        _ => {}
    }

    result
}

fn clean_temp_by_ext(root: &Path) -> CleanResult {
    let mut result = CleanResult::new();
    let exts = ["tmp", "cache"];

    fn walk(dir: &Path, exts: &[&str], result: &mut CleanResult) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, exts, result);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    if fs::remove_file(&path).is_ok() {
                        result.add_freed(size);
                    }
                }
            }
        }
    }

    walk(root, &exts, &mut result);
    result
}

fn clean_temp_files_c() -> CleanResult {
    let root = PathBuf::from("C:\\");
    if root.exists() {
        clean_temp_by_ext(&root)
    } else {
        CleanResult::new()
    }
}

fn clean_msp_files() -> CleanResult {
    fn walk(dir: &Path, result: &mut CleanResult) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, result);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("msp"))
                .unwrap_or(false)
            {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    result.add_freed(size);
                }
            }
        }
    }

    let mut result = CleanResult::new();
    let root = PathBuf::from("C:\\");
    if root.exists() {
        walk(&root, &mut result);
    }
    result
}

fn clean_browser_cache() -> CleanResult {
    let mut result = CleanResult::new();

    let chrome = local_appdata()
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join("Default")
        .join("Cache");
    if chrome.exists() {
        result.merge(clean_folder_contents(&chrome));
    }

    let edge = local_appdata()
        .join("Microsoft")
        .join("Edge")
        .join("User Data")
        .join("Default")
        .join("Cache");
    if edge.exists() {
        result.merge(clean_folder_contents(&edge));
    }

    result
}

fn disable_hibernation() -> CleanResult {
    let mut result = CleanResult::new();

    match std::process::Command::new("powercfg")
        .args(["/h", "off"])
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                result.add_freed(0);
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let msg = stderr.trim();
                if !msg.is_empty() {
                    result.add_error(format!("powercfg: {}", msg));
                }
            }
        }
        Err(e) => {
            result.add_error(format!("Cannot run powercfg: {}", e));
        }
    }

    result
}

fn clean_delivery_optimization() -> CleanResult {
    let path = windows_dir()
        .join("SoftwareDistribution")
        .join("DeliveryOptimization");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

fn clean_thumbnails() -> CleanResult {
    let path = local_appdata()
        .join("Microsoft")
        .join("Windows")
        .join("Explorer");
    if !path.exists() {
        return CleanResult::new();
    }

    let mut result = CleanResult::new();
    let entries = match fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", path.display(), e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("thumbcache_") && name.ends_with(".db") {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    match fs::remove_file(&p) {
                        Ok(_) => result.add_freed(size),
                        Err(e) => result.add_error(format!("Skip {}: {}", p.display(), e)),
                    }
                }
            }
        }
    }

    result
}

fn clean_defender() -> CleanResult {
    let path = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows Defender\\Scans\\History");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

fn clean_inet_cache() -> CleanResult {
    let path = local_appdata()
        .join("Microsoft")
        .join("Windows")
        .join("INetCache");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

fn clean_wer_reports() -> CleanResult {
    let mut result = CleanResult::new();

    let system_wer = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\WER");
    if system_wer.exists() {
        result.merge(clean_folder_contents(&system_wer));
    }

    let user_wer = local_appdata()
        .join("Microsoft")
        .join("Windows")
        .join("WER");
    if user_wer.exists() {
        result.merge(clean_folder_contents(&user_wer));
    }

    result
}

fn clean_shader_cache() -> CleanResult {
    let mut result = CleanResult::new();

    let dx_cache = local_appdata()
        .join("Microsoft")
        .join("DirectX Shader Cache");
    if dx_cache.exists() {
        result.merge(clean_folder_contents(&dx_cache));
    }

    let nv_cache = local_appdata().join("NVIDIA").join("DXCache");
    if nv_cache.exists() {
        result.merge(clean_folder_contents(&nv_cache));
    }

    let amd_cache = local_appdata().join("AMD").join("DxCache");
    if amd_cache.exists() {
        result.merge(clean_folder_contents(&amd_cache));
    }

    let intel_cache = local_appdata().join("Intel").join("ShaderCache");
    if intel_cache.exists() {
        result.merge(clean_folder_contents(&intel_cache));
    }

    result
}

fn clean_nvidia_debug_logs() -> CleanResult {
    let mut result = CleanResult::new();

    let roots = [
        PathBuf::from("C:\\ProgramData\\NVIDIA Corporation"),
        PathBuf::from(std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into()))
            .join("NVIDIA Corporation"),
        local_appdata().join("NVIDIA Corporation"),
    ];

    for root in &roots {
        if !root.exists() {
            continue;
        }
        delete_named_files_recursive(root, "debug.log", &mut result);
    }

    result
}

fn clean_wps_backups() -> CleanResult {
    let mut result = CleanResult::new();
    let appdata = PathBuf::from(std::env::var("APPDATA").unwrap_or_default());

    let backup_dir = appdata.join("Kingsoft").join("office6").join("backup");
    if backup_dir.exists() {
        result.merge(clean_folder_contents(&backup_dir));
    }

    let local = local_appdata().join("Kingsoft");
    if local.exists() {
        collect_and_clean_backup_dirs(&local, &mut result);
    }

    let roaming = appdata.join("Kingsoft");
    if roaming.exists() {
        collect_and_clean_backup_dirs(&roaming, &mut result);
    }

    result
}

fn collect_and_clean_backup_dirs(dir: &Path, result: &mut CleanResult) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if dir_name.eq_ignore_ascii_case("backup") {
                result.merge(clean_folder_contents(&path));
            } else {
                collect_and_clean_backup_dirs(&path, result);
            }
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let lower = name.to_lowercase();
            if lower.ends_with(".bak") || lower.ends_with(".wbk") {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    result.add_freed(size);
                }
            }
        }
    }
}

fn delete_named_files_recursive(dir: &Path, target_name: &str, result: &mut CleanResult) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            delete_named_files_recursive(&path, target_name, result);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case(target_name) {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    result.add_freed(size);
                }
            }
        }
    }
}

const QUICK_CLEAN_OPTIONS: &[CleanOption] = &[
    CleanOption::Prefetch,
    CleanOption::UserTemp,
    CleanOption::WindowsTemp,
    CleanOption::SystemLogs,
    CleanOption::SoftwareDistribution,
    CleanOption::BrowserCache,
    CleanOption::TempFilesC,
    CleanOption::RestorePoints,
    CleanOption::Hibernation,
    CleanOption::DeliveryOptimization,
    CleanOption::Thumbnails,
    CleanOption::Defender,
    CleanOption::InetCache,
    CleanOption::WerReports,
    CleanOption::ShaderCache,
    CleanOption::NvidiaDebugLogs,
    CleanOption::WpsBackups,
    CleanOption::Winapp2,
];

pub fn run_clean_option(option: CleanOption) -> CleanResult {
    let name = format!("{:?}", option);
    let result = match option {
        CleanOption::Prefetch => clean_prefetch(),
        CleanOption::UserTemp => clean_user_temp(),
        CleanOption::WindowsTemp => clean_windows_temp(),
        CleanOption::SystemLogs => clean_system_logs(),
        CleanOption::SoftwareDistribution => clean_software_distribution(),
        CleanOption::BrowserCache => clean_browser_cache(),
        CleanOption::TempFilesC => clean_temp_files_c(),
        CleanOption::RestorePoints => clean_restore_points(),
        CleanOption::Hibernation => disable_hibernation(),
        CleanOption::DeliveryOptimization => clean_delivery_optimization(),
        CleanOption::Thumbnails => clean_thumbnails(),
        CleanOption::Defender => clean_defender(),
        CleanOption::InetCache => clean_inet_cache(),
        CleanOption::WerReports => clean_wer_reports(),
        CleanOption::ShaderCache => clean_shader_cache(),
        CleanOption::NvidiaDebugLogs => clean_nvidia_debug_logs(),
        CleanOption::WpsBackups => clean_wps_backups(),
        CleanOption::Winapp2 => crate::winapp2::clean_winapp2(),
        CleanOption::MspFiles => clean_msp_files(),
    };
    if result.bytes_freed > 0 || !result.errors.is_empty() {
        crate::logger::info(
            "clean",
            &format!("{}: {} 字节", name, result.bytes_freed),
            &format!("errors={}", result.errors.len()),
        );
    }
    for e in &result.errors {
        crate::logger::warn("clean", &name, e);
    }
    result
}

fn run_clean_options(options: &[CleanOption]) -> CleanResult {
    let mut result = CleanResult::new();
    let before = measure_c_free().unwrap_or(0);

    for option in options {
        result.merge(run_clean_option(*option));
    }

    let after = measure_c_free().unwrap_or(before);
    result.bytes_freed = after.saturating_sub(before);
    result
}

fn measure_c_free() -> Result<u64, String> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy();
        if mount.starts_with("C:") || mount.starts_with("c:") {
            return Ok(disk.available_space());
        }
    }
    Err("C: drive not found".into())
}

#[tauri::command]
pub async fn quick_clean() -> CleanResult {
    crate::logger::info("clean", "快速清理 开始", "");
    let r = tauri::async_runtime::spawn_blocking(|| run_clean_options(QUICK_CLEAN_OPTIONS))
        .await
        .unwrap_or_else(|_| {
            let mut err = CleanResult::new();
            err.add_error("Clean task panicked".into());
            err
        });
    crate::logger::info(
        "clean",
        &format!("快速清理 完成: 释放 {} 字节", r.bytes_freed),
        &format!("errors={}", r.errors.len()),
    );
    for e in &r.errors {
        crate::logger::warn("clean", "清理错误", e);
    }
    r
}

#[tauri::command]
pub async fn deep_clean() -> CleanResult {
    quick_clean().await
}

#[tauri::command]
pub async fn clean_selected(request: CleanRequest) -> CleanResult {
    crate::logger::info("clean", "自定义清理 开始", &format!("{} 项", request.options.len()));
    let r = tauri::async_runtime::spawn_blocking(move || run_clean_options(&request.options))
        .await
        .unwrap_or_else(|_| {
            let mut err = CleanResult::new();
            err.add_error("Clean task panicked".into());
            err
        });
    crate::logger::info(
        "clean",
        &format!("自定义清理 完成: 释放 {} 字节", r.bytes_freed),
        &format!("errors={}", r.errors.len()),
    );
    for e in &r.errors {
        crate::logger::warn("clean", "清理错误", e);
    }
    r
}
