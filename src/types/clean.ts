export interface CleanResult {
  bytes_freed: number;
  errors: string[];
}

export type CleanOptionId =
  | "prefetch"
  | "user_temp"
  | "windows_temp"
  | "system_logs"
  | "software_distribution"
  | "browser_cache"
  | "temp_files_c"
  | "restore_points"
  | "hibernation"
  | "delivery_optimization"
  | "thumbnails"
  | "defender"
  | "inet_cache"
  | "wer_reports"
  | "shader_cache"
  | "nvidia_debug_logs"
  | "wps_backups"
  | "winapp2"
  | "msp_files"
  | "recycle_bin"
  | "log_files_c"
  | "memory_dumps"
  | "old_windows"
  | "node_modules"
  | "scattered_thumbs"
  | "bak_files"
  | "editor_temp";

export interface CleanOptionConfig {
  id: CleanOptionId;
  defaultChecked: boolean;
  risky?: boolean;
}

export interface CleanRequest {
  options: CleanOptionId[];
}
