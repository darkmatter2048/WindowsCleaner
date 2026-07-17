import { makeStyles, tokens, Button } from "@fluentui/react-components";
import {
  Broom24Regular,
  Options24Regular,
} from "@fluentui/react-icons";
import { useTranslation } from "react-i18next";

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
    fontSize: "20px",
    fontWeight: 700,
    borderRadius: tokens.borderRadiusLarge,
    display: "flex",
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingHorizontalS,
  },
  buttonIcon: {
    fontSize: "28px",
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
});

export default function ActionButtons() {
  const styles = useStyles();
  const { t } = useTranslation();

  const handleQuickClean = () => {
    // TODO: implement quick clean in future iteration
    console.log("Quick clean triggered");
  };

  const handleCustomClean = () => {
    // TODO: implement custom clean in future iteration
    console.log("Custom clean triggered");
  };

  return (
    <div className={styles.container}>
      <div className={styles.row}>
        <Button
          className={`${styles.button} ${styles.primaryBtn}`}
          appearance="primary"
          size="large"
          icon={<Broom24Regular className={styles.buttonIcon} />}
          onClick={handleQuickClean}
        >
          {t("buttons.quickClean")}
        </Button>
        <Button
          className={`${styles.button} ${styles.outlineBtn}`}
          appearance="outline"
          size="large"
          icon={<Options24Regular className={styles.buttonIcon} />}
          onClick={handleCustomClean}
        >
          {t("buttons.customClean")}
        </Button>
      </div>
    </div>
  );
}
