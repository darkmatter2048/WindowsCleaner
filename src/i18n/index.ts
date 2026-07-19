import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

// ---------------------------------------------------------------------------
// Auto-discover: every JSON under ./locales/ becomes a language.
// The JSON's `_meta.nativeName` provides the display name.
//
// To add a language: drop a `<code>.json` file into ./locales/ — that's it.
// ---------------------------------------------------------------------------

const localeModules = import.meta.glob<{ default: Record<string, unknown> }>(
  "./locales/*.json",
  { eager: true },
);

interface LanguageMeta {
  code: string;
  nativeName: string;
}

const LANGUAGES: LanguageMeta[] = [];

function buildResources(): Record<string, { translation: Record<string, unknown> }> {
  const resources: Record<string, { translation: Record<string, unknown> }> = {};
  for (const [path, mod] of Object.entries(localeModules)) {
    const code = path.replace("./locales/", "").replace(".json", "");
    const translation = mod.default;
    resources[code] = { translation };
    LANGUAGES.push({
      code,
      nativeName: (translation._meta as { nativeName?: string } | undefined)?.nativeName ?? code,
    });
  }
  return resources;
}

export function getLanguages(): readonly LanguageMeta[] {
  return LANGUAGES;
}

export function getLanguageMeta(code: string): LanguageMeta | undefined {
  return LANGUAGES.find((l) => l.code === code);
}

export function isAppLanguage(value: string): boolean {
  return LANGUAGES.some((l) => l.code === value);
}

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: buildResources(),
    fallbackLng: "zh-CN",
    detection: {
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
      lookupLocalStorage: "wc-lang",
    },
    interpolation: {
      escapeValue: false,
    },
    returnObjects: false,
  });

export default i18n;
