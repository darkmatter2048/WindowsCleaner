import { useState } from "react";
import {
  makeStyles,
  tokens,
  Button,
  Spinner,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
} from "@fluentui/react-components";
import {
  Broom28Regular,
  Options28Regular,
  CheckmarkCircle24Regular,
  ErrorCircle24Regular,
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import type { CleanResult } from "../../types/clean";

const useStyles = makeStyles({
  container: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingVerticalL,
    padding: `${tokens.spacingVerticalM} ${tokens.spacingHorizontalS}`,
  },
  row: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalM,
    alignItems: "center",
  },
  button: {
    width: "220px",
    height: "80px",
    fontFamily: "'AlimamaShuHeiTi', sans-serif",
    fontSize: "26px",
    fontWeight: 700,
    borderRadius: tokens.borderRadiusLarge,
    display: "flex",
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingHorizontalS,
  },
  buttonIcon: {
    fontSize: "36px",
  },
  primaryBtn: {
    background: `linear-gradient(135deg, ${tokens.colorBrandBackground} 0%, ${tokens.colorBrandBackgroundHover} 100%)`,
    border: "none",
    boxShadow: `0 4px 16px ${tokens.colorBrandBackground}44`,
    ":hover": {
      boxShadow: `0 6px 24px ${tokens.colorBrandBackground}66`,
      transform: "translateY(-1px)",
    },
    transition: "all 0.2s ease",
  },
  outlineBtn: {
    border: `2px solid ${tokens.colorNeutralStroke1}`,
    ":hover": {
      border: `2px solid ${tokens.colorCompoundBrandStroke}`,
      backgroundColor: tokens.colorBrandBackground2,
      transform: "translateY(-1px)",
    },
    transition: "all 0.2s ease",
  },
  dialogIcon: {
    fontSize: "48px",
  },
  dialogOk: {
    color: tokens.colorPaletteGreenForeground1,
  },
  dialogErr: {
    color: tokens.colorPaletteRedForeground1,
  },
  freedText: {
    fontFamily: "'DingTalkJinBu', sans-serif",
    fontSize: "16px",
  },
});

function formatFreed(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
}

interface Props {
  onCleaned: () => void;
}

export default function ActionButtons({ onCleaned }: Props) {
  const styles = useStyles();
  const { t } = useTranslation();
  const [cleaning, setCleaning] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [dialogResult, setDialogResult] = useState<{
    ok: boolean;
    message: string;
  }>({ ok: true, message: "" });

  const doClean = async (mode: "quick_clean" | "deep_clean", label: string) => {
    setCleaning(label);
    try {
      const result = await invoke<CleanResult>(mode);
      const freed = formatFreed(result.bytes_freed);
      setDialogResult({
        ok: true,
        message: result.bytes_freed > 0
          ? `${t("clean.freed")}: ${freed}`
          : t("clean.nothing"),
      });
      onCleaned(); // trigger disk info refresh
    } catch (err) {
      setDialogResult({
        ok: false,
        message: `${t("clean.failed")}: ${String(err)}`,
      });
    } finally {
      setCleaning(null);
      setDialogOpen(true);
    }
  };

  const busy = cleaning !== null;

  return (
    <div className={styles.container}>
      <div className={styles.row}>
        <Button
          className={`${styles.button} ${styles.primaryBtn}`}
          appearance="primary"
          size="large"
          icon={cleaning === t("buttons.quickClean") ? <Spinner size="tiny" /> : <Broom28Regular className={styles.buttonIcon} />}
          onClick={() => doClean("quick_clean", t("buttons.quickClean"))}
          disabled={busy}
        >
          {cleaning === t("buttons.quickClean") ? t("clean.cleaning") : t("buttons.quickClean")}
        </Button>
        <Button
          className={`${styles.button} ${styles.outlineBtn}`}
          appearance="outline"
          size="large"
          icon={cleaning === t("buttons.customClean") ? <Spinner size="tiny" /> : <Options28Regular className={styles.buttonIcon} />}
          onClick={() => doClean("deep_clean", t("buttons.customClean"))}
          disabled={busy}
        >
          {cleaning === t("buttons.customClean") ? t("clean.cleaning") : t("buttons.customClean")}
        </Button>
      </div>

      <Dialog open={dialogOpen} onOpenChange={(_, data) => setDialogOpen(data.open)}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>
              {dialogResult.ok ? (
                <CheckmarkCircle24Regular className={`${styles.dialogIcon} ${styles.dialogOk}`} />
              ) : (
                <ErrorCircle24Regular className={`${styles.dialogIcon} ${styles.dialogErr}`} />
              )}
            </DialogTitle>
            <DialogContent>
              <span className={styles.freedText}>{dialogResult.message}</span>
            </DialogContent>
            <DialogActions>
              <DialogTrigger disableButtonEnhancement>
                <Button appearance="primary">{t("clean.ok")}</Button>
              </DialogTrigger>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </div>
  );
}
