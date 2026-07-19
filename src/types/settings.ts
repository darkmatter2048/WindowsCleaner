export type AppLanguage = "zh-CN" | "en-US";
export type AppTheme = "dark" | "light" | "system";
export type CloseBehavior = "minimizeToTray" | "exit";

export interface AppSettings {
  language: AppLanguage;
  theme: AppTheme;
  autostart: boolean;
  closeBehavior: CloseBehavior;
  updateCheckOnStartup: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  language: "zh-CN",
  theme: "system",
  autostart: false,
  closeBehavior: "minimizeToTray",
  updateCheckOnStartup: false,
};
