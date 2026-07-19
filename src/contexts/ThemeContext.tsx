import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  useRef,
  type ReactNode,
} from "react";
import {
  FluentProvider,
  webDarkTheme,
  webLightTheme,
  type Theme,
} from "@fluentui/react-components";
import { getCurrentWindow, Effect } from "@tauri-apps/api/window";
import { loadSettings, saveSettings } from "../constants/settings";
import type { AppTheme } from "../types/settings";

type Resolved = "dark" | "light";

interface ThemeContextValue {
  theme: AppTheme;
  resolvedTheme: Resolved;
  setTheme: (t: AppTheme) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  theme: "system",
  resolvedTheme: "dark",
  setTheme: () => {},
  toggleTheme: () => {},
});

const STORAGE_KEY = "wc-theme";

function readSystem(): Resolved {
  if (typeof window === "undefined") return "dark";
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function getInitial(): AppTheme {
  const s = loadSettings();
  if (s.theme === "dark" || s.theme === "light" || s.theme === "system") {
    return s.theme;
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === "dark" || raw === "light") return raw;
  } catch { /* noop */ }
  return "system";
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<AppTheme>(getInitial);
  const micaOnce = useRef(false);

  const resolved: Resolved = theme === "system" ? readSystem() : theme;
  const fluentTheme: Theme = resolved === "dark" ? webDarkTheme : webLightTheme;

  const setTheme = useCallback((next: AppTheme) => {
    setThemeState(next);
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((prev) => {
      const r: Resolved = prev === "system" ? readSystem() : prev;
      return r === "dark" ? "light" : "dark";
    });
  }, []);

  // Persist + native window
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, theme);
    const s = loadSettings();
    saveSettings({ ...s, theme });

    const win = getCurrentWindow();
    const bg = resolved === "dark" ? "#1b1b1b" : "#f3f3f3";
    win.setTheme(resolved).catch(() => {});
    win.setBackgroundColor(bg).catch(() => {});

    if (!micaOnce.current) {
      micaOnce.current = true;
      win.setEffects({ effects: [Effect.Mica] }).catch(() => {});
    }
  }, [theme, resolved]);

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme: resolved, setTheme, toggleTheme }}>
      <FluentProvider
        theme={fluentTheme}
        style={{ height: "100%", background: resolved === "dark" ? "#1b1b1b" : "#f3f3f3" }}
      >
        {children}
      </FluentProvider>
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  return useContext(ThemeContext);
}
