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
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
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
      </div>
    </section>
  );
}
