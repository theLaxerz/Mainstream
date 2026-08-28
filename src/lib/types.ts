export type Note = {
  id: number;
  title: string;
  body: string;
  createdAt: string;
  updatedAt: string;
};

export type ShortcutKind = "url" | "app";

export type Shortcut = {
  id: number;
  label: string;
  kind: ShortcutKind;
  target: string;
  sortOrder: number;
  createdAt: string;
};

export type NewsPref = {
  id: number;
  feedUrl: string;
  title: string | null;
  weight: number;
  enabled: boolean;
  muted: boolean;
};

export type NewsItem = {
  id: number;
  sourceId: string;
  sourceTitle: string | null;
  title: string;
  url: string;
  summary: string | null;
  publishedAt: string | null;
  fetchedAt: string;
  score: number;
  liked: boolean;
  hidden: boolean;
};

export type NewsFeedbackAction =
  | "like"
  | "hide"
  | "follow_source"
  | "mute_source";

export type NewsRefreshResult = {
  fetchedFeeds: number;
  upsertedItems: number;
  errors: string[];
};

export type Setting = {
  key: string;
  value: string;
};

export type EmailMessage = {
  id: number;
  uid: number;
  mailbox: string;
  messageId: string | null;
  fromAddr: string | null;
  fromName: string | null;
  toAddrs: string;
  subject: string;
  preview: string;
  dateIso: string | null;
  isUnread: boolean;
  isImportant: boolean;
  importanceScore: number;
  hasListUnsubscribe: boolean;
  isJunk: boolean;
  messageUrl: string | null;
  syncedAt: string;
};

export type EmailSettings = {
  provider: string;
  auth: string;
  host: string;
  port: number;
  user: string;
  mailbox: string;
  hasPassword: boolean;
  hasOauth: boolean;
  connected: boolean;
  displayName: string | null;
  mailappAccount: string | null;
  googleClientId: string;
  microsoftClientId: string;
};

export type MailAppAccount = {
  name: string;
  userName: string;
  kind: string;
  accountType: string;
};

export type MailAppAccountsResult = {
  status: string;
  detail: string | null;
  accounts: MailAppAccount[];
};

export type EmailSyncResult = {
  fetched: number;
  important: number;
  mailbox: string;
};

export type PhysicalMailPiece = {
  id: number;
  emailId: number;
  digestDate: string | null;
  pieceIndex: number;
  ocrText: string;
  imagePath: string | null;
  subject: string;
  syncedAt: string;
};

export type PhysicalMailSyncResult = {
  digests: number;
  pieces: number;
  ocrRan: number;
};

export type HealthDay = {
  day: string;
  steps: number;
  sleepMinutes: number;
  avgHeartRate: number | null;
  importedAt: string;
};

export type HealthSettings = {
  exportPath: string;
};

export type HealthImportResult = {
  daysUpdated: number;
  exportPath: string;
};

export type HomeDevice = {
  id: string;
  name: string;
  vendor: string;
  deviceType: string;
  status: string;
  detail: string | null;
  thumbnailAvailable: boolean;
  snapshotReady: boolean;
  networkId: string | null;
  cameraId: string | null;
};

export type HomeSettings = {
  ringConnected: boolean;
  blinkConnected: boolean;
  blinkEmail: string;
};

export type BlinkLoginResult = {
  status: string;
  detail: string | null;
};

export type YoutubePref = {
  id: number;
  channelId: string;
  title: string | null;
  enabled: boolean;
};

export type YoutubeItem = {
  id: number;
  videoId: string;
  channelId: string;
  channelTitle: string | null;
  title: string;
  url: string;
  publishedAt: string | null;
  fetchedAt: string;
};

export type StreamingProvider = {
  id: string;
  name: string;
  tmdbProviderId: number;
};

export type StreamingItem = {
  id: number;
  providerId: string;
  providerName: string;
  kind: string;
  tmdbId: number;
  mediaType: string;
  title: string;
  overview: string | null;
  posterPath: string | null;
  releaseDate: string | null;
  score: number;
  fetchedAt: string;
};

export type StreamingSettings = {
  hasApiKey: boolean;
  enabledProviders: string[];
};

export type ModuleRefreshResult = {
  module: string;
  status: "ok" | "skipped" | "error" | string;
  detail: string | null;
};

export type DashboardRefreshResult = {
  startedAt: string;
  finishedAt: string;
  modules: ModuleRefreshResult[];
};

export type WeatherPlace = {
  name: string;
  latitude: number;
  longitude: number;
  admin: string | null;
  country: string | null;
  units: string;
};

export type WeatherSnapshot = {
  place: WeatherPlace;
  temperature: number;
  high: number | null;
  low: number | null;
  weatherCode: number;
  condition: string;
  humidity: number | null;
  windSpeed: number | null;
  fetchedAt: string;
};
