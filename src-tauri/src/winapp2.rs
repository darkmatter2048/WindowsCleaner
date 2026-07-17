use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::clean::CleanResult;

// ─── INI Parsing ───

#[allow(dead_code)]
struct Entry {
    #[allow(dead_code)]
    name: String,
    detects: Vec<String>,       // registry keys that must exist
    detect_files: Vec<String>,  // files/folders that must exist
    file_keys: Vec<String>,     // "path|pattern|options"
    reg_keys: Vec<(String, String)>, // (key, value_name)
}

/// Parse winapp2.ini into entries. Returns entries whose Detect conditions pass.
fn parse_and_detect(ini_content: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    let mut current: Option<Entry> = None;

    for line in ini_content.lines() {
        let trimmed = line.trim();

        // Skip comments and blanks
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }

        // Section header: [App Name *]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Save previous entry
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            let name = &trimmed[1..trimmed.len() - 1];
            current = Some(Entry {
                name: name.to_string(),
                detects: Vec::new(),
                detect_files: Vec::new(),
                file_keys: Vec::new(),
                reg_keys: Vec::new(),
            });
            continue;
        }

        // Key=Value within a section
        if let Some(ref mut entry) = current {
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                if key.starts_with("DetectFile") {
                    entry.detect_files.push(expand_env(value));
                } else if key.starts_with("Detect") && !key.starts_with("DetectOS") {
                    entry.detects.push(value.to_string());
                } else if key.starts_with("FileKey") {
                    entry.file_keys.push(value.to_string());
                } else if key.starts_with("RegKey") {
                    if let Some((k, v)) = value.split_once('|') {
                        entry.reg_keys.push((k.to_string(), v.to_string()));
                    } else {
                        entry.reg_keys.push((value.to_string(), String::new()));
                    }
                }
                // Ignore LangSecRef, Section, DetectOS, etc.
            }
        }
    }

    // Don't forget the last entry
    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    // Filter: keep only entries where detection passes
    entries.retain(|e| detect(e));

    entries
}

// ─── Environment variable expansion ───

fn expand_env(s: &str) -> String {
    let vars: HashMap<&str, &str> = [
        ("LocalAppData", "LOCALAPPDATA"),
        ("AppData", "APPDATA"),
        ("WinDir", "SystemRoot"),
        ("SystemDrive", "SystemDrive"),
        ("UserProfile", "USERPROFILE"),
        ("ProgramFiles", "ProgramFiles"),
        ("CommonAppData", "ProgramData"),
        ("LocalAppDataLow", "LOCALAPPDATA"),
        ("Documents", "USERPROFILE"),
    ]
    .into_iter()
    .collect();

    let mut result = s.to_string();
    for (placeholder, env_var) in &vars {
        let pattern = format!("%{}%", placeholder);
        if result.contains(&pattern) {
            let expanded = std::env::var(env_var).unwrap_or_default();
            result = result.replace(&pattern, &expanded);
        }
    }
    result
}

// ─── Detection ───

fn detect(entry: &Entry) -> bool {
    if entry.detects.is_empty() && entry.detect_files.is_empty() {
        return false; // no detection = skip (avoids false positives)
    }

    // Check registry detects
    for reg_key in &entry.detects {
        if !reg_exists(reg_key) {
            return false;
        }
    }

    // Check file/folder detects
    for file_path in &entry.detect_files {
        let p = Path::new(file_path);
        if !p.exists() {
            return false;
        }
    }

    true
}

fn reg_exists(key: &str) -> bool {
    let hkey = match key {
        k if k.starts_with("HKCU\\") => {
            (winreg::enums::HKEY_CURRENT_USER, &k[5..])
        }
        k if k.starts_with("HKLM\\") => {
            (winreg::enums::HKEY_LOCAL_MACHINE, &k[5..])
        }
        k if k.starts_with("HKU\\") => {
            // HKU detection is rare; skip for now
            return false;
        }
        _ => return false,
    };

    winreg::RegKey::predef(hkey.0)
        .open_subkey(hkey.1)
        .is_ok()
}

// ─── Execution ───

