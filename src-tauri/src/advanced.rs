use serde::{Deserialize, Serialize};
use winreg::enums::*;
use winreg::RegKey;

// ---------------------------------------------------------------------------
// Combined status (single invoke → no focus flicker)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedStatus {
    pub defender_disabled: bool,
    pub update_disabled: bool,
    pub page_file: PageFileInfo,
}

#[tauri::command]
pub async fn get_advanced_status() -> Result<AdvancedStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let defender_disabled = get_defender_status_raw()?;
        let update_disabled = match get_update_status_raw() {
            Ok(s) => s.disabled,
            Err(_) => false,
        };
        let page_file = get_page_file_info_raw()?;
        Ok(AdvancedStatus {
            defender_disabled,
            update_disabled,
            page_file,
        })
    })
    .await
    .unwrap_or_else(|_| {
        Err("Failed to read advanced status".into())
    })
}

fn get_defender_status_raw() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey(DEFENDER_KEY) {
        Ok(k) => {
            let val: u32 = k.get_value("DisableAntiSpyware").unwrap_or(0);
            Ok(val == 1)
        }
        Err(_) => Ok(false),
    }
}

fn get_update_status_raw() -> Result<UpdateStatus, String> {
    let disabled = read_service_disabled("wuauserv");
    Ok(UpdateStatus {
        disabled,
        service_running: !disabled,
    })
}

fn get_page_file_info_raw() -> Result<PageFileInfo, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;
    let paging_files: String = key.get_value("PagingFiles").unwrap_or_default();
    if paging_files.is_empty() || paging_files.trim().is_empty() {
        return Ok(PageFileInfo { initial_mb: 0, max_mb: 0, system_managed: true });
    }
    let first_line = paging_files.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let (initial, max) = if parts.len() >= 3 {
        (parts[1].parse::<u32>().unwrap_or(0), parts[2].parse::<u32>().unwrap_or(0))
    } else {
        (0, 0)
    };
    Ok(PageFileInfo { initial_mb: initial, max_mb: max, system_managed: initial == 0 && max == 0 })
}

// ---------------------------------------------------------------------------
// Windows Defender
// ---------------------------------------------------------------------------

const DEFENDER_KEY: &str = "SOFTWARE\\Policies\\Microsoft\\Windows Defender";

#[tauri::command]
pub fn get_defender_status() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(DEFENDER_KEY);
    match key {
        Ok(k) => {
            let val: u32 = k.get_value("DisableAntiSpyware").unwrap_or(0);
            Ok(val == 1)
        }
        Err(_) => Ok(false), // key doesn't exist = defender is ON (not disabled)
    }
}

#[tauri::command]
pub fn set_defender_disabled(disabled: bool) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm.create_subkey(DEFENDER_KEY).map_err(|e| e.to_string())?;

    if disabled {
        key.set_value("DisableAntiSpyware", &1u32)
            .map_err(|e| format!("Failed to disable Defender: {}", e))?;
        // Also disable real-time monitoring via policy
        let _ = key.set_value("DisableRealtimeMonitoring", &1u32);
        let _ = key.set_value("DisableBehaviorMonitoring", &1u32);
        let _ = key.set_value("DisableOnAccessProtection", &1u32);
        let _ = key.set_value("DisableScanOnRealtimeEnable", &1u32);
    } else {
        // Delete the values to re-enable
        let _ = key.delete_value("DisableAntiSpyware");
        let _ = key.delete_value("DisableRealtimeMonitoring");
        let _ = key.delete_value("DisableBehaviorMonitoring");
        let _ = key.delete_value("DisableOnAccessProtection");
        let _ = key.delete_value("DisableScanOnRealtimeEnable");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Windows Update
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct UpdateStatus {
    pub disabled: bool,
    pub service_running: bool,
}

/// Read service Start value from registry — no process spawn.
/// HKLM\SYSTEM\CurrentControlSet\Services\<name>\Start:
///   2=AUTO, 3=MANUAL, 4=DISABLED
fn read_service_disabled(name: &str) -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!("SYSTEM\\CurrentControlSet\\Services\\{}", name);
    hklm.open_subkey(&path)
        .ok()
        .and_then(|k| k.get_value::<u32, _>("Start").ok())
        .map(|v| v == 4)
        .unwrap_or(false)
}

/// Write service Start value via registry — no process spawn.
fn write_service_disabled(name: &str, disabled: bool) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!("SYSTEM\\CurrentControlSet\\Services\\{}", name);
    let (key, _) = hklm.create_subkey(&path).map_err(|e| e.to_string())?;
    let start_val: u32 = if disabled { 4 } else { 2 }; // 4=DISABLED, 2=AUTO
    key.set_value("Start", &start_val)
        .map_err(|e| format!("Failed to set {} Start: {}", name, e))
}

#[tauri::command]
pub fn get_update_status() -> Result<UpdateStatus, String> {
    let disabled = read_service_disabled("wuauserv");
    Ok(UpdateStatus {
        disabled,
        service_running: !disabled,
    })
}

#[tauri::command]
pub fn set_update_disabled(disabled: bool) -> Result<(), String> {
    write_service_disabled("wuauserv", disabled)?;
    let _ = write_service_disabled("UsoSvc", disabled);
    let _ = write_service_disabled("WaaSMedicSvc", disabled);
    // Registry change takes effect on next reboot
    Ok(())
}

// ---------------------------------------------------------------------------
// Virtual Memory (Page File)
// ---------------------------------------------------------------------------

const MEMORY_MGMT_KEY: &str =
    "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageFileInfo {
    pub initial_mb: u32,
    pub max_mb: u32,
    /// System-managed = true, custom size = false
    pub system_managed: bool,
}

#[tauri::command]
pub fn get_page_file_info() -> Result<PageFileInfo, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;

    let paging_files: String = key
        .get_value("PagingFiles")
        .unwrap_or_default();

    // "PagingFiles" is REG_MULTI_SZ but may be read as REG_SZ.
    // Format per line: "<disk>:\pagefile.sys <min> <max>"
    if paging_files.is_empty() || paging_files.trim().is_empty() {
        return Ok(PageFileInfo {
            initial_mb: 0,
            max_mb: 0,
            system_managed: true,
        });
    }

    let first_line = paging_files.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    // Expected: "C:\pagefile.sys 0 0" or "C:\pagefile.sys 2048 4096"
    let (initial, max) = if parts.len() >= 3 {
        (parts[1].parse::<u32>().unwrap_or(0), parts[2].parse::<u32>().unwrap_or(0))
    } else {
        (0, 0)
    };

    Ok(PageFileInfo {
        initial_mb: initial,
        max_mb: max,
        // When both are 0 it means system-managed
        system_managed: initial == 0 && max == 0,
    })
}

#[tauri::command]
pub fn set_page_file(initial_mb: u32, max_mb: u32) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm
        .create_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;

    // Find the system drive letter
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let entry = format!("{}\\pagefile.sys {} {}", system_drive, initial_mb, max_mb);
    key.set_value("PagingFiles", &entry)
        .map_err(|e| format!("Failed to set page file: {}", e))?;

    // Also set the existing page file to avoid conflicts
    key.set_value("ExistingPagingFiles", &entry)
        .map_err(|e| format!("Failed to set existing page file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn reset_page_file_to_system_managed() -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm
        .create_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;

    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let entry = format!("{}\\pagefile.sys 0 0", system_drive);
    key.set_value("PagingFiles", &entry)
        .map_err(|e| format!("Failed to reset page file: {}", e))?;

    Ok(())
}
