mod advanced;
mod auto_clean;
mod clean;
mod logger;
mod software_move;
mod winapp2;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::Disks;
use tauri::{Manager, WindowEvent, Wry};
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
    hide_on_startup: Mutex<bool>,
}

// ---------------------------------------------------------------------------
// Hide-on-startup persistence (same pattern as auto_clean)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
struct StartupConfig {
    hide_on_startup: bool,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self { hide_on_startup: false }
    }
}

fn startup_config_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("startup_config.json")
}

fn load_startup_config(app: &tauri::AppHandle) -> StartupConfig {
    let path = startup_config_path(app);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_startup_config(app: &tauri::AppHandle, config: &StartupConfig) -> Result<(), String> {
    let path = startup_config_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct DiskInfo {
    total: u64,
    used: u64,
    free: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum CloseBehavior {
    MinimizeToTray,
    Exit,
}

impl Default for CloseBehavior {
    fn default() -> Self {
        CloseBehavior::Exit
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
        crate::logger::info("autostart", "开机自启 已启用", "");
    } else {
        let _ = run.delete_value(APP_RUN_KEY);
        crate::logger::info("autostart", "开机自启 已禁用", "");
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
    crate::logger::info("settings", &format!("关闭行为 → {:?}", behavior), "");
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
fn get_hide_on_startup(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(load_startup_config(&app).hide_on_startup)
}

#[tauri::command]
fn set_hide_on_startup(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut hide = state.hide_on_startup.lock().map_err(|_| "lock error")?;
    *hide = enabled;
    let config = StartupConfig { hide_on_startup: enabled };
    save_startup_config(&app, &config)
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

fn build_tray(
    app: &tauri::AppHandle<Wry>,
    icon: tauri::image::Image<'_>,
) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("show", "打开主界面").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "设置").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show, &settings, &quit])
        .build()?;

    let tooltip = format!("WindowsCleaner v{}", env!("CARGO_PKG_VERSION"));
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip(tooltip)
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
            logger::init(&app.handle());

            // Decode shared icon for tray + all windows
            let png = include_bytes!("../icons/logo.png");
            let img = image::load_from_memory(png).expect("Failed to decode icon");
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let raw = rgba.into_raw();
            let tray_icon = tauri::image::Image::new_owned(raw.clone(), w, h);
            let win_icon = tauri::image::Image::new_owned(raw, w, h);

            build_tray(&app.handle(), tray_icon)?;
            auto_clean::spawn_auto_clean_watcher(app.handle().clone());

            // Set taskbar icons for all windows
            for label in &["main", "custom-clean", "settings"] {
                if let Some(win) = app.get_webview_window(label) {
                    let _ = win.set_icon(win_icon.clone());
                }
            }

            // Apply hide-on-startup config: main is initially hidden,
            // show it only if hide_on_startup is disabled
            {
                let config = load_startup_config(&app.handle());
                if let Ok(mut hide) = app.state::<AppState>().hide_on_startup.lock() {
                    *hide = config.hide_on_startup;
                }
                if !config.hide_on_startup {
                    if let Some(main) = app.get_webview_window("main") {
                        let _ = main.show();
                    }
                }
            }

            // Intercept main window close (taskbar X, Alt+F4, etc.)
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                let handle = app.handle().clone();
                w.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let state = handle.state::<AppState>();
                        let behavior = *state.close_behavior.lock().unwrap();
                        match behavior {
                            CloseBehavior::MinimizeToTray => {
                                api.prevent_close();
                                let _ = if let Some(main) = handle.get_webview_window("main") {
                                    main.hide()
                                } else {
                                    Err(tauri::Error::WebviewNotFound)
                                };
                            }
                            CloseBehavior::Exit => {
                                handle.exit(0);
                            }
                        }
                    }
                });
            }

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
            get_hide_on_startup,
            set_hide_on_startup,
            handle_main_close,
            clean::quick_clean,
            clean::deep_clean,
            clean::clean_selected,
            winapp2::clean_winapp2,
            auto_clean::get_auto_clean_settings,
            auto_clean::save_auto_clean_settings,
            advanced::get_advanced_status,
            advanced::get_defender_status,
            advanced::set_defender_disabled,
            advanced::get_update_status,
            advanced::set_update_disabled,
            advanced::get_page_file_info,
            advanced::set_page_file,
            advanced::reset_page_file_to_system_managed,
            advanced::get_user_folders,
            advanced::scan_user_folder_sizes,
            advanced::move_user_folder,
            advanced::undo_user_folder,
            software_move::scan_software,
            software_move::get_available_drives,
            software_move::check_running_processes,
            software_move::kill_running_processes,
            software_move::move_software,
            software_move::undo_software_move,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
