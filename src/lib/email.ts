import type { EmailMessage } from "./types";

export function emailSenderLabel(msg: EmailMessage): string {
  if (msg.fromName?.trim()) return msg.fromName.trim();
  if (msg.fromAddr?.trim()) return msg.fromAddr.trim();
  return "Unknown sender";
}

export function formatEmailDate(iso: string | null): string {
  if (!iso) return "";
  try {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(d);
  } catch {
    return iso;
  }
}
