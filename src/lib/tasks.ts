import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./browserPreview";
import {
  localDateKey,
  shiftLocalDate,
  sortTasks,
  summarizeTasks,
} from "./taskLogic";
import type { Task, TaskSummary } from "./types";

export {
  briefingDetail,
  formatDueOn,
  groupTasks,
  localDateKey,
  parseLocalDate,
  shiftLocalDate,
  sortTasks,
  summarizeTasks,
  taskBucket,
  type TaskBucket,
  type TaskGroup,
} from "./taskLogic";

type PreviewStore = {
  nextId: number;
  tasks: Task[];
};

function seedPreviewTasks(): Task[] {
  const today = new Date();
  const iso = today.toISOString();
  return [
    {
      id: 1,
      title: "Reply to the lease renewal",
      notes: "",
      dueOn: shiftLocalDate(-1, today),
      priority: 1,
      completed: false,
      completedAt: null,
      createdAt: iso,
      updatedAt: iso,
    },
    {
      id: 2,
      title: "Pick up dry cleaning",
      notes: "",
      dueOn: localDateKey(today),
      priority: 0,
      completed: false,
      completedAt: null,
      createdAt: iso,
      updatedAt: iso,
    },
    {
      id: 3,
      title: "Book dentist",
      notes: "Morning slot if possible",
      dueOn: shiftLocalDate(3, today),
      priority: 0,
      completed: false,
      completedAt: null,
      createdAt: iso,
      updatedAt: iso,
    },
  ];
}

let previewStore: PreviewStore | null = null;

function preview(): PreviewStore {
  if (!previewStore) {
    previewStore = { nextId: 4, tasks: seedPreviewTasks() };
  }
  return previewStore;
}

export function resetPreviewTasks(): void {
  previewStore = { nextId: 4, tasks: seedPreviewTasks() };
}

function nowIso(): string {
  return new Date().toISOString();
}

export async function listTasks(
  limit?: number,
  includeCompleted = true,
): Promise<Task[]> {
  if (!isTauriRuntime()) {
    const rows = includeCompleted
      ? preview().tasks
      : preview().tasks.filter((t) => !t.completed);
    return sortTasks(rows).slice(0, limit ?? 40);
  }
  return invoke("list_tasks", {
    limit: limit ?? null,
    includeCompleted,
  });
}

export async function getTaskSummary(): Promise<TaskSummary> {
  if (!isTauriRuntime()) {
    return summarizeTasks(preview().tasks);
  }
  return invoke("task_summary");
}

export async function createTask(input: {
  title: string;
  notes?: string;
  dueOn?: string | null;
  priority?: number;
}): Promise<Task> {
  if (!isTauriRuntime()) {
    const store = preview();
    const task: Task = {
      id: store.nextId++,
      title: input.title.trim(),
      notes: input.notes ?? "",
      dueOn: input.dueOn?.trim() || null,
      priority: (input.priority ?? 0) > 0 ? 1 : 0,
      completed: false,
      completedAt: null,
      createdAt: nowIso(),
      updatedAt: nowIso(),
    };
    store.tasks.push(task);
    return task;
  }
  return invoke("create_task", {
    input: {
      title: input.title,
      notes: input.notes ?? "",
      dueOn: input.dueOn ?? null,
      priority: input.priority ?? 0,
    },
  });
}

export async function updateTask(
  id: number,
  patch: {
    title?: string;
    notes?: string;
    dueOn?: string | null;
    priority?: number;
  },
): Promise<Task> {
  if (!isTauriRuntime()) {
    const store = preview();
    const task = store.tasks.find((t) => t.id === id);
    if (!task) throw new Error(`task ${id} not found`);
    if (patch.title !== undefined) task.title = patch.title.trim();
    if (patch.notes !== undefined) task.notes = patch.notes;
    if (patch.dueOn !== undefined) task.dueOn = patch.dueOn?.trim() || null;
    if (patch.priority !== undefined) task.priority = patch.priority > 0 ? 1 : 0;
    task.updatedAt = nowIso();
    return task;
  }
  return invoke("update_task", { input: { id, ...patch } });
}

export async function setTaskCompleted(id: number, completed: boolean): Promise<Task> {
  if (!isTauriRuntime()) {
    const store = preview();
    const task = store.tasks.find((t) => t.id === id);
    if (!task) throw new Error(`task ${id} not found`);
    task.completed = completed;
    task.completedAt = completed ? nowIso() : null;
    task.updatedAt = nowIso();
    return task;
  }
  return invoke("set_task_completed", { input: { id, completed } });
}

export async function deleteTask(id: number): Promise<void> {
  if (!isTauriRuntime()) {
    const store = preview();
    store.tasks = store.tasks.filter((t) => t.id !== id);
    return;
  }
  return invoke("delete_task", { id });
}

export async function clearCompletedTasks(): Promise<number> {
  if (!isTauriRuntime()) {
    const store = preview();
    const before = store.tasks.length;
    store.tasks = store.tasks.filter((t) => !t.completed);
    return before - store.tasks.length;
  }
  return invoke("clear_completed_tasks");
}
