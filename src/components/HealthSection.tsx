import { useEffect, useState, type FormEvent } from "react";
import {
  getHealthSettings,
  healthTodaySummary,
  importHealthExport,
  listHealthDays,
  saveHealthSettings,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { HealthDay } from "../lib/types";
import { ModuleSection } from "./ModuleSection";

function formatSleep(minutes: number): string {
  const h = Math.floor(minutes / 60);
  const m = Math.round(minutes % 60);
  return `${h}h ${m}m`;
}

export function HealthSection() {
  const [today, setToday] = useState<HealthDay | null>(null);
  const [history, setHistory] = useState<HealthDay[]>([]);
  const [exportPath, setExportPath] = useState("");
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);

  async function refresh() {
    try {
      const settings = await getHealthSettings();
      setExportPath(settings.exportPath);
      const [t, days] = await Promise.all([
        healthTodaySummary(),
        listHealthDays(7),
      ]);
      setToday(t);
      setHistory(days);
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

  async function onSavePath(e: FormEvent) {
    e.preventDefault();
    await saveHealthSettings(exportPath);
    setStatus("Export path saved.");
  }

  async function onImport() {
    setImporting(true);
    setStatus(null);
    try {
      const result = await importHealthExport();
      setStatus(`Imported ${result.daysUpdated} days from Apple Health export.`);
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setImporting(false);
    }
  }

  return (
    <ModuleSection
      title="Health"
      eyebrow="Apple Health"
      action={
        <div className="row-actions">
          <button
            type="button"
            className="btn btn-ghost"
            onClick={() => setShowSettings((v) => !v)}
          >
            {showSettings ? "Hide" : "Setup"}
          </button>
          <button
            type="button"
            className="btn btn-ghost"
            disabled={importing || !exportPath}
            onClick={() => void onImport()}
          >
            {importing ? "Importing…" : "Import"}
          </button>
        </div>
      }
      style={{ animationDelay: "0.14s" }}
    >
      {showSettings ? (
        <form className="notes-form" onSubmit={onSavePath}>
          <div className="field-row">
            <input
              className="field"
              value={exportPath}
              onChange={(e) => setExportPath(e.target.value)}
              placeholder="Path to export.zip or export.xml"
              aria-label="Apple Health export path"
            />
            <button type="submit" className="btn btn-primary">
              Save
            </button>
          </div>
          <p className="module-empty">
            Export from Health app on iPhone → Profile → Export All Health Data,
            then point Mainstream at the zip on your Mac.
          </p>
        </form>
      ) : null}

      {error ? <p className="module-empty">{error}</p> : null}
      {status ? <p className="module-empty">{status}</p> : null}
      {loading ? <p className="module-empty">Loading health…</p> : null}

      {!loading && today ? (
        <div className="finance-totals">
          <div>
            <p className="module-row-meta">Steps</p>
            <p className="module-row-title finance-total">{today.steps}</p>
          </div>
          <div>
            <p className="module-row-meta">Sleep</p>
            <p className="module-row-title finance-total">
              {formatSleep(today.sleepMinutes)}
            </p>
          </div>
          {today.avgHeartRate ? (
            <div>
              <p className="module-row-meta">Heart</p>
              <p className="module-row-title finance-total">
                {Math.round(today.avgHeartRate)} bpm
              </p>
            </div>
          ) : null}
        </div>
      ) : null}

      {!loading && !today ? (
        <p className="module-empty">
          No health data yet — set export path and import.
        </p>
      ) : null}

      {history.length > 1 ? (
        <>
          <p className="module-eyebrow finance-subhead">Recent days</p>
          <ul className="module-list">
            {history.slice(0, 5).map((d) => (
              <li key={d.day}>
                <div className="module-row-main">
                  <p className="module-row-title">{d.day}</p>
                  <p className="module-row-meta">
                    {d.steps} steps · {formatSleep(d.sleepMinutes)}
                  </p>
                </div>
              </li>
            ))}
          </ul>
        </>
      ) : null}
    </ModuleSection>
  );
}
