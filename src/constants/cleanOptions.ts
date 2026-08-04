import type { CleanOptionConfig } from "../types/clean";

export const GENERAL_CLEAN_OPTIONS: CleanOptionConfig[] = [
  { id: "prefetch", defaultChecked: true },
  { id: "user_temp", defaultChecked: true },
  { id: "windows_temp", defaultChecked: true },
  { id: "system_logs", defaultChecked: true },
  { id: "software_distribution", defaultChecked: true },
  { id: "browser_cache", defaultChecked: true },
  { id: "temp_files_c", defaultChecked: true },
  { id: "log_files_c", defaultChecked: false },
  { id: "restore_points", defaultChecked: true, risky: true },
  { id: "hibernation", defaultChecked: true, risky: true },
  { id: "delivery_optimization", defaultChecked: true },
  { id: "thumbnails", defaultChecked: true },
  { id: "defender", defaultChecked: true },
  { id: "inet_cache", defaultChecked: true },
  { id: "wer_reports", defaultChecked: true },
  { id: "shader_cache", defaultChecked: true },
  { id: "nvidia_debug_logs", defaultChecked: true },
  { id: "wps_backups", defaultChecked: true },
  { id: "winapp2", defaultChecked: true },
  { id: "recycle_bin", defaultChecked: false },
  { id: "msp_files", defaultChecked: false, risky: true },
];

export const DEFAULT_GENERAL_CLEAN_OPTION_IDS = GENERAL_CLEAN_OPTIONS
  .filter((option) => option.defaultChecked)
  .map((option) => option.id);
