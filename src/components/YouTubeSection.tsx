import { useEffect, useState, type FormEvent } from "react";
import {
  deleteYoutubePref,
  listYoutubeItems,
  listYoutubePrefs,
  openYoutubeItem,
  refreshYoutube,
  upsertYoutubePref,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { YoutubeItem, YoutubePref } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";

type Props = { limit?: number };

export function YouTubeSection({ limit = 10 }: Props) {
  const [videos, setVideos] = useState<YoutubeItem[]>([]);
  const [prefs, setPrefs] = useState<YoutubePref[]>([]);
  const [channelId, setChannelId] = useState("");
  const [title, setTitle] = useState("");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [manageOpen, setManageOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  async function refresh(options?: { fetch?: boolean }) {
    try {
      if (options?.fetch) {
        setRefreshing(true);
        const result = await refreshYoutube();
        setStatus(
          `Fetched ${result.upserted} videos from ${result.channels} channels`,
        );
      }
      const [items, prefRows] = await Promise.all([
        listYoutubeItems(limit),
        listYoutubePrefs(),
      ]);
      setVideos(items);
      setPrefs(prefRows);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setRefreshing(false);
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    return onDashboardRefresh(() => void refresh());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [limit]);

  async function onAddChannel(e: FormEvent) {
    e.preventDefault();
    if (!channelId.trim()) return;
    await upsertYoutubePref({
      channelId: channelId.trim(),
      title: title.trim() || null,
    });
    setChannelId("");
    setTitle("");
    await refresh({ fetch: true });
  }

  return (
    <>
      <ModuleSection
        title="YouTube"
        eyebrow="Subscriptions"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setManageOpen(true)}
            >
              Channels
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              disabled={refreshing}
              onClick={() => void refresh({ fetch: true })}
            >
              {refreshing ? "Syncing…" : "Sync"}
            </button>
          </div>
        }
        count={!loading ? videos.length : null}
      >
        {error ? <p className="module-empty">{error}</p> : null}
        {status ? <p className="module-empty">{status}</p> : null}
        {loading ? <p className="module-empty">Loading feed…</p> : null}
        {!loading && videos.length === 0 ? (
          <p className="module-empty">
            Add channel IDs to build your YouTube feed (public RSS).
          </p>
        ) : null}
        <ul className="module-list">
          {videos.map((v) => (
            <li key={v.id}>
              <button
                type="button"
                className="module-row-main shortcut-open"
                onClick={() => void openYoutubeItem(v.id)}
              >
                <p className="module-row-title">{v.title}</p>
                <p className="module-row-meta">
                  {v.channelTitle ?? v.channelId}
                  {v.publishedAt ? ` · ${v.publishedAt.slice(0, 10)}` : ""}
                </p>
              </button>
            </li>
          ))}
        </ul>
      </ModuleSection>

      <DetailDrawer
        open={manageOpen}
        title="YouTube channels"
        eyebrow="RSS feeds"
        onClose={() => setManageOpen(false)}
      >
        <form onSubmit={onAddChannel}>
          <div className="field-row">
            <input
              className="field"
              value={channelId}
              onChange={(e) => setChannelId(e.target.value)}
              placeholder="Channel ID or youtube.com/channel/… URL"
              aria-label="YouTube channel"
            />
            <input
              className="field"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Label (optional)"
              aria-label="Channel label"
            />
            <button type="submit" className="btn btn-primary">
              Add
            </button>
          </div>
        </form>
        <ul className="module-list">
          {prefs.map((p) => (
            <li key={p.id}>
              <div className="module-row-main">
                <p className="module-row-title">{p.title ?? p.channelId}</p>
                <p className="module-row-meta">{p.channelId}</p>
              </div>
              <button
                type="button"
                className="btn btn-danger btn-icon"
                onClick={() => void deleteYoutubePref(p.id).then(() => refresh())}
              >
                Del
              </button>
            </li>
          ))}
        </ul>
      </DetailDrawer>
    </>
  );
}
