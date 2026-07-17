use serde::Serialize;
use sysinfo::Disks;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_disk_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
