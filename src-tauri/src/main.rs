// Hide console window on Windows in all builds
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn ensure_admin() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    extern "system" {
        fn IsUserAnAdmin() -> i32;
        fn ShellExecuteW(
            hwnd: isize,
            lpOperation: *const u16,
            lpFile: *const u16,
            lpParameters: *const u16,
            lpDirectory: *const u16,
            nShowCmd: i32,
        ) -> isize;
    }

    unsafe {
        if IsUserAnAdmin() != 0 {
            return; // already elevated
        }

        let exe = std::env::current_exe().unwrap_or_default();
        let exe_wide: Vec<u16> = exe
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let op: Vec<u16> = OsStr::new("runas")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let ret = ShellExecuteW(
            0,
            op.as_ptr(),
            exe_wide.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1, // SW_NORMAL
        );

        // ret > 32 means success, <= 32 means error
        if ret > 32 {
            std::process::exit(0);
        }
        // If ShellExecute fails (e.g. user clicked "No" on UAC), continue without admin
    }
}

#[cfg(not(target_os = "windows"))]
fn ensure_admin() {}

fn main() {
    ensure_admin();
    wc_lib::run()
}
