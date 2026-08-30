import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { Calendar } from "../components/Calendar";
import { CalendarSection } from "../components/CalendarSection";
import { Clock } from "../components/Clock";
import { CommandBar } from "../components/CommandBar";
import { CommandPalette } from "../components/CommandPalette";
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
import { TasksSection } from "../components/TasksSection";
import { TodayBriefing } from "../components/TodayBriefing";
import { Weather } from "../components/Weather";
import { YouTubeSection } from "../components/YouTubeSection";
import {
  applyTheme,
  cycleTheme,
  loadRefreshIntervalMinutes,
  loadThemePreference,
  saveThemePreference,
  themeButtonLabel,
  type ThemePreference,
} from "../lib/appearance";
import { refreshDashboard } from "../lib/api";
import {
  defaultLayout,
  enabledModules,
  loadDashboardLayout,
  type DashboardLayout,
  type ModuleId,
  type ModuleLayoutEntry,
} from "../lib/moduleLayout";
import { requestDashboardRefresh } from "../lib/refresh";
import {
  buildSyncState,
  ModuleSyncScope,
  SyncStateProvider,
  type SyncState,
} from "../lib/syncStatus";
import "./Dashboard.css";

function renderModule(entry: ModuleLayoutEntry): ReactNode {
  const { id, listLimit } = entry;
  switch (id) {
    case "messages":
      return <MessagesSection limit={listLimit} />;
    case "calendar":
      return <CalendarSection limit={listLimit} />;
    case "tasks":
      return <TasksSection limit={listLimit} />;
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
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [refreshStatus, setRefreshStatus] = useState<string | null>(null);
  const [syncState, setSyncState] = useState<SyncState>({
    finishedAt: null,
    byModule: {},
  });
  const [themePref, setThemePref] = useState<ThemePreference>("auto");

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

  useEffect(() => {
    let cancelled = false;
    void loadThemePreference().then((pref) => {
      if (cancelled) return;
      setThemePref(pref);
      applyTheme(pref);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    applyTheme(themePref);
    if (themePref !== "auto") return;
    const id = window.setInterval(() => applyTheme("auto"), 60_000);
    return () => window.clearInterval(id);
  }, [themePref]);

  const modules = useMemo(() => enabledModules(layout), [layout]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      const result = await refreshDashboard();
      requestDashboardRefresh();
      setSyncState(buildSyncState(result.finishedAt, result.modules));
      const synced = result.modules.filter((m) => m.status === "ok").length;
      const errors = result.modules.filter((m) => m.status === "error").length;
      setRefreshStatus(
        errors > 0
          ? `Synced ${synced} · ${errors} error(s)`
          : `Synced ${synced} module(s)`,
      );
    } catch (e) {
      setRefreshStatus(e instanceof Error ? e.message : String(e));
    } finally {
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;
    void loadRefreshIntervalMinutes().then((mins) => {
      if (cancelled || mins <= 0) return;
      timer = window.setInterval(() => {
        if (document.hidden) return;
        void onRefresh();
      }, mins * 60_000);
    });
    return () => {
      cancelled = true;
      if (timer) window.clearInterval(timer);
    };
  }, [onRefresh]);

  const onCustomize = useCallback(() => setCustomizeOpen(true), []);
  const onClosePalette = useCallback(() => setPaletteOpen(false), []);
  const onPalette = useCallback(() => setPaletteOpen((open) => !open), []);
  const onToggleTheme = useCallback(() => {
    const next = cycleTheme(themePref);
    setThemePref(next);
    void saveThemePreference(next);
  }, [themePref]);

  const paletteHandlers = useMemo(
    () => ({
      onRefresh,
      onCustomize,
      onToggleTheme,
      themeLabel: themeButtonLabel(themePref),
    }),
    [onRefresh, onCustomize, onToggleTheme, themePref],
  );

  return (
    <SyncStateProvider value={syncState}>
      <div className="dashboard">
        <CommandBar
          onRefresh={onRefresh}
          onCustomize={onCustomize}
          onPalette={onPalette}
          onToggleTheme={onToggleTheme}
          themeLabel={themeButtonLabel(themePref)}
          refreshing={refreshing}
          refreshStatus={refreshStatus}
        />

        <header className="dashboard-hero">
          <p className="brand">Mainstream</p>
          <div className="hero-time-row">
            <Clock />
            <Weather />
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
              <ModuleSyncScope id={entry.id}>
                {renderModule(entry)}
              </ModuleSyncScope>
            </div>
          ))}
        </div>

        {modules.length === 0 ? (
          <p className="dashboard-empty-layout">
            All modules are hidden. Open Layout to turn some back on.
          </p>
        ) : null}

        <p className="dashboard-palette-hint">⌘K command palette · ⌘⇧R refresh</p>

        <LayoutCustomize
          open={customizeOpen}
          layout={layout}
          onClose={() => setCustomizeOpen(false)}
          onChange={setLayout}
        />
        <CommandPalette
          open={paletteOpen}
          onClose={onClosePalette}
          handlers={paletteHandlers}
        />
      </div>
    </SyncStateProvider>
  );
}
