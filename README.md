# Mainstream

macOS Life OS — a local-first personal command center built with **Tauri 2 + React + TypeScript**.

Clock hero, unread Messages, important email (IMAP), tailored news, local finance ledger, notes, and app/website shortcuts. Data stays on-device; the web is only used for RSS and launching shortcuts.

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

## Layout

- `src/` — React UI (dashboard, clock, module sections, detail drawers)
- `src-tauri/` — Rust core, SQLite (`app.db`), Tauri commands
- `feeds.default.json` — starter RSS feeds for the news module

SQLite lives in the app data directory as `app.db` (created on first launch).

## Known limits (v1)

- macOS only
- No send/reply from Messages inside Mainstream
- No Plaid / live bank APIs — CSV import + local ledger only
- No multi-user sync or cloud backup
