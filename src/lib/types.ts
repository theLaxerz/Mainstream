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
  host: string;
  port: number;
  user: string;
  mailbox: string;
  hasPassword: boolean;
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
