import { makeStyles, tokens, Button, Tooltip } from "@fluentui/react-components";
import {
  Dismiss16Regular,
  Subtract16Regular,
} from "@fluentui/react-icons";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import appIcon from "../../assets/clean.png";

const useStyles = makeStyles({
  titlebar: {
    display: "flex",
    alignItems: "center",
    height: "36px",
    paddingLeft: tokens.spacingHorizontalM,
    backgroundColor: tokens.colorNeutralBackground2,
    borderBottom: `1px solid ${tokens.colorNeutralStroke1}`,
    flexShrink: 0,
  },
  brand: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalS,
    flex: 1,
    pointerEvents: "none",
  },
  logo: {
    width: "18px",
    height: "18px",
    flexShrink: 0,
  },
  title: {
    fontFamily: "'DingTalkJinBu', sans-serif",
    fontSize: "13px",
    color: tokens.colorNeutralForeground2,
    fontWeight: 400,
  },
  controls: {
    display: "flex",
    height: "100%",
  },
  controlBtn: {
    width: "46px",
    height: "100%",
    minWidth: "46px",
    borderRadius: "0",
    color: tokens.colorNeutralForeground2,
    ":hover": {
      backgroundColor: tokens.colorNeutralBackground4,
      color: tokens.colorNeutralForeground1,
    },
  },
  closeBtn: {
    ":hover": {
      backgroundColor: "#c42b1c !important",
      color: "#ffffff !important",
    },
  },
});

export default function TitleBar() {
  const styles = useStyles();
  const { t } = useTranslation();
  const appWindow = getCurrentWindow();

  return (
    <div className={styles.titlebar} data-tauri-drag-region>
      <div className={styles.brand}>
        <img src={appIcon} className={styles.logo} alt="" />
        <span className={styles.title}>{t("titleBar.appName")}</span>
      </div>
      <div className={styles.controls}>
        <Tooltip content="最小化" relationship="label">
          <Button
            className={styles.controlBtn}
            appearance="transparent"
            size="small"
            icon={<Subtract16Regular />}
            onClick={() => appWindow.minimize()}
            aria-label="Minimize"
          />
        </Tooltip>
        <Tooltip content="关闭" relationship="label">
          <Button
            className={`${styles.controlBtn} ${styles.closeBtn}`}
            appearance="transparent"
            size="small"
            icon={<Dismiss16Regular />}
            onClick={() => appWindow.close()}
            aria-label="Close"
          />
        </Tooltip>
      </div>
    </div>
  );
}
