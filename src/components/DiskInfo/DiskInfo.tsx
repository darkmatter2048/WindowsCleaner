import { makeStyles, tokens, Text, Spinner } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";
import { useDiskInfo } from "../../hooks/useDiskInfo";
import diskSvg from "../../assets/disk.svg";

const useStyles = makeStyles({
  container: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingHorizontalL,
    padding: `${tokens.spacingVerticalM} ${tokens.spacingHorizontalS}`,
  },
  icon: {
    width: "56px",
    height: "56px",
    flexShrink: 0,
    opacity: 0.9,
  },
  info: {
    display: "flex",
    flexDirection: "column",
    gap: "2px",
  },
  label: {
    fontFamily: "'DingTalkJinBu', sans-serif",
    fontSize: "13px",
    lineHeight: "20px",
  },
  labelText: {
    color: tokens.colorNeutralForeground2,
  },
  totalLabel: {
    color: tokens.colorNeutralForeground2,
  },
  freeValue: {
    color: tokens.colorPaletteGreenForeground1,
  },
  usedValue: {
    color: tokens.colorPaletteRedForeground1,
  },
  percent: {
    fontFamily: "'DingTalkJinBu', sans-serif",
    fontSize: "12px",
    marginLeft: tokens.spacingHorizontalXS,
    opacity: 0.8,
  },
  // Two-color progress bar
  progressContainer: {
    width: "100%",
    height: "8px",
    borderRadius: tokens.borderRadiusMedium,
    overflow: "hidden",
    display: "flex",
    marginTop: tokens.spacingVerticalS,
    backgroundColor: tokens.colorNeutralBackground5,
  },
  progressUsed: {
    height: "100%",
    backgroundColor: tokens.colorPaletteRedForeground1,
    transition: "width 0.5s ease",
    borderRadius: `${tokens.borderRadiusMedium} 0 0 ${tokens.borderRadiusMedium}`,
    minWidth: "0px",
  },
  progressFree: {
    height: "100%",
    backgroundColor: tokens.colorPaletteGreenForeground1,
    transition: "width 0.5s ease",
    borderRadius: `0 ${tokens.borderRadiusMedium} ${tokens.borderRadiusMedium} 0`,
    minWidth: "0px",
  },
  // States
  loading: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalS,
    padding: tokens.spacingVerticalL,
    color: tokens.colorNeutralForeground3,
  },
  error: {
    color: tokens.colorPaletteRedForeground1,
    fontFamily: "'DingTalkJinBu', sans-serif",
  },
});

function formatGB(bytes: number): string {
  return (bytes / (1024 * 1024 * 1024)).toFixed(1);
}

export default function DiskInfo() {
  const styles = useStyles();
  const { t } = useTranslation();
  const { data, loading, error } = useDiskInfo();

  if (error) {
    return (
      <div className={styles.container}>
        <Text className={`${styles.label} ${styles.error}`}>
          {t("disk.totalSpace")}: {error}
        </Text>
      </div>
    );
  }

  const totalGB = data ? formatGB(data.total) : "--";
  const freeGB = data ? formatGB(data.free) : "--";
  const usedGB = data ? formatGB(data.used) : "--";
  const freePercent = data ? ((data.free / data.total) * 100).toFixed(1) : "0";
  const usedPercent = data
    ? ((data.used / data.total) * 100).toFixed(1)
    : "0";
  const freePctNum = data ? (data.free / data.total) * 100 : 0;
  const usedPctNum = data ? (data.used / data.total) * 100 : 0;

  return (
    <div className={styles.container}>
      <img src={diskSvg} className={styles.icon} alt="Disk" />

      <div className={styles.info}>
        {/* Total */}
        <div className={`${styles.label} ${styles.totalLabel}`}>
          {t("disk.totalSpace")}: {totalGB} {t("disk.unit")}
        </div>

        {/* Free */}
        <div className={styles.label}>
          <span className={styles.labelText}>{t("disk.freeSpace")}: </span>
          <span className={styles.freeValue}>{freeGB} {t("disk.unit")}</span>
          <span className={`${styles.percent} ${styles.freeValue}`}>({freePercent}%)</span>
        </div>

        {/* Used */}
        <div className={styles.label}>
          <span className={styles.labelText}>{t("disk.usedSpace")}: </span>
          <span className={styles.usedValue}>{usedGB} {t("disk.unit")}</span>
          <span className={`${styles.percent} ${styles.usedValue}`}>({usedPercent}%)</span>
        </div>

        {/* Two-color progress bar */}
        {loading ? (
          <div className={styles.loading}>
            <Spinner size="extra-tiny" />
          </div>
        ) : (
          <div className={styles.progressContainer}>
            <div
              className={styles.progressUsed}
              style={{ width: `${usedPctNum}%` }}
            />
            <div
              className={styles.progressFree}
              style={{ width: `${freePctNum}%` }}
            />
          </div>
        )}
      </div>
    </div>
  );
}
