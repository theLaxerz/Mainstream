import { invoke } from "@tauri-apps/api/core";
import type {
  EmailMessage,
  EmailSettings,
  EmailSyncResult,
  PhysicalMailPiece,
  PhysicalMailSyncResult,
  NewsFeedbackAction,
  NewsItem,
  NewsPref,
  NewsRefreshResult,
  Note,
  Setting,
  Shortcut,
  ShortcutKind,
} from "./types";

export async function listNotes(limit?: number): Promise<Note[]> {
  return invoke("list_notes", { limit: limit ?? null });
}

export async function getNote(id: number): Promise<Note | null> {
  return invoke("get_note", { id });
}

export async function createNote(title: string, body = ""): Promise<Note> {
  return invoke("create_note", { input: { title, body } });
}

export async function updateNote(
  id: number,
  patch: { title?: string; body?: string },
): Promise<Note> {
  return invoke("update_note", { input: { id, ...patch } });
}

export async function deleteNote(id: number): Promise<void> {
  return invoke("delete_note", { id });
}

export async function listShortcuts(): Promise<Shortcut[]> {
  return invoke("list_shortcuts");
}

export async function createShortcut(input: {
  label: string;
  kind: ShortcutKind;
  target: string;
  sortOrder?: number;
}): Promise<Shortcut> {
  return invoke("create_shortcut", {
    input: {
      label: input.label,
      kind: input.kind,
      target: input.target,
      sortOrder: input.sortOrder ?? null,
    },
  });
}

export async function updateShortcut(
  id: number,
  patch: {
    label?: string;
    kind?: ShortcutKind;
    target?: string;
    sortOrder?: number;
  },
): Promise<Shortcut> {
  return invoke("update_shortcut", { input: { id, ...patch } });
}

export async function deleteShortcut(id: number): Promise<void> {
  return invoke("delete_shortcut", { id });
}

export async function openShortcut(id: number): Promise<void> {
  return invoke("open_shortcut", { id });
}

export async function openTarget(kind: ShortcutKind, target: string): Promise<void> {
  return invoke("open_target", { kind, target });
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting_cmd", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting_cmd", { key, value });
}

export async function listSettings(): Promise<Setting[]> {
  return invoke("list_settings");
}

export async function listNewsPrefs(): Promise<NewsPref[]> {
  return invoke("list_news_prefs");
}

export async function upsertNewsPref(input: {
  feedUrl: string;
  title?: string | null;
  weight?: number;
  enabled?: boolean;
  muted?: boolean;
}): Promise<NewsPref> {
  return invoke("upsert_news_pref", { input });
}

export async function deleteNewsPref(id: number): Promise<void> {
  return invoke("delete_news_pref", { id });
}

export async function getEmailSettings(): Promise<EmailSettings> {
  return invoke("get_email_settings");
}

export async function saveEmailSettings(input: {
  host: string;
  port?: number;
  user: string;
  mailbox?: string;
  password?: string;
}): Promise<EmailSettings> {
  return invoke("save_email_settings", {
    input: {
      host: input.host,
      port: input.port ?? null,
      user: input.user,
      mailbox: input.mailbox ?? null,
      password: input.password ?? null,
    },
  });
}

export async function syncEmail(): Promise<EmailSyncResult> {
  return invoke("sync_email");
}

export async function listImportantEmails(limit?: number): Promise<EmailMessage[]> {
  return invoke("list_important_emails", { limit: limit ?? null });
}

export async function listAllImportantEmails(): Promise<EmailMessage[]> {
  return invoke("list_all_important_emails");
}

export async function openEmail(id: number): Promise<void> {
  return invoke("open_email", { id });
}

export async function syncPhysicalMail(): Promise<PhysicalMailSyncResult> {
  return invoke("sync_physical_mail");
}

export async function listPhysicalMail(
  limit?: number,
): Promise<PhysicalMailPiece[]> {
  return invoke("list_physical_mail", { limit: limit ?? null });
}

export async function physicalMailImageBase64(
  id: number,
): Promise<string | null> {
  return invoke("physical_mail_image_base64", { id });
}

export async function seedDefaultNewsFeeds(): Promise<number> {
  return invoke("seed_default_news_feeds");
}

export async function refreshNews(): Promise<NewsRefreshResult> {
  return invoke("refresh_news");
}

export async function listNews(
  limit?: number,
  includeHidden = false,
): Promise<NewsItem[]> {
  return invoke("list_news", {
    limit: limit ?? null,
    includeHidden,
  });
}

export async function newsFeedback(
  itemId: number,
  action: NewsFeedbackAction,
): Promise<NewsItem> {
  return invoke("news_feedback", { input: { itemId, action } });
}

export async function openNewsItem(id: number): Promise<void> {
  return invoke("open_news_item", { id });
}

export async function rerankNews(): Promise<number> {
  return invoke("rerank_news");
}

export async function getNewsLastRefresh(): Promise<string | null> {
  return invoke("get_news_last_refresh");
}
