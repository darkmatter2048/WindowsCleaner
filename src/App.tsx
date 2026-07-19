import { useEffect, useState, useCallback } from "react";
import { makeStyles, tokens } from "@fluentui/react-components";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import TitleBar from "./components/TitleBar/TitleBar";
import DiskInfo from "./components/DiskInfo/DiskInfo";
import ActionButtons from "./components/ActionButtons/ActionButtons";
import Footer from "./components/Footer/Footer";
import CustomCleanWindow from "./components/CustomCleanWindow/CustomCleanWindow";
import SettingsWindow from "./components/SettingsWindow/SettingsWindow";
import { useTheme } from "./contexts/ThemeContext";
import { loadSettings, saveSettings } from "./constants/settings";
import type { AppSettings, AppTheme } from "./types/settings";

const useStyles = makeStyles({
  root: {
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
    height: "100vh",
    backgroundColor: tokens.colorNeutralBackground1,
    overflow: "hidden",
  },
  main: {
    display: "flex",
    flexDirection: "column",
    justifyContent: "center",
    gap: "12px",
    overflow: "hidden",
  },
});

function App() {
  const styles = useStyles();
  const { i18n } = useTranslation();
  const { setTheme } = useTheme();
  const [refreshKey, setRefreshKey] = useState(0);

  const handleCleaned = useCallback(() => {
    setRefreshKey((k) => k + 1);
  }, []);

  useEffect(() => {
    const settings = loadSettings();


    if (i18n.language !== settings.language) {
      void i18n.changeLanguage(settings.language);
    }

    const unlisten = listen("settings-changed", (event) => {
      const payload = event.payload as AppSettings;
      if (!payload) return;
      // Persist to this WebView's localStorage so
      // loadSettings() (e.g. TitleBar close) reads the latest value.
      saveSettings(payload);
      if (payload.language && i18n.language !== payload.language) {
        void i18n.changeLanguage(payload.language);
      }
      if (payload.theme) {
        const v = payload.theme as AppTheme;
        if (v === "dark" || v === "light" || v === "system") {
          setTheme(v);
        }
      }
    }).catch(() => {});

    return () => {
      unlisten.then((fn) => fn?.()).catch(() => {});
    };
  }, [i18n, setTheme]);

  if (window.location.hash === "#/custom-clean") {
    return <CustomCleanWindow />;
  }

  if (window.location.hash === "#/settings") {
    return <SettingsWindow />;
  }

  return (
    <div className={styles.root}>
      <TitleBar />
      <div className={styles.main}>
        <DiskInfo key={refreshKey} />
        <ActionButtons onCleaned={handleCleaned} />
      </div>
      <Footer />
    </div>
  );
}

export default App;
