import type { Task, TaskSummary } from "./types";

export type TaskBucket = "overdue" | "today" | "upcoming" | "someday" | "done";

export type TaskGroup = {
  bucket: TaskBucket;
  label: string;
  tasks: Task[];
};

const BUCKET_ORDER: TaskBucket[] = [
  "overdue",
  "today",
  "upcoming",
  "someday",
  "done",
];

const BUCKET_LABEL: Record<TaskBucket, string> = {
  overdue: "Overdue",
  today: "Today",
  upcoming: "Upcoming",
  someday: "Someday",
  done: "Done",
};

export function localDateKey(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function shiftLocalDate(days: number, from = new Date()): string {
  const next = new Date(from.getFullYear(), from.getMonth(), from.getDate() + days);
  return localDateKey(next);
}

export function parseLocalDate(key: string): Date {
  const [y, m, d] = key.split("-").map(Number);
  return new Date(y, (m ?? 1) - 1, d ?? 1);
}

export function taskBucket(task: Task, today = localDateKey(new Date())): TaskBucket {
  if (task.completed) return "done";
  if (!task.dueOn) return "someday";
  if (task.dueOn < today) return "overdue";
  if (task.dueOn === today) return "today";
  return "upcoming";
}

export function sortTasks(tasks: Task[], today = localDateKey(new Date())): Task[] {
  const rank = (task: Task) => BUCKET_ORDER.indexOf(taskBucket(task, today));
  return [...tasks].sort((a, b) => {
    const bucketDelta = rank(a) - rank(b);
    if (bucketDelta !== 0) return bucketDelta;
    if (a.priority !== b.priority) return b.priority - a.priority;
    if (a.dueOn && b.dueOn && a.dueOn !== b.dueOn) {
      return a.dueOn < b.dueOn ? -1 : 1;
    }
    if (a.dueOn && !b.dueOn) return -1;
    if (!a.dueOn && b.dueOn) return 1;
    return a.id - b.id;
  });
}

export function groupTasks(tasks: Task[], today = localDateKey(new Date())): TaskGroup[] {
  const groups = new Map<TaskBucket, Task[]>();
  for (const bucket of BUCKET_ORDER) {
    groups.set(bucket, []);
  }
  for (const task of sortTasks(tasks, today)) {
    groups.get(taskBucket(task, today))!.push(task);
  }
  return BUCKET_ORDER.filter((bucket) => (groups.get(bucket)?.length ?? 0) > 0).map(
    (bucket) => ({
      bucket,
      label: BUCKET_LABEL[bucket],
      tasks: groups.get(bucket)!,
    }),
  );
}

export function formatDueOn(dueOn: string | null, today = localDateKey(new Date())): string {
  if (!dueOn) return "No date";
  if (dueOn === today) return "Today";
  if (dueOn === shiftLocalDate(-1, parseLocalDate(today))) return "Yesterday";
  if (dueOn === shiftLocalDate(1, parseLocalDate(today))) return "Tomorrow";
  const date = parseLocalDate(dueOn);
  if (Number.isNaN(date.getTime())) return dueOn;
  return new Intl.DateTimeFormat(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  }).format(date);
}

export function summarizeTasks(tasks: Task[], today = localDateKey(new Date())): TaskSummary {
  const open = tasks.filter((t) => !t.completed);
  return {
    open: open.length,
    overdue: open.filter((t) => taskBucket(t, today) === "overdue").length,
    dueToday: open.filter((t) => taskBucket(t, today) === "today").length,
    upcoming: open.filter((t) => taskBucket(t, today) === "upcoming").length,
  };
}

export function briefingDetail(summary: TaskSummary): string {
  if (summary.overdue > 0) {
    return `${summary.overdue} overdue`;
  }
  if (summary.dueToday > 0) {
    return `${summary.dueToday} due today`;
  }
  if (summary.open > 0) {
    return `${summary.open} open`;
  }
  return "All clear";
}
