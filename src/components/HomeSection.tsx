import { useEffect, useState, type FormEvent } from "react";
import {
  getHomeSettings,
  listHomeDevices,
  saveHomeCredentials,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { HomeDevice } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";

type Props = { limit?: number };

export function HomeSection({ limit = 8 }: Props) {
  const [devices, setDevices] = useState<HomeDevice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [manageOpen, setManageOpen] = useState(false);
  const [ringToken, setRingToken] = useState("");
  const [blinkEmail, setBlinkEmail] = useState("");
  const [blinkPassword, setBlinkPassword] = useState("");
  const [blinkUid, setBlinkUid] = useState("");

  async function refresh() {
    try {
      const settings = await getHomeSettings();
      setBlinkUid(settings.blinkDeviceUid);
      const list = await listHomeDevices();
      setDevices(list);
      setError(null);
    } catch (e) {
      setDevices([]);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    return onDashboardRefresh(() => void refresh());
  }, []);

  async function onSave(e: FormEvent) {
    e.preventDefault();
    await saveHomeCredentials({
      ringRefreshToken: ringToken.trim() || undefined,
      blinkEmail: blinkEmail.trim() || undefined,
      blinkPassword: blinkPassword.trim() || undefined,
      blinkDeviceUid: blinkUid.trim() || undefined,
    });
    setRingToken("");
    setBlinkPassword("");
    await refresh();
  }

  return (
    <>
      <ModuleSection
        title="Home"
        eyebrow="Ring & Blink"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setManageOpen(true)}
            >
              Connect
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void refresh()}
            >
              Refresh
            </button>
          </div>
        }
        count={!loading ? devices.length : null}
      >
        {error ? <p className="module-empty">{error}</p> : null}
        {loading ? <p className="module-empty">Loading devices…</p> : null}
        {!loading && devices.length === 0 && !error ? (
          <p className="module-empty">
            Connect Ring (refresh token) and/or Blink to see cameras and
            doorbells.
          </p>
        ) : null}
        <ul className="module-list">
          {devices.slice(0, limit).map((d) => (
            <li key={d.id}>
              <div className="module-row-main">
                <p className="module-row-title">{d.name}</p>
                <p className="module-row-meta">
                  {d.vendor} · {d.deviceType} · {d.status}
                  {d.detail ? ` · ${d.detail}` : ""}
                </p>
              </div>
            </li>
          ))}
        </ul>
      </ModuleSection>

      <DetailDrawer
        open={manageOpen}
        title="Home connectors"
        eyebrow="Ring & Blink"
        onClose={() => setManageOpen(false)}
      >
        <form onSubmit={onSave}>
          <p className="module-eyebrow">Ring</p>
          <input
            className="field"
            type="password"
            value={ringToken}
            onChange={(e) => setRingToken(e.target.value)}
            placeholder="Ring refresh token (Keychain)"
            aria-label="Ring refresh token"
          />
          <p className="module-eyebrow finance-subhead">Blink</p>
          <div className="field-row">
            <input
              className="field"
              value={blinkEmail}
              onChange={(e) => setBlinkEmail(e.target.value)}
              placeholder="Blink email"
              aria-label="Blink email"
            />
            <input
              className="field"
              type="password"
              value={blinkPassword}
              onChange={(e) => setBlinkPassword(e.target.value)}
              placeholder="Blink password"
              aria-label="Blink password"
            />
          </div>
          <input
            className="field"
            value={blinkUid}
            onChange={(e) => setBlinkUid(e.target.value)}
            placeholder="Blink device UID"
            aria-label="Blink device UID"
          />
          <button type="submit" className="btn btn-primary">
            Save credentials
          </button>
        </form>
        <p className="module-empty">
          Credentials stay in the macOS Keychain. Ring uses a refresh token;
          Blink uses your Amazon/Blink login.
        </p>
      </DetailDrawer>
    </>
  );
}
