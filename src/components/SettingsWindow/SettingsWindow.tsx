import { useEffect, useState } from "react";
import {
  Dropdown,
  Field,
  makeStyles,
  Option,
  Spinner,
  Switch,
  Text,
  tokens,
} from "@fluentui/react-components";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import TitleBar from "../TitleBar/TitleBar";
import { loadSettings, saveSettings } from "../../constants/settings";
import type { AppSettings, AppLanguage, AppTheme, CloseBehavior } from "../../types/settings";
import { useTheme } from "../../contexts/ThemeContext";

interface StartupSettings {
  autostart: boolean;
  closeBehavior: CloseBehavior;
  updateCheckOnStartup: boolean;
}

const useStyles = makeStyles({
  root: {
    display: "grid",
    gridTemplateRows: "auto 1fr",
    height: "100vh",
    backgroundColor: tokens.colorNeutralBackground1,
    overflow: "hidden",
  },
  content: {
    overflowY: "auto",
    padding: tokens.spacingHorizontalXL,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalXL,
  },
  section: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalM,
    padding: tokens.spacingHorizontalL,
    borderRadius: tokens.borderRadiusLarge,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground2,
  },
  sectionTitle: {
    fontSize: "18px",
    fontWeight: 700,
    color: tokens.colorNeutralForeground1,
  },
});

export default function SettingsWindow() {
  const styles = useStyles();
  const { t, i18n } = useTranslation();
  const { setTheme } = useTheme();
  const [settings, setSettings] = useState<AppSettings>(loadSettings());
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    const hydrate = async () => {
      try {
        const startup = await invoke<StartupSettings>("get_startup_settings");
        if (!mounted) return;
        // Only autostart is read-only from the registry; the rest is
        // already loaded from localStorage (the single source of truth).
        setSettings((current) => ({
          ...current,
          autostart: startup.autostart,
        }));
      } finally {
        if (mounted) setLoading(false);
      }
    };
    hydrate();
    return () => { mounted = false; };
  }, []);

  const persist = (next: AppSettings) => {
    setSettings(next);
    saveSettings(next);
    emit("settings-changed", next).catch(() => {});
  };

  const changeLanguage = (language: AppLanguage) => {
    const next = { ...settings, language };
    persist(next);
    void i18n.changeLanguage(language);
  };

  const changeTheme = (nextTheme: AppTheme) => {
    const next = { ...settings, theme: nextTheme };
    persist(next);
    setTheme(nextTheme);
  };

  const changeAutostart = async (enabled: boolean) => {
    await invoke("set_autostart_enabled", { enabled });
    persist({ ...settings, autostart: enabled });
  };

  const changeCloseBehavior = async (behavior: CloseBehavior) => {
    await invoke("set_close_behavior", { behavior });
    persist({ ...settings, closeBehavior: behavior });
  };

  const changeUpdateCheck = async (enabled: boolean) => {
    await invoke("set_update_check_on_startup", { enabled });
    persist({ ...settings, updateCheckOnStartup: enabled });
  };

  return (
    <div className={styles.root}>
      <TitleBar showIcon={false} title="设置" />
      <div className={styles.content}>
        {loading ? (
          <Spinner label={t("settings.loading")} />
        ) : (
          <>
            <section className={styles.section}>
              <Text className={styles.sectionTitle}>{t("settings.language.title")}</Text>
              <Dropdown
                selectedOptions={[settings.language]}
                value={settings.language === "zh-CN" ? t("settings.language.zhCN") : t("settings.language.enUS")}
                onOptionSelect={(_, data) => {
                  if (data.optionValue) changeLanguage(data.optionValue as AppLanguage);
                }}
              >
                <Option value="zh-CN">{t("settings.language.zhCN")}</Option>
                <Option value="en-US">{t("settings.language.enUS")}</Option>
              </Dropdown>
            </section>

            <section className={styles.section}>
              <Text className={styles.sectionTitle}>{t("settings.theme.title")}</Text>
              <Dropdown
                  selectedOptions={[settings.theme]}
                  value={
                    settings.theme === "dark"
                      ? t("settings.theme.dark")
                      : settings.theme === "light"
                        ? t("settings.theme.light")
                        : t("settings.theme.system")
                  }
                  onOptionSelect={(_, data) => {
                    if (data.optionValue) changeTheme(data.optionValue as AppTheme);
                  }}
                >
                  <Option value="system" text={`${t("settings.theme.system")} ${t("settings.theme.restartHint")}`}>{t("settings.theme.system")} {t("settings.theme.restartHint")}</Option>
                  <Option value="dark">{t("settings.theme.dark")}</Option>
                  <Option value="light">{t("settings.theme.light")}</Option>
                </Dropdown>
            </section>

            <section className={styles.section}>
              <Text className={styles.sectionTitle}>{t("settings.autostart.title")}</Text>
              <Switch
                checked={settings.autostart}
                label={t("settings.autostart.label")}
                onChange={(_, data) => void changeAutostart(Boolean(data.checked))}
              />
            </section>

            <section className={styles.section}>
              <Text className={styles.sectionTitle}>{t("settings.closeBehavior.title")}</Text>
              <Field label={t("settings.closeBehavior.label")}>
                <Dropdown
                  selectedOptions={[settings.closeBehavior]}
                  value={
                    settings.closeBehavior === "minimizeToTray"
                      ? t("settings.closeBehavior.minimizeToTray")
                      : t("settings.closeBehavior.exit")
                  }
                  onOptionSelect={(_, data) => {
                    if (data.optionValue) void changeCloseBehavior(data.optionValue as CloseBehavior);
                  }}
                >
                  <Option value="minimizeToTray">{t("settings.closeBehavior.minimizeToTray")}</Option>
                  <Option value="exit">{t("settings.closeBehavior.exit")}</Option>
                </Dropdown>
              </Field>
            </section>

            <section className={styles.section}>
              <Text className={styles.sectionTitle}>{t("settings.updates.title")}</Text>
              <Switch
                checked={settings.updateCheckOnStartup}
                label={t("settings.updates.label")}
                onChange={(_, data) => void changeUpdateCheck(Boolean(data.checked))}
              />
            </section>
          </>
        )}
      </div>
    </div>
  );
}
