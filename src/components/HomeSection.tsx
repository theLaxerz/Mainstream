import { useEffect, useState, type FormEvent } from "react";
import {
  blinkCaptureSnapshot,
  blinkDisconnect,
  blinkStartLogin,
  blinkVerifyPin,
  getHomeSettings,
  homeDeviceImageBase64,
  listHomeDevices,
  saveHomeCredentials,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { HomeDevice, HomeSettings } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import "./HomeSection.css";

type Props = { limit?: number };

function statusClass(status: string): string {
  const s = status.toLowerCase();
  if (s === "online" || s === "on") return "is-online";
  if (s === "offline" || s === "disabled") return `is-${s}`;
  if (s === "alert") return "is-alert";
  return "";
}

function CameraThumb({
  id,
  nonce,
  available,
  className = "home-thumb",
}: {
  id: string;
  nonce: number;
  available: boolean;
  className?: string;
}) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    if (!available) {
      setSrc(null);
      return;
    }
    let cancelled = false;
    void homeDeviceImageBase64(id).then((data) => {
      if (!cancelled) setSrc(data);
    });
    return () => {
      cancelled = true;
    };
  }, [id, nonce, available]);

  if (!src) {
    return <div className={`${className} home-thumb-empty`} aria-hidden />;
  }
  return <img className={className} src={src} alt="" />;
}

