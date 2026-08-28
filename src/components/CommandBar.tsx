import { useEffect, useRef, useState } from "react";
import "./CommandBar.css";

function scrollParentOf(el: HTMLElement | null): Element | Window {
  let node = el?.parentElement ?? null;
  while (node) {
    const { overflowY } = getComputedStyle(node);
    if (overflowY === "auto" || overflowY === "scroll") return node;
    node = node.parentElement;
  }
  return window;
}

type Props = {
  onRefresh: () => void;
  onCustomize: () => void;
  onPalette: () => void;
  onToggleTheme: () => void;
  themeLabel: string;
  refreshing?: boolean;
  refreshStatus?: string | null;
};

export function CommandBar({
  onRefresh,
  onCustomize,
  onPalette,
  onToggleTheme,
  themeLabel,
  refreshing,
  refreshStatus,
}: Props) {
  const barRef = useRef<HTMLDivElement>(null);
  const [now, setNow] = useState(() => new Date());
  const [compact, setCompact] = useState(false);

  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    const scroller = scrollParentOf(barRef.current);
    const readY = () =>
      scroller instanceof Element ? scroller.scrollTop : window.scrollY;
    const onScroll = () => setCompact(readY() > 220);
    onScroll();
    scroller.addEventListener("scroll", onScroll, { passive: true });
    return () => scroller.removeEventListener("scroll", onScroll);
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      if (meta && e.shiftKey && e.key.toLowerCase() === "r") {
        e.preventDefault();
        onRefresh();
      }
      if (meta && e.key === ",") {
        e.preventDefault();
        onCustomize();
      }
      if (meta && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onPalette();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onRefresh, onCustomize, onPalette]);

  const time = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
  }).format(now);

  const date = new Intl.DateTimeFormat(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  }).format(now);

  return (
    <div
      ref={barRef}
      className={`command-bar ${compact ? "is-compact" : ""}`}
    >
      <div className="command-bar-inner">
        <div className="command-bar-brand">
          <span className="command-bar-mark">Mainstream</span>
          <span className="command-bar-clock" aria-live="polite">
            {date} · {time}
          </span>
          {refreshStatus ? (
            <span className="command-bar-status" role="status">
              {refreshStatus}
            </span>
          ) : null}
        </div>
        <div className="command-bar-actions">
          <button
            type="button"
            className="btn btn-ghost"
            onClick={onPalette}
            title="Command palette (⌘K)"
          >
            ⌘K
          </button>
          <button
            type="button"
            className="btn btn-ghost"
            onClick={onToggleTheme}
            title="Cycle theme: auto, dusk, light"
          >
            {themeLabel}
          </button>
          <button
            type="button"
            className="btn btn-ghost"
            onClick={onCustomize}
            title="Customize layout (⌘,)"
          >
            Layout
          </button>
          <button
            type="button"
            className="btn btn-primary"
            onClick={onRefresh}
            disabled={refreshing}
            title="Refresh all modules (⌘⇧R)"
          >
            {refreshing ? "Refreshing…" : "Refresh all"}
          </button>
        </div>
      </div>
    </div>
  );
}
