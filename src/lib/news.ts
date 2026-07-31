/** Frontend refresh interval while the app is open. */
export const NEWS_REFRESH_MS = 15 * 60 * 1000;

export const NEWS_DASHBOARD_LIMIT = 8;
export const NEWS_MORE_LIMIT = 40;

export function formatNewsTime(iso: string | null | undefined): string {
  if (!iso) return "";
  try {
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export function sourceLabel(
  sourceTitle: string | null | undefined,
  sourceId: string,
): string {
  if (sourceTitle?.trim()) return sourceTitle.trim();
  try {
    return new URL(sourceId).hostname.replace(/^www\./, "");
  } catch {
    return sourceId;
  }
}
