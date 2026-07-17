import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
import {
  FluentProvider,
  webDarkTheme,
  webLightTheme,
  type Theme,
} from "@fluentui/react-components";
import { getCurrentWindow, Effect } from "@tauri-apps/api/window";

type ThemeMode = "dark" | "light";

interface ThemeContextValue {
  theme: ThemeMode;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  theme: "dark",
  toggleTheme: () => {},
});

const STORAGE_KEY = "wc-theme";

function getInitialTheme(): ThemeMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark") return stored;
  } catch {
    // localStorage unavailable
  }
  return "dark";
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<ThemeMode>(getInitialTheme);

  const fluentTheme: Theme =
    theme === "dark" ? webDarkTheme : webLightTheme;

  const toggleTheme = useCallback(() => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  }, []);

  // Persist theme + sync with native
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      // ignore
    }
    const appWindow = getCurrentWindow();
    appWindow.setTheme(theme).catch(() => {});
  }, [theme]);

  // Mica effect on mount
  useEffect(() => {
    const appWindow = getCurrentWindow();
    appWindow
      .setEffects({ effects: [Effect.Mica] })
      .catch(async () => {
        // Fallback for Windows 10: solid dark background
        await appWindow
          .setBackgroundColor("#1b1b1b")
          .catch(() => {});
      });
  }, []);

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <FluentProvider
        theme={fluentTheme}
        style={{ height: "100%", background: "transparent" }}
      >
        {children}
      </FluentProvider>
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  return useContext(ThemeContext);
}
