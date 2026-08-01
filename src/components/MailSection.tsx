import { useEffect, useState } from "react";
import {
  getEmailSettings,
  listPhysicalMail,
  openEmail,
  physicalMailImageBase64,
  syncPhysicalMail,
} from "../lib/api";
import { onDashboardRefresh } from "../lib/refresh";
import type { PhysicalMailPiece } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import { PermissionCallout } from "./PermissionCallout";
import "./MailSection.css";

function ocrPreview(text: string, max = 160): string {
  const line = text.split("\n").find((l) => l.trim().length > 0) ?? text;
  const t = line.trim();
  if (t.length <= max) return t;
  return `${t.slice(0, max)}…`;
}

function MailThumb({ pieceId }: { pieceId: number }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void physicalMailImageBase64(pieceId).then((data) => {
      if (!cancelled) setSrc(data);
    });
    return () => {
      cancelled = true;
    };
  }, [pieceId]);

  if (!src) {
    return <div className="mail-thumb mail-thumb-empty" aria-hidden />;
  }
  return <img className="mail-thumb" src={src} alt="" />;
}

export function MailSection() {
  const [pieces, setPieces] = useState<PhysicalMailPiece[]>([]);
  const [configured, setConfigured] = useState(false);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [detail, setDetail] = useState<PhysicalMailPiece | null>(null);

  async function loadList() {
    const settings = await getEmailSettings();
    const ready = Boolean(
      settings.host && settings.user && settings.hasPassword,
    );
    setConfigured(ready);
    if (!ready) {
      setPieces([]);
      return;
    }
    const rows = await listPhysicalMail(12);
    setPieces(rows);
  }

  async function refresh(options?: { sync?: boolean }) {
    setError(null);
    setStatus(null);
    try {
      if (options?.sync) {
        setSyncing(true);
        const result = await syncPhysicalMail();
        setStatus(
          `Synced ${result.pieces} piece${result.pieces === 1 ? "" : "s"} from ${result.digests} digest${result.digests === 1 ? "" : "s"} (${result.ocrRan} OCR)`,
        );
      }
      await loadList();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSyncing(false);
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    return onDashboardRefresh(() => void refresh());
  }, []);

  return (
    <>
      <ModuleSection
        title="Mail"
        eyebrow="Informed Delivery"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              disabled={syncing || !configured}
              onClick={() => void refresh({ sync: true })}
            >
              {syncing ? "Syncing…" : "Sync"}
            </button>
          </div>
        }
        style={{ animationDelay: "0.12s" }}
      >
        {!configured ? (
          <PermissionCallout
            title="Connect email first"
            body="Physical mail is read from USPS Informed Delivery digests in your synced inbox. Set up IMAP under Email, then sync here."
            steps={[
              "Subscribe to Informed Delivery at usps.com",
              "Use the same mailbox in Email settings",
              "Sync Mail to pull digests and OCR envelope scans",
            ]}
            actionLabel="Open Email module"
            onAction={() => {
              document
                .querySelector("[data-module='email']")
                ?.scrollIntoView({ behavior: "smooth", block: "start" });
            }}
          />
        ) : null}

        {error ? <p className="module-empty">{error}</p> : null}
        {status ? <p className="module-empty">{status}</p> : null}
        {loading ? <p className="module-empty">Loading mail…</p> : null}

        {!loading && configured && pieces.length === 0 ? (
          <p className="module-empty">
            No Informed Delivery pieces yet — sync after USPS digests arrive in
            your inbox.
          </p>
        ) : null}

        {configured && pieces.length > 0 ? (
          <ul className="module-list mail-list">
            {pieces.map((piece) => (
              <li key={piece.id} className="mail-row">
                <MailThumb pieceId={piece.id} />
                <button
                  type="button"
                  className="module-row-main shortcut-open"
                  onClick={() => setDetail(piece)}
                >
                  <p className="module-row-title">
                    {ocrPreview(piece.ocrText, 72)}
                  </p>
                  <p className="module-row-meta">
                    {piece.digestDate ?? "Recent digest"}
                    {piece.pieceIndex > 0
                      ? ` · piece ${piece.pieceIndex + 1}`
                      : ""}
                  </p>
                </button>
                <div className="row-actions">
                  <button
                    type="button"
                    className="btn btn-primary btn-icon"
                    onClick={() => void openEmail(piece.emailId)}
                  >
                    Email
                  </button>
                </div>
              </li>
            ))}
          </ul>
        ) : null}
      </ModuleSection>

      <DetailDrawer
        open={detail !== null}
        title="Mail piece"
        eyebrow={detail?.digestDate ?? "OCR"}
        onClose={() => setDetail(null)}
      >
        {detail ? (
          <div className="mail-detail">
            <div className="mail-detail-thumb">
              <MailThumb pieceId={detail.id} />
            </div>
            <pre className="mail-ocr-body">{detail.ocrText}</pre>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void openEmail(detail.emailId)}
            >
              Open digest in Mail
            </button>
          </div>
        ) : null}
      </DetailDrawer>
    </>
  );
}
