import { invoke } from "@tauri-apps/api/core";

export type CalendarAccessStatus = "ok" | "needs_permission" | "unavailable" | "error";

export type CalendarAccess = {
  status: CalendarAccessStatus;
  detail: string | null;
};

export type CalendarEvent = {
  id: string;
  title: string;
  start: string;
  end: string;
  isAllDay: boolean;
  location: string | null;
  calendarName: string | null;
};

export type CalendarEventsResult = {
  access: CalendarAccess;
  events: CalendarEvent[];
};

export async function listCalendarEvents(
  limit?: number,
  daysAhead?: number,
  daysBack?: number,
): Promise<CalendarEventsResult> {
  return invoke("list_calendar_events", {
    limit: limit ?? null,
    daysAhead: daysAhead ?? null,
    daysBack: daysBack ?? null,
  });
}

export async function calendarAccessStatus(): Promise<CalendarAccess> {
  return invoke("calendar_access_status");
}

export async function openCalendarPrivacySettings(): Promise<void> {
  return invoke("open_calendar_privacy_settings");
}

export async function openCalendarEvent(startIso: string): Promise<void> {
  return invoke("open_calendar_event", { startIso });
}

export function formatEventWhen(event: CalendarEvent): string {
  const start = new Date(event.start);
  const end = new Date(event.end);
  if (Number.isNaN(start.getTime())) return event.start;

  if (event.isAllDay) {
    return new Intl.DateTimeFormat(undefined, {
      weekday: "short",
      month: "short",
      day: "numeric",
    }).format(start);
  }

  const sameDay =
    start.getFullYear() === end.getFullYear() &&
    start.getMonth() === end.getMonth() &&
    start.getDate() === end.getDate();

  const dateFmt = new Intl.DateTimeFormat(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  });
  const timeFmt = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });

  if (sameDay) {
    return `${dateFmt.format(start)} · ${timeFmt.format(start)} – ${timeFmt.format(end)}`;
  }

  return `${dateFmt.format(start)} ${timeFmt.format(start)} → ${dateFmt.format(end)} ${timeFmt.format(end)}`;
}

export function eventMeta(event: CalendarEvent): string {
  const parts = [event.calendarName, event.location].filter(Boolean);
  return parts.join(" · ");
}

export function startOfLocalDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

export function eventCoversLocalDay(event: CalendarEvent, day: Date): boolean {
  const start = new Date(event.start);
  if (Number.isNaN(start.getTime())) return false;
  const endRaw = new Date(event.end);
  const end = Number.isNaN(endRaw.getTime())
    ? new Date(start.getTime() + 1)
    : endRaw;
  const dayStart = startOfLocalDay(day).getTime();
  const dayEnd = dayStart + 24 * 60 * 60 * 1000;
  return start.getTime() < dayEnd && end.getTime() > dayStart;
}

export function formatAgendaTime(event: CalendarEvent): string {
  if (event.isAllDay) return "All day";
  const start = new Date(event.start);
  if (Number.isNaN(start.getTime())) return "";
  return new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  }).format(start);
}

export function nextUpcomingEvent(
  events: CalendarEvent[],
  now = new Date(),
): CalendarEvent | null {
  const nowMs = now.getTime();
  const upcoming = events
    .filter((event) => {
      const end = new Date(event.end);
      const start = new Date(event.start);
      const endMs = Number.isNaN(end.getTime()) ? start.getTime() : end.getTime();
      return Number.isFinite(endMs) && endMs >= nowMs;
    })
    .sort(
      (a, b) => new Date(a.start).getTime() - new Date(b.start).getTime(),
    );
  return upcoming[0] ?? null;
}
