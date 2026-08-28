import { useEffect, useState } from "react";
import {
  formatEventWhen,
  eventMeta,
  listCalendarEvents,
  openCalendarEvent,
  openCalendarPrivacySettings,
  type CalendarEvent,
} from "../lib/calendar";
import { onDashboardRefresh } from "../lib/refresh";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import { PermissionCallout } from "./PermissionCallout";

type Props = { limit?: number };

export function CalendarSection({ limit = 10 }: Props) {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [accessStatus, setAccessStatus] = useState<string | null>(null);
  const [accessDetail, setAccessDetail] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [drawerOpen, setDrawerOpen] = useState(false);

  async function refresh() {
    try {
      const result = await listCalendarEvents(Math.max(limit, 50), 21);
      setEvents(result.events);
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
  }, [limit]);

  async function onOpenEvent(event: CalendarEvent) {
    try {
      await openCalendarEvent(event.start);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  const needsPermission = accessStatus === "needs_permission";
  const unavailable = accessStatus === "unavailable";
  const accessError = accessStatus === "error";

  function eventRows(rows: CalendarEvent[]) {
    return (
      <ul className="module-list">
        {rows.map((event) => (
          <li key={event.id}>
            <button
              type="button"
              className="module-row-main shortcut-open"
              onClick={() => void onOpenEvent(event)}
            >
              <p className="module-row-title">{event.title}</p>
              <p className="module-row-meta">
                {formatEventWhen(event)}
                {eventMeta(event) ? ` · ${eventMeta(event)}` : ""}
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
        title="Calendar"
        eyebrow="Upcoming"
        count={events.length || null}
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void refresh()}
            >
              Refresh
            </button>
            {events.length > limit ? (
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setDrawerOpen(true)}
              >
                See all
              </button>
            ) : null}
          </div>
        }
      >
        {loading ? <p className="module-empty">Loading events…</p> : null}
        {error ? <p className="module-empty">{error}</p> : null}
        {needsPermission ? (
          <PermissionCallout
            title="Calendar access needed"
            body="Mainstream needs Calendar access to show upcoming events. macOS should ask once; if no prompt appeared, enable Mainstream under Calendars."
            steps={[
              "Open System Settings → Privacy & Security → Calendars",
              "Enable Mainstream (or your terminal / IDE if running in dev)",
              "Return here and refresh",
            ]}
            actionLabel="Open Calendar privacy"
            onAction={() => void openCalendarPrivacySettings()}
          />
        ) : null}
        {unavailable ? (
          <p className="module-empty">
            {accessDetail ?? "Calendar is unavailable on this system."}
          </p>
        ) : null}
        {accessError ? (
          <p className="module-empty">
            {accessDetail ?? "Could not read calendars."}
          </p>
        ) : null}
        {!loading &&
        !needsPermission &&
        !unavailable &&
        !accessError &&
        events.length === 0 ? (
          <p className="module-empty">No upcoming events in the next few weeks.</p>
        ) : null}
        {!needsPermission && !unavailable && !accessError && events.length > 0
          ? eventRows(events.slice(0, limit))
          : null}
      </ModuleSection>

      <DetailDrawer
        open={drawerOpen}
        title="Calendar"
        eyebrow="Upcoming"
        onClose={() => setDrawerOpen(false)}
        wide
      >
        {eventRows(events)}
      </DetailDrawer>
    </>
  );
}
