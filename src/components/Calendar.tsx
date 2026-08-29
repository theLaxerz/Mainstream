import { useEffect, useMemo, useState } from "react";
import {
  eventCoversLocalDay,
  formatAgendaTime,
  listCalendarEvents,
  openCalendarEvent,
  type CalendarEvent,
} from "../lib/calendar";
import { isTauriRuntime, previewCalendarEvents } from "../lib/browserPreview";
import { onDashboardRefresh } from "../lib/refresh";
import "./Calendar.css";

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;

function startOfMonth(d: Date) {
  return new Date(d.getFullYear(), d.getMonth(), 1);
}

function sameDay(a: Date, b: Date) {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

function buildCells(view: Date): Date[] {
  const first = startOfMonth(view);
  const startOffset = first.getDay();
  const gridStart = new Date(first);
  gridStart.setDate(first.getDate() - startOffset);

  return Array.from({ length: 42 }, (_, i) => {
    const day = new Date(gridStart);
    day.setDate(gridStart.getDate() + i);
    return day;
  });
}

function rangeForView(view: Date, today: Date) {
  const first = startOfMonth(view);
  const last = new Date(view.getFullYear(), view.getMonth() + 1, 0);
  const ms = 24 * 60 * 60 * 1000;
  const daysBack = Math.max(
    0,
    Math.ceil((today.getTime() - first.getTime()) / ms) + 7,
  );
  const daysAhead = Math.max(
    1,
    Math.ceil((last.getTime() - today.getTime()) / ms) + 7,
  );
  return {
    daysBack: Math.min(90, daysBack),
    daysAhead: Math.min(90, daysAhead),
  };
}

export function Calendar() {
  const today = useMemo(() => {
    const n = new Date();
    return new Date(n.getFullYear(), n.getMonth(), n.getDate());
  }, []);

  const [view, setView] = useState(() => startOfMonth(today));
  const [selected, setSelected] = useState<Date>(today);
  const [events, setEvents] = useState<CalendarEvent[]>([]);

  const cells = useMemo(() => buildCells(view), [view]);
  const { daysBack, daysAhead } = rangeForView(view, today);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const result = await listCalendarEvents(200, daysAhead, daysBack);
        if (!cancelled && result.access.status === "ok") {
          setEvents(result.events);
        } else if (!cancelled && !isTauriRuntime()) {
          setEvents(previewCalendarEvents());
        }
      } catch {
        if (!cancelled) {
          setEvents(isTauriRuntime() ? [] : previewCalendarEvents());
        }
      }
    }
    void load();
    return onDashboardRefresh(() => void load());
  }, [daysAhead, daysBack]);

  const monthLabel = new Intl.DateTimeFormat(undefined, {
    month: "long",
    year: "numeric",
  }).format(view);

  const selectedEvents = events.filter((event) =>
    eventCoversLocalDay(event, selected),
  );
  const selectedLabel = new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "short",
    day: "numeric",
  }).format(selected);

  function shiftMonth(delta: number) {
    setView((prev) => new Date(prev.getFullYear(), prev.getMonth() + delta, 1));
  }

  function goToday() {
    setView(startOfMonth(today));
    setSelected(today);
  }

  async function onOpenEvent(event: CalendarEvent) {
    try {
      await openCalendarEvent(event.start);
    } catch {
      /* Calendar.app deep-link is best-effort */
    }
  }

  return (
    <div className="calendar-panel" aria-label={`Calendar for ${monthLabel}`}>
      <div className="calendar-toolbar">
        <button
          type="button"
          className="calendar-nav"
          aria-label="Previous month"
          onClick={() => shiftMonth(-1)}
        >
          ‹
        </button>
        <div className="calendar-heading">
          <p className="calendar-month">{monthLabel}</p>
          <button type="button" className="calendar-today" onClick={goToday}>
            Today
          </button>
        </div>
        <button
          type="button"
          className="calendar-nav"
          aria-label="Next month"
          onClick={() => shiftMonth(1)}
        >
          ›
        </button>
      </div>

      <div className="calendar-weekdays" aria-hidden="true">
        {WEEKDAYS.map((d) => (
          <span key={d} className="calendar-weekday">
            {d}
          </span>
        ))}
      </div>

      <div className="calendar-grid" role="grid" aria-label={monthLabel}>
        {cells.map((day) => {
          const inMonth = day.getMonth() === view.getMonth();
          const isToday = sameDay(day, today);
          const isSelected = sameDay(day, selected);
          const hasEvents = events.some((event) => eventCoversLocalDay(event, day));
          const label = new Intl.DateTimeFormat(undefined, {
            weekday: "long",
            month: "long",
            day: "numeric",
            year: "numeric",
          }).format(day);

          return (
            <button
              key={day.toISOString()}
              type="button"
              role="gridcell"
              className={[
                "calendar-day",
                inMonth ? "in-month" : "out-month",
                isToday ? "is-today" : "",
                isSelected ? "is-selected" : "",
                hasEvents ? "has-events" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              aria-label={hasEvents ? `${label}, has events` : label}
              aria-current={isToday ? "date" : undefined}
              aria-pressed={isSelected}
              onClick={() => setSelected(day)}
            >
              <span className="calendar-day-num">{day.getDate()}</span>
              {hasEvents ? <span className="calendar-day-dot" /> : null}
            </button>
          );
        })}
      </div>

      <div className="calendar-agenda" aria-live="polite">
        <p className="calendar-agenda-label">{selectedLabel}</p>
        {selectedEvents.length === 0 ? (
          <p className="calendar-agenda-empty">Nothing scheduled</p>
        ) : (
          <ul className="calendar-agenda-list">
            {selectedEvents.slice(0, 3).map((event) => (
              <li key={event.id}>
                <button
                  type="button"
                  className="calendar-agenda-item"
                  onClick={() => void onOpenEvent(event)}
                >
                  <span className="calendar-agenda-time">
                    {formatAgendaTime(event)}
                  </span>
                  <span className="calendar-agenda-title">{event.title}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
        {selectedEvents.length > 3 ? (
          <p className="calendar-agenda-more">
            +{selectedEvents.length - 3} more
          </p>
        ) : null}
      </div>
    </div>
  );
}
