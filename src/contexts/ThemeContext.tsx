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
  const micaInitializedRef = useRef(false);

  const fluentTheme: Theme =
    theme === "dark" ? webDarkTheme : webLightTheme;

  const toggleTheme = useCallback(() => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      // ignore
    }

    const appWindow = getCurrentWindow();
    const fallbackBackground = theme === "dark" ? "#1b1b1b" : "#f3f3f3";

    appWindow.setTheme(theme).catch(() => {});
    appWindow.setBackgroundColor(fallbackBackground).catch(() => {});

    if (!micaInitializedRef.current) {
      micaInitializedRef.current = true;
      appWindow
        .setEffects({ effects: [Effect.Mica] })
        .catch(() => {});
    }
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <FluentProvider
        theme={fluentTheme}
        style={{ height: "100%", background: theme === "dark" ? "#1b1b1b" : "#f3f3f3" }}
      >
        {children}
      </FluentProvider>
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  return useContext(ThemeContext);
}
