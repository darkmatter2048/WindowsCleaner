mod clean;
mod winapp2;

use serde::Serialize;
use sysinfo::Disks;
use tauri::Manager;

#[derive(Serialize)]
pub struct DiskInfo {
    total: u64,
    used: u64,
    free: u64,
}

#[tauri::command]
fn get_disk_info() -> Result<DiskInfo, String> {
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy();
        if mount.starts_with("C:") || mount.starts_with("c:") {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total.saturating_sub(free);
            return Ok(DiskInfo { total, used, free });
        }
    }
    Err("C: drive not found".into())
}

#[tauri::command]
async fn open_custom_clean_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("custom-clean") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        "custom-clean",
        tauri::WebviewUrl::App("index.html#/custom-clean".into()),
    )
    .title("自定义清理")
    .inner_size(760.0, 520.0)
    .min_inner_size(680.0, 460.0)
    .resizable(false)
    .decorations(false)
    .transparent(false)
    .center()
    .build()
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_disk_info,
            open_custom_clean_window,
            clean::quick_clean,
            clean::deep_clean,
            clean::clean_selected,
            winapp2::clean_winapp2,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
