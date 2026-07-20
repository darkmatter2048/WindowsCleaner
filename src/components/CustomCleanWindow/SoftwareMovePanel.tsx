import { useCallback, useEffect, useState } from "react";
import {
  makeStyles,
  tokens,
  Button,
  Text,
  Spinner,
  Dropdown,
  Option,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  ProgressBar,
} from "@fluentui/react-components";
import {
  ArrowSwap24Regular,
  ArrowUndo24Regular,
  CheckmarkCircle24Regular,
  Folder24Regular,
  Warning24Regular,
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
  desc: {
    display: "block",
    fontSize: "13px",
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalXS,
  },
  body: {
    overflowY: "auto",
    padding: tokens.spacingHorizontalXL,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalL,
  },
  // Risk warning
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
  // Drive selector row
  driveRow: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalM,
  },
  driveLabel: {
    fontWeight: 600,
    fontSize: "14px",
    whiteSpace: "nowrap",
  },
  // App list table
  table: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalS,
  },
  tableHeader: {
    display: "grid",
    gridTemplateColumns: "1fr 140px 100px",
    gap: tokens.spacingHorizontalS,
    padding: `0 ${tokens.spacingHorizontalM}`,
    fontSize: "12px",
    fontWeight: 600,
    color: tokens.colorNeutralForeground3,
    textTransform: "uppercase" as const,
  },
  appRow: {
    display: "grid",
    gridTemplateColumns: "1fr 140px 100px",
    gap: tokens.spacingHorizontalS,
    alignItems: "center",
    padding: `${tokens.spacingVerticalS} ${tokens.spacingHorizontalM}`,
    borderRadius: tokens.borderRadiusMedium,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
  },
  movedRow: {
    border: "1px solid #4caf50",
    backgroundColor: "#1b3a1b",
  },
  appInfo: {
    display: "flex",
    flexDirection: "column",
    gap: "2px",
    minWidth: 0,
  },
  appName: {
    fontSize: "14px",
    fontWeight: 600,
    color: tokens.colorNeutralForeground1,
  },
  appPath: {
    fontSize: "11px",
    color: tokens.colorNeutralForeground3,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap" as const,
  },
  sizeText: {
    fontSize: "13px",
    fontWeight: 500,
    color: tokens.colorNeutralForeground2,
    whiteSpace: "nowrap" as const,
  },
  fileCount: {
    fontSize: "11px",
    color: tokens.colorNeutralForeground3,
  },
  movedTag: {
    display: "inline-flex",
    alignItems: "center",
    gap: "4px",
    fontSize: "12px",
    fontWeight: 600,
    color: tokens.colorPaletteGreenForeground1,
  },
  // Progress inside row
  progressCell: {
    display: "flex",
    flexDirection: "column",
    gap: "2px",
  },
  empty: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: tokens.spacingVerticalM,
    padding: "48px 0",
    color: tokens.colorNeutralForeground3,
  },
  // Toast bubble — floating at top center
  toastBase: {
    position: "fixed",
    top: "56px",
    left: "50%",
    transform: "translateX(-50%)",
    maxWidth: "520px",
    padding: "12px 20px",
    borderRadius: "24px",
    fontSize: "14px",
    fontWeight: 600,
    display: "flex",
    alignItems: "center",
    gap: "8px",
    zIndex: 1000,
    boxShadow: "0 4px 16px rgba(0,0,0,0.4)",
    animation: "slideDown 0.3s ease",
  },
  toastSuccess: {
    backgroundColor: "#1b3a1b",
    color: "#6fcf6f",
    border: "1px solid #4caf50",
  },
  toastError: {
    backgroundColor: "#3d1414",
    color: "#f57c7c",
    border: "1px solid #d32f2f",
  },
});

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface SoftwareInfo {
  key: string;
  displayPath: string;
  sizeBytes: number;
  fileCount: number;
  exists: boolean;
  isMoved: boolean;
  movedTo: string | null;
}

interface DriveInfo {
  letter: string;
  freeBytes: number;
  totalBytes: number;
}

interface MoveProgress {
  key: string;
  percent: number;
  status: string;
  message?: string;
}

interface ProcessInfo {
  name: string;
  pid: number;
}

interface MoveResult {
  skippedFiles: string[];
  junctionCreated: boolean;
}

