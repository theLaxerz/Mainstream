import { useEffect, useMemo, useState, type FormEvent } from "react";
import { onDashboardRefresh, requestDashboardRefresh } from "../lib/refresh";
import {
  briefingDetail,
  clearCompletedTasks,
  createTask,
  deleteTask,
  formatDueOn,
  groupTasks,
  listTasks,
  localDateKey,
  setTaskCompleted,
  shiftLocalDate,
  summarizeTasks,
  updateTask,
} from "../lib/tasks";
import type { Task } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import "./TasksSection.css";

type Props = { limit?: number };

export function TasksSection({ limit = 12 }: Props) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [allTasks, setAllTasks] = useState<Task[]>([]);
  const [title, setTitle] = useState("");
  const [dueOn, setDueOn] = useState("");
  const [high, setHigh] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [drawerLoading, setDrawerLoading] = useState(false);

  async function refresh() {
    try {
      const rows = await listTasks(Math.max(limit, 24), true);
      setTasks(rows);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [limit]);

  const summary = useMemo(() => summarizeTasks(tasks), [tasks]);
  const groups = useMemo(() => {
    const openFirst = groupTasks(tasks).filter((g) => g.bucket !== "done");
    const done = groupTasks(tasks).find((g) => g.bucket === "done");
    const visible = openFirst.flatMap((g) => g.tasks).slice(0, limit);
    const visibleIds = new Set(visible.map((t) => t.id));
    const trimmed = openFirst
      .map((g) => ({ ...g, tasks: g.tasks.filter((t) => visibleIds.has(t.id)) }))
      .filter((g) => g.tasks.length > 0);
    if (done && done.tasks.length > 0) {
      trimmed.push({ ...done, tasks: done.tasks.slice(0, 3) });
    }
    return trimmed;
  }, [tasks, limit]);

  function resetForm() {
    setTitle("");
    setDueOn("");
    setHigh(false);
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!title.trim()) return;
    try {
      await createTask({
        title: title.trim(),
        dueOn: dueOn || null,
        priority: high ? 1 : 0,
      });
      resetForm();
      await refresh();
      requestDashboardRefresh();
      if (drawerOpen) {
        setAllTasks(await listTasks(200, true));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onToggle(task: Task) {
    try {
      await setTaskCompleted(task.id, !task.completed);
      await refresh();
      requestDashboardRefresh();
      if (drawerOpen) {
        setAllTasks(await listTasks(200, true));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onPriority(task: Task) {
    try {
      await updateTask(task.id, { priority: task.priority > 0 ? 0 : 1 });
      await refresh();
      if (drawerOpen) {
        setAllTasks(await listTasks(200, true));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onDelete(id: number) {
    try {
      await deleteTask(id);
      await refresh();
      requestDashboardRefresh();
      if (drawerOpen) {
        setAllTasks(await listTasks(200, true));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onClearDone() {
    try {
      await clearCompletedTasks();
      await refresh();
      requestDashboardRefresh();
      if (drawerOpen) {
        setAllTasks(await listTasks(200, true));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function openAll() {
    setDrawerOpen(true);
    setDrawerLoading(true);
    try {
      setAllTasks(await listTasks(200, true));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setDrawerLoading(false);
    }
  }

  function taskRows(rows: Task[]) {
    const grouped = groupTasks(rows);
    return (
      <div className="tasks-groups">
        {grouped.map((group) => (
          <div key={group.bucket} className={`tasks-group is-${group.bucket}`}>
            <p className="tasks-group-label">{group.label}</p>
            <ul className="module-list tasks-list">
              {group.tasks.map((task) => (
                <li
                  key={task.id}
                  className={`task-row ${task.completed ? "is-done" : ""} ${
                    task.priority > 0 ? "is-high" : ""
                  }`}
                >
                  <button
                    type="button"
                    className={`task-check ${task.completed ? "is-checked" : ""}`}
                    onClick={() => void onToggle(task)}
                    aria-pressed={task.completed}
                    aria-label={
                      task.completed
                        ? `Mark ${task.title} as open`
                        : `Complete ${task.title}`
                    }
                  >
                    {task.completed ? "✓" : ""}
                  </button>
                  <div className="module-row-main">
                    <p className="module-row-title">{task.title}</p>
                    <p className="module-row-meta">
                      {formatDueOn(task.dueOn)}
                      {task.notes
                        ? ` · ${task.notes.slice(0, 48)}${task.notes.length > 48 ? "…" : ""}`
                        : ""}
                    </p>
                  </div>
                  <div className="row-actions">
                    <button
                      type="button"
                      className={`btn btn-ghost btn-icon ${task.priority > 0 ? "is-flagged" : ""}`}
                      onClick={() => void onPriority(task)}
                      aria-label={
                        task.priority > 0
                          ? `Clear priority on ${task.title}`
                          : `Mark ${task.title} as high priority`
                      }
                    >
                      !
                    </button>
                    <button
                      type="button"
                      className="btn btn-danger btn-icon"
                      onClick={() => void onDelete(task.id)}
                      aria-label={`Delete ${task.title}`}
                    >
                      Del
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    );
  }

  const eyebrow =
    summary.overdue > 0
      ? "Overdue"
      : summary.dueToday > 0
        ? "Due today"
        : "Open";

  return (
    <>
      <ModuleSection
        title="Tasks"
        eyebrow={eyebrow}
        accent="accent"
        action={
          <div className="row-actions">
            <button type="button" className="btn btn-ghost" onClick={() => void refresh()}>
              Refresh
            </button>
            <button type="button" className="btn btn-ghost" onClick={() => void openAll()}>
              All
            </button>
          </div>
        }
        count={!loading ? summary.open : null}
      >
        <form className="notes-form tasks-form" onSubmit={onSubmit}>
          <div className="field-row">
            <input
              className="field"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Add a task"
              aria-label="Task title"
            />
            <button type="submit" className="btn btn-primary">
              Add
            </button>
          </div>
          <div className="tasks-quick">
            <button
              type="button"
              className={`tasks-chip ${dueOn === localDateKey(new Date()) ? "is-on" : ""}`}
              onClick={() =>
                setDueOn((prev) =>
                  prev === localDateKey(new Date()) ? "" : localDateKey(new Date()),
                )
              }
            >
              Today
            </button>
            <button
              type="button"
              className={`tasks-chip ${dueOn === shiftLocalDate(1) ? "is-on" : ""}`}
              onClick={() =>
                setDueOn((prev) => (prev === shiftLocalDate(1) ? "" : shiftLocalDate(1)))
              }
            >
              Tomorrow
            </button>
            <label className="tasks-date">
              <span className="visually-hidden">Due date</span>
              <input
                type="date"
                className="field tasks-date-input"
                value={dueOn}
                onChange={(e) => setDueOn(e.target.value)}
              />
            </label>
            <button
              type="button"
              className={`tasks-chip ${high ? "is-on is-high" : ""}`}
              onClick={() => setHigh((v) => !v)}
              aria-pressed={high}
            >
              High
            </button>
          </div>
        </form>

        {error ? <p className="module-empty">{error}</p> : null}
        {loading ? <p className="module-empty">Loading tasks…</p> : null}
        {!loading && summary.open === 0 && tasks.every((t) => t.completed) ? (
          <p className="module-empty">
            {tasks.length === 0
              ? "Nothing on the list — capture something above."
              : "All clear. Add the next thing when it shows up."}
          </p>
        ) : null}

        {!loading ? taskRows(groups.flatMap((g) => g.tasks)) : null}

        {summary.open > 0 ? (
          <p className="tasks-pulse">{briefingDetail(summary)}</p>
        ) : null}
      </ModuleSection>

      <DetailDrawer
        open={drawerOpen}
        title="All tasks"
        eyebrow="Due date · priority"
        onClose={() => setDrawerOpen(false)}
      >
        {drawerLoading ? (
          <p className="module-empty">Loading…</p>
        ) : allTasks.length === 0 ? (
          <p className="module-empty">No tasks yet.</p>
        ) : (
          <>
            {taskRows(allTasks)}
            {allTasks.some((t) => t.completed) ? (
              <button
                type="button"
                className="btn btn-ghost tasks-clear"
                onClick={() => void onClearDone()}
              >
                Clear completed
              </button>
            ) : null}
          </>
        )}
      </DetailDrawer>
    </>
  );
}
