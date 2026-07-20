import { makeStyles, tokens, Button, Tooltip } from "@fluentui/react-components";
import {
  Dismiss16Regular,
  Subtract16Regular,
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import appIcon from "../../assets/clean.png";
import { loadSettings } from "../../constants/settings";

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
    fontFamily: "'DingTalkJinBu', 'Microsoft YaHei UI', 'Microsoft YaHei', 'Segoe UI', sans-serif",
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

interface TitleBarProps {
  showIcon?: boolean;
  title?: string;
}

export default function TitleBar({ showIcon = true, title }: TitleBarProps) {
  const styles = useStyles();
  const { t } = useTranslation();
  const appWindow = getCurrentWindow();

  const handleClose = async () => {
    // Only the main window should trigger close-behavior logic;
    // every other window just hides on close.
    if (appWindow.label !== "main") {
      await appWindow.hide();
      return;
    }

    const settings = loadSettings();
    await invoke("handle_main_close", { behavior: settings.closeBehavior });
  };

  const label = title ?? t("titleBar.appName");

  return (
    <div className={styles.titlebar} data-tauri-drag-region>
      <div className={styles.brand}>
        {showIcon && <img src={appIcon} className={styles.logo} alt="" />}
        <span className={styles.title}>{label}</span>
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
            onClick={() => void handleClose()}
            aria-label="Close"
          />
        </Tooltip>
      </div>
    </div>
  );
}
