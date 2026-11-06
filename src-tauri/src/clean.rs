use crate::file_cleanup::{CleanupPlan, FileFilter};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone)]
pub struct CleanResult {
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
    RecycleBin,
    LogFilesC,
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

// Serialize separate manual/automatic requests; each request uses the shared worker pool.
static CLEAN_RUN: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn windows_dir() -> PathBuf {
    PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into()))
}

fn add_env_folder(plan: &mut CleanupPlan, variable: &str, suffix: &str) {
    if let Some(root) = std::env::var_os(variable).filter(|value| !value.is_empty()) {
        plan.add_folder(&PathBuf::from(root).join(suffix));
    }
}

fn add_option_rules(plan: &mut CleanupPlan, option: CleanOption, result: &mut CleanResult) {
    match option {
        CleanOption::Prefetch => plan.add_rule(
            &windows_dir().join("Prefetch"),
            FileFilter::Extensions(vec!["pf".into()]),
            false,
            false,
            false,
        ),
        CleanOption::UserTemp => add_env_folder(plan, "TEMP", ""),
        CleanOption::WindowsTemp => plan.add_folder(&windows_dir().join("Temp")),
        CleanOption::SystemLogs => plan.add_folder(&windows_dir().join("Logs")),
        CleanOption::SoftwareDistribution => {
            if is_windows_update_active() {
                result.add_error(
                    "Windows Update is active, skipping SoftwareDistribution cleanup".into(),
                );
            } else {
                plan.add_folder(&windows_dir().join("SoftwareDistribution").join("Download"));
            }
        }
        CleanOption::BrowserCache => {
            add_env_folder(
                plan,
                "LOCALAPPDATA",
                "Google\\Chrome\\User Data\\Default\\Cache",
            );
            add_env_folder(
                plan,
                "LOCALAPPDATA",
                "Microsoft\\Edge\\User Data\\Default\\Cache",
            );
        }
        CleanOption::TempFilesC | CleanOption::LogFilesC | CleanOption::MspFiles => {
            let extensions = match option {
                CleanOption::TempFilesC => vec!["tmp".into(), "cache".into()],
                CleanOption::LogFilesC => vec!["log".into()],
                _ => vec!["msp".into()],
            };
            plan.add_rule(
                Path::new("C:\\"),
                FileFilter::Extensions(extensions),
                true,
                false,
                false,
            );
        }
        CleanOption::DeliveryOptimization => plan.add_folder(
            &windows_dir()
                .join("SoftwareDistribution")
                .join("DeliveryOptimization"),
        ),
        CleanOption::Thumbnails => {
            if let Some(root) = std::env::var_os("LOCALAPPDATA").filter(|value| !value.is_empty()) {
                plan.add_rule(
                    &PathBuf::from(root).join("Microsoft\\Windows\\Explorer"),
                    FileFilter::Pattern("thumbcache_*.db".into()),
                    false,
                    false,
                    false,
                );
            }
        }
        CleanOption::Defender => plan.add_folder(Path::new(
            "C:\\ProgramData\\Microsoft\\Windows Defender\\Scans\\History",
        )),
        CleanOption::InetCache => {
            add_env_folder(plan, "LOCALAPPDATA", "Microsoft\\Windows\\INetCache")
        }
        CleanOption::WerReports => {
            plan.add_folder(Path::new("C:\\ProgramData\\Microsoft\\Windows\\WER"));
            add_env_folder(plan, "LOCALAPPDATA", "Microsoft\\Windows\\WER");
        }
        CleanOption::ShaderCache => {
            for suffix in [
                "Microsoft\\DirectX Shader Cache",
                "NVIDIA\\DXCache",
                "AMD\\DxCache",
                "Intel\\ShaderCache",
            ] {
                add_env_folder(plan, "LOCALAPPDATA", suffix);
            }
        }
        CleanOption::NvidiaDebugLogs => {
            let mut roots = vec![
                PathBuf::from("C:\\ProgramData\\NVIDIA Corporation"),
                PathBuf::from(
                    std::env::var_os("ProgramFiles").unwrap_or_else(|| "C:\\Program Files".into()),
                )
                .join("NVIDIA Corporation"),
            ];
            if let Some(local) = std::env::var_os("LOCALAPPDATA").filter(|value| !value.is_empty())
            {
                roots.push(PathBuf::from(local).join("NVIDIA Corporation"));
            }
            for root in roots {
                plan.add_rule(
                    &root,
                    FileFilter::Pattern("debug.log".into()),
                    true,
                    false,
                    false,
                );
            }
        }
        CleanOption::WpsBackups => {
            for variable in ["LOCALAPPDATA", "APPDATA"] {
                if let Some(root) = std::env::var_os(variable).filter(|value| !value.is_empty()) {
                    plan.add_rule(
                        &PathBuf::from(root).join("Kingsoft"),
                        FileFilter::WpsBackups,
                        true,
                        false,
                        false,
                    );
                }
            }
        }
        CleanOption::RecycleBin => plan.add_folder(Path::new("C:\\$Recycle.Bin")),
        CleanOption::RestorePoints | CleanOption::Hibernation | CleanOption::Winapp2 => {}
    }
}

pub(crate) fn run_clean_options(options: &[CleanOption]) -> CleanResult {
    let _run = CLEAN_RUN.lock().unwrap_or_else(|error| error.into_inner());
    let before = measure_c_free().ok();
    let mut result = CleanResult::new();
    let mut plan = CleanupPlan::default();
    let mut seen = Vec::new();
    let mut registry_keys = Vec::new();
    // Collect all rules before changing files, including Winapp2 detection.
    for option in options {
        if seen.contains(option) {
            continue;
        }
        seen.push(*option);
        if *option == CleanOption::Winapp2 {
            registry_keys = crate::winapp2::append_file_rules(&mut plan);
        } else {
            add_option_rules(&mut plan, *option, &mut result);
        }
    }
    result.merge(plan.run());
    crate::winapp2::clean_registry_keys(&registry_keys);
    for option in seen {
        match option {
            CleanOption::RestorePoints => result.merge(clean_restore_points()),
            CleanOption::Hibernation => result.merge(disable_hibernation()),
            _ => {}
        }
    }
    if let (Some(before), Ok(after)) = (before, measure_c_free()) {
        result.bytes_freed = after.saturating_sub(before);
    }
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
    crate::logger::info(
        "clean",
        "自定义清理 开始",
        &format!("{} 项", request.options.len()),
    );
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
