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
  type Theme as FluentTheme,
} from "@fluentui/react-components";
import { getCurrentWindow, Effect, type Theme as NativeTheme } from "@tauri-apps/api/window";
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

function normalizeTheme(value: NativeTheme | null | undefined): Resolved | null {
  return value === "dark" || value === "light" ? value : null;
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
  const [systemTheme, setSystemTheme] = useState<Resolved>(readSystem);
  const micaOnce = useRef(false);

  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const updateSystemTheme = (event?: MediaQueryListEvent) => {
      setSystemTheme(event ? (event.matches ? "dark" : "light") : readSystem());
    };

    updateSystemTheme();
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener("change", updateSystemTheme);
    } else {
      mediaQuery.addListener(updateSystemTheme);
    }

    return () => {
      if (mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener("change", updateSystemTheme);
      } else {
        mediaQuery.removeListener(updateSystemTheme);
      }
    };
  }, []);

  useEffect(() => {
    const win = getCurrentWindow();
    let active = true;
    let unlisten: (() => void) | undefined;

    // WebView2 can report a stale matchMedia value during startup. The native
    // window theme is the authoritative initial value on Windows.
    win.theme().then((nativeTheme) => {
      const next = normalizeTheme(nativeTheme);
      if (active && next) setSystemTheme(next);
    }).catch(() => {});

    win.onThemeChanged(({ payload }) => {
      const next = normalizeTheme(payload);
      if (active && next) setSystemTheme(next);
    }).then((stop) => {
      if (active) {
        unlisten = stop;
      } else {
        stop();
      }
    }).catch(() => {});

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  const resolved: Resolved = theme === "system" ? systemTheme : theme;
  const fluentTheme: FluentTheme = resolved === "dark" ? webDarkTheme : webLightTheme;

  const setTheme = useCallback((next: AppTheme) => {
    setThemeState(next);
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((prev) => {
      const r: Resolved = prev === "system" ? systemTheme : prev;
      return r === "dark" ? "light" : "dark";
    });
  }, [systemTheme]);

  // Persist + native window
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, theme);
    const s = loadSettings();
    saveSettings({ ...s, theme });

    const win = getCurrentWindow();
    const bg = resolved === "dark" ? "#1b1b1b" : "#f3f3f3";
    document.documentElement.style.colorScheme = resolved;
    document.documentElement.style.setProperty("--app-background", bg);

    // Passing null keeps the native window connected to the OS theme. Read
    // the effective native theme after doing so to update immediately when
    // the user selects "system" in settings.
    let active = true;
    win.setTheme(theme === "system" ? null : resolved)
      .then(() => theme === "system" ? win.theme() : null)
      .then((nativeTheme) => {
        const next = normalizeTheme(nativeTheme);
        if (active && next) setSystemTheme(next);
      })
      .catch(() => {});
    win.setBackgroundColor(bg).catch(() => {});

    if (!micaOnce.current) {
      micaOnce.current = true;
      win.setEffects({ effects: [Effect.Mica] }).catch(() => {});
    }

    return () => {
      active = false;
    };
  }, [theme, resolved]);

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme: resolved, setTheme, toggleTheme }}>
      <FluentProvider
        theme={fluentTheme}
        style={{ height: "100%", background: "var(--app-background)" }}
      >
        {children}
      </FluentProvider>
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  return useContext(ThemeContext);
}
