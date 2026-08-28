# Mainstream

macOS Life OS — a local-first personal command center built with **Tauri 2 + React + TypeScript**.

Clock hero, unread Messages, important email (IMAP), USPS Informed Delivery mail (OCR), tailored news, local finance ledger, notes, **Health** (Apple Health export), **Home** (Ring & Blink), **YouTube** (channel RSS), **Streaming** (what's hot / new via TMDB), and app/website shortcuts. Data stays on-device; the web is only used for RSS and launching shortcuts.

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

### Email (IMAP)

IMAP host/user live in SQLite settings; passwords are stored only in the **macOS Keychain**. Use an app-specific password for iCloud or Gmail.

### Physical mail (Informed Delivery)

The **Mail** module searches your synced mailbox for USPS Informed Delivery digests, extracts envelope scan images, and runs **macOS Vision OCR** locally. Configure Email (IMAP) first, then use **Mail → Sync**. Scan images are cached under your app data directory.

### Health, Home, YouTube, Streaming

- **Health** — Import Apple Health `export.zip` / `export.xml` (steps, sleep, heart rate).
- **Home** — Ring (refresh token) and Blink (email/password) in Keychain; lists cameras and doorbells.
- **YouTube** — Add channel IDs; sync uses public YouTube RSS feeds.
- **Streaming** — Free [TMDB](https://www.themoviedb.org/) API key; pick services (Prime, Apple TV+, Paramount+, Peacock, AMC+, Netflix, Max, Disney+, Hulu). **What's hot** and **New & available** lists refresh on sync.

## Layout

- Sticky **command bar**: live clock, **Layout** customizer, **Refresh all**
- Shortcuts: `⌘,` customize layout · `⌘⇧R` refresh all modules
- Layout prefs (enable/order/width/item counts) persist in SQLite settings
- `src/` — React UI (dashboard, clock, module sections, detail drawers)
- `src-tauri/` — Rust core, SQLite (`app.db`), Tauri commands
- `feeds.default.json` — starter RSS feeds for the news module

SQLite lives in the app data directory as `app.db` (created on first launch).

## What's next

v1 already covers the command-center loop: modules, layout, unified refresh, and Calendar. The next increment (this branch) makes the **hero feel alive** — digital time + greeting, EventKit dots/agenda on the month grid, and a Today briefing that jumps to Messages, Calendar, Email, and Health.

Suggested follow-ups, in roughly this order:

1. **Weather in the hero** — local conditions via Open-Meteo (no API key), sitting beside the clock.
2. **⌘K command palette** — jump to a module, open a shortcut, or capture a note without leaving the keyboard.
3. **Quiet auto-refresh** — interval sync with per-module last-synced timestamps instead of only a manual Refresh all.
4. **Dusk theme** — time-of-day palette (and a manual toggle) so the glass cards work at night.
5. **Health sparklines** — steps/sleep as a 7-day chart, not just a list of days.
6. **Tonight on streaming** — one cinema-style featured title above the hot/new lists.
7. **Deeper Messages / finance** — in-app reply is still out of scope; charts and better CSV mapping would help the ledger before any bank API.

## Known limits (v1)

- macOS only
- No send/reply from Messages inside Mainstream
- No Plaid / live bank APIs — CSV import + local ledger only
- No multi-user sync or cloud backup
