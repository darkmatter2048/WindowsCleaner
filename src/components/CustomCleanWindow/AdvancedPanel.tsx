import { useEffect, useState } from "react";
import {
  makeStyles,
  tokens,
  Switch,
  Button,
  Text,
  Spinner,
} from "@fluentui/react-components";
import {
  ShieldDismiss24Regular,
  ArrowSync24Regular,
  Memory16Regular,
  FolderOpen24Regular,
  ArrowSwap24Regular,
  ArrowUndo24Regular,
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const useStyles = makeStyles({
  root: {
    height: "100%",
    display: "grid",
    gridTemplateRows: "auto 1fr",
    minHeight: 0,
  },
  header: {
    padding: `${tokens.spacingVerticalL} ${tokens.spacingHorizontalXL}`,
    borderBottom: `1px solid ${tokens.colorNeutralStroke1}`,
  },
  title: {
    display: "block",
    fontSize: "22px",
    fontWeight: 700,
    color: tokens.colorNeutralForeground1,
  },
  body: {
    overflowY: "auto",
    padding: tokens.spacingHorizontalXL,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalL,
  },
  section: {
    padding: tokens.spacingHorizontalL,
    borderRadius: tokens.borderRadiusLarge,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalM,
  },
  sectionHeader: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalS,
  },
  sectionIcon: {
    fontSize: "20px",
    flexShrink: 0,
  },
  sectionTitle: {
    fontWeight: 700,
    fontSize: "16px",
    color: tokens.colorNeutralForeground1,
  },
  sectionDesc: {
    fontSize: "13px",
    color: tokens.colorNeutralForeground3,
    lineHeight: "20px",
  },
  riskBox: {
    padding: tokens.spacingHorizontalM,
    borderRadius: tokens.borderRadiusMedium,
    backgroundColor: "#3d1f1f",
    borderLeft: `3px solid ${tokens.colorPaletteRedBorderActive}`,
  },
  riskText: {
    fontSize: "12px",
    color: tokens.colorPaletteRedForeground1,
    lineHeight: "18px",
  },
  infoBox: {
    padding: tokens.spacingHorizontalM,
    borderRadius: tokens.borderRadiusMedium,
    backgroundColor: "#1f2e3d",
    borderLeft: `3px solid ${tokens.colorBrandForeground1}`,
  },
  infoText: {
    fontSize: "12px",
    color: tokens.colorBrandForeground1,
    lineHeight: "18px",
  },
  row: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalM,
  },
  numberInput: {
    width: "100px",
    padding: "4px 8px",
    fontSize: "14px",
    borderRadius: tokens.borderRadiusMedium,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground1,
    color: tokens.colorNeutralForeground1,
    outline: "none",
  },
  statusBadge: {
    display: "inline-block",
    fontSize: "12px",
    padding: "2px 10px",
    borderRadius: "12px",
    fontWeight: 600,
  },
  statusOn: {
    backgroundColor: tokens.colorPaletteGreenBackground2,
    color: tokens.colorPaletteGreenForeground1,
  },
  statusOff: {
    backgroundColor: tokens.colorPaletteRedBackground2,
    color: tokens.colorPaletteRedForeground1,
  },
  vmInputGroup: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalS,
  },
  vmRow: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalM,
  },
  // User folder migration
  folderRow: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalM,
    padding: `${tokens.spacingVerticalS} ${tokens.spacingHorizontalM}`,
    borderRadius: tokens.borderRadiusMedium,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground1,
  },
  folderRowMoved: {
    border: "1px solid #4caf50",
    backgroundColor: "#1b3a1b",
  },
  folderToast: {
    padding: "10px 16px",
    borderRadius: "24px",
    fontSize: "13px",
    fontWeight: 600,
    animation: "slideDown 0.3s ease",
  },
  folderToastSuccess: {
    backgroundColor: "#1b3a1b",
    color: "#6fcf6f",
    border: "1px solid #4caf50",
  },
  folderToastError: {
    backgroundColor: "#3d1414",
    color: "#f57c7c",
    border: "1px solid #d32f2f",
  },
  progressCell: {
    display: "flex",
    alignItems: "center",
    gap: "4px",
  },
});

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface AdvancedStatus {
  defenderDisabled: boolean;
  updateDisabled: boolean;
  pageFile: {
    initialMb: number;
    maxMb: number;
    systemManaged: boolean;
  };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function AdvancedPanel() {
  const styles = useStyles();
  const { t } = useTranslation();

  const [loading, setLoading] = useState(true);
  const [defenderDisabled, setDefenderDisabled] = useState(false);
  const [updateDisabled, setUpdateDisabled] = useState(false);
  const [pfSystemManaged, setPfSystemManaged] = useState(true);
  const [pfInitial, setPfInitial] = useState("2048");
  const [pfMax, setPfMax] = useState("4096");
  const [pageFile, setPageFile] = useState({
    initialMb: 0,
    maxMb: 0,
    systemManaged: true,
  });

  // ---- User Folder Migration ----
  interface UserFolder {
    key: string;
    displayName: string;
    currentPath: string;
    drive: string;
    sizeBytes: number;
    onCDrive: boolean;
  }
  const [userFolders, setUserFolders] = useState<UserFolder[]>([]);
  const [folderSizesLoading, setFolderSizesLoading] = useState(false);
  const [folderMoving, setFolderMoving] = useState<string | null>(null);
  const [folderProgress, setFolderProgress] = useState(0);
  const [folderOriginalPaths, setFolderOriginalPaths] = useState<Record<string, string>>({});
  const [folderNotification, setFolderNotification] = useState<{
    type: "success" | "error"; message: string;
  } | null>(null);

  useEffect(() => {
    // Load folder list first (fast), then scan sizes (slow)
    invoke<UserFolder[]>("get_user_folders")
      .then((f) => setUserFolders(f))
      .catch(() => {});
    setFolderSizesLoading(true);
    invoke<UserFolder[]>("scan_user_folder_sizes")
      .then((f) => { setUserFolders(f); setFolderSizesLoading(false); })
      .catch(() => setFolderSizesLoading(false));
  }, []);

  // Listen for folder move progress
  useEffect(() => {
    const unlisten = listen<{ key: string; percent: number; status: string }>(
      "folder-move-progress",
      (event) => {
        if (event.payload.status === "done") {
          setFolderMoving(null);
          setFolderProgress(0);
          // Refresh folder info
          invoke<UserFolder[]>("scan_user_folder_sizes")
            .then((f) => setUserFolders(f))
            .catch(() => {});
        } else {
          setFolderProgress(event.payload.percent);
        }
      },
    );
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Defer invoke so it doesn't overlap with React mount / paint cycle
  useEffect(() => {
    let mounted = true;
    const timer = setTimeout(() => {
      if (!mounted) return;
      invoke<AdvancedStatus>("get_advanced_status")
        .then((s) => {
          if (!mounted) return;
          setDefenderDisabled(s.defenderDisabled);
          setUpdateDisabled(s.updateDisabled);
          setPageFile(s.pageFile);
          setPfSystemManaged(s.pageFile.systemManaged);
          if (!s.pageFile.systemManaged && s.pageFile.initialMb > 0) {
            setPfInitial(String(s.pageFile.initialMb));
            setPfMax(String(s.pageFile.maxMb));
          }
        })
        .catch(() => {})
        .finally(() => { if (mounted) setLoading(false); });
    }, 150);
    return () => { mounted = false; clearTimeout(timer); };
  }, []);

  // -------- Defender --------
  const toggleDefender = async (disable: boolean) => {
    setDefenderDisabled(disable);
    const ok = await invoke("set_defender_disabled", { disabled: disable })
      .then(() => true)
      .catch(() => false);
    if (!ok) {
      setDefenderDisabled(!disable);
      return;
    }
    // Re-read to confirm
    const actual = await invoke<boolean>("get_defender_status").catch(() => !disable);
    if (actual !== disable) setDefenderDisabled(actual);
  };

  // -------- Update --------
  const toggleUpdate = async (disable: boolean) => {
    setUpdateDisabled(disable);
    const ok = await invoke("set_update_disabled", { disabled: disable })
      .then(() => true)
      .catch(() => false);
    if (!ok) {
      setUpdateDisabled(!disable);
      return;
    }
    const actual = await invoke<{ disabled: boolean }>("get_update_status").catch(() => null);
    if (actual && actual.disabled !== disable) setUpdateDisabled(actual.disabled);
  };

  // -------- Page File --------
  const applyPageFile = async () => {
    const init = parseInt(pfInitial, 10);
    const max = parseInt(pfMax, 10);
    if (isNaN(init) || isNaN(max) || init < 0 || max < init) return;

    await invoke("set_page_file", { initialMb: init, maxMb: max }).catch(() => {});
    setPageFile({ initialMb: init, maxMb: max, systemManaged: false });
    setPfSystemManaged(false);
  };

  // ---- User Folder Move ----
  const moveFolder = async (folder: UserFolder) => {
    setFolderMoving(folder.key);
    setFolderProgress(0);
    setFolderNotification(null);
    // Save original path for undo
    setFolderOriginalPaths((prev) => ({ ...prev, [folder.key]: folder.currentPath }));
    try {
      await invoke("move_user_folder", {
        folderKey: folder.key,
        targetDrive: "D",
      });
      setFolderNotification({
        type: "success",
        message: `${folder.displayName} 已迁移到 D: 盘`,
      });
      setTimeout(() => setFolderNotification(null), 4000);
    } catch (e) {
      setFolderNotification({ type: "error", message: String(e) });
      setTimeout(() => setFolderNotification(null), 8000);
    }
    setFolderMoving(null);
  };

  const undoFolder = async (folder: UserFolder) => {
    // Prefer stored original path; fall back to default C: location
    const originalPath =
      folderOriginalPaths[folder.key] ||
      folder.currentPath.replace(/^[A-Z]:/, "C:");
    setFolderMoving(folder.key);
    setFolderProgress(0);
    setFolderNotification(null);
    try {
      await invoke("undo_user_folder", {
        folderKey: folder.key,
        originalPath,
      });
      setFolderNotification({
        type: "success",
        message: `${folder.displayName} 已还原到 C 盘`,
      });
      setTimeout(() => setFolderNotification(null), 4000);
    } catch (e) {
      setFolderNotification({ type: "error", message: String(e) });
      setTimeout(() => setFolderNotification(null), 8000);
    }
    setFolderMoving(null);
  };

  const resetPageFile = async () => {
    await invoke("reset_page_file_to_system_managed").catch(() => {});
    setPageFile({ initialMb: 0, maxMb: 0, systemManaged: true });
    setPfSystemManaged(true);
    setPfInitial("2048");
    setPfMax("4096");
  };

  return (
    <section className={styles.root}>
      <header className={styles.header}>
        <Text className={styles.title}>{t("advanced.title")}</Text>
      </header>

      <div className={styles.body}>
        {/* ================================================================ */}
        {/* Windows Defender                                                 */}
        {/* ================================================================ */}
        <div className={styles.section}>
          <div className={styles.sectionHeader}>
            <ShieldDismiss24Regular className={styles.sectionIcon} />
            <Text className={styles.sectionTitle}>{t("advanced.defender.title")}</Text>
          </div>
          <Text className={styles.sectionDesc}>{t("advanced.defender.desc")}</Text>

          {loading ? (
            <Spinner size="tiny" />
          ) : (
            <span
              className={`${styles.statusBadge} ${defenderDisabled ? styles.statusOff : styles.statusOn}`}
            >
              {defenderDisabled
                ? t("advanced.defender.disabled")
                : t("advanced.defender.enabled")}
            </span>
          )}

          <div className={styles.riskBox}>
            <Text className={styles.riskText}>{t("advanced.defender.risk")}</Text>
          </div>

          <div className={styles.row}>
            <Switch
              checked={defenderDisabled}
              onChange={(_, data) => toggleDefender(Boolean(data.checked))}
              label={defenderDisabled
                ? t("advanced.defender.turnOff")
                : t("advanced.defender.turnOn")}
            />
          </div>
        </div>

        {/* ================================================================ */}
        {/* Windows Update                                                   */}
        {/* ================================================================ */}
        <div className={styles.section}>
          <div className={styles.sectionHeader}>
            <ArrowSync24Regular className={styles.sectionIcon} />
            <Text className={styles.sectionTitle}>{t("advanced.update.title")}</Text>
          </div>
          <Text className={styles.sectionDesc}>{t("advanced.update.desc")}</Text>

          {loading ? (
            <Spinner size="tiny" />
          ) : (
            <span
              className={`${styles.statusBadge} ${updateDisabled ? styles.statusOff : styles.statusOn}`}
            >
              {updateDisabled
                ? t("advanced.update.disabled")
                : t("advanced.update.enabled")}
            </span>
          )}

          <div className={styles.riskBox}>
            <Text className={styles.riskText}>{t("advanced.update.risk")}</Text>
          </div>

          <div className={styles.row}>
            <Switch
              checked={updateDisabled}
              onChange={(_, data) => toggleUpdate(Boolean(data.checked))}
              label={updateDisabled
                ? t("advanced.update.turnOff")
                : t("advanced.update.turnOn")}
            />
          </div>
        </div>

        {/* ================================================================ */}
        {/* Virtual Memory (Page File)                                       */}
        {/* ================================================================ */}
        <div className={styles.section}>
          <div className={styles.sectionHeader}>
            <Memory16Regular className={styles.sectionIcon} />
            <Text className={styles.sectionTitle}>{t("advanced.virtualMemory.title")}</Text>
          </div>
          <Text className={styles.sectionDesc}>{t("advanced.virtualMemory.desc")}</Text>

          {loading ? (
            <Spinner size="tiny" />
          ) : (
            <span
              className={`${styles.statusBadge} ${pfSystemManaged ? styles.statusOn : styles.statusOff}`}
            >
              {pfSystemManaged
                ? t("advanced.virtualMemory.systemManaged")
                : `${pageFile.initialMb} MB – ${pageFile.maxMb} MB`}
            </span>
          )}

          <div className={styles.infoBox}>
            <Text className={styles.infoText}>{t("advanced.virtualMemory.restartHint")}</Text>
          </div>

          <div className={styles.vmInputGroup}>
            <Switch
              checked={pfSystemManaged}
              onChange={(_, data) => {
                const v = Boolean(data.checked);
                setPfSystemManaged(v);
                if (v) resetPageFile();
              }}
              label={t("advanced.virtualMemory.systemManagedLabel")}
            />

            {!pfSystemManaged && (
              <>
                <div className={styles.vmRow}>
                  <Text>{t("advanced.virtualMemory.initial")}</Text>
                  <input
                    className={styles.numberInput}
                    type="number"
                    min={256}
                    value={pfInitial}
                    onChange={(e) => setPfInitial(e.target.value)}
                  />
                  <Text>MB</Text>
                </div>
                <div className={styles.vmRow}>
                  <Text>{t("advanced.virtualMemory.maximum")}</Text>
                  <input
                    className={styles.numberInput}
                    type="number"
                    min={256}
                    value={pfMax}
                    onChange={(e) => setPfMax(e.target.value)}
                  />
                  <Text>MB</Text>
                </div>
                <div className={styles.row}>
                  <Button appearance="primary" onClick={applyPageFile}>
                    {t("advanced.virtualMemory.apply")}
                  </Button>
                </div>
              </>
            )}
          </div>
        </div>

        {/* ================================================================ */}
        {/* User Folder Migration                                            */}
        {/* ================================================================ */}
        <div className={styles.section}>
          <div className={styles.sectionHeader}>
            <FolderOpen24Regular className={styles.sectionIcon} />
            <Text className={styles.sectionTitle}>{t("advanced.folders.title")}</Text>
          </div>
          <Text className={styles.sectionDesc}>{t("advanced.folders.desc")}</Text>

          <div className={styles.riskBox}>
            <Text className={styles.riskText}>{t("advanced.folders.risk")}</Text>
          </div>

          {/* Notification */}
          {folderNotification && (
            <div
              className={`${styles.folderToast} ${
                folderNotification.type === "success"
                  ? styles.folderToastSuccess
                  : styles.folderToastError
              }`}
            >
              <Text style={{ fontSize: "13px", fontWeight: 600 }}>
                {folderNotification.message}
              </Text>
            </div>
          )}

          {/* Movable folders (on C:) */}
          {userFolders.filter((f) => f.onCDrive).map((f) => {
            const isMoving = folderMoving === f.key;
            const sizeStr = f.sizeBytes >= 1024 * 1024 * 1024
              ? `${(f.sizeBytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
              : f.sizeBytes >= 1024 * 1024
                ? `${(f.sizeBytes / (1024 * 1024)).toFixed(0)} MB`
                : `${(f.sizeBytes / 1024).toFixed(0)} KB`;
            return (
              <div key={f.key} className={styles.folderRow}>
                <div style={{ display: "flex", flexDirection: "column", gap: "2px", flex: 1 }}>
                  <Text style={{ fontWeight: 600, fontSize: "14px" }}>{f.displayName}</Text>
                  <Text style={{ fontSize: "11px", color: "var(--colorNeutralForeground3)" }}>
                    {f.currentPath}
                  </Text>
                  <Text style={{ fontSize: "12px", color: "var(--colorNeutralForeground2)" }}>
                    {folderSizesLoading ? "Calculating..." : sizeStr}
                  </Text>
                </div>
                {isMoving ? (
                  <div className={styles.progressCell}>
                    <Text style={{ fontSize: "11px" }}>{folderProgress}%</Text>
                  </div>
                ) : (
                  <Button
                    size="small"
                    appearance="primary"
                    icon={<ArrowSwap24Regular />}
                    disabled={folderMoving !== null}
                    onClick={() => moveFolder(f)}
                  >
                    {t("advanced.folders.move")}
                  </Button>
                )}
              </div>
            );
          })}

          {/* Moved folders (not on C:) */}
          {userFolders.filter((f) => !f.onCDrive).map((f) => {
            const isMoving = folderMoving === f.key;
            return (
              <div key={f.key} className={`${styles.folderRow} ${styles.folderRowMoved}`}>
                <div style={{ display: "flex", flexDirection: "column", gap: "2px", flex: 1 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <Text style={{ fontWeight: 600, fontSize: "14px" }}>{f.displayName}</Text>
                    <span style={{
                      display: "inline-block",
                      fontSize: "11px",
                      padding: "0 6px",
                      borderRadius: "8px",
                      fontWeight: 600,
                      backgroundColor: "var(--colorPaletteGreenBackground2)",
                      color: "var(--colorPaletteGreenForeground1)",
                    }}>
                      {f.drive}:
                    </span>
                  </div>
                  <Text style={{ fontSize: "11px", color: "var(--colorNeutralForeground3)" }}>
                    {f.currentPath}
                  </Text>
                </div>
                {isMoving ? (
                  <div className={styles.progressCell}>
                    <Text style={{ fontSize: "11px" }}>{folderProgress}%</Text>
                  </div>
                ) : (
                  <Button
                    size="small"
                    appearance="outline"
                    icon={<ArrowUndo24Regular />}
                    disabled={folderMoving !== null}
                    onClick={() => undoFolder(f)}
                  >
                    {t("advanced.folders.undo")}
                  </Button>
                )}
              </div>
            );
          })}

          {userFolders.length === 0 && !folderSizesLoading && (
            <Text style={{ fontSize: "13px", color: "var(--colorNeutralForeground3)" }}>
              {t("advanced.folders.noneOnC")}
            </Text>
          )}

          {folderSizesLoading && (
            <Spinner size="tiny" label={t("advanced.folders.scanning")} />
          )}
        </div>
      </div>
    </section>
  );
}
