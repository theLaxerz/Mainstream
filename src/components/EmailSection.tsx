import { useEffect, useState, type FormEvent } from "react";
import {
  disconnectEmail,
  getEmailSettings,
  listAllImportantEmails,
  listImportantEmails,
  listMailAccounts,
  openEmail,
  openInternetAccounts,
  openTarget,
  saveEmailSettings,
  startEmailOauth,
  syncEmail,
  useMailAccount,
} from "../lib/api";
import { emailSenderLabel, formatEmailDate } from "../lib/email";
import {
  EMAIL_CONNECTORS,
  connectorIdForHost,
  emailAuthLabel,
  getEmailConnector,
  type EmailConnectorId,
} from "../lib/emailConnectors";
import { onDashboardRefresh } from "../lib/refresh";
import type {
  EmailMessage,
  EmailSettings,
  MailAppAccount,
  MailAppAccountsResult,
} from "../lib/types";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";
import { PermissionCallout } from "./PermissionCallout";

type OauthProvider = "google" | "microsoft";

type Props = { limit?: number };

export function EmailSection({ limit = 10 }: Props) {
  const [top, setTop] = useState<EmailMessage[]>([]);
  const [all, setAll] = useState<EmailMessage[]>([]);
  const [showAll, setShowAll] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showImap, setShowImap] = useState(false);
  const [settings, setSettings] = useState<EmailSettings | null>(null);
  const [accounts, setAccounts] = useState<MailAppAccountsResult | null>(null);
  const [connectorId, setConnectorId] = useState<EmailConnectorId>("icloud");
  const [host, setHost] = useState("");
  const [port, setPort] = useState("993");
  const [user, setUser] = useState("");
  const [mailbox, setMailbox] = useState("INBOX");
  const [password, setPassword] = useState("");
  const [googleClientId, setGoogleClientId] = useState("");
  const [microsoftClientId, setMicrosoftClientId] = useState("");
  const [oauthPrompt, setOauthPrompt] = useState<OauthProvider | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [saving, setSaving] = useState(false);
  const [busyProvider, setBusyProvider] = useState<string | null>(null);

  const configured = Boolean(settings?.connected);

  async function loadSettings() {
    const s = await getEmailSettings();
    setSettings(s);
    setHost(s.host);
    setPort(String(s.port || 993));
    setUser(s.user);
    setMailbox(s.mailbox || "INBOX");
    setConnectorId(s.host ? connectorIdForHost(s.host) : "icloud");
    setGoogleClientId(s.googleClientId);
    setMicrosoftClientId(s.microsoftClientId);
    if (!s.connected) {
      setShowSettings(true);
    }
    return s;
  }

  async function loadAccounts() {
    try {
      setAccounts(await listMailAccounts());
    } catch (e) {
      setAccounts({
        status: "unavailable",
        detail: e instanceof Error ? e.message : String(e),
        accounts: [],
      });
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
      const s = await loadSettings();
      if (!s.connected) {
        await loadAccounts();
      }
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

  async function onOauth(provider: OauthProvider) {
    const existing = provider === "google" ? googleClientId : microsoftClientId;
    if (!existing.trim()) {
      setOauthPrompt(provider);
      setShowSettings(true);
      return;
    }
    setBusyProvider(provider);
    setError(null);
    setStatus(
      "Waiting for the browser — click the account you’re already signed into.",
    );
    try {
      const saved = await startEmailOauth({
        provider,
        clientId: existing.trim(),
      });
      setSettings(saved);
      setOauthPrompt(null);
      setStatus(`Connected ${saved.displayName ?? saved.user}. Syncing…`);
      await refresh({ sync: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyProvider(null);
    }
  }

  async function onOauthWithClientId(provider: OauthProvider) {
    const clientId =
      provider === "google" ? googleClientId.trim() : microsoftClientId.trim();
    if (!clientId) {
      setError(
        provider === "google"
          ? "Paste a Google Desktop OAuth client ID first."
          : "Paste a Microsoft public client ID first.",
      );
      return;
    }
    setBusyProvider(provider);
    setError(null);
    setStatus(
      "Waiting for the browser — click the account you’re already signed into.",
    );
    try {
      const saved = await startEmailOauth({ provider, clientId });
      setSettings(saved);
      setOauthPrompt(null);
      setStatus(`Connected ${saved.displayName ?? saved.user}. Syncing…`);
      await refresh({ sync: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyProvider(null);
    }
  }

  async function onUseAccount(account: MailAppAccount) {
    setBusyProvider(`mailapp:${account.name}`);
    setError(null);
    try {
      const saved = await useMailAccount(account.name);
      setSettings(saved);
      setStatus(`Using ${account.userName || account.name} from Mail.app. Syncing…`);
      await refresh({ sync: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyProvider(null);
    }
  }

  async function onDisconnect() {
    setBusyProvider("disconnect");
    setError(null);
    try {
      const saved = await disconnectEmail();
      setSettings(saved);
      setTop([]);
      setAll([]);
      setStatus("Disconnected. Pick an account or sign in with your browser.");
      setShowSettings(true);
      await loadAccounts();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyProvider(null);
    }
  }

  function oauthCard(provider: OauthProvider) {
    const label = provider === "google" ? "Continue with Google" : "Continue with Microsoft";
    const hint =
      provider === "google"
        ? "Opens Google in your browser so you can click the Gmail account already signed in on this Mac."
        : "Opens Microsoft in your browser so you can click the Outlook / 365 account already signed in on this Mac.";
    const clientId = provider === "google" ? googleClientId : microsoftClientId;
    const setClientId = provider === "google" ? setGoogleClientId : setMicrosoftClientId;
    const needsId = oauthPrompt === provider || !clientId.trim();
    const placeholder =
      provider === "google"
        ? "Google Desktop client ID"
        : "Microsoft public client ID";
    const help =
      provider === "google"
        ? "One-time: Google Cloud Console → Credentials → Create Desktop client. No secret."
        : "One-time: Azure app registration → public client / mobile & desktop. Redirect http://127.0.0.1";
    const busy = busyProvider === provider;
    return (
      <div className={"email-oauth-card" + (oauthPrompt === provider ? " is-open" : "")}>
        <button
          type="button"
          className="email-oauth-btn"
          disabled={Boolean(busyProvider)}
          onClick={() => void onOauth(provider)}
        >
          <span className={"email-oauth-mark is-" + provider} aria-hidden>
            {provider === "google" ? "G" : "M"}
          </span>
          <span>
            <strong>{label}</strong>
            <em>{hint}</em>
          </span>
        </button>
        {needsId ? (
          <div className="email-oauth-id">
            <p className="email-connector-hint">{help}</p>
            <div className="field-row">
              <input
                className="field"
                value={clientId}
                onChange={(e) => setClientId(e.target.value)}
                placeholder={placeholder}
                aria-label={placeholder}
                autoComplete="off"
              />
              <button
                type="button"
                className="btn btn-primary"
                disabled={Boolean(busyProvider) || !clientId.trim()}
                onClick={() => void onOauthWithClientId(provider)}
              >
                {busy ? "Waiting…" : "Sign in"}
              </button>
            </div>
          </div>
        ) : null}
      </div>
    );
  }

  function mailAppList(rows: MailAppAccount[]) {
    if (!rows.length) return null;
    return (
      <div className="email-account-list">
        <p className="email-account-kicker">Accounts on this Mac</p>
        <ul className="module-list">
          {rows.map((account) => (
            <li key={account.name}>
              <button
                type="button"
                className="module-row-main shortcut-open"
                disabled={Boolean(busyProvider)}
                onClick={() => void onUseAccount(account)}
              >
                <p className="module-row-title">
                  {account.userName || account.name}
                </p>
                <p className="module-row-meta">
                  {account.name}
                  {account.kind ? ` · ${account.kind}` : ""}
                  {" · Mail.app"}
                </p>
              </button>
              <div className="row-actions">
                <button
                  type="button"
                  className="btn btn-primary btn-icon"
                  disabled={Boolean(busyProvider)}
                  onClick={() => void onUseAccount(account)}
                >
                  {busyProvider === `mailapp:${account.name}` ? "…" : "Use"}
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    );
  }

  function imapForm() {
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

  function settingsPanel() {
    return (
      <div className="email-connect">
        {configured && settings ? (
          <div className="email-connected">
            <p className="module-row-title">
              {settings.displayName || settings.user || "Connected"}
            </p>
            <p className="module-row-meta">{emailAuthLabel(settings)}</p>
            <button
              type="button"
              className="btn btn-danger"
              disabled={Boolean(busyProvider)}
              onClick={() => void onDisconnect()}
            >
              {busyProvider === "disconnect" ? "Disconnecting…" : "Disconnect"}
            </button>
          </div>
        ) : (
          <>
            <p className="email-connector-hint">
              Click an account already on this Mac, or sign in with your browser.
              Mainstream maps that login to your inbox — no IMAP app password
              for Google or Microsoft.
            </p>
            {accounts?.status === "needs_permission" ? (
              <PermissionCallout
                title="Allow Mail.app"
                body={
                  accounts.detail ??
                  "Mainstream can list Google and Microsoft accounts already signed into Mail."
                }
                steps={[
                  "System Settings → Privacy & Security → Automation",
                  "Enable Mail for Mainstream",
                  "Return here and pick an account",
                ]}
              />
            ) : null}
            {accounts?.status === "unavailable" ? (
              <PermissionCallout
                title="Mail.app didn’t respond"
                body={
                  accounts.detail ??
                  "Mail may be busy signing into Outlook. Finish that window, then try again."
                }
                steps={[
                  "Finish any Mail or Outlook sign-in dialogs",
                  "Return here and try again",
                ]}
                actionLabel="Try again"
                onAction={() => void loadAccounts()}
              />
            ) : null}
            {mailAppList(accounts?.accounts ?? [])}
            {oauthCard("google")}
            {oauthCard("microsoft")}
            <div className="row-actions email-extra-actions">
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => void openInternetAccounts()}
              >
                Internet Accounts…
              </button>
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setShowImap((v) => !v)}
              >
                {showImap ? "Hide other IMAP" : "Other IMAP"}
              </button>
            </div>
          </>
        )}
        {showImap || (configured && settings?.auth === "password") ? imapForm() : null}
      </div>
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
              onClick={() => {
                setShowSettings((v) => {
                  const next = !v;
                  if (next) void loadAccounts();
                  return next;
                });
              }}
            >
              {showSettings ? "Hide" : configured ? "Account" : "Connect"}
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
            title="Connect a mailbox"
            body="Use the Google or Microsoft account already signed in on this Mac, or sign in with your browser. App passwords are only needed for other IMAP providers."
            steps={[
              "Click Continue with Google or Microsoft — your browser lets you pick the signed-in account",
              "Or tap an account Mail.app already has",
              "Sync pulls important unread mail into Mainstream",
            ]}
            actionLabel="Connect"
            onAction={() => setShowSettings(true)}
          />
        ) : null}

        {showSettings ? settingsPanel() : null}

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
