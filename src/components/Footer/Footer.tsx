import { makeStyles, tokens, Link } from "@fluentui/react-components";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";

const useStyles = makeStyles({
  footer: {
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    gap: tokens.spacingHorizontalL,
    padding: `${tokens.spacingVerticalM} ${tokens.spacingHorizontalL}`,
    backgroundColor: tokens.colorNeutralBackground2,
    borderTop: `1px solid ${tokens.colorNeutralStroke1}`,
    flexShrink: 0,
  },
  link: {
    fontSize: "12px",
    fontFamily: "'DingTalkJinBu', 'Microsoft YaHei UI', 'Microsoft YaHei', 'Segoe UI', sans-serif",
    color: tokens.colorNeutralForeground3,
    ":hover": {
      color: tokens.colorBrandForeground1,
    },
  },
  separator: {
    color: tokens.colorNeutralStrokeAccessible,
    fontSize: "12px",
    userSelect: "none",
  },
});

export default function Footer() {
  const styles = useStyles();
  const { t } = useTranslation();

  const handleSettings = async () => {
    try {
      await invoke("open_settings_window");
    } catch (error) {
      console.error("Failed to open settings window", error);
    }
  };

  const handleAbout = () => {
    invoke("open_about_window").catch(() => {});
  };

  const handleDonate = () => {
    invoke("open_donate_window").catch(() => {});
  };

  return (
    <div className={styles.footer}>
      <Link className={styles.link} onClick={handleSettings}>
        {t("footer.settings")}
      </Link>
      <span className={styles.separator}>|</span>
      <Link className={styles.link} onClick={handleAbout}>
        {t("footer.about")}
      </Link>
      <span className={styles.separator}>|</span>
      <Link className={styles.link} onClick={handleDonate}>
        {t("footer.donate")}
      </Link>
    </div>
  );
}
