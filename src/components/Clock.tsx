import { useEffect, useState } from "react";
import { greetingFor } from "../lib/pulse";
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

  const hours12 = now.getHours() % 12;
  const minutes = now.getMinutes();
  const seconds = now.getSeconds();

  const secondDeg = seconds * 6;
  const minuteDeg = minutes * 6 + seconds * 0.1;
  const hourDeg = hours12 * 30 + minutes * 0.5;

  const dateLabel = new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
  }).format(now);

  const hourLabel = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  }).format(now);

  const meridiem = now.getHours() >= 12 ? "PM" : "AM";
  const displayHour = hours12 === 0 ? 12 : hours12;
  const timeLabel = `${pad(now.getHours())}:${pad(minutes)}:${pad(seconds)}`;
  const greeting = greetingFor(now);

  return (
    <div className="clock-hero" aria-label={`${greeting}. Current time ${timeLabel}`}>
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
          className={`clock-hand second${seconds === 0 ? " no-tick" : ""}`}
          style={{ transform: `rotate(${secondDeg}deg)` }}
        />
        <div className="clock-center" />
      </div>
      <div className="clock-readout">
        <p className="clock-greeting">{greeting}</p>
        <p className="clock-digital" aria-hidden="true">
          <span className="clock-hm">
            {displayHour}:{pad(minutes)}
          </span>
          <span className="clock-sec">{pad(seconds)}</span>
          <span className="clock-meridiem">{meridiem}</span>
        </p>
        <p className="clock-date">{dateLabel}</p>
        <span className="visually-hidden">{hourLabel}</span>
      </div>
    </div>
  );
}
