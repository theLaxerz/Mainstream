import { useEffect, useState } from "react";
import "./Clock.css";

function pad(n: number) {
  return String(n).padStart(2, "0");
}

export function Clock() {
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(id);
  }, []);

  const hours = now.getHours() % 12;
  const minutes = now.getMinutes();
  const seconds = now.getSeconds();

  const secondDeg = seconds * 6;
  const minuteDeg = minutes * 6 + seconds * 0.1;
  const hourDeg = hours * 30 + minutes * 0.5;

  const dateLabel = new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  }).format(now);

  const timeLabel = `${pad(now.getHours())}:${pad(minutes)}:${pad(seconds)}`;

  return (
    <div className="clock-hero" aria-label={`Current time ${timeLabel}`}>
      <div className="clock-face" role="img" aria-hidden="true">
        <div className="clock-ring" />
        <div className="clock-ticks">
          {Array.from({ length: 12 }, (_, i) => (
            <span
              key={i}
              className="clock-tick"
              style={{ transform: `rotate(${i * 30}deg)` }}
            />
          ))}
        </div>
        <div
          className="clock-hand hour"
          style={{ transform: `rotate(${hourDeg}deg)` }}
        />
        <div
          className="clock-hand minute"
          style={{ transform: `rotate(${minuteDeg}deg)` }}
        />
        <div
          className="clock-hand second"
          style={{ transform: `rotate(${secondDeg}deg)` }}
        />
        <div className="clock-center" />
      </div>
      <p className="clock-date">{dateLabel}</p>
    </div>
  );
}
