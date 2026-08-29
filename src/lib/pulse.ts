import { healthTodaySummary, listImportantEmails } from "./api";
import { isTauriRuntime, previewCalendarEvents, previewFinanceSummary } from "./browserPreview";
import {
  listCalendarEvents,
  nextUpcomingEvent,
  type CalendarEvent,
} from "./calendar";
import { formatMoney, getFinanceSummary } from "./finance";
import { groupUnreadByChat } from "./messageGroups";
import { listAllUnreadMessages } from "./messages";

export type PulseChipKind =
  | "calendar"
  | "messages"
  | "email"
  | "health"
  | "finance";

export type PulseChip = {
  kind: PulseChipKind;
  label: string;
  detail: string;
  moduleId: PulseChipKind;
};

export type DashboardPulse = {
  greeting: string;
  nextEvent: CalendarEvent | null;
  unreadChats: number;
  importantEmail: number;
  steps: number | null;
  spentThisMonth: number | null;
  chips: PulseChip[];
};

export function greetingFor(now: Date): string {
  const hour = now.getHours();
  if (hour < 5) return "Still going";
  if (hour < 12) return "Good morning";
  if (hour < 17) return "Good afternoon";
  if (hour < 21) return "Good evening";
  return "Good night";
}

function formatChipTime(event: CalendarEvent): string {
  if (event.isAllDay) return "All day";
  const start = new Date(event.start);
  if (Number.isNaN(start.getTime())) return event.title;
  const today = new Date();
  const sameDay =
    start.getFullYear() === today.getFullYear() &&
    start.getMonth() === today.getMonth() &&
    start.getDate() === today.getDate();
  const time = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  }).format(start);
  if (sameDay) return time;
  const date = new Intl.DateTimeFormat(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  }).format(start);
  return `${date} · ${time}`;
}

export async function loadDashboardPulse(): Promise<DashboardPulse> {
  if (!isTauriRuntime()) {
    const events = previewCalendarEvents();
    const finance = previewFinanceSummary();
    const nextEvent = nextUpcomingEvent(events);
    const chips: PulseChip[] = [];
    if (nextEvent) {
      chips.push({
        kind: "calendar",
        label: "Next up",
        detail: `${formatChipTime(nextEvent)} · ${nextEvent.title}`,
        moduleId: "calendar",
      });
    }
    chips.push({
      kind: "finance",
      label: "Spent",
      detail: `${formatMoney(finance.spentThisMonth)} this month`,
      moduleId: "finance",
    });
    return {
      greeting: greetingFor(new Date()),
      nextEvent,
      unreadChats: 0,
      importantEmail: 0,
      steps: null,
      spentThisMonth: finance.spentThisMonth,
      chips,
    };
  }
  const settled = await Promise.allSettled([
    listCalendarEvents(24, 14, 0),
    listAllUnreadMessages(),
    listImportantEmails(20),
    healthTodaySummary(),
    getFinanceSummary(),
  ]);

  const calendar =
    settled[0].status === "fulfilled" ? settled[0].value : null;
  const messages =
    settled[1].status === "fulfilled" ? settled[1].value : null;
  const emails = settled[2].status === "fulfilled" ? settled[2].value : [];
  const health = settled[3].status === "fulfilled" ? settled[3].value : null;
  const finance = settled[4].status === "fulfilled" ? settled[4].value : null;

  const nextEvent =
    calendar && calendar.access.status === "ok"
      ? nextUpcomingEvent(calendar.events)
      : null;

  const unreadChats =
    messages && messages.access.status === "ok"
      ? groupUnreadByChat(messages.messages).length
      : 0;

  const importantEmail = Array.isArray(emails) ? emails.length : 0;
  const steps = health ? health.steps : null;
  const spentThisMonth = finance ? finance.spentThisMonth : null;

  const chips: PulseChip[] = [];

  if (nextEvent) {
    chips.push({
      kind: "calendar",
      label: "Next up",
      detail: `${formatChipTime(nextEvent)} · ${nextEvent.title}`,
      moduleId: "calendar",
    });
  } else if (calendar?.access.status === "ok") {
    chips.push({
      kind: "calendar",
      label: "Calendar",
      detail: "Nothing upcoming",
      moduleId: "calendar",
    });
  }

  if (messages?.access.status === "ok") {
    chips.push({
      kind: "messages",
      label: "Messages",
      detail:
        unreadChats === 0
          ? "Inbox zero"
          : `${unreadChats} unread chat${unreadChats === 1 ? "" : "s"}`,
      moduleId: "messages",
    });
  }

  if (importantEmail > 0) {
    chips.push({
      kind: "email",
      label: "Email",
      detail: `${importantEmail} important`,
      moduleId: "email",
    });
  }

  if (steps !== null) {
    chips.push({
      kind: "health",
      label: "Health",
      detail: `${steps.toLocaleString()} steps`,
      moduleId: "health",
    });
  }

  if (spentThisMonth !== null && spentThisMonth > 0) {
    chips.push({
      kind: "finance",
      label: "Spent",
      detail: `${formatMoney(spentThisMonth)} this month`,
      moduleId: "finance",
    });
  }

  return {
    greeting: greetingFor(new Date()),
    nextEvent,
    unreadChats,
    importantEmail,
    steps,
    spentThisMonth,
    chips,
  };
}
