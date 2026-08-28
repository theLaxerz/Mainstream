import { getSetting, setSetting } from "./api";

export type ThemePreference = "auto" | "light" | "dusk";
export type ResolvedTheme = "light" | "dusk";

export const THEME_SETTING = "appearance.theme";
export const REFRESH_INTERVAL_SETTING = "refresh.intervalMinutes";
export const DEFAULT_REFRESH_MINUTES = 15;

export function isDuskHour(date = new Date()): boolean {
  const hour = date.getHours();
  return hour >= 19 || hour < 7;
}

export function resolveTheme(
  preference: ThemePreference,
  date = new Date(),
): ResolvedTheme {
  if (preference === "auto") return isDuskHour(date) ? "dusk" : "light";
  return preference;
}

export function applyTheme(preference: ThemePreference, date = new Date()) {
  document.documentElement.dataset.theme = resolveTheme(preference, date);
}

export function cycleTheme(current: ThemePreference): ThemePreference {
  if (current === "auto") return "dusk";
  if (current === "dusk") return "light";
  return "auto";
}

export async function loadThemePreference(): Promise<ThemePreference> {
  try {
    const value = await getSetting(THEME_SETTING);
    if (value === "auto" || value === "light" || value === "dusk") return value;
  } catch {
    /* browser / first run */
  }
  return "auto";
}

export async function saveThemePreference(
  preference: ThemePreference,
): Promise<void> {
  applyTheme(preference);
  try {
    await setSetting(THEME_SETTING, preference);
  } catch {
    /* persist is best-effort outside Tauri */
  }
}

export async function loadRefreshIntervalMinutes(): Promise<number> {
  try {
    const raw = await getSetting(REFRESH_INTERVAL_SETTING);
    const n = Number(raw);
    if (Number.isFinite(n) && n >= 0 && n <= 180) return Math.round(n);
  } catch {
    /* default */
  }
  return DEFAULT_REFRESH_MINUTES;
}

export function themeButtonLabel(preference: ThemePreference): string {
  if (preference === "dusk") return "Dusk";
  if (preference === "light") return "Light";
  return "Auto";
}
