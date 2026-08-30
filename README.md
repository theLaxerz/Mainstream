# Mainstream

macOS Life OS — a local-first personal command center built with **Tauri 2 + React + TypeScript**.

Clock hero with weather, unread Messages, important email (Google / Microsoft browser sign-in or Mail.app), USPS Informed Delivery mail (OCR), tailored news, local finance ledger, **Tasks**, notes, **Health** (Apple Health export), **Home** (Ring & Blink), **YouTube** (channel RSS), **Streaming** (what's hot / new via TMDB), and app/website shortcuts. Data stays on-device; the web is only used for RSS, weather, OAuth sign-in, and launching shortcuts.

## Run (dev)

```bash
npm install
npm run tauri dev
```

If `cargo` hits a disk-full error, free space first (e.g. `cargo clean --manifest-path src-tauri/Cargo.toml`).

## Checks

```bash
npm run build          # tsc + vite
npm run check:rust     # cargo check
```

## Package (.app)

```bash
npm run tauri build
```

Output lands under `src-tauri/target/release/bundle/macos/`. Signing uses your local identity when configured; unsigned builds still produce a runnable `.app` for local use.

### Full Disk Access (Messages)

Messages reads `~/Library/Messages/chat.db`. macOS requires **Full Disk Access**:

1. System Settings → Privacy & Security → Full Disk Access
2. Enable **Mainstream** (or your terminal / IDE when using `tauri dev`)
3. Quit and reopen the app, then Refresh

There is no entitlement that grants FDA — it is a user TCC grant. The app deep-links to the settings pane and shows an empty state until access works.

### Calendar

Calendar reads upcoming events through EventKit. macOS requires **Calendars** access:

1. Load the dashboard so Mainstream can request access (a system prompt should appear)
2. If you declined, or no prompt appeared: System Settings → Privacy & Security → Calendars
3. Enable **Mainstream** (or your terminal / IDE when using `tauri dev`), then Refresh

The first request has to come from Mainstream itself. After that, Mainstream shows up in the Calendars list so you can toggle it.

### Email

Google and Microsoft use **browser sign-in** (PKCE). Mainstream opens your browser so you can click the account already signed in on this Mac. Refresh tokens stay in the **macOS Keychain**; mail is still fetched over IMAP with XOAUTH2.

You can also click an account already configured in **Mail.app** (Internet Accounts). iCloud / Yahoo / Fastmail still use IMAP + an app password.

One-time OAuth app IDs (public desktop / public client — no secret):

- Google: Cloud Console → APIs & Services → Credentials → Desktop client
- Microsoft: Azure app registration → public client / mobile & desktop, redirect `http://127.0.0.1`

### Physical mail (Informed Delivery)

The **Mail** module searches your synced mailbox for USPS Informed Delivery digests, extracts envelope scan images, and runs **macOS Vision OCR** locally. Connect Email first, then use **Mail → Sync**. Scan images are cached under your app data directory.

### Health, Home, YouTube, Streaming

- **Health** — Import Apple Health `export.zip` / `export.xml` (steps, sleep, heart rate) with 7-day sparklines.
- **Home** — Ring (refresh token in Keychain) and Blink (OAuth + 2FA PIN). Blink refresh tokens stay in Keychain; camera stills are cached locally and can be refreshed with **Snap**.
- **YouTube** — Add channel IDs; sync uses public YouTube RSS feeds.
- **Streaming** — Free [TMDB](https://www.themoviedb.org/) API key; pick services (Prime, Apple TV+, Paramount+, Peacock, AMC+, Netflix, Max, Disney+, Hulu). A **Tonight** featured title sits above **What's hot** and **New & available**.
- **Weather** — Pin a city in the hero; [Open-Meteo](https://open-meteo.com/) forecast, no API key.
- **Finance** — Local accounts + CSV import (Apple Card, Chase, Bank of America, Capital One, Citi, Discover, or generic). The card shows a 14-day spend chart, this-month total, and category chips. No bank APIs.
- **Tasks** — Local due list beside Calendar (overdue / today / upcoming / someday). Quick-add with Today / Tomorrow chips, high priority, and complete-in-place. ⌘K can capture a task due today. No EventKit Reminders sync yet.

## Layout

- Sticky **command bar**: live clock, **⌘K** palette, theme (auto / dusk / light), **Layout** customizer, **Refresh all** with last-sync status
- **Today briefing**: next event, unread chats, important email, steps, this-month spend, and due tasks — chips scroll to the matching module
- Hero calendar shows EventKit dots and a selected-day agenda (looks back within the visible month)
- Shortcuts: `⌘K` command palette (jump, note, or task) · `⌘,` customize layout · `⌘⇧R` refresh all modules
- Quiet auto-refresh every 15 minutes (pauses while the window is hidden); modules show last-synced time
- Layout prefs (enable/order/width/item counts) persist in SQLite settings
- `src/` — React UI (dashboard, clock, weather, module sections, command palette)
- `src-tauri/` — Rust core, SQLite (`app.db`), Tauri commands
- `feeds.default.json` — starter RSS feeds for the news module

SQLite lives in the app data directory as `app.db` (created on first launch).

## Known limits (v1)

- macOS only
- Google / Microsoft browser sign-in needs a one-time public OAuth client ID (Desktop / public client). Mail.app accounts do not.
- No send/reply from Messages inside Mainstream
- Blink uses the unofficial OAuth API (same as Home Assistant / blinkpy); Amazon does not offer an official third-party camera API
- No Plaid / live bank APIs — CSV import + local ledger only
- Tasks are local SQLite — they do not sync with Apple Reminders yet
- No multi-user sync or cloud backup

## Later

- EventKit Reminders sync (read/write when TCC allows)
- In-app Messages reply
- First-run onboarding
- Signing, notarization, and auto-update
