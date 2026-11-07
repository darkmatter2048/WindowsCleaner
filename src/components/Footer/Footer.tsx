import { useState } from "react";
import {
  Button,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  DialogTrigger,
  Link,
  makeStyles,
  Text,
  tokens,
} from "@fluentui/react-components";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { openFreeSupport } from "../../constants/support";
import FreeSupportButton from "../Support/FreeSupportButton";

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
    whiteSpace: "nowrap",
    ":hover": {
      color: tokens.colorBrandForeground1,
    },
  },
  separator: {
    color: tokens.colorNeutralStrokeAccessible,
    fontSize: "12px",
    userSelect: "none",
  },
  supportSurface: {
    width: "min(360px, calc(100vw - 24px))",
    maxWidth: "calc(100vw - 24px)",
  },
  supportOptions: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalM,
  },
  supportError: {
    color: tokens.colorPaletteRedForeground1,
    fontSize: "12px",
  },
});

export default function Footer() {
  const styles = useStyles();
  const { t } = useTranslation();
  const [supportOpen, setSupportOpen] = useState(false);
  const [supportError, setSupportError] = useState<string | null>(null);

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

  const handleFreeSupport = () => {
    setSupportOpen(false);
    window.requestAnimationFrame(() => {
      openFreeSupport().catch((error) => {
        console.error("Failed to open support page", error);
        setSupportError(t("support.openFailed"));
        setSupportOpen(true);
      });
    });
  };

  const handleDirectDonate = () => {
    setSupportOpen(false);
    window.requestAnimationFrame(() => {
      invoke("open_donate_window").catch((error) => {
        console.error("Failed to open donation window", error);
        setSupportError(t("support.donateOpenFailed"));
        setSupportOpen(true);
      });
    });
  };

  return (
    <div className={styles.footer} data-auto-size-footer>
      <Link className={styles.link} onClick={handleSettings}>
        {t("footer.settings")}
      </Link>
      <span className={styles.separator}>|</span>
      <Link className={styles.link} onClick={handleAbout}>
        {t("footer.about")}
      </Link>
      <span className={styles.separator}>|</span>
      <Link className={styles.link} onClick={() => { setSupportError(null); setSupportOpen(true); }}>
        {t("footer.donate")}
      </Link>

      <Dialog
        open={supportOpen}
        onOpenChange={(_, data) => {
          setSupportOpen(data.open);
          if (!data.open) setSupportError(null);
        }}
      >
        <DialogSurface className={styles.supportSurface}>
          <DialogBody>
            <DialogTitle>{t("support.chooseTitle")}</DialogTitle>
            <DialogContent className={styles.supportOptions}>
              <FreeSupportButton fill onClick={handleFreeSupport} />
              <Button appearance="secondary" onClick={handleDirectDonate}>
                {t("support.direct")}
              </Button>
              {supportError && <Text className={styles.supportError} role="alert">{supportError}</Text>}
            </DialogContent>
            <DialogActions>
              <DialogTrigger disableButtonEnhancement>
                <Button appearance="subtle">{t("support.cancel")}</Button>
              </DialogTrigger>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </div>
  );
}
