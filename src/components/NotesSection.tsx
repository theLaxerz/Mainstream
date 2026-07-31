import { useEffect, useState, type FormEvent } from "react";
import { createNote, deleteNote, listNotes, updateNote } from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { Note } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";

function formatUpdated(iso: string) {
  try {
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export function NotesSection() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [allNotes, setAllNotes] = useState<Note[]>([]);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [drawerLoading, setDrawerLoading] = useState(false);

  async function refresh() {
    try {
      const rows = await listNotes(10);
      setNotes(rows);
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
  }, []);

  function resetForm() {
    setTitle("");
    setBody("");
    setEditingId(null);
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!title.trim()) return;
    try {
      if (editingId != null) {
        await updateNote(editingId, { title: title.trim(), body });
      } else {
        await createNote(title.trim(), body);
      }
      resetForm();
      await refresh();
      if (drawerOpen) {
        const rows = await listNotes(200);
        setAllNotes(rows);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  function startEdit(note: Note) {
    setEditingId(note.id);
    setTitle(note.title);
    setBody(note.body);
  }

  async function onDelete(id: number) {
    try {
      await deleteNote(id);
      if (editingId === id) resetForm();
      await refresh();
      if (drawerOpen) {
        const rows = await listNotes(200);
        setAllNotes(rows);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function openAll() {
    setDrawerOpen(true);
    setDrawerLoading(true);
    try {
      const rows = await listNotes(200);
      setAllNotes(rows);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setDrawerLoading(false);
    }
  }

  function noteRows(rows: Note[]) {
    return (
      <ul className="module-list">
        {rows.map((note) => (
          <li key={note.id}>
            <div className="module-row-main">
              <p className="module-row-title">{note.title}</p>
              <p className="module-row-meta">
                {note.body
                  ? `${note.body.slice(0, 72)}${note.body.length > 72 ? "…" : ""} · `
                  : ""}
                {formatUpdated(note.updatedAt)}
              </p>
            </div>
            <div className="row-actions">
              <button
                type="button"
                className="btn btn-ghost btn-icon"
                onClick={() => startEdit(note)}
                aria-label={`Edit ${note.title}`}
              >
                Edit
              </button>
              <button
                type="button"
                className="btn btn-danger btn-icon"
                onClick={() => void onDelete(note.id)}
                aria-label={`Delete ${note.title}`}
              >
                Del
              </button>
            </div>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <>
      <ModuleSection
        title="Notes"
        eyebrow="Recent"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void refresh()}
            >
              Refresh
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void openAll()}
            >
              All
            </button>
          </div>
        }
        style={{ animationDelay: "0.18s" }}
      >
        <form className="notes-form" onSubmit={onSubmit}>
          <div className="field-row">
            <input
              className="field"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Note title"
              aria-label="Note title"
            />
            <button type="submit" className="btn btn-primary">
              {editingId != null ? "Save" : "Add"}
            </button>
            {editingId != null ? (
              <button type="button" className="btn btn-ghost" onClick={resetForm}>
                Cancel
              </button>
            ) : null}
          </div>
          <textarea
            className="field-area"
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder="Write something…"
            aria-label="Note body"
          />
        </form>

        {error ? <p className="module-empty">{error}</p> : null}
        {loading ? <p className="module-empty">Loading notes…</p> : null}
        {!loading && notes.length === 0 ? (
          <p className="module-empty">No notes yet — capture a thought above.</p>
        ) : null}

        {noteRows(notes)}
      </ModuleSection>

      <DetailDrawer
        open={drawerOpen}
        title="All notes"
        eyebrow="Sorted by update"
        onClose={() => setDrawerOpen(false)}
      >
        {drawerLoading ? (
          <p className="module-empty">Loading…</p>
        ) : allNotes.length === 0 ? (
          <p className="module-empty">No notes yet.</p>
        ) : (
          noteRows(allNotes)
        )}
      </DetailDrawer>
    </>
  );
}
