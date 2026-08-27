import { useEffect, useState } from "react";
import {
  formatMessageTime,
  listAllUnreadMessages,
  openFullDiskAccessSettings,
  openMessageConversation,
  type UnreadMessage,
} from "../lib/messages";
import {
  groupPreviewLabel,
  groupUnreadByChat,
  type GroupedUnreadMessage,
} from "../lib/messageGroups";
import { onDashboardRefresh } from "../lib/refresh";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import { PermissionCallout } from "./PermissionCallout";

type Props = { limit?: number };

export function MessagesSection({ limit = 10 }: Props) {
  const topCount = limit;
  const [groups, setGroups] = useState<GroupedUnreadMessage[]>([]);
  const [accessStatus, setAccessStatus] = useState<string | null>(null);
  const [accessDetail, setAccessDetail] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [drawerOpen, setDrawerOpen] = useState(false);

  async function refresh() {
    try {
      // Fetch every unread row and group by conversation client-side so a
      // busy group chat (many individual replies) doesn't crowd out other
      // conversations — each chat collapses to one row with an unread count.
      const result = await listAllUnreadMessages();
      setGroups(groupUnreadByChat(result.messages));
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

  async function onOpenConversation(msg: UnreadMessage) {
    try {
      await openMessageConversation(msg.chatIdentifier, msg.chatGuid);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  const needsPermission = accessStatus === "needs_permission";
  const unavailable = accessStatus === "unavailable";

  function messageRows(rows: GroupedUnreadMessage[]) {
    return (
      <ul className="module-list">
        {rows.map((msg) => (
          <li key={msg.chatGuid || msg.chatIdentifier || msg.chatId}>
            <button
              type="button"
              className="module-row-main shortcut-open"
              onClick={() => void onOpenConversation(msg)}
            >
              <p className="module-row-title">
                {msg.displayName}
                {msg.unreadCount > 1 ? (
                  <span className="module-row-badge">{msg.unreadCount}</span>
                ) : null}
              </p>
              <p className="module-row-meta">
                {groupPreviewLabel(msg)} · {formatMessageTime(msg.date)}
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
            {!needsPermission && !unavailable && groups.length > topCount ? (
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setDrawerOpen(true)}
              >
                All
              </button>
            ) : null}
          </div>
        }
        count={
          !needsPermission && !unavailable && !loading ? groups.length : null
        }
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

        {!loading && !needsPermission && !unavailable && groups.length === 0 ? (
          <p className="module-empty">No unread messages.</p>
        ) : null}

        {!needsPermission && !unavailable
          ? messageRows(groups.slice(0, topCount))
          : null}
      </ModuleSection>

      <DetailDrawer
        open={drawerOpen}
        title="All messages"
        eyebrow="Unread"
        onClose={() => setDrawerOpen(false)}
      >
        {groups.length === 0 ? (
          <p className="module-empty">No unread messages.</p>
        ) : (
          messageRows(groups)
        )}
      </DetailDrawer>
    </>
  );
}
