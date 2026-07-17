import { makeStyles, tokens, Link } from "@fluentui/react-components";
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
    fontFamily: "'DingTalkJinBu', sans-serif",
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

  const handleSettings = () => {
    // TODO: navigate to settings page
    console.log("Settings");
  };

  const handleAbout = () => {
    // TODO: navigate to about page
    console.log("About");
  };

  const handleDonate = () => {
    // TODO: navigate to donate page
    console.log("Donate");
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
