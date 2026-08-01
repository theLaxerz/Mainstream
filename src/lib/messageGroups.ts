import type { UnreadMessage } from "./messages";

export type GroupedUnreadMessage = UnreadMessage & {
  unreadCount: number;
};

/**
 * Collapse multiple unread rows from the same chat into one list item
 * (latest message + count) so the dashboard shows more distinct conversations.
 */
export function groupUnreadByChat(messages: UnreadMessage[]): GroupedUnreadMessage[] {
  const byChat = new Map<string, GroupedUnreadMessage>();

  for (const msg of messages) {
    const key =
      msg.chatGuid?.trim() ||
      msg.chatIdentifier?.trim() ||
      String(msg.chatId);
    const existing = byChat.get(key);
    if (!existing) {
      byChat.set(key, { ...msg, unreadCount: 1 });
      continue;
    }
    existing.unreadCount += 1;
    const existingTime = Date.parse(existing.date);
    const msgTime = Date.parse(msg.date);
    if (Number.isFinite(msgTime) && msgTime > existingTime) {
      Object.assign(existing, msg);
    }
  }

  return [...byChat.values()].sort(
    (a, b) => Date.parse(b.date) - Date.parse(a.date),
  );
}

export function groupPreviewLabel(msg: GroupedUnreadMessage): string {
  if (msg.unreadCount <= 1) return msg.text;
  return `${msg.unreadCount} unread · ${msg.text}`;
}
