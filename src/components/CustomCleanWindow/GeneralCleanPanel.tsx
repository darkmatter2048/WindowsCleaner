import { useMemo, useState } from "react";
import {
  makeStyles,
  tokens,
  Button,
  Checkbox,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  DialogTrigger,
  Spinner,
  Text,
} from "@fluentui/react-components";
import {
  CheckmarkCircle24Regular,
  ErrorCircle24Regular,
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { GENERAL_CLEAN_OPTIONS, DEFAULT_GENERAL_CLEAN_OPTION_IDS } from "../../constants/cleanOptions";
import type { CleanOptionId, CleanResult } from "../../types/clean";

const useStyles = makeStyles({
  root: {
    height: "100%",
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
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
    marginBottom: tokens.spacingVerticalXS,
  },
  subtitle: {
    color: tokens.colorNeutralForeground3,
  },
  list: {
    minHeight: 0,
    overflowY: "auto",
    display: "grid",
    gridTemplateColumns: "1fr 1fr",
    alignContent: "start",
    gap: tokens.spacingVerticalS,
    padding: tokens.spacingHorizontalXL,
  },
  option: {
    display: "flex",
    alignItems: "flex-start",
    gap: tokens.spacingHorizontalS,
    minHeight: "76px",
    padding: tokens.spacingHorizontalM,
    borderRadius: tokens.borderRadiusMedium,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
  },
  optionText: {
    display: "flex",
    flexDirection: "column",
    gap: "2px",
  },
  optionTitle: {
    fontWeight: 600,
    color: tokens.colorNeutralForeground1,
  },
  optionDesc: {
    fontSize: "12px",
    color: tokens.colorNeutralForeground3,
    lineHeight: "18px",
  },
  risky: {
    color: tokens.colorPaletteYellowForeground2,
  },
  footer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: `${tokens.spacingVerticalM} ${tokens.spacingHorizontalXL}`,
    borderTop: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
  },
  footerActions: {
    display: "flex",
    gap: tokens.spacingHorizontalS,
  },
  count: {
    color: tokens.colorNeutralForeground3,
  },
  dialogIcon: {
    fontSize: "40px",
  },
  dialogOk: {
    color: tokens.colorPaletteGreenForeground1,
  },
  dialogErr: {
    color: tokens.colorPaletteRedForeground1,
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
  selectedOptions: CleanOptionId[];
  onSelectedOptionsChange: (options: CleanOptionId[]) => void;
}

export default function GeneralCleanPanel({ selectedOptions, onSelectedOptionsChange }: Props) {
  const styles = useStyles();
  const { t } = useTranslation();
  const [cleaning, setCleaning] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [dialogResult, setDialogResult] = useState<{ ok: boolean; message: string }>({
    ok: true,
    message: "",
  });

  const selectedSet = useMemo(() => new Set(selectedOptions), [selectedOptions]);

  const toggleOption = (id: CleanOptionId, checked: boolean) => {
    if (checked) {
      if (!selectedOptions.includes(id)) onSelectedOptionsChange([...selectedOptions, id]);
    } else {
      onSelectedOptionsChange(selectedOptions.filter((item) => item !== id));
    }
  };

  const runClean = async () => {
    if (selectedOptions.length === 0) return;

    setCleaning(true);
    try {
      const result = await invoke<CleanResult>("clean_selected", {
        request: { options: selectedOptions },
      });
      const freed = formatFreed(result.bytes_freed);
      const errorNote = result.errors.length > 0
        ? ` ${t("customClean.general.errors", { count: result.errors.length })}`
        : "";
      setDialogResult({
        ok: true,
        message: `${result.bytes_freed > 0 ? `${t("clean.freed")}: ${freed}` : t("clean.nothing")}${errorNote}`,
      });
    } catch (err) {
      setDialogResult({
        ok: false,
        message: `${t("clean.failed")}: ${String(err)}`,
      });
    } finally {
      setCleaning(false);
      setDialogOpen(true);
    }
  };

  return (
    <section className={styles.root}>
      <header className={styles.header}>
        <Text className={styles.title}>{t("customClean.general.title")}</Text>
        <Text className={styles.subtitle}>{t("customClean.general.subtitle")}</Text>
      </header>

      <div className={styles.list}>
        {GENERAL_CLEAN_OPTIONS.map((option) => (
          <label className={styles.option} key={option.id}>
            <Checkbox
              checked={selectedSet.has(option.id)}
              onChange={(_, data) => toggleOption(option.id, Boolean(data.checked))}
            />
            <span className={styles.optionText}>
              <Text className={`${styles.optionTitle} ${option.risky ? styles.risky : ""}`}>
                {t(`customClean.options.${option.id}.title`)}
              </Text>
              <Text className={styles.optionDesc}>
                {t(`customClean.options.${option.id}.description`)}
              </Text>
            </span>
          </label>
        ))}
      </div>

      <footer className={styles.footer}>
        <Text className={styles.count}>{t("customClean.general.selected", { count: selectedOptions.length })}</Text>
        <div className={styles.footerActions}>
          <Button appearance="subtle" onClick={() => onSelectedOptionsChange(DEFAULT_GENERAL_CLEAN_OPTION_IDS)} disabled={cleaning}>
            {t("customClean.general.reset")}
          </Button>
          <Button appearance="secondary" onClick={() => onSelectedOptionsChange(GENERAL_CLEAN_OPTIONS.map((option) => option.id))} disabled={cleaning}>
            {t("customClean.general.selectAll")}
          </Button>
          <Button appearance="primary" onClick={runClean} disabled={cleaning || selectedOptions.length === 0}>
            {cleaning ? <Spinner size="tiny" /> : t("customClean.general.run")}
          </Button>
        </div>
      </footer>

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
            <DialogContent>{dialogResult.message}</DialogContent>
            <DialogActions>
              <DialogTrigger disableButtonEnhancement>
                <Button appearance="primary">{t("clean.ok")}</Button>
              </DialogTrigger>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </section>
  );
}