export function HomeSection({ limit = 8 }: Props) {
  const [devices, setDevices] = useState<HomeDevice[]>([]);
  const [settings, setSettings] = useState<HomeSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [manageOpen, setManageOpen] = useState(false);
  const [detail, setDetail] = useState<HomeDevice | null>(null);
  const [ringToken, setRingToken] = useState("");
  const [blinkEmail, setBlinkEmail] = useState("");
  const [blinkPassword, setBlinkPassword] = useState("");
  const [blinkPin, setBlinkPin] = useState("");
  const [pinRequired, setPinRequired] = useState(false);
  const [busy, setBusy] = useState(false);
  const [snappingId, setSnappingId] = useState<string | null>(null);
  const [thumbNonce, setThumbNonce] = useState<Record<string, number>>({});

  async function refresh() {
    try {
      const next = await getHomeSettings();
      setSettings(next);
      setBlinkEmail((prev) => prev || next.blinkEmail);
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

  function bumpThumb(id: string) {
    setThumbNonce((prev) => ({ ...prev, [id]: (prev[id] ?? 0) + 1 }));
  }

  async function onSaveRing(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await saveHomeCredentials({
        ringRefreshToken: ringToken.trim() || undefined,
        blinkEmail: blinkEmail.trim() || undefined,
      });
      setRingToken("");
      setStatus("Ring credentials saved.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function onConnectBlink(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const result = await blinkStartLogin(blinkEmail.trim(), blinkPassword);
      setBlinkPassword("");
      if (result.status === "pin_required") {
        setPinRequired(true);
        setStatus(result.detail ?? "Enter the Blink verification code.");
        return;
      }
      setPinRequired(false);
      setStatus(result.detail ?? "Blink is connected.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function onVerifyPin(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result = await blinkVerifyPin(blinkPin.trim());
      setBlinkPin("");
      setPinRequired(false);
      setStatus(result.detail ?? "Blink is connected.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function onDisconnectBlink() {
    setBusy(true);
    setError(null);
    try {
      await blinkDisconnect();
      setPinRequired(false);
      setBlinkPin("");
      setStatus("Blink disconnected.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function onSnap(device: HomeDevice) {
    setSnappingId(device.id);
    setError(null);
    try {
      const next = await blinkCaptureSnapshot(device.id);
      bumpThumb(device.id);
      setDevices((prev) => prev.map((d) => (d.id === next.id ? next : d)));
      setDetail((prev) => (prev?.id === next.id ? next : prev));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSnappingId(null);
    }
  }

  const shown = devices.slice(0, limit);
  const blinkConnected = Boolean(settings?.blinkConnected);
  const ringConnected = Boolean(settings?.ringConnected);

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
        {status && !manageOpen ? <p className="module-empty">{status}</p> : null}
        {loading ? <p className="module-empty">Loading cameras…</p> : null}
        {!loading && devices.length === 0 && !error ? (
          <p className="module-empty">
            Connect Blink with email, password, and the SMS/email PIN Blink
            sends. Camera stills load here after sign-in. Ring still uses a
            refresh token.
          </p>
        ) : null}
        <ul className="module-list home-list">
          {shown.map((d) => (
            <li key={d.id} className="home-row">
              <CameraThumb
                id={d.id}
                nonce={thumbNonce[d.id] ?? 0}
                available={d.thumbnailAvailable}
              />
              <button
                type="button"
                className="module-row-main shortcut-open"
                onClick={() => setDetail(d)}
              >
                <p className="module-row-title">{d.name}</p>
                <p className="module-row-meta">
                  <span className="home-status">
                    <span
                      className={`home-status-dot ${statusClass(d.status)}`}
                    />
                    {d.vendor} · {d.deviceType} · {d.status}
                  </span>
                  {d.detail ? ` · ${d.detail}` : ""}
                </p>
              </button>
              {d.vendor === "blink" && d.snapshotReady ? (
                <div className="row-actions">
                  <button
                    type="button"
                    className="btn btn-ghost"
                    disabled={snappingId === d.id}
                    onClick={() => void onSnap(d)}
                  >
                    {snappingId === d.id ? "Snapping…" : "Snap"}
                  </button>
                </div>
              ) : null}
            </li>
          ))}
        </ul>
      </ModuleSection>

      <DetailDrawer
        open={detail !== null}
        title={detail?.name ?? "Camera"}
        eyebrow={detail ? `${detail.vendor} · ${detail.deviceType}` : "Home"}
        onClose={() => setDetail(null)}
        wide
      >
        {detail ? (
          <div>
            <div className="home-detail-thumb">
              <CameraThumb
                id={detail.id}
                nonce={thumbNonce[detail.id] ?? 0}
                available={detail.thumbnailAvailable}
              />
            </div>
            <p className="home-connect-status">
              <span className="home-status">
                <span
                  className={`home-status-dot ${statusClass(detail.status)}`}
                />
                {detail.status}
              </span>
              {detail.detail ? ` · ${detail.detail}` : ""}
            </p>
            {detail.vendor === "blink" && detail.snapshotReady ? (
              <button
                type="button"
                className="btn btn-primary"
                disabled={snappingId === detail.id}
                onClick={() => void onSnap(detail)}
              >
                {snappingId === detail.id ? "Capturing still…" : "Snap new still"}
              </button>
            ) : null}
          </div>
        ) : null}
      </DetailDrawer>

      <DetailDrawer
        open={manageOpen}
        title="Home connectors"
        eyebrow="Ring & Blink"
        onClose={() => {
          setManageOpen(false);
          setPinRequired(false);
        }}
      >
        <p className="home-connect-status">
          Ring: {ringConnected ? "connected" : "not connected"}
          {" · "}
          Blink: {blinkConnected ? "connected" : "not connected"}
        </p>
        {status ? <p className="module-empty">{status}</p> : null}

        <form onSubmit={onSaveRing}>
          <p className="module-eyebrow">Ring</p>
          <div className="field-row">
            <input
              className="field"
              type="password"
              value={ringToken}
              onChange={(e) => setRingToken(e.target.value)}
              placeholder="Ring refresh token (Keychain)"
              aria-label="Ring refresh token"
            />
            <button type="submit" className="btn btn-primary" disabled={busy}>
              Save Ring
            </button>
          </div>
        </form>

        <p className="module-eyebrow finance-subhead">Blink</p>
        {blinkConnected ? (
          <>
            <p className="home-connect-status">
              Signed in as {settings?.blinkEmail || "Blink account"}. Refresh
              tokens stay in the macOS Keychain; stills are cached on disk.
            </p>
            <button
              type="button"
              className="btn btn-danger"
              disabled={busy}
              onClick={() => void onDisconnectBlink()}
            >
              Disconnect Blink
            </button>
          </>
        ) : pinRequired ? (
          <form onSubmit={onVerifyPin}>
            <p className="module-empty">
              Blink sent a verification code to your phone or email.
            </p>
            <div className="field-row">
              <input
                className="field"
                value={blinkPin}
                onChange={(e) => setBlinkPin(e.target.value)}
                placeholder="6-digit PIN"
                inputMode="numeric"
                autoComplete="one-time-code"
                aria-label="Blink verification code"
              />
              <button type="submit" className="btn btn-primary" disabled={busy}>
                {busy ? "Verifying…" : "Verify PIN"}
              </button>
            </div>
          </form>
        ) : (
          <form onSubmit={onConnectBlink}>
            <div className="field-row">
              <input
                className="field"
                type="email"
                value={blinkEmail}
                onChange={(e) => setBlinkEmail(e.target.value)}
                placeholder="Blink email"
                autoComplete="username"
                aria-label="Blink email"
              />
              <input
                className="field"
                type="password"
                value={blinkPassword}
                onChange={(e) => setBlinkPassword(e.target.value)}
                placeholder="Blink password"
                autoComplete="current-password"
                aria-label="Blink password"
              />
            </div>
            <button type="submit" className="btn btn-primary" disabled={busy}>
              {busy ? "Connecting…" : "Connect Blink"}
            </button>
          </form>
        )}
        <p className="module-empty">
          Blink retired the old password login. Mainstream now uses Blink’s
          current OAuth sign-in (same unofficial path as Home Assistant). The
          password is used once to get a refresh token and is not stored.
        </p>
      </DetailDrawer>
    </>
  );
}
