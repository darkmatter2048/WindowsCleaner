mod clean;
mod winapp2;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::Disks;
use tauri::{Manager, Wry};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use winreg::enums::*;
use winreg::RegKey;

const APP_RUN_KEY: &str = "WindowsCleaner";

#[derive(Default)]
struct AppState {
    close_behavior: Mutex<CloseBehavior>,
    update_check_on_startup: Mutex<bool>,
    should_exit: Mutex<bool>,
}

#[derive(Serialize)]
pub struct DiskInfo {
    total: u64,
    used: u64,
    free: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum CloseBehavior {
    MinimizeToTray,
    Exit,
}

impl Default for CloseBehavior {
    fn default() -> Self {
        CloseBehavior::MinimizeToTray
    }
}

#[derive(Serialize)]
struct StartupSettings {
    autostart: bool,
    close_behavior: CloseBehavior,
    update_check_on_startup: bool,
}

fn current_exe_quoted() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(format!("\"{}\"", exe.display()))
}

fn read_autostart_enabled() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| e.to_string())?;
    Ok(run.get_value::<String, _>(APP_RUN_KEY).is_ok())
}

fn write_autostart_enabled(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| e.to_string())?;

    if enabled {
        let exe = current_exe_quoted()?;
        run.set_value(APP_RUN_KEY, &exe).map_err(|e| e.to_string())?;
    } else {
        let _ = run.delete_value(APP_RUN_KEY);
    }

    Ok(())
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

fn open_named_window(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    Err(format!("Window not found: {label}"))
}

#[tauri::command]
async fn open_custom_clean_window(app: tauri::AppHandle) -> Result<(), String> {
    open_named_window(&app, "custom-clean")
}

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    open_named_window(&app, "settings")
}

#[tauri::command]
fn get_startup_settings(state: tauri::State<AppState>) -> Result<StartupSettings, String> {
    let autostart = read_autostart_enabled()?;
    let close_behavior = *state.close_behavior.lock().map_err(|_| "Failed to read close behavior")?;
    let update_check_on_startup = *state
        .update_check_on_startup
        .lock()
        .map_err(|_| "Failed to read update setting")?;

    Ok(StartupSettings {
        autostart,
        close_behavior,
        update_check_on_startup,
    })
}

#[tauri::command]
fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    write_autostart_enabled(enabled)
}

#[tauri::command]
fn set_close_behavior(state: tauri::State<AppState>, behavior: CloseBehavior) -> Result<(), String> {
    let mut close_behavior = state.close_behavior.lock().map_err(|_| "Failed to update close behavior")?;
    *close_behavior = behavior;
    Ok(())
}

#[tauri::command]
fn set_update_check_on_startup(state: tauri::State<AppState>, enabled: bool) -> Result<(), String> {
    let mut update_setting = state
        .update_check_on_startup
        .lock()
        .map_err(|_| "Failed to update update check setting")?;
    *update_setting = enabled;
    Ok(())
}

#[tauri::command]
fn handle_main_close(app: tauri::AppHandle<Wry>, state: tauri::State<AppState>, behavior: CloseBehavior) -> Result<(), String> {
    // Sync Rust state with what the frontend persisted
    {
        let mut stored = state.close_behavior.lock().map_err(|_| "Failed to update close behavior")?;
        *stored = behavior;
    }
    match behavior {
        CloseBehavior::MinimizeToTray => {
            if let Some(window) = app.get_webview_window("main") {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
        CloseBehavior::Exit => {
            let mut should_exit = state.should_exit.lock().map_err(|_| "Failed to update exit flag")?;
            *should_exit = true;
            app.exit(0);
        }
    }
    Ok(())
}

fn build_tray(app: &tauri::AppHandle<Wry>) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("show", "打开主界面").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "设置").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show, &settings, &quit])
        .build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let _ = open_named_window(app, "main");
            }
            "settings" => {
                let _ = open_named_window(app, "settings");
            }
            "quit" => {
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(mut should_exit) = state.should_exit.lock() {
                        *should_exit = true;
                    }
                }
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            build_tray(&app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_disk_info,
            open_custom_clean_window,
            open_settings_window,
            get_startup_settings,
            set_autostart_enabled,
            set_close_behavior,
            set_update_check_on_startup,
            handle_main_close,
            clean::quick_clean,
            clean::deep_clean,
            clean::clean_selected,
            winapp2::clean_winapp2,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
