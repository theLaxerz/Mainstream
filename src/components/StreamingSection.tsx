import { useEffect, useState, type FormEvent } from "react";
import {
  getStreamingSettings,
  listStreamingHot,
  listStreamingNew,
  listStreamingProviders,
  openStreamingItem,
  refreshStreaming,
  saveStreamingSettings,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { StreamingItem, StreamingProvider } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import "./StreamingSection.css";

type Props = { limit?: number };

export function StreamingSection({ limit = 8 }: Props) {
  const [hot, setHot] = useState<StreamingItem[]>([]);
  const [fresh, setFresh] = useState<StreamingItem[]>([]);
  const [providers, setProviders] = useState<StreamingProvider[]>([]);
  const [enabled, setEnabled] = useState<string[]>([]);
  const [apiKey, setApiKey] = useState("");
  const [hasKey, setHasKey] = useState(false);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  async function loadLists() {
    const [hotRows, newRows] = await Promise.all([
      listStreamingHot(limit),
      listStreamingNew(limit),
    ]);
    setHot(hotRows);
    setFresh(newRows);
  }

  async function refresh(options?: { sync?: boolean }) {
    try {
      const [prov, settings] = await Promise.all([
        listStreamingProviders(),
        getStreamingSettings(),
      ]);
      setProviders(prov);
      setEnabled(settings.enabledProviders);
      setHasKey(settings.hasApiKey);
      if (options?.sync) {
        setRefreshing(true);
        const result = await refreshStreaming();
        setStatus(
          `Updated ${result.upserted} titles across ${result.providers} services`,
        );
      }
      await loadLists();
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

  function toggleProvider(id: string) {
    setEnabled((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id],
    );
  }

  async function onSaveSettings(e: FormEvent) {
    e.preventDefault();
    await saveStreamingSettings({
      apiKey: apiKey.trim() || undefined,
      enabledProviders: enabled,
    });
    setApiKey("");
    setSettingsOpen(false);
    await refresh({ sync: true });
  }

  function row(item: StreamingItem) {
    return (
      <li key={item.id} className="streaming-row">
        {item.posterPath ? (
          <img className="streaming-poster" src={item.posterPath} alt="" />
        ) : (
          <div className="streaming-poster streaming-poster-empty" />
        )}
        <button
          type="button"
          className="module-row-main shortcut-open"
          onClick={() => void openStreamingItem(item.id)}
        >
          <p className="module-row-title">{item.title}</p>
          <p className="module-row-meta">
            {item.providerName} · {item.mediaType}
            {item.releaseDate ? ` · ${item.releaseDate}` : ""}
          </p>
        </button>
      </li>
    );
  }

  return (
    <>
      <ModuleSection
        title="Streaming"
        eyebrow="What's on"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setSettingsOpen(true)}
            >
              Services
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              disabled={refreshing || !hasKey}
              onClick={() => void refresh({ sync: true })}
            >
              {refreshing ? "Syncing…" : "Sync"}
            </button>
          </div>
        }
        className="module-streaming"
        count={!loading ? hot.length + fresh.length : null}
      >
        {!hasKey ? (
          <p className="module-empty">
            Add a free TMDB API key under Services to load Prime, Apple TV+,
            Paramount+, Peacock, AMC+, and more.
          </p>
        ) : null}
        {error ? <p className="module-empty">{error}</p> : null}
        {status ? <p className="module-empty">{status}</p> : null}
        {loading ? <p className="module-empty">Loading streaming…</p> : null}

        <p className="module-eyebrow finance-subhead">What's hot</p>
        {hot.length === 0 ? (
          <p className="module-empty">Sync to load trending picks.</p>
        ) : (
          <ul className="module-list">{hot.map(row)}</ul>
        )}

        <p className="module-eyebrow finance-subhead">New & available</p>
        {fresh.length === 0 ? (
          <p className="module-empty">Recent arrivals appear here after sync.</p>
        ) : (
          <ul className="module-list">{fresh.map(row)}</ul>
        )}
      </ModuleSection>

      <DetailDrawer
        open={settingsOpen}
        title="Streaming services"
        eyebrow="TMDB"
        onClose={() => setSettingsOpen(false)}
      >
        <form onSubmit={onSaveSettings}>
          <input
            className="field"
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={
              hasKey ? "New TMDB API key (optional)" : "TMDB API key (required)"
            }
            aria-label="TMDB API key"
          />
          <p className="module-eyebrow">Enabled services</p>
          <div className="streaming-provider-grid">
            {providers.map((p) => (
              <label key={p.id} className="streaming-provider-chip">
                <input
                  type="checkbox"
                  checked={enabled.includes(p.id)}
                  onChange={() => toggleProvider(p.id)}
                />
                {p.name}
              </label>
            ))}
          </div>
          <button type="submit" className="btn btn-primary">
            Save & sync
          </button>
        </form>
      </DetailDrawer>
    </>
  );
}