fn exec_file_key(raw: &str) -> CleanResult {
    let mut result = CleanResult::new();

    // Format: path|pattern|options
    let parts: Vec<&str> = raw.splitn(3, '|').collect();
    if parts.len() < 2 {
        return result;
    }

    let base = expand_env(parts[0]);
    let pattern = parts[1];
    let options = parts.get(2).unwrap_or(&"");

    let base_path = Path::new(&base);
    let recurse = options.contains("RECURSE");
    let remove_self = options.contains("REMOVESELF");

    // If pattern is "*" and RECURSE, clean entire folder
    if pattern == "*" && recurse {
        if base_path.exists() {
            return clean_dir_contents(base_path);
        }
    }

    // If pattern is "*" without RECURSE, clean files in this dir only
    if pattern == "*" {
        if base_path.exists() {
            return clean_dir_files_only(base_path);
        }
    }

    // Pattern-based cleanup
    clean_matching_files(base_path, pattern, recurse, &mut result);

    // REMOVESELF: delete the base directory itself if empty
    if remove_self && base_path.exists() {
        let _ = fs::remove_dir(base_path);
    }

    result
}

fn clean_dir_contents(dir: &Path) -> CleanResult {
    let mut result = CleanResult::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", dir.display(), e));
            return result;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            match fs::remove_dir_all(&path) {
                Ok(_) => {}
                Err(e) => {
                    result.add_error(format!("Cannot delete {}: {}", path.display(), e))
                }
            }
        } else {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            match fs::remove_file(&path) {
                Ok(_) => result.add_freed(size),
                Err(_) => {} // locked
            }
        }
    }
    result
}

fn clean_dir_files_only(dir: &Path) -> CleanResult {
    let mut result = CleanResult::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", dir.display(), e));
            return result;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            match fs::remove_file(&path) {
                Ok(_) => result.add_freed(size),
                Err(_) => {}
            }
        }
    }
    result
}

fn clean_matching_files(dir: &Path, pattern: &str, recurse: bool, result: &mut CleanResult) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && recurse {
            clean_matching_files(&path, pattern, recurse, result);
        } else if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if wildmatch(name, pattern) {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    match fs::remove_file(&path) {
                        Ok(_) => result.add_freed(size),
                        Err(_) => {}
                    }
                }
            }
        }
    }
}

/// Simple wildcard match: supports * and ?
fn wildmatch(name: &str, pattern: &str) -> bool {
    let name = name.to_lowercase();
    let pattern = pattern.to_lowercase();

    // Fast path: exact match or single *
    if pattern == "*" || pattern == "*.*" {
        return true;
    }

    // Multiple semicolon-separated patterns
    for pat in pattern.split(';') {
        let pat = pat.trim();
        if pat.is_empty() {
            continue;
        }
        if wildmatch_single(&name, pat) {
            return true;
        }
    }
    false
}

fn wildmatch_single(name: &str, pattern: &str) -> bool {
    let n: Vec<char> = name.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let mut dp = vec![vec![false; p.len() + 1]; n.len() + 1];
    dp[0][0] = true;

    for j in 1..=p.len() {
        if p[j - 1] == '*' {
            dp[0][j] = dp[0][j - 1];
        }
    }

    for i in 1..=n.len() {
        for j in 1..=p.len() {
            if p[j - 1] == '*' {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if p[j - 1] == '?' || p[j - 1] == n[i - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[n.len()][p.len()]
}

fn exec_reg_key(key: &str, value: &str) {
    let hkey = if key.starts_with("HKCU\\") {
        (winreg::enums::HKEY_CURRENT_USER, &key[5..])
    } else if key.starts_with("HKLM\\") {
        (winreg::enums::HKEY_LOCAL_MACHINE, &key[5..])
    } else {
        return;
    };

    if let Ok(k) = winreg::RegKey::predef(hkey.0).open_subkey_with_flags(
        hkey.1,
        winreg::enums::KEY_SET_VALUE | winreg::enums::KEY_QUERY_VALUE,
    ) {
        if value.is_empty() {
            // Delete the entire subkey
            let _ = winreg::RegKey::predef(hkey.0).delete_subkey_all(hkey.1);
        } else {
            let _ = k.delete_value(value);
        }
    }
}

// ─── Public API ───

const WINAPP2_INI: &str = include_str!("../resources/Winapp2.ini");

/// Scan installed software via winapp2.ini and clean their caches.
/// Only cleans entries whose Detect conditions match (software is actually installed).
#[tauri::command]
pub fn clean_winapp2() -> CleanResult {
    let mut result = CleanResult::new();

    let entries = parse_and_detect(WINAPP2_INI);

    for entry in entries {
        // Execute file cleanup rules
        for fk in &entry.file_keys {
            result.merge(exec_file_key(fk));
        }
        // Execute registry cleanup rules
        for (key, value) in &entry.reg_keys {
            exec_reg_key(key, value);
        }
    }

    result
}
