import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { Calendar } from "../components/Calendar";
import { CalendarSection } from "../components/CalendarSection";
import { Clock } from "../components/Clock";
import { CommandBar } from "../components/CommandBar";
import { EmailSection } from "../components/EmailSection";
import { FinanceSection } from "../components/FinanceSection";
import { HealthSection } from "../components/HealthSection";
import { HomeSection } from "../components/HomeSection";
import { LayoutCustomize } from "../components/LayoutCustomize";
import { MailSection } from "../components/MailSection";
import { MessagesSection } from "../components/MessagesSection";
import { NewsSection } from "../components/NewsSection";
import { NotesSection } from "../components/NotesSection";
import { ShortcutsSection } from "../components/ShortcutsSection";
import { StreamingSection } from "../components/StreamingSection";
import { TodayBriefing } from "../components/TodayBriefing";
import { YouTubeSection } from "../components/YouTubeSection";
import {
  defaultLayout,
  enabledModules,
  loadDashboardLayout,
  type DashboardLayout,
  type ModuleId,
  type ModuleLayoutEntry,
} from "../lib/moduleLayout";
import { refreshDashboard } from "../lib/api";
import { requestDashboardRefresh } from "../lib/refresh";
import "./Dashboard.css";

function renderModule(entry: ModuleLayoutEntry): ReactNode {
  const { id, listLimit } = entry;
  switch (id) {
    case "messages":
      return <MessagesSection limit={listLimit} />;
    case "calendar":
      return <CalendarSection limit={listLimit} />;
    case "email":
      return <EmailSection limit={listLimit} />;
    case "mail":
      return <MailSection limit={listLimit} />;
    case "news":
      return <NewsSection limit={listLimit} />;
    case "finance":
      return <FinanceSection limit={listLimit} />;
    case "notes":
      return <NotesSection limit={listLimit} />;
    case "health":
      return <HealthSection limit={listLimit} />;
    case "home":
      return <HomeSection limit={listLimit} />;
    case "youtube":
      return <YouTubeSection limit={listLimit} />;
    case "streaming":
      return <StreamingSection limit={listLimit} />;
    case "shortcuts":
      return <ShortcutsSection limit={listLimit} />;
    default: {
      const _exhaustive: never = id;
      return _exhaustive;
    }
  }
}

export function Dashboard() {
  const [layout, setLayout] = useState<DashboardLayout>(() => defaultLayout());
  const [layoutReady, setLayoutReady] = useState(false);
  const [customizeOpen, setCustomizeOpen] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [refreshStatus, setRefreshStatus] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void loadDashboardLayout().then((next) => {
      if (!cancelled) {
        setLayout(next);
        setLayoutReady(true);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const modules = useMemo(() => enabledModules(layout), [layout]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    setRefreshStatus(null);
    try {
      const result = await refreshDashboard();
      requestDashboardRefresh();
      const synced = result.modules.filter((m) => m.status === "ok").length;
      const errors = result.modules.filter((m) => m.status === "error").length;
      setRefreshStatus(
        errors > 0
          ? `Synced ${synced} module(s) · ${errors} error(s)`
          : `Synced ${synced} module(s)`,
      );
    } catch (e) {
      setRefreshStatus(e instanceof Error ? e.message : String(e));
    } finally {
      setRefreshing(false);
    }
  }, []);

  const onCustomize = useCallback(() => setCustomizeOpen(true), []);

  return (
    <div className="dashboard">
      <CommandBar
        onRefresh={onRefresh}
        onCustomize={onCustomize}
        refreshing={refreshing}
        refreshStatus={refreshStatus}
      />

      <header className="dashboard-hero">
        <p className="brand">Mainstream</p>
        <div className="hero-time-row">
          <Clock />
          <Calendar />
        </div>
      </header>

      <TodayBriefing />

      <div className="dashboard-modules-head">
        <div>
          <p className="dashboard-modules-eyebrow">Command center</p>
          <h2 className="dashboard-modules-title">Live modules</h2>
        </div>
        <p className="dashboard-modules-meta">
          {layoutReady
            ? `${modules.length} active · customize anytime`
            : "Loading layout…"}
        </p>
      </div>

      <div className="dashboard-grid">
        {modules.map((entry, index) => (
          <div
            key={entry.id as ModuleId}
            id={`module-${entry.id}`}
            className={`dashboard-module-slot placement-${entry.placement}`}
            style={{ animationDelay: `${0.04 + index * 0.045}s` }}
          >
            {renderModule(entry)}
          </div>
        ))}
      </div>

      {modules.length === 0 ? (
        <p className="dashboard-empty-layout">
          All modules are hidden. Open Layout to turn some back on.
        </p>
      ) : null}

      <div className="dashboard-footer">
        <button
          type="button"
          className="btn btn-ghost"
          onClick={onCustomize}
        >
          Customize layout
        </button>
        <button
          type="button"
          className="btn btn-primary dashboard-refresh"
          onClick={onRefresh}
          disabled={refreshing}
        >
          {refreshing ? "Refreshing…" : "Refresh all"}
        </button>
      </div>

      <LayoutCustomize
        open={customizeOpen}
        layout={layout}
        onClose={() => setCustomizeOpen(false)}
        onChange={setLayout}
      />
    </div>
  );
}
