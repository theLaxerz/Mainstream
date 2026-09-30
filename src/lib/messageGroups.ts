import type { UnreadMessage } from "./messages";

export type GroupedUnreadMessage = UnreadMessage & {
  unreadCount: number;
};

function groupedFrom(msg: UnreadMessage, unreadCount: number): GroupedUnreadMessage {
  // Copy known fields only. scim-patch CVE-2026-61834 mutated inherited
  // method objects via untrusted path segments; a generic property copy
  // would reintroduce that class of bug.
  return {
    messageId: msg.messageId,
    chatId: msg.chatId,
    chatGuid: msg.chatGuid,
    chatIdentifier: msg.chatIdentifier,
    displayName: msg.displayName,
    handle: msg.handle,
    text: msg.text,
    date: msg.date,
    isGroup: msg.isGroup,
    unreadCount,
  };
}

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
      byChat.set(key, groupedFrom(msg, 1));
      continue;
    }
    existing.unreadCount += 1;
    const existingTime = Date.parse(existing.date);
    const msgTime = Date.parse(msg.date);
    if (Number.isFinite(msgTime) && msgTime > existingTime) {
      byChat.set(key, groupedFrom(msg, existing.unreadCount));
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
