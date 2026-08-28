import { useEffect, useMemo, useRef, useState } from "react";
import { createNote, listShortcuts, openShortcut } from "../lib/api";
import { MODULE_IDS, MODULE_META, type ModuleId } from "../lib/moduleLayout";
import { requestDashboardRefresh } from "../lib/refresh";
import type { Shortcut } from "../lib/types";
import "./CommandPalette.css";

export type PaletteHandlers = {
  onRefresh: () => void;
  onCustomize: () => void;
  onToggleTheme: () => void;
  themeLabel: string;
};

type Item = {
  id: string;
  group: string;
  label: string;
  hint?: string;
  run: () => void | Promise<void>;
};

function matches(query: string, ...parts: Array<string | undefined>): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return parts.some((p) => p?.toLowerCase().includes(q));
}

type Props = {
  open: boolean;
  onClose: () => void;
  handlers: PaletteHandlers;
};

export function CommandPalette({ open, onClose, handlers }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setActive(0);
    setStatus(null);
    void listShortcuts()
      .then(setShortcuts)
      .catch(() => setShortcuts([]));
    const id = window.setTimeout(() => inputRef.current?.focus(), 20);
    return () => window.clearTimeout(id);
  }, [open]);

  const items = useMemo<Item[]>(() => {
    const goTo: Item[] = MODULE_IDS.filter((id) =>
      matches(query, MODULE_META[id].title, MODULE_META[id].blurb, id),
    ).map((id: ModuleId) => ({
      id: `go-${id}`,
      group: "Go to",
      label: MODULE_META[id].title,
      hint: MODULE_META[id].blurb,
      run: () => {
        document.getElementById(`module-${id}`)?.scrollIntoView({
          behavior: "smooth",
          block: "start",
        });
        onClose();
      },
    }));

    const shortcutItems: Item[] = shortcuts
      .filter((s) => matches(query, s.label, s.target, s.kind))
      .map((s) => ({
        id: `sc-${s.id}`,
        group: "Shortcuts",
        label: s.label,
        hint: s.kind === "app" ? "App" : s.target,
        run: async () => {
          await openShortcut(s.id);
          onClose();
        },
      }));

    const noteTitle = query.trim() || "Quick note";
    const actions: Item[] = [
      {
        id: "act-refresh",
        group: "Actions",
        label: "Refresh all",
        hint: "⌘⇧R",
        run: () => {
          handlers.onRefresh();
          onClose();
        },
      },
      {
        id: "act-layout",
        group: "Actions",
        label: "Customize layout",
        hint: "⌘,",
        run: () => {
          handlers.onCustomize();
          onClose();
        },
      },
      {
        id: "act-theme",
        group: "Actions",
        label: `Theme: ${handlers.themeLabel}`,
        hint: "Cycle auto / dusk / light",
        run: () => {
          handlers.onToggleTheme();
          onClose();
        },
      },
      {
        id: "act-note",
        group: "Actions",
        label: query.trim() ? `Note: ${noteTitle}` : "Capture a note",
        hint: "Uses the search text as the title",
        run: async () => {
          await createNote(noteTitle, "");
          requestDashboardRefresh();
          onClose();
        },
      },
    ].filter((item) => matches(query, item.label, item.hint));

    return [...goTo, ...shortcutItems, ...actions];
  }, [query, shortcuts, handlers, onClose]);

  useEffect(() => {
    setActive(0);
  }, [query, items.length]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (items.length === 0) return;
        setActive((i) => Math.min(items.length - 1, i + 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (items.length === 0) return;
        setActive((i) => Math.max(0, i - 1));
      } else if (e.key === "Enter") {
        e.preventDefault();
        const item = items[active];
        if (item) void runItem(item);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, items, active, onClose]);

  async function runItem(item: Item) {
    try {
      await item.run();
    } catch (e) {
      setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  if (!open) return null;

  let lastGroup = "";

  return (
    <div className="palette-root" role="dialog" aria-modal="true" aria-label="Command palette">
      <button
        type="button"
        className="palette-backdrop"
        aria-label="Close command palette"
        onClick={onClose}
      />
      <div className="palette-panel">
        <input
          ref={inputRef}
          className="palette-input"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Jump to a module, open a shortcut, or capture a note"
          aria-label="Command search"
        />
        <ul className="palette-list">
          {items.length === 0 ? (
            <li className="palette-empty">Nothing matches.</li>
          ) : (
            items.map((item, index) => {
              const showGroup = item.group !== lastGroup;
              lastGroup = item.group;
              return (
                <li key={item.id}>
                  {showGroup ? (
                    <p className="palette-group">{item.group}</p>
                  ) : null}
                  <button
                    type="button"
                    className={`palette-item ${index === active ? "is-active" : ""}`}
                    onMouseEnter={() => setActive(index)}
                    onClick={() => void runItem(item)}
                  >
                    <span className="palette-item-label">{item.label}</span>
                    {item.hint ? (
                      <span className="palette-item-hint">{item.hint}</span>
                    ) : null}
                  </button>
                </li>
              );
            })
          )}
        </ul>
        {status ? <p className="palette-status">{status}</p> : null}
      </div>
    </div>
  );
}
