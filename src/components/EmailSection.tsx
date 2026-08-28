import { useEffect, useState, type FormEvent } from "react";
import {
  getEmailSettings,
  listAllImportantEmails,
  listImportantEmails,
  openEmail,
  openTarget,
  saveEmailSettings,
  syncEmail,
} from "../lib/api";
import { emailSenderLabel, formatEmailDate } from "../lib/email";
import {
  EMAIL_CONNECTORS,
  getEmailConnector,
  type EmailConnectorId,
} from "../lib/emailConnectors";
import { onDashboardRefresh } from "../lib/refresh";
import type { EmailMessage, EmailSettings } from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import { PermissionCallout } from "./PermissionCallout";

function connectorIdForHost(host: string): EmailConnectorId {
  const match = EMAIL_CONNECTORS.find(
    (c) => c.host && c.host.toLowerCase() === host.trim().toLowerCase(),
  );
  return match?.id ?? "custom";
}

type Props = { limit?: number };

export function EmailSection({ limit = 10 }: Props) {
  const [top, setTop] = useState<EmailMessage[]>([]);
  const [all, setAll] = useState<EmailMessage[]>([]);
  const [showAll, setShowAll] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<EmailSettings | null>(null);
  const [connectorId, setConnectorId] = useState<EmailConnectorId>("icloud");
  const [host, setHost] = useState("");
  const [port, setPort] = useState("993");
  const [user, setUser] = useState("");
  const [mailbox, setMailbox] = useState("INBOX");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [saving, setSaving] = useState(false);

  async function loadSettings() {
    const s = await getEmailSettings();
    setSettings(s);
    setHost(s.host);
    setPort(String(s.port || 993));
    setUser(s.user);
    setMailbox(s.mailbox || "INBOX");
    setConnectorId(s.host ? connectorIdForHost(s.host) : "icloud");
    if (!s.host || !s.user || !s.hasPassword) {
      setShowSettings(true);
    }
  }

  function onPickConnector(id: EmailConnectorId) {
    setConnectorId(id);
    if (id === "custom") return;
    const connector = getEmailConnector(id);
    setHost(connector.host);
    setPort(String(connector.port));
    setMailbox(connector.mailbox);
  }

  async function loadLists() {
    const [topRows, allRows] = await Promise.all([
      listImportantEmails(limit),
      listAllImportantEmails(),
    ]);
    setTop(topRows);
    setAll(allRows);
  }

  async function refresh(options?: { sync?: boolean }) {
    setError(null);
    setStatus(null);
    try {
      await loadSettings();
      if (options?.sync) {
        setSyncing(true);
        const result = await syncEmail();
        setStatus(
          `Synced ${result.fetched} unread · ${result.important} important (${result.mailbox})`,
        );
      }
      await loadLists();
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
    // eslint-disable-next-line react-hooks/exhaustive-deps -- refresh closes over limit
  }, [limit]);

  async function onSaveSettings(e: FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    setStatus(null);
    try {
      const portNum = Number.parseInt(port, 10);
      const saved = await saveEmailSettings({
        host: host.trim(),
        port: Number.isFinite(portNum) ? portNum : 993,
        user: user.trim(),
        mailbox: mailbox.trim() || "INBOX",
        password: password.trim() || undefined,
      });
      setSettings(saved);
      setPassword("");
      setStatus(
        saved.hasPassword
          ? "IMAP settings saved (password in Keychain)."
          : "IMAP settings saved — add a password to sync.",
      );
      if (saved.hasPassword) {
        await refresh({ sync: true });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  }

  async function onOpen(id: number) {
    try {
      await openEmail(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  const configured = Boolean(
    settings?.host && settings?.user && settings?.hasPassword,
  );

  function settingsForm() {
    const connector = getEmailConnector(connectorId);
    const isCustom = connectorId === "custom";
    return (
      <form className="notes-form" onSubmit={onSaveSettings}>
        <div className="email-connector-picker">
          {EMAIL_CONNECTORS.map((c) => (
            <button
              key={c.id}
              type="button"
              className={
                "email-connector-chip" +
                (c.id === connectorId ? " is-selected" : "")
              }
              onClick={() => onPickConnector(c.id)}
              title={c.description}
            >
              {c.name}
            </button>
          ))}
        </div>

        {connector.setupHint ? (
          <p className="email-connector-hint">
            {connector.setupHint}
            {connector.helpUrl ? (
              <>
                {" "}
                <button
                  type="button"
                  className="link-button"
                  onClick={() => {
                    const url = connector.helpUrl;
                    if (url) void openTarget("url", url).catch(() => {});
                  }}
                >
                  Learn more
                </button>
              </>
            ) : null}
          </p>
        ) : null}

        {isCustom ? (
          <div className="field-row">
            <input
              className="field"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              placeholder="IMAP host (e.g. imap.mail.me.com)"
              aria-label="IMAP host"
              autoComplete="off"
            />
            <input
              className="field"
              style={{ flex: "0 0 5.5rem" }}
              value={port}
              onChange={(e) => setPort(e.target.value)}
              placeholder="993"
              aria-label="IMAP port"
              inputMode="numeric"
            />
          </div>
        ) : null}
        <div className="field-row">
          <input
            className="field"
            value={user}
            onChange={(e) => setUser(e.target.value)}
            placeholder="Username / email"
            aria-label="IMAP username"
            autoComplete="username"
          />
          <input
            className="field"
            value={mailbox}
            onChange={(e) => setMailbox(e.target.value)}
            placeholder="INBOX"
            aria-label="Mailbox"
          />
        </div>
        <div className="field-row">
          <input
            className="field"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder={
              settings?.hasPassword
                ? "Password stored in Keychain — leave blank to keep"
                : "App password (Keychain only)"
            }
            aria-label="IMAP password"
            autoComplete="new-password"
          />
          <button type="submit" className="btn btn-primary" disabled={saving}>
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </form>
    );
  }

  function emailRows(rows: EmailMessage[]) {
    return (
      <ul className="module-list">
        {rows.map((msg) => (
          <li key={msg.id}>
            <button
              type="button"
              className="module-row-main shortcut-open"
              onClick={() => void onOpen(msg.id)}
            >
              <p className="module-row-title">{msg.subject || "(no subject)"}</p>
              <p className="module-row-meta">
                {emailSenderLabel(msg)}
                {msg.dateIso ? ` · ${formatEmailDate(msg.dateIso)}` : ""}
                {msg.preview
                  ? ` · ${msg.preview.slice(0, 64)}${msg.preview.length > 64 ? "…" : ""}`
                  : ""}
              </p>
            </button>
            <div className="row-actions">
              <button
                type="button"
                className="btn btn-primary btn-icon"
                onClick={() => void onOpen(msg.id)}
              >
                Open
              </button>
            </div>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <>
      <div data-module="email">
      <ModuleSection
        title="Email"
        eyebrow="Important"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setShowSettings((v) => !v)}
            >
              {showSettings ? "Hide" : "Settings"}
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              disabled={syncing || !configured}
              onClick={() => void refresh({ sync: true })}
            >
              {syncing ? "Syncing…" : "Sync"}
            </button>
            {configured && all.length > 0 ? (
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setShowAll(true)}
              >
                All
              </button>
            ) : null}
          </div>
        }
        count={configured && !loading ? all.length : null}
        accent="accent"
      >
        {!configured && !showSettings ? (
          <PermissionCallout
            title="IMAP setup"
            body="Add host, username, and an app password to sync important unread mail. Credentials stay in the macOS Keychain."
            steps={[
              "Use an app-specific password for iCloud or Gmail",
              "Save settings — password never touches SQLite",
              "Sync to pull important unread into Mainstream",
            ]}
            actionLabel="Open settings"
            onAction={() => setShowSettings(true)}
          />
        ) : null}

        {showSettings ? settingsForm() : null}

        {error ? <p className="module-empty">{error}</p> : null}
        {status ? <p className="module-empty">{status}</p> : null}
        {loading ? <p className="module-empty">Loading mail…</p> : null}

        {!loading && configured && top.length === 0 ? (
          <p className="module-empty">
            No important unread mail yet — sync after filtering junk and
            newsletters.
          </p>
        ) : null}

        {configured ? emailRows(top) : null}
      </ModuleSection>
      </div>

      <DetailDrawer
        open={showAll}
        title="All important mail"
        eyebrow="Email"
        onClose={() => setShowAll(false)}
      >
        {all.length === 0 ? (
          <p className="module-empty">No important unread mail.</p>
        ) : (
          emailRows(all)
        )}
      </DetailDrawer>
    </>
  );
}
