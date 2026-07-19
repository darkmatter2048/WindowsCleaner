import { useEffect, useState, useCallback } from "react";
import {
  makeStyles,
  tokens,
  Switch,
  Dropdown,
  Option,
  Input,
  Button,
  Text,
  Spinner,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  DialogTrigger,
} from "@fluentui/react-components";
import {
  CheckmarkCircle24Regular,
  Dismiss24Regular,
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import type { CleanOptionId } from "../../types/clean";
import { loadSettings, saveSettings } from "../../constants/settings";

// ---------------------------------------------------------------------------
// Types shared with Rust
// ---------------------------------------------------------------------------

type AutoCleanMode = "scheduled" | "lowDisk";

interface AutoCleanSettings {
  enabled: boolean;
  mode: AutoCleanMode;
  intervalDays: number;
  thresholdGb: number;
  lastCleanTime: string;
  options: string[];
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const useStyles = makeStyles({
  root: {
    height: "100%",
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
    minHeight: 0,
    position: "relative",
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
    marginBottom: tokens.spacingVerticalXS,
  },
  body: {
    overflowY: "auto",
    padding: tokens.spacingHorizontalXL,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalL,
  },
  row: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalM,
  },
  numberInput: {
    width: "80px",
  },
  section: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalM,
    padding: tokens.spacingHorizontalL,
    borderRadius: tokens.borderRadiusLarge,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
  },
  sectionTitle: {
    fontWeight: 600,
    color: tokens.colorNeutralForeground1,
  },
  hint: {
    fontSize: "12px",
    color: tokens.colorNeutralForeground3,
  },
  optionTag: {
    display: "inline-block",
    fontSize: "12px",
    padding: "2px 8px",
    borderRadius: tokens.borderRadiusSmall,
    backgroundColor: tokens.colorNeutralBackground4,
    color: tokens.colorNeutralForeground2,
  },
  optionList: {
    display: "flex",
    flexWrap: "wrap",
    gap: tokens.spacingHorizontalXS,
  },
  footer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "flex-end",
    padding: `${tokens.spacingVerticalM} ${tokens.spacingHorizontalXL}`,
    borderTop: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
  },
  // -------- Green toast --------
  toast: {
    position: "absolute",
    top: "12px",
    right: "16px",
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalS,
    padding: `${tokens.spacingVerticalS} ${tokens.spacingHorizontalM}`,
    borderRadius: tokens.borderRadiusMedium,
    backgroundColor: "#107c10",
    color: "#fff",
    fontSize: "13px",
    fontWeight: 500,
    boxShadow: tokens.shadow16,
    zIndex: 100,
    animation: "slideIn 0.3s ease-out",
  },
  toastDismiss: {
    background: "none",
    border: "none",
    color: "#fff",
    cursor: "pointer",
    padding: "2px",
    display: "flex",
    alignItems: "center",
    opacity: 0.8,
    ":hover": { opacity: 1 },
  },
});

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface Props {
  selectedOptions: CleanOptionId[];
}

