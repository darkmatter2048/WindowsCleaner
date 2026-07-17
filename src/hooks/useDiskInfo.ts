import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DiskInfo } from "../types/disk";

interface UseDiskInfoResult {
  data: DiskInfo | null;
  loading: boolean;
  error: string | null;
}

export function useDiskInfo(): UseDiskInfoResult {
  const [data, setData] = useState<DiskInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<DiskInfo>("get_disk_info")
      .then((result) => {
        setData(result);
        setLoading(false);
      })
      .catch((err) => {
        setError(String(err));
        setLoading(false);
      });
  }, []);

  return { data, loading, error };
}
