import { useEffect, useState } from "react";
import {
  defaultLayout,
  MODULE_META,
  moveModule,
  saveDashboardLayout,
  updateModule,
  type DashboardLayout,
  type ModuleId,
  type ModulePlacement,
} from "../lib/moduleLayout";
import { DetailDrawer } from "./DetailDrawer";
import "./LayoutCustomize.css";

type Props = {
  open: boolean;
  layout: DashboardLayout;
  onClose: () => void;
  onChange: (layout: DashboardLayout) => void;
};

export function LayoutCustomize({ open, layout, onClose, onChange }: Props) {
  const [draft, setDraft] = useState(layout);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setDraft(layout);
      setError(null);
    }
  }, [open, layout]);

  const ordered = [...draft.modules].sort((a, b) => a.order - b.order);

  async function persist(next: DashboardLayout) {
    setDraft(next);
    setSaving(true);
    setError(null);
    try {
      await saveDashboardLayout(next);
      onChange(next);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }

  function patch(id: ModuleId, updates: Parameters<typeof updateModule>[2]) {
    void persist(updateModule(draft, id, updates));
  }

  return (
    <DetailDrawer
      open={open}
      title="Dashboard layout"
      eyebrow="Customize"
      onClose={onClose}
      wide
    >
      <p className="layout-lead">
        Toggle modules, set how many items each shows, choose column span, and
        reorder your command center. Changes save automatically.
      </p>

      {error ? <p className="module-empty">{error}</p> : null}
      {saving ? <p className="layout-saving">Saving…</p> : null}

      <ul className="layout-list">
        {ordered.map((entry, index) => {
          const meta = MODULE_META[entry.id];
          return (
            <li
              key={entry.id}
              className={`layout-item ${entry.enabled ? "" : "is-disabled"}`}
            >
              <div className="layout-item-top">
                <label className="layout-toggle">
                  <input
                    type="checkbox"
                    checked={entry.enabled}
                    onChange={(e) =>
                      patch(entry.id, { enabled: e.target.checked })
                    }
                  />
                  <span>
                    <strong>{meta.title}</strong>
                    <small>{meta.blurb}</small>
                  </span>
                </label>
                <div className="layout-move">
                  <button
                    type="button"
                    className="btn btn-ghost btn-icon"
                    aria-label={`Move ${meta.title} up`}
                    disabled={index === 0}
                    onClick={() => void persist(moveModule(draft, entry.id, -1))}
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    className="btn btn-ghost btn-icon"
                    aria-label={`Move ${meta.title} down`}
                    disabled={index === ordered.length - 1}
                    onClick={() => void persist(moveModule(draft, entry.id, 1))}
                  >
                    ↓
                  </button>
                </div>
              </div>

              <div className="layout-controls">
                <label className="layout-field">
                  <span>Items</span>
                  <input
                    className="field"
                    type="number"
                    min={3}
                    max={50}
                    value={entry.listLimit}
                    disabled={!entry.enabled}
                    onChange={(e) =>
                      patch(entry.id, { listLimit: Number(e.target.value) })
                    }
                  />
                </label>
                <label className="layout-field">
                  <span>Column</span>
                  <select
                    className="field-select"
                    value={entry.placement}
                    disabled={!entry.enabled}
                    onChange={(e) =>
                      patch(entry.id, {
                        placement: e.target.value as ModulePlacement,
                      })
                    }
                  >
                    <option value="auto">Auto</option>
                    <option value="left">Left column</option>
                    <option value="right">Right column</option>
                    <option value="full">Full width</option>
                  </select>
                </label>
              </div>
            </li>
          );
        })}
      </ul>

      <div className="layout-footer">
        <button
          type="button"
          className="btn btn-ghost"
          onClick={() => void persist(defaultLayout())}
        >
          Reset defaults
        </button>
        <p className="layout-hint">⌘, layout · ⌘⇧R refresh all</p>
      </div>
    </DetailDrawer>
  );
}
