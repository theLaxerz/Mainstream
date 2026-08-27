import { useEffect, useState, type FormEvent } from "react";
import {
  createShortcut,
  deleteShortcut,
  listShortcuts,
  openShortcut,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { Shortcut, ShortcutKind } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";

type Props = { limit?: number };

export function ShortcutsSection({ limit = 12 }: Props) {
  const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);
  const [label, setLabel] = useState("");
  const [kind, setKind] = useState<ShortcutKind>("url");
  const [target, setTarget] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [manageOpen, setManageOpen] = useState(false);

  async function refresh() {
    try {
      const rows = await listShortcuts();
      setShortcuts(rows);
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

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!label.trim() || !target.trim()) return;
    try {
      await createShortcut({
        label: label.trim(),
        kind,
        target: target.trim(),
        sortOrder: shortcuts.length,
      });
      setLabel("");
      setTarget("");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onOpen(id: number) {
    try {
      await openShortcut(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onDelete(id: number) {
    try {
      await deleteShortcut(id);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <>
      <ModuleSection
        title="Shortcuts"
        eyebrow="Launch"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setManageOpen(true)}
            >
              Manage
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void refresh()}
            >
              Refresh
            </button>
          </div>
        }
        count={!loading ? shortcuts.length : null}
      >
        {error && !manageOpen ? <p className="module-empty">{error}</p> : null}
        {loading ? <p className="module-empty">Loading shortcuts…</p> : null}
        {!loading && shortcuts.length === 0 ? (
          <p className="module-empty">
            No shortcuts yet — open Manage to add a website or macOS app.
          </p>
        ) : null}

        <ul className="module-list">
          {shortcuts.slice(0, limit).map((item) => (
            <li key={item.id}>
              <button
                type="button"
                className="module-row-main shortcut-open"
                onClick={() => void onOpen(item.id)}
              >
                <p className="module-row-title">{item.label}</p>
                <p className="module-row-meta">
                  {item.kind} · {item.target}
                </p>
              </button>
              <div className="row-actions">
                <button
                  type="button"
                  className="btn btn-primary btn-icon"
                  onClick={() => void onOpen(item.id)}
                >
                  Open
                </button>
              </div>
            </li>
          ))}
        </ul>
      </ModuleSection>

      <DetailDrawer
        open={manageOpen}
        title="Shortcuts manager"
        eyebrow="Add & organize"
        onClose={() => setManageOpen(false)}
      >
        <form onSubmit={onSubmit}>
          <div className="field-row">
            <input
              className="field"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder="Label"
              aria-label="Shortcut label"
            />
            <select
              className="field-select"
              value={kind}
              onChange={(e) => setKind(e.target.value as ShortcutKind)}
              aria-label="Shortcut kind"
            >
              <option value="url">URL</option>
              <option value="app">App</option>
            </select>
          </div>
          <div className="field-row">
            <input
              className="field"
              value={target}
              onChange={(e) => setTarget(e.target.value)}
              placeholder={
                kind === "url" ? "https://…" : "App path, name, or bundle id"
              }
              aria-label="Shortcut target"
            />
            <button type="submit" className="btn btn-primary">
              Add
            </button>
          </div>
        </form>

        {error ? <p className="module-empty">{error}</p> : null}
        {shortcuts.length === 0 ? (
          <p className="module-empty">
            Add a website or macOS app to launch it with one click.
          </p>
        ) : (
          <ul className="module-list">
            {shortcuts.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  className="module-row-main shortcut-open"
                  onClick={() => void onOpen(item.id)}
                >
                  <p className="module-row-title">{item.label}</p>
                  <p className="module-row-meta">
                    {item.kind} · {item.target}
                  </p>
                </button>
                <div className="row-actions">
                  <button
                    type="button"
                    className="btn btn-primary btn-icon"
                    onClick={() => void onOpen(item.id)}
                  >
                    Open
                  </button>
                  <button
                    type="button"
                    className="btn btn-danger btn-icon"
                    onClick={() => void onDelete(item.id)}
                  >
                    Del
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </DetailDrawer>
    </>
  );
}
