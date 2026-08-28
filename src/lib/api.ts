import { invoke } from "@tauri-apps/api/core";
import type {
  EmailMessage,
  EmailSettings,
  EmailSyncResult,
  PhysicalMailPiece,
  PhysicalMailSyncResult,
  HealthDay,
  HealthSettings,
  HealthImportResult,
  BlinkLoginResult,
  HomeDevice,
  HomeSettings,
  YoutubeItem,
  YoutubePref,
  StreamingItem,
  StreamingProvider,
  StreamingSettings,
  DashboardRefreshResult,
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

export async function getHealthSettings(): Promise<HealthSettings> {
  return invoke("get_health_settings");
}

export async function saveHealthSettings(exportPath: string): Promise<HealthSettings> {
  return invoke("save_health_settings", { exportPath });
}

export async function importHealthExport(path?: string): Promise<HealthImportResult> {
  return invoke("import_health_export", { path: path ?? null });
}

export async function listHealthDays(limit?: number): Promise<HealthDay[]> {
  return invoke("list_health_days", { limit: limit ?? null });
}

export async function healthTodaySummary(): Promise<HealthDay | null> {
  return invoke("health_today_summary");
}

export async function getHomeSettings(): Promise<HomeSettings> {
  return invoke("get_home_settings");
}

export async function saveHomeCredentials(input: {
  ringRefreshToken?: string;
  blinkEmail?: string;
}): Promise<HomeSettings> {
  return invoke("save_home_credentials", { input });
}

export async function listHomeDevices(): Promise<HomeDevice[]> {
  return invoke("list_home_devices");
}

export async function blinkStartLogin(
  email: string,
  password: string,
): Promise<BlinkLoginResult> {
  return invoke("blink_start_login", { email, password });
}

export async function blinkVerifyPin(pin: string): Promise<BlinkLoginResult> {
  return invoke("blink_verify_pin", { pin });
}

export async function blinkDisconnect(): Promise<void> {
  return invoke("blink_disconnect");
}

export async function homeDeviceImageBase64(id: string): Promise<string | null> {
  return invoke("home_device_image_base64", { id });
}

export async function blinkCaptureSnapshot(id: string): Promise<HomeDevice> {
  return invoke("blink_capture_snapshot", { id });
}

export async function listYoutubePrefs(): Promise<YoutubePref[]> {
  return invoke("list_youtube_prefs");
}

export async function upsertYoutubePref(input: {
  channelId: string;
  title?: string | null;
  enabled?: boolean;
}): Promise<YoutubePref> {
  return invoke("upsert_youtube_pref", { input });
}

export async function deleteYoutubePref(id: number): Promise<void> {
  return invoke("delete_youtube_pref", { id });
}

export async function refreshYoutube(): Promise<{
  channels: number;
  upserted: number;
  errors: string[];
}> {
  return invoke("refresh_youtube");
}

export async function listYoutubeItems(limit?: number): Promise<YoutubeItem[]> {
  return invoke("list_youtube_items", { limit: limit ?? null });
}

export async function openYoutubeItem(id: number): Promise<void> {
  return invoke("open_youtube_item", { id });
}

export async function listStreamingProviders(): Promise<StreamingProvider[]> {
  return invoke("list_streaming_providers");
}

export async function getStreamingSettings(): Promise<StreamingSettings> {
  return invoke("get_streaming_settings");
}

export async function saveStreamingSettings(input: {
  apiKey?: string;
  enabledProviders?: string[];
}): Promise<StreamingSettings> {
  return invoke("save_streaming_settings", {
    apiKey: input.apiKey ?? null,
    enabledProviders: input.enabledProviders ?? null,
  });
}

export async function refreshStreaming(): Promise<{
  providers: number;
  upserted: number;
  errors: string[];
}> {
  return invoke("refresh_streaming");
}

export async function listStreamingHot(limit?: number): Promise<StreamingItem[]> {
  return invoke("list_streaming_hot", { limit: limit ?? null });
}

export async function listStreamingNew(limit?: number): Promise<StreamingItem[]> {
  return invoke("list_streaming_new", { limit: limit ?? null });
}

export async function openStreamingItem(id: number): Promise<void> {
  return invoke("open_streaming_item", { id });
}

export async function refreshDashboard(): Promise<DashboardRefreshResult> {
  return invoke("refresh_dashboard");
}
