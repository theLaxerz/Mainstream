import { useEffect, useState, type FormEvent } from "react";
import {
  deleteNewsPref,
  listNews,
  listNewsPrefs,
  newsFeedback,
  openNewsItem,
  refreshNews,
  upsertNewsPref,
} from "../lib/api";
import {
  formatNewsTime,
  NEWS_DASHBOARD_LIMIT,
  NEWS_MORE_LIMIT,
  NEWS_REFRESH_MS,
  sourceLabel,
} from "../lib/news";
import { onDashboardRefresh } from "../lib/refresh";
import type { NewsFeedbackAction, NewsItem, NewsPref } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import "./NewsSection.css";

type Props = { limit?: number };

export function NewsSection({ limit: dashboardLimit }: Props) {
  const [stories, setStories] = useState<NewsItem[]>([]);
  const [prefs, setPrefs] = useState<NewsPref[]>([]);
  const [showMore, setShowMore] = useState(false);
  const [showPrefs, setShowPrefs] = useState(false);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [newFeedUrl, setNewFeedUrl] = useState("");
  const [newFeedTitle, setNewFeedTitle] = useState("");

  const baseLimit = dashboardLimit ?? NEWS_DASHBOARD_LIMIT;
  const limit = showMore ? NEWS_MORE_LIMIT : baseLimit;

  async function loadStories(nextLimit = limit) {
    const rows = await listNews(nextLimit);
    setStories(rows);
  }

  async function loadPrefs() {
    const rows = await listNewsPrefs();
    setPrefs(rows);
  }

  async function bootstrap(forceRefresh = false) {
    try {
      setError(null);
      await loadPrefs();
      const existing = await listNews(limit);
      if (forceRefresh || existing.length === 0) {
        setRefreshing(true);
        const result = await refreshNews();
        if (result.errors.length > 0) {
          setStatus(
            `Updated ${result.upsertedItems} stories · ${result.errors.length} feed error(s)`,
          );
        } else {
          setStatus(
            `Updated ${result.upsertedItems} stories from ${result.fetchedFeeds} feeds`,
          );
        }
        await loadPrefs();
      }
      await loadStories(limit);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }

  useEffect(() => {
    void bootstrap(false);
    return onDashboardRefresh(() => void bootstrap(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- mount-only bootstrap
  }, []);

  useEffect(() => {
    if (!loading) {
      void loadStories(limit).catch((e) => {
        setError(e instanceof Error ? e.message : String(e));
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- react to limit toggles
  }, [showMore]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      void (async () => {
        try {
          await refreshNews();
          await loadStories(limit);
          setStatus("Background refresh complete");
        } catch {
          /* keep last good state during background refresh */
        }
      })();
    }, NEWS_REFRESH_MS);
    return () => window.clearInterval(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [limit]);

  async function onRefresh() {
    setRefreshing(true);
    setStatus(null);
    try {
      const result = await refreshNews();
      await loadStories(limit);
      await loadPrefs();
      setError(null);
      setStatus(
        result.errors.length > 0
          ? `Refreshed with ${result.errors.length} feed error(s)`
          : `Refreshed ${result.upsertedItems} stories`,
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setRefreshing(false);
    }
  }

  async function onFeedback(itemId: number, action: NewsFeedbackAction) {
    try {
      await newsFeedback(itemId, action);
      await loadStories(limit);
      if (action === "follow_source" || action === "mute_source") {
        await loadPrefs();
      }
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function onOpen(id: number) {
    try {
      await openNewsItem(id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function onAddFeed(e: FormEvent) {
    e.preventDefault();
    if (!newFeedUrl.trim()) return;
    try {
      await upsertNewsPref({
        feedUrl: newFeedUrl.trim(),
        title: newFeedTitle.trim() || null,
        weight: 1,
        enabled: true,
        muted: false,
      });
      setNewFeedUrl("");
      setNewFeedTitle("");
      await loadPrefs();
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onTogglePref(pref: NewsPref, patch: Partial<NewsPref>) {
    try {
      await upsertNewsPref({
        feedUrl: pref.feedUrl,
        title: patch.title !== undefined ? patch.title : pref.title,
        weight: patch.weight ?? pref.weight,
        enabled: patch.enabled ?? pref.enabled,
        muted: patch.muted ?? pref.muted,
      });
      await loadPrefs();
      await loadStories(limit);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onDeletePref(id: number) {
    try {
      await deleteNewsPref(id);
      await loadPrefs();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <>
      <ModuleSection
        title="News"
        eyebrow="For you"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => {
                setShowPrefs(true);
                void loadPrefs();
              }}
            >
              Prefs
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void onRefresh()}
              disabled={refreshing}
            >
              {refreshing ? "…" : "Refresh"}
            </button>
          </div>
        }
        count={!loading ? stories.length : null}
      >
        {error ? <p className="module-empty">{error}</p> : null}
        {status && !error ? <p className="news-status">{status}</p> : null}

        {loading ? <p className="module-empty">Loading stories…</p> : null}
        {!loading && stories.length === 0 ? (
          <p className="module-empty">
            No stories yet — refresh to pull your feeds.
          </p>
        ) : null}

        <ul className="module-list">
          {stories.map((story) => (
            <li key={story.id} className="news-row">
              <button
                type="button"
                className="module-row-main shortcut-open"
                onClick={() => void onOpen(story.id)}
              >
                <p className="module-row-title">{story.title}</p>
                <p className="module-row-meta">
                  {sourceLabel(story.sourceTitle, story.sourceId)}
                  {story.publishedAt
                    ? ` · ${formatNewsTime(story.publishedAt)}`
                    : ""}
                  {story.liked ? " · liked" : ""}
                </p>
              </button>
              <div className="row-actions news-row-actions">
                <button
                  type="button"
                  className="btn btn-ghost btn-icon"
                  onClick={() => void onFeedback(story.id, "like")}
                  aria-label={`Like ${story.title}`}
                  title="Like"
                >
                  Like
                </button>
                <button
                  type="button"
                  className="btn btn-ghost btn-icon"
                  onClick={() => void onFeedback(story.id, "hide")}
                  aria-label={`Hide ${story.title}`}
                  title="Hide"
                >
                  Hide
                </button>
                <button
                  type="button"
                  className="btn btn-ghost btn-icon"
                  onClick={() => void onFeedback(story.id, "follow_source")}
                  aria-label={`Follow source for ${story.title}`}
                  title="Follow source"
                >
                  Follow
                </button>
                <button
                  type="button"
                  className="btn btn-danger btn-icon"
                  onClick={() => void onFeedback(story.id, "mute_source")}
                  aria-label={`Mute source for ${story.title}`}
                  title="Mute source"
                >
                  Mute
                </button>
              </div>
            </li>
          ))}
        </ul>

        <div className="news-footer">
          <button
            type="button"
            className="btn btn-ghost"
            onClick={() => setShowMore((v) => !v)}
          >
            {showMore ? "Top stories" : "More stories"}
          </button>
        </div>
      </ModuleSection>

      <DetailDrawer
        open={showPrefs}
        title="News preferences"
        eyebrow="Feeds"
        onClose={() => setShowPrefs(false)}
      >
        <div className="news-prefs">
          <form className="news-pref-form" onSubmit={onAddFeed}>
            <div className="field-row">
              <input
                className="field"
                value={newFeedTitle}
                onChange={(e) => setNewFeedTitle(e.target.value)}
                placeholder="Feed title"
                aria-label="Feed title"
              />
              <input
                className="field"
                value={newFeedUrl}
                onChange={(e) => setNewFeedUrl(e.target.value)}
                placeholder="https://…/rss.xml"
                aria-label="Feed URL"
              />
              <button type="submit" className="btn btn-primary">
                Add
              </button>
            </div>
          </form>

          {prefs.length === 0 ? (
            <p className="module-empty">No feeds yet — refresh to seed defaults.</p>
          ) : (
            <ul className="module-list">
              {prefs.map((pref) => (
                <li key={pref.id}>
                  <div className="module-row-main">
                    <p className="module-row-title">
                      {pref.title?.trim() || sourceLabel(null, pref.feedUrl)}
                    </p>
                    <p className="module-row-meta">
                      weight {pref.weight.toFixed(2)}
                      {pref.muted ? " · muted" : ""}
                      {!pref.enabled ? " · off" : ""}
                      {" · "}
                      {pref.feedUrl}
                    </p>
                  </div>
                  <div className="row-actions">
                    <button
                      type="button"
                      className="btn btn-ghost btn-icon"
                      onClick={() =>
                        void onTogglePref(pref, { enabled: !pref.enabled })
                      }
                    >
                      {pref.enabled ? "Off" : "On"}
                    </button>
                    <button
                      type="button"
                      className="btn btn-ghost btn-icon"
                      onClick={() =>
                        void onTogglePref(pref, { muted: !pref.muted })
                      }
                    >
                      {pref.muted ? "Unmute" : "Mute"}
                    </button>
                    <button
                      type="button"
                      className="btn btn-danger btn-icon"
                      onClick={() => void onDeletePref(pref.id)}
                    >
                      Del
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </DetailDrawer>
    </>
  );
}
