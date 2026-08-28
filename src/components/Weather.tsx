import { useEffect, useState, type FormEvent } from "react";
import {
  clearWeatherPlace,
  getWeather,
  saveWeatherPlace,
  searchWeatherPlaces,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { WeatherPlace, WeatherSnapshot } from "../lib/types";
import "./Weather.css";

function weatherGlyph(code: number): string {
  if (code === 0) return "☀";
  if (code === 1) return "🌤";
  if (code === 2) return "⛅";
  if (code === 3) return "☁";
  if (code === 45 || code === 48) return "🌫";
  if (code >= 51 && code <= 57) return "🌦";
  if (code >= 71 && code <= 77) return "❄";
  if (code === 85 || code === 86) return "❄";
  if (code >= 95) return "⛈";
  return "🌧";
}

function formatTemp(n: number, units: string): string {
  const rounded = Math.round(n);
  return units === "celsius" ? `${rounded}°C` : `${rounded}°`;
}

function placeLabel(place: WeatherPlace): string {
  const bits = [place.name, place.admin].filter(Boolean);
  return bits.join(", ");
}

export function Weather() {
  const [snapshot, setSnapshot] = useState<WeatherSnapshot | null>(null);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<WeatherPlace[]>([]);
  const [searching, setSearching] = useState(false);
  const [editing, setEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function load() {
    try {
      const next = await getWeather();
      setSnapshot(next);
      setError(null);
      if (next) setEditing(false);
    } catch {
      setSnapshot(null);
      setError(null);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
    return onDashboardRefresh(() => void load());
  }, []);

  async function onSearch(e: FormEvent) {
    e.preventDefault();
    const q = query.trim();
    if (q.length < 2) return;
    setSearching(true);
    try {
      const rows = await searchWeatherPlaces(q);
      setResults(rows);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSearching(false);
    }
  }

  async function onPick(place: WeatherPlace) {
    try {
      const next = await saveWeatherPlace({
        name: place.name,
        latitude: place.latitude,
        longitude: place.longitude,
        admin: place.admin,
        country: place.country,
        units: snapshot?.place.units ?? "fahrenheit",
      });
      setSnapshot(next);
      setResults([]);
      setQuery("");
      setEditing(false);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onClear() {
    try {
      await clearWeatherPlace();
    } catch {
      /* ignore outside Tauri */
    }
    setSnapshot(null);
    setEditing(true);
    setResults([]);
  }

  const showSearch = editing || !snapshot;

  return (
    <div className="weather-panel">
      <p className="weather-eyebrow">Outside</p>
      {loading && !snapshot ? (
        <p className="weather-empty">Checking the sky…</p>
      ) : null}

      {snapshot && !showSearch ? (
        <div className="weather-current">
          <div className="weather-glyph" aria-hidden="true">
            {weatherGlyph(snapshot.weatherCode)}
          </div>
          <div className="weather-readout">
            <p className="weather-temp">
              {formatTemp(snapshot.temperature, snapshot.place.units)}
            </p>
            <p className="weather-condition">{snapshot.condition}</p>
            <p className="weather-place">{placeLabel(snapshot.place)}</p>
            {snapshot.high != null && snapshot.low != null ? (
              <p className="weather-range">
                H {formatTemp(snapshot.high, snapshot.place.units)} · L{" "}
                {formatTemp(snapshot.low, snapshot.place.units)}
              </p>
            ) : null}
          </div>
          <button
            type="button"
            className="weather-change"
            onClick={() => setEditing(true)}
          >
            Change
          </button>
        </div>
      ) : null}

      {showSearch ? (
        <form className="weather-search" onSubmit={onSearch}>
          <input
            className="field"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="City or town"
            aria-label="Search for a city"
            autoComplete="off"
          />
          <button type="submit" className="btn btn-primary" disabled={searching}>
            {searching ? "…" : "Find"}
          </button>
          {snapshot ? (
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setEditing(false)}
            >
              Cancel
            </button>
          ) : null}
        </form>
      ) : null}

      {results.length > 0 ? (
        <ul className="weather-results">
          {results.map((place) => (
            <li key={`${place.name}-${place.latitude}-${place.longitude}`}>
              <button type="button" onClick={() => void onPick(place)}>
                <span className="weather-result-name">{placeLabel(place)}</span>
                {place.country ? (
                  <span className="weather-result-meta">{place.country}</span>
                ) : null}
              </button>
            </li>
          ))}
        </ul>
      ) : null}

      {showSearch && snapshot ? (
        <button type="button" className="weather-clear" onClick={() => void onClear()}>
          Remove place
        </button>
      ) : null}

      {error ? <p className="weather-empty">{error}</p> : null}
      {!loading && !snapshot && !error && results.length === 0 ? (
        <p className="weather-empty">Set a city to pin the forecast here.</p>
      ) : null}
    </div>
  );
}
