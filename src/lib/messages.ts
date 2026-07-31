import { invoke } from "@tauri-apps/api/core";

export type MessagesAccessStatus = "ok" | "needs_permission" | "unavailable";

export type MessagesAccess = {
  status: MessagesAccessStatus;
  detail: string | null;
};

export type UnreadMessage = {
  messageId: number;
  chatId: number;
  chatGuid: string;
  chatIdentifier: string;
  displayName: string;
  handle: string | null;
  text: string;
  date: string;
  isGroup: boolean;
};

export type UnreadMessagesResult = {
  access: MessagesAccess;
  messages: UnreadMessage[];
};

export async function listUnreadMessages(
  limit = 10,
): Promise<UnreadMessagesResult> {
  return invoke("list_unread_messages", { limit });
}

export async function listAllUnreadMessages(): Promise<UnreadMessagesResult> {
  return invoke("list_all_unread_messages");
}

export async function messagesAccessStatus(): Promise<MessagesAccess> {
  return invoke("messages_access_status");
}

export async function openFullDiskAccessSettings(): Promise<void> {
  return invoke("open_full_disk_access_settings");
}

export async function openMessageConversation(
  chatIdentifier: string,
  chatGuid?: string,
): Promise<void> {
  return invoke("open_message_conversation", {
    chatIdentifier,
    chatGuid: chatGuid ?? null,
  });
}

export function formatMessageTime(iso: string): string {
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
