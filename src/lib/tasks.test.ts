import {
  briefingDetail,
  formatDueOn,
  groupTasks,
  localDateKey,
  shiftLocalDate,
  sortTasks,
  summarizeTasks,
  taskBucket,
} from "./taskLogic.ts";
import type { Task } from "./types.ts";

function assertEqual(actual: unknown, expected: unknown, label: string) {
  if (actual !== expected) {
    throw new Error(
      `${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
}

function task(partial: Partial<Task> & Pick<Task, "id" | "title">): Task {
  return {
    notes: "",
    dueOn: null,
    priority: 0,
    completed: false,
    completedAt: null,
    createdAt: "2026-08-30T00:00:00.000Z",
    updatedAt: "2026-08-30T00:00:00.000Z",
    ...partial,
  };
}

const today = "2026-08-30";

assertEqual(localDateKey(new Date(2026, 7, 30)), "2026-08-30", "localDateKey");
assertEqual(shiftLocalDate(1, new Date(2026, 7, 30)), "2026-08-31", "tomorrow");
assertEqual(shiftLocalDate(-1, new Date(2026, 7, 30)), "2026-08-29", "yesterday");

assertEqual(
  taskBucket(task({ id: 1, title: "a", dueOn: "2026-08-29" }), today),
  "overdue",
  "overdue bucket",
);
assertEqual(
  taskBucket(task({ id: 2, title: "b", dueOn: today }), today),
  "today",
  "today bucket",
);
assertEqual(
  taskBucket(task({ id: 3, title: "c", dueOn: "2026-09-02" }), today),
  "upcoming",
  "upcoming bucket",
);
assertEqual(
  taskBucket(task({ id: 4, title: "d", dueOn: null }), today),
  "someday",
  "someday bucket",
);
assertEqual(
  taskBucket(task({ id: 5, title: "e", dueOn: today, completed: true }), today),
  "done",
  "done bucket",
);

const sorted = sortTasks(
  [
    task({ id: 10, title: "later", dueOn: "2026-09-04" }),
    task({ id: 11, title: "high today", dueOn: today, priority: 1 }),
    task({ id: 12, title: "overdue", dueOn: "2026-08-20" }),
    task({ id: 13, title: "done", dueOn: today, completed: true }),
    task({ id: 14, title: "someday" }),
  ],
  today,
).map((t) => t.id);

assertEqual(sorted.join(","), "12,11,10,14,13", "sort order");

const groups = groupTasks(
  [
    task({ id: 1, title: "over", dueOn: "2026-08-01" }),
    task({ id: 2, title: "today", dueOn: today }),
  ],
  today,
);
assertEqual(groups[0]?.bucket, "overdue", "first group overdue");
assertEqual(groups[1]?.bucket, "today", "second group today");

assertEqual(formatDueOn(today, today), "Today", "format today");
assertEqual(formatDueOn("2026-08-31", today), "Tomorrow", "format tomorrow");
assertEqual(formatDueOn("2026-08-29", today), "Yesterday", "format yesterday");
assertEqual(formatDueOn(null, today), "No date", "format none");

const summary = summarizeTasks(
  [
    task({ id: 1, title: "a", dueOn: "2026-08-01" }),
    task({ id: 2, title: "b", dueOn: today }),
    task({ id: 3, title: "c", dueOn: today }),
    task({ id: 4, title: "d", dueOn: "2026-09-01" }),
    task({ id: 5, title: "e", completed: true, dueOn: today }),
  ],
  today,
);
assertEqual(summary.open, 4, "open count");
assertEqual(summary.overdue, 1, "overdue count");
assertEqual(summary.dueToday, 2, "due today count");
assertEqual(summary.upcoming, 1, "upcoming count");
assertEqual(briefingDetail(summary), "1 overdue", "briefing overdue wins");
assertEqual(
  briefingDetail({ open: 2, overdue: 0, dueToday: 2, upcoming: 0 }),
  "2 due today",
  "briefing today",
);
assertEqual(
  briefingDetail({ open: 3, overdue: 0, dueToday: 0, upcoming: 1 }),
  "3 open",
  "briefing open",
);
assertEqual(
  briefingDetail({ open: 0, overdue: 0, dueToday: 0, upcoming: 0 }),
  "All clear",
  "briefing clear",
);

console.log("tasks tests ok");