export default function ScheduledCleanPanel({ selectedOptions }: Props) {
  const styles = useStyles();
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const [enabled, setEnabled] = useState(false);
  const [mode, setMode] = useState<AutoCleanMode>("scheduled");
  const [intervalDays, setIntervalDays] = useState(7);
  const [thresholdGb, setThresholdGb] = useState(10);

  // Autostart suggestion dialog
  const [autostartDialogOpen, setAutostartDialogOpen] = useState(false);

  // Green success toast
  const [toastVisible, setToastVisible] = useState(false);

  // Load saved settings on mount
  useEffect(() => {
    let mounted = true;
    invoke<AutoCleanSettings>("get_auto_clean_settings")
      .then((s) => {
        if (!mounted) return;
        setEnabled(s.enabled);
        setMode(s.mode);
        setIntervalDays(s.intervalDays || 7);
        setThresholdGb(s.thresholdGb || 10);
      })
      .catch(() => {})
      .finally(() => {
        if (mounted) setLoading(false);
      });
    return () => {
      mounted = false;
    };
  }, []);

  // -------- Autostart check when toggling ON --------
  const handleToggle = useCallback(
    (checked: boolean) => {
      setEnabled(checked);
      if (checked) {
        const settings = loadSettings();
        if (!settings.autostart) {
          setAutostartDialogOpen(true);
        }
      }
    },
    [],
  );

  const enableAutostart = useCallback(() => {
    const settings = loadSettings();
    settings.autostart = true;
    saveSettings(settings);
    invoke("set_autostart_enabled", { enabled: true }).catch(() => {});
    emit("settings-changed", settings).catch(() => {});
    setAutostartDialogOpen(false);
  }, []);

  // -------- Save --------
  const save = async () => {
    setSaving(true);
    try {
      await invoke("save_auto_clean_settings", {
        settings: {
          enabled,
          mode,
          intervalDays,
          thresholdGb,
          lastCleanTime: "",
          options: selectedOptions,
        },
      });
      // Green toast
      setToastVisible(true);
      setTimeout(() => setToastVisible(false), 2500);
    } catch {
      // silently ignore
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className={styles.root}>
      {/* Green success toast */}
      {toastVisible && (
        <div className={styles.toast}>
          <CheckmarkCircle24Regular />
          {t("autoClean.saved")}
          <button
            className={styles.toastDismiss}
            onClick={() => setToastVisible(false)}
            aria-label="Dismiss"
          >
            <Dismiss24Regular fontSize={14} />
          </button>
        </div>
      )}

      <header className={styles.header}>
        <Text className={styles.title}>{t("autoClean.title")}</Text>
      </header>

      <div className={styles.body}>
        {loading ? (
          <Spinner />
        ) : (
          <>
            {/* Enable / disable toggle */}
            <div className={styles.section}>
              <Switch
                checked={enabled}
                onChange={(_, data) => handleToggle(Boolean(data.checked))}
                label={t("autoClean.enable")}
              />
            </div>

            {/* Mode selector */}
            <div className={styles.section}>
              <Text className={styles.sectionTitle}>{t("autoClean.mode.title")}</Text>
              <Dropdown
                disabled={!enabled}
                selectedOptions={[mode]}
                value={
                  mode === "scheduled"
                    ? t("autoClean.mode.scheduled")
                    : t("autoClean.mode.lowDisk")
                }
                onOptionSelect={(_, data) => {
                  if (data.optionValue) setMode(data.optionValue as AutoCleanMode);
                }}
              >
                <Option value="scheduled">{t("autoClean.mode.scheduled")}</Option>
                <Option value="lowDisk">{t("autoClean.mode.lowDisk")}</Option>
              </Dropdown>
            </div>

            {/* Interval / Threshold */}
            <div className={styles.section}>
              {mode === "scheduled" ? (
                <div className={styles.row}>
                  <Text>{t("autoClean.every")}</Text>
                  <Input
                    disabled={!enabled}
                    className={styles.numberInput}
                    type="number"
                    min={1}
                    max={180}
                    value={String(intervalDays)}
                    onChange={(_, data) => {
                      const v = parseInt(data.value, 10);
                      if (v >= 1 && v <= 180) setIntervalDays(v);
                    }}
                  />
                  <Text>{t("autoClean.days")}</Text>
                </div>
              ) : (
                <div className={styles.row}>
                  <Text>{t("autoClean.whenFreeBelow")}</Text>
                  <Input
                    disabled={!enabled}
                    className={styles.numberInput}
                    type="number"
                    min={1}
                    max={8192}
                    value={String(thresholdGb)}
                    onChange={(_, data) => {
                      const v = parseInt(data.value, 10);
                      if (v >= 1 && v <= 8192) setThresholdGb(v);
                    }}
                  />
                  <Text>{t("autoClean.gb")}</Text>
                </div>
              )}
              <Text className={styles.hint}>
                {mode === "scheduled"
                  ? t("autoClean.scheduleHint")
                  : t("autoClean.lowDiskHint")}
              </Text>
            </div>

            {/* Selected options preview */}
            <div className={styles.section}>
              <Text className={styles.sectionTitle}>{t("autoClean.cleanList")}</Text>
              <Text className={styles.hint}>{t("autoClean.cleanListHint")}</Text>
              <div className={styles.optionList}>
                {selectedOptions.length > 0 ? (
                  selectedOptions.map((id) => (
                    <span key={id} className={styles.optionTag}>
                      {t(`customClean.options.${id}.title`)}
                    </span>
                  ))
                ) : (
                  <Text className={styles.hint}>{t("autoClean.noOptions")}</Text>
                )}
              </div>
            </div>
          </>
        )}
      </div>

      <footer className={styles.footer}>
        <Button appearance="primary" onClick={save} disabled={loading || saving}>
          {saving ? <Spinner size="tiny" /> : t("autoClean.save")}
        </Button>
      </footer>

      {/* Autostart suggestion dialog */}
      <Dialog open={autostartDialogOpen} onOpenChange={(_, data) => setAutostartDialogOpen(data.open)}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>{t("autoClean.autostartDialog.title")}</DialogTitle>
            <DialogContent>{t("autoClean.autostartDialog.content")}</DialogContent>
            <DialogActions>
              <DialogTrigger disableButtonEnhancement>
                <Button appearance="secondary">{t("autoClean.autostartDialog.cancel")}</Button>
              </DialogTrigger>
              <Button appearance="primary" onClick={enableAutostart}>
                {t("autoClean.autostartDialog.enable")}
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </section>
  );
}
