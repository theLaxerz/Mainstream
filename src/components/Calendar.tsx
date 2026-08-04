import { useMemo, useState } from "react";
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

export function Calendar() {
  const today = useMemo(() => {
    const n = new Date();
    return new Date(n.getFullYear(), n.getMonth(), n.getDate());
  }, []);

  const [view, setView] = useState(() => startOfMonth(today));
  const [selected, setSelected] = useState<Date>(today);

  const cells = useMemo(() => buildCells(view), [view]);

  const monthLabel = new Intl.DateTimeFormat(undefined, {
    month: "long",
    year: "numeric",
  }).format(view);

  function shiftMonth(delta: number) {
    setView((prev) => new Date(prev.getFullYear(), prev.getMonth() + delta, 1));
  }

  function goToday() {
    setView(startOfMonth(today));
    setSelected(today);
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
              ]
                .filter(Boolean)
                .join(" ")}
              aria-label={label}
              aria-current={isToday ? "date" : undefined}
              aria-pressed={isSelected}
              onClick={() => setSelected(day)}
            >
              <span className="calendar-day-num">{day.getDate()}</span>
              {isToday ? <span className="calendar-day-dot" /> : null}
            </button>
          );
        })}
      </div>
    </div>
  );
}
