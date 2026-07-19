/**
 * Language code (e.g. "zh-CN", "en-US"). Validated at runtime against
 * the locale JSON files auto-discovered by the i18n engine.
 */
export type AppLanguage = string;

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
  closeBehavior: "exit",
  updateCheckOnStartup: false,
};
