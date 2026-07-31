import { useEffect, useState } from "react";
import {
  formatMessageTime,
  listAllUnreadMessages,
  listUnreadMessages,
  openFullDiskAccessSettings,
  openMessageConversation,
  type UnreadMessage,
} from "../lib/messages";
import { onDashboardRefresh } from "../lib/refresh";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import { PermissionCallout } from "./PermissionCallout";

export function MessagesSection() {
  const [messages, setMessages] = useState<UnreadMessage[]>([]);
  const [allMessages, setAllMessages] = useState<UnreadMessage[]>([]);
  const [accessStatus, setAccessStatus] = useState<string | null>(null);
  const [accessDetail, setAccessDetail] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [drawerLoading, setDrawerLoading] = useState(false);

  async function refresh() {
    try {
      const result = await listUnreadMessages(10);
      setMessages(result.messages);
      setAccessStatus(result.access.status);
      setAccessDetail(result.access.detail);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    return onDashboardRefresh(() => void refresh());
  }, []);

  async function openAll() {
    setDrawerOpen(true);
    setDrawerLoading(true);
    try {
      const result = await listAllUnreadMessages();
      setAllMessages(result.messages);
      setAccessStatus(result.access.status);
      setAccessDetail(result.access.detail);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setDrawerLoading(false);
    }
  }

  async function onOpenConversation(msg: UnreadMessage) {
    try {
      await openMessageConversation(msg.chatIdentifier, msg.chatGuid);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  const needsPermission = accessStatus === "needs_permission";
  const unavailable = accessStatus === "unavailable";

  function messageRows(rows: UnreadMessage[]) {
    return (
      <ul className="module-list">
        {rows.map((msg) => (
          <li key={msg.messageId}>
            <button
              type="button"
              className="module-row-main shortcut-open"
              onClick={() => void onOpenConversation(msg)}
            >
              <p className="module-row-title">{msg.displayName}</p>
              <p className="module-row-meta">
                {msg.text} · {formatMessageTime(msg.date)}
              </p>
            </button>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <>
      <ModuleSection
        title="Messages"
        eyebrow="Unread"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void refresh()}
            >
              Refresh
            </button>
            {!needsPermission && !unavailable ? (
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => void openAll()}
              >
                All
              </button>
            ) : null}
          </div>
        }
        style={{ animationDelay: "0.05s" }}
      >
        {needsPermission ? (
          <PermissionCallout
            title="Full Disk Access required"
            body={
              accessDetail ??
              "Mainstream needs Full Disk Access to show unread Messages."
            }
            steps={[
              "Open System Settings → Privacy & Security → Full Disk Access",
              "Enable Mainstream (or your terminal if running in dev)",
              "Quit and reopen Mainstream, then refresh",
            ]}
            actionLabel="Open Full Disk Access"
            onAction={() => void openFullDiskAccessSettings()}
          />
        ) : null}

        {unavailable ? (
          <p className="module-empty">
            {accessDetail ?? "Messages are unavailable on this Mac."}
          </p>
        ) : null}

        {error ? <p className="module-empty">{error}</p> : null}
        {loading ? <p className="module-empty">Loading messages…</p> : null}

        {!loading && !needsPermission && !unavailable && messages.length === 0 ? (
          <p className="module-empty">No unread messages.</p>
        ) : null}

        {!needsPermission && !unavailable ? messageRows(messages) : null}
      </ModuleSection>

      <DetailDrawer
        open={drawerOpen}
        title="All messages"
        eyebrow="Unread"
        onClose={() => setDrawerOpen(false)}
      >
        {drawerLoading ? (
          <p className="module-empty">Loading…</p>
        ) : allMessages.length === 0 ? (
          <p className="module-empty">No unread messages.</p>
        ) : (
          messageRows(allMessages)
        )}
      </DetailDrawer>
    </>
  );
}