type MoveState =
  | { phase: "idle" }
  | { phase: "confirming"; app: SoftwareInfo; checking: boolean; runningProcs: ProcessInfo[] }
  | { phase: "moving"; key: string; percent: number }
  | { phase: "undoing"; key: string; percent: number };

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  }
  return `${(bytes / 1024).toFixed(0)} KB`;
}

function formatDriveInfo(d: DriveInfo): string {
  return `${d.letter}: (${formatBytes(d.freeBytes)} free)`;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function SoftwareMovePanel() {
  const styles = useStyles();
  const { t } = useTranslation();

  const [scanning, setScanning] = useState(true);
  const [apps, setApps] = useState<SoftwareInfo[]>([]);
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [targetDrive, setTargetDrive] = useState("");
  const [moveState, setMoveState] = useState<MoveState>({ phase: "idle" });
  const [notification, setNotification] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);

  // Map app key to translated name
  const appName = useCallback(
    (key: string) => t(`softwareMove.apps.${key}`, key),
    [t],
  );

  // -------------------- Initial scan --------------------
  useEffect(() => {
    let mounted = true;

    const run = async () => {
      const [software, driveList] = await Promise.all([
        invoke<SoftwareInfo[]>("scan_software").catch(() => [] as SoftwareInfo[]),
        invoke<DriveInfo[]>("get_available_drives").catch(() => [] as DriveInfo[]),
      ]);
      if (!mounted) return;
      // Only show apps that exist and are not already moved (we'll show those separately)
      const present = software.filter((s) => s.exists);
      setApps(present);
      setDrives(driveList);
      if (driveList.length > 0) {
        // Default to the drive with most free space
        const best = driveList.reduce((a, b) => (a.freeBytes > b.freeBytes ? a : b));
        setTargetDrive(best.letter);
      }
      setScanning(false);
    };

    // Defer slightly so React paint cycle isn't blocked
    const timer = setTimeout(run, 100);

    // Listen for move progress
    const unlisten = listen<MoveProgress>("software-move-progress", (event) => {
      const { key, percent, status } = event.payload;
      setMoveState((prev) => {
        if (status === "done") {
          return { phase: "idle" };
        }
        if (prev.phase === "moving" && prev.key === key) {
          return { phase: "moving", key, percent };
        }
        if (prev.phase === "undoing" && prev.key === key) {
          return { phase: "undoing", key, percent };
        }
        return prev;
      });
    });

    return () => {
      mounted = false;
      clearTimeout(timer);
      unlisten.then((fn) => fn());
    };
  }, []);

  // -------------------- Check processes & move --------------------
  const executeMove = async (app: SoftwareInfo) => {
    if (!targetDrive) return;
    setMoveState({ phase: "moving", key: app.key, percent: 0 });
    setNotification(null);

    try {
      await invoke<MoveResult>("move_software", {
        key: app.key,
        sourcePathTemplate: app.displayPath,
        targetDrive,
      });
      setApps((prev) =>
        prev.map((a) =>
          a.key === app.key
            ? { ...a, isMoved: true, movedTo: `${targetDrive}:\\MovedApps\\${app.key}` }
            : a,
        ),
      );
      setNotification({
        type: "success",
        message: t("softwareMove.moveSuccess", { name: appName(app.key) }),
      });
      setTimeout(() => setNotification(null), 4000);
    } catch (e) {
      setNotification({ type: "error", message: String(e) });
      setTimeout(() => setNotification(null), 8000);
    }
    setMoveState({ phase: "idle" });
  };

  const confirmAndCheckProcesses = async (app: SoftwareInfo) => {
    // Show checking state
    setMoveState({ phase: "confirming", app, checking: true, runningProcs: [] });

    const procs = await invoke<ProcessInfo[]>("check_running_processes", {
      key: app.key,
    }).catch(() => [] as ProcessInfo[]);

    if (procs.length === 0) {
      // No running processes — proceed directly
      executeMove(app);
    } else {
      // Show dialog with running processes
      setMoveState({ phase: "confirming", app, checking: false, runningProcs: procs });
    }
  };

  const forceCloseAndMove = async () => {
    if (moveState.phase !== "confirming") return;
    const app = moveState.app;

    // Kill processes
    await invoke("kill_running_processes", { key: app.key }).catch(() => {});

    // Proceed with move
    executeMove(app);
  };

  // -------------------- Undo --------------------
  const startUndo = async (app: SoftwareInfo) => {
    setMoveState({ phase: "undoing", key: app.key, percent: 0 });
    setNotification(null);

    try {
      await invoke("undo_software_move", {
        key: app.key,
        sourcePathTemplate: app.displayPath,
      });
      setApps((prev) =>
        prev.map((a) =>
          a.key === app.key ? { ...a, isMoved: false, movedTo: null } : a,
        ),
      );
      setNotification({
        type: "success",
        message: t("softwareMove.undoSuccess", { name: appName(app.key) }),
      });
      setTimeout(() => setNotification(null), 4000);
    } catch (e) {
      setNotification({ type: "error", message: String(e) });
      setTimeout(() => setNotification(null), 8000);
    }
    setMoveState({ phase: "idle" });
  };

  // -------------------- Render --------------------

  const movedApps = apps.filter((a) => a.isMoved);
  const pendingApps = apps.filter((a) => !a.isMoved);

  return (
    <section className={styles.root}>
      <header className={styles.header}>
        <Text className={styles.title}>{t("softwareMove.title")}</Text>
        <Text className={styles.desc}>{t("softwareMove.desc")}</Text>
      </header>

      <div className={styles.body}>
        {/* Risk warning */}
        <div className={styles.riskBox}>
          <Text className={styles.riskText}>{t("softwareMove.riskWarning")}</Text>
        </div>

        {/* Notification banner */}
        {notification && (
          <div
            className={`${styles.toastBase} ${
              notification.type === "success" ? styles.toastSuccess : styles.toastError
            }`}
          >
            {notification.type === "success" ? (
              <CheckmarkCircle24Regular
                style={{ fontSize: "18px", flexShrink: 0 }}
              />
            ) : (
              <Warning24Regular
                style={{ fontSize: "18px", flexShrink: 0 }}
              />
            )}
            <Text style={{ fontSize: "14px", fontWeight: 600 }}>
              {notification.message}
            </Text>
          </div>
        )}

        {/* Drive selector */}
        {drives.length > 0 && (
          <div className={styles.driveRow}>
            <Text className={styles.driveLabel}>{t("softwareMove.targetDrive")}</Text>
            <Dropdown
              value={targetDrive}
              selectedOptions={[targetDrive]}
              onOptionSelect={(_, data) => setTargetDrive(data.optionValue ?? targetDrive)}
            >
              {drives.map((d) => (
                <Option key={d.letter} value={d.letter}>
                  {formatDriveInfo(d)}
                </Option>
              ))}
            </Dropdown>
          </div>
        )}

        {/* Loading */}
        {scanning && (
          <div className={styles.empty}>
            <Spinner size="medium" />
            <Text>{t("softwareMove.scanning")}</Text>
          </div>
        )}

        {/* Empty state */}
        {!scanning && apps.length === 0 && (
          <div className={styles.empty}>
            <Folder24Regular style={{ fontSize: "48px" }} />
            <Text>{t("softwareMove.empty")}</Text>
          </div>
        )}

        {/* Pending (not moved) apps */}
        {!scanning && pendingApps.length > 0 && (
          <>
            <div className={styles.tableHeader}>
              <Text>{t("softwareMove.appName")}</Text>
              <Text>{t("softwareMove.size")}</Text>
              <Text style={{ textAlign: "right" }}>{t("softwareMove.action")}</Text>
            </div>
            <div className={styles.table}>
              {pendingApps.map((app) => {
                const isMoving =
                  moveState.phase === "moving" && moveState.key === app.key;
                return (
                  <div key={app.key} className={styles.appRow}>
                    <div className={styles.appInfo}>
                      <Text className={styles.appName}>{appName(app.key)}</Text>
                      <Text className={styles.appPath}>{app.displayPath}</Text>
                      {app.fileCount > 0 && (
                        <Text className={styles.fileCount}>
                          {app.fileCount.toLocaleString()} {t("softwareMove.files")}
                        </Text>
                      )}
                    </div>
                    <div>
                      <Text className={styles.sizeText}>{formatBytes(app.sizeBytes)}</Text>
                    </div>
                    <div>
                      {isMoving ? (
                        <div className={styles.progressCell}>
                          <ProgressBar value={moveState.percent / 100} />
                          <Text style={{ fontSize: "11px" }}>{moveState.percent}%</Text>
                        </div>
                      ) : (
                        <Button
                          size="small"
                          appearance="primary"
                          icon={<ArrowSwap24Regular />}
                          disabled={!targetDrive || moveState.phase !== "idle"}
                          onClick={() =>
                            confirmAndCheckProcesses(app)
                          }
                        >
                          {t("softwareMove.move")}
                        </Button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </>
        )}

        {/* Moved apps */}
        {!scanning && movedApps.length > 0 && (
          <>
            <Text style={{ fontWeight: 600, fontSize: "14px", marginTop: "8px" }}>
              {t("softwareMove.movedSection")}
            </Text>
            <div className={styles.table}>
              {movedApps.map((app) => {
                const isUndoing =
                  moveState.phase === "undoing" && moveState.key === app.key;
                return (
                  <div key={app.key} className={`${styles.appRow} ${styles.movedRow}`}>
                    <div className={styles.appInfo}>
                      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                        <Text className={styles.appName}>{appName(app.key)}</Text>
                        <span className={styles.movedTag}>
                          <CheckmarkCircle24Regular style={{ fontSize: "14px" }} />
                          {t("softwareMove.movedTag")}
                        </span>
                      </div>
                      <Text className={styles.appPath}>
                        {app.movedTo || app.displayPath}
                      </Text>
                    </div>
                    <Text className={styles.sizeText}>{formatBytes(app.sizeBytes)}</Text>
                    <div>
                      {isUndoing ? (
                        <div className={styles.progressCell}>
                          <ProgressBar value={moveState.percent / 100} />
                          <Text style={{ fontSize: "11px" }}>{moveState.percent}%</Text>
                        </div>
                      ) : (
                        <Button
                          size="small"
                          appearance="outline"
                          icon={<ArrowUndo24Regular />}
                          disabled={moveState.phase !== "idle"}
                          onClick={() => startUndo(app)}
                        >
                          {t("softwareMove.undo")}
                        </Button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </>
        )}

        {/* Confirm dialog — step 1: check processes, step 2: force-close warning */}
        <Dialog
          open={moveState.phase === "confirming"}
          onOpenChange={() => setMoveState({ phase: "idle" })}
        >
          <DialogSurface>
            {moveState.phase === "confirming" &&
              moveState.checking &&
              moveState.runningProcs.length === 0 && (
                <>
                  <DialogTitle>{t("softwareMove.confirmTitle")}</DialogTitle>
                  <DialogContent>
                    <DialogBody>
                      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                        <Spinner size="tiny" />
                        <Text>{t("softwareMove.checkingProcesses")}</Text>
                      </div>
                    </DialogBody>
                  </DialogContent>
                </>
              )}

            {moveState.phase === "confirming" &&
              !moveState.checking &&
              moveState.runningProcs.length > 0 && (
                <>
                  <DialogTitle>
                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                      <Warning24Regular style={{ color: "var(--colorPaletteRedForeground1)" }} />
                      {t("softwareMove.processesRunningTitle")}
                    </div>
                  </DialogTitle>
                  <DialogContent>
                    <DialogBody>
                      <Text>
                        {t("softwareMove.processesRunningContent", {
                          name: appName(moveState.app.key),
                        })}
                      </Text>
                      <div
                        style={{
                          marginTop: "12px",
                          padding: "8px 12px",
                          borderRadius: "4px",
                          backgroundColor: "var(--colorNeutralBackground2)",
                          fontSize: "13px",
                          fontFamily: "monospace",
                        }}
                      >
                        {moveState.runningProcs.map((p) => (
                          <div key={p.pid}>
                            {p.name} (PID: {p.pid})
                          </div>
                        ))}
                      </div>
                      <div className={styles.riskBox} style={{ marginTop: "12px" }}>
                        <Text className={styles.riskText}>
                          {t("softwareMove.forceCloseWarning")}
                        </Text>
                      </div>
                    </DialogBody>
                  </DialogContent>
                  <DialogActions>
                    <DialogTrigger disableButtonEnhancement>
                      <Button appearance="secondary">{t("softwareMove.cancel")}</Button>
                    </DialogTrigger>
                    <Button
                      appearance="primary"
                      style={{ backgroundColor: "var(--colorPaletteRedBackground2)" }}
                      onClick={forceCloseAndMove}
                    >
                      {t("softwareMove.forceCloseAndContinue")}
                    </Button>
                  </DialogActions>
                </>
              )}
          </DialogSurface>
        </Dialog>
      </div>

    </section>
  );
}
