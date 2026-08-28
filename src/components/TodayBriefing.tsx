import { useEffect, useState } from "react";
import {
  loadDashboardPulse,
  type DashboardPulse,
  type PulseChipKind,
} from "../lib/pulse";
import { onDashboardRefresh } from "../lib/refresh";
import "./TodayBriefing.css";

const KIND_HINT: Record<PulseChipKind, string> = {
  calendar: "Jump to Calendar",
  messages: "Jump to Messages",
  email: "Jump to Email",
  health: "Jump to Health",
};

export function TodayBriefing() {
  const [pulse, setPulse] = useState<DashboardPulse | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const next = await loadDashboardPulse();
        if (!cancelled) setPulse(next);
      } catch {
        if (!cancelled) setPulse(null);
      }
    }
    void load();
    return onDashboardRefresh(() => void load());
  }, []);

  function jumpTo(moduleId: string) {
    const el = document.getElementById(`module-${moduleId}`);
    el?.scrollIntoView({ behavior: "smooth", block: "center" });
  }

  if (!pulse) return null;

  return (
    <section className="today-briefing" aria-label="Today at a glance">
      <div className="today-briefing-copy">
        <p className="today-briefing-eyebrow">{pulse.greeting}</p>
        <h2 className="today-briefing-title">At a glance</h2>
        <p className="today-briefing-sub">
          {pulse.nextEvent
            ? "Your day, gathered in one calm place."
            : pulse.chips.length > 0
              ? "Nothing urgent — the rest of the day is yours."
              : "Connect Calendar, Messages, Email, or Health and this becomes your morning briefing."}
        </p>
      </div>
      {pulse.chips.length > 0 ? (
        <ul className="today-briefing-chips">
          {pulse.chips.map((chip) => (
            <li key={chip.kind}>
              <button
                type="button"
                className={`today-chip kind-${chip.kind}`}
                onClick={() => jumpTo(chip.moduleId)}
                title={KIND_HINT[chip.kind]}
              >
                <span className="today-chip-label">{chip.label}</span>
                <span className="today-chip-detail">{chip.detail}</span>
              </button>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
