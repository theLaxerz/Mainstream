# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-08-31**. Version is still `0.1.0`. This is the queue for the next few increments — not a dump of every possible idea.

Mainstream already feels like a Life OS: analog + digital clock, weather, EventKit month + agenda, Today briefing chips, twelve live modules, ⌘K, dusk/light/auto, and a layout customizer. The next work should make first launch obvious, make the visual system actually ship, and add one or two moments that feel magical — not another module that sits empty.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark, analog clock + digital readout, Open-Meteo weather, EventKit month grid with selected-day agenda
- Today briefing: greeting, next event, unread chats, important email, steps, spend
- Modules: Messages (grouped unread → Messages.app), Calendar, Email (Google / Microsoft PKCE, Mail.app picker, IMAP fallback), USPS Informed Delivery OCR, News (ranked RSS), Finance (local ledger + CSV + 14-day spend chart), Notes, Health (Apple Health zip), Home (Ring + Blink stills), YouTube RSS, Streaming (TMDB Tonight / hot / new), Shortcuts
- Chrome: sticky command bar, ⌘K (jump / shortcuts / quick note), theme cycle, layout persist in SQLite, 15-minute quiet refresh, per-module last-sync
- Hardening: Tauri CSP, `freezePrototype`, Keychain secrets, IPC validators

Open PRs — do not rebuild these on another branch:

| PR | What | Action |
| --- | --- | --- |
| [#15](https://github.com/theLaxerz/Mainstream/pull/15) | Local Tasks module (due buckets, ⌘K “Task: …”, briefing chip) | Merge first. Highest-value missing module. |
| [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | IPC / zero-day hardening | Review one, close the other as duplicate |

Do **not** re-do: weather city picker, ⌘K skeleton, dusk theme tokens, health sparklines, Streaming Tonight card, today briefing, digital clock, hero agenda, finance spend chart / extra CSV banks, email browser sign-in, Blink OAuth stills.

README “Later” still lists “Reminders / Tasks” as unbuilt. After #15 lands, strike that line.

---

## Priority queue

### 1. First-run onboarding — **next increment**

The dashboard is a wall of empty modules until FDA, Calendars, a mailbox, a weather city, and (optionally) TMDB / Health / Home are set. New users cannot discover that path from the hero.

**Build a 5-step overlay** (skip-able, resumable, stored as `onboarding.completed` in settings):

1. **Permissions** — request Calendars from the app (required for the system prompt), deep-link Full Disk Access, show live status chips.
2. **Place** — weather city search (reuse `searchWeatherPlaces`).
3. **Inbox** — same Google / Microsoft / Mail.app cards as Email, just framed as “connect one account.”
4. **Taste** — TMDB key + 2–3 streaming services, or skip.
5. **Done** — “Refresh all” and jump to Today briefing.

**Looks:** full-viewport glass sheet, Fraunces headlines, one primary action per step — not a settings dump.

**Acceptance:** cold launch with empty `app.db` never shows twelve “Connect / Add / Sync” cards as the first impression. Returning users never see the overlay.

### 2. Ship the typeface (looks bug)

`theme.css` pulls Fraunces + Figtree from Google Fonts, but production CSP is `font-src 'self'`. Packaged builds fall back to Iowan / Avenir. The wordmark and module titles are designed around those faces.

Self-host the woff2 files under `src/assets/fonts/`, `@font-face` them, keep CSP tight. Biggest single visual upgrade for zero new UI.

### 3. YouTube posters + richer rows (coolness)

Streaming has TMDB posters and a Tonight hero. YouTube is a text list even though every RSS item has a public `hqdefault` / `mqdefault` thumb (`i.ytimg.com`).

- Persist `thumbnail_url` on `youtube_items`
- Card row: 16:9 thumb + title + relative time
- Allow `https://i.ytimg.com` in `img-src` (nothing else)

Same visual language as Streaming, much higher “this is a TV” feeling.

### 4. Native window chrome (looks + coolness)

The window is a generic 1280×860 chrome box. Life OS should feel like a Mac app.

- Hidden title bar + traffic lights inset
- `titleBarStyle: "overlay"` and a drag region on the command bar
- Optional vibrancy (`sidebar` / `underWindow`) behind the page gradient so dusk actually blends with the desktop
- Remember window size / position

Small Tauri config change, large “this belongs on my Mac” lift.

### 5. Menu-bar pulse (coolness)

A menu extra that shows next event + unread chat count, with “Open Mainstream” and “Refresh.” Uses data already in `loadDashboardPulse`. Makes the app useful when the window is closed — the gap between “dashboard” and “command center.”

### 6. After #15: EventKit Reminders sync

Do not start until Tasks is on master. Then optionally read/write Apple Reminders when Reminders TCC is granted. Local SQLite stays source of truth if access is denied. Keep the due-bucket UI; add a “Reminders” account chip, not a second module.

### 7. Command palette, round 2

⌘K already jumps, launches, and captures a note. After Tasks: `Task: …`, natural-time notes (`Note dinner ideas`), and a “pin city / connect email” setup action when those are empty. Fuzzy match (not `includes`). Keyboard hint row.

### 8. Refresh that feels alive

`refresh_dashboard` runs modules **sequentially** on the Rust side. Email + Informed Delivery + news + YouTube + TMDB + Health + Home + weather can stall the primary button for a long time. Status is a single string.

- Parallelize independent modules (email → mail still ordered)
- Stream per-module status into the command bar (or a tiny sync popover)
- Surface `refresh.intervalMinutes` in Layout (already persisted, never exposed)

### 9. Health without the zip ritual

Importing `export.zip` is correct for v1 and privacy-preserving, but it goes stale. Next: watch the saved export path, show “last imported” age, and a one-click re-import. Live HealthKit is a later entitlement fight — don’t start it until onboarding + fonts + window chrome are done.

### 10. Signing, notarization, auto-update

Still `0.1.0`, no updater plugin, README already documents unsigned local `.app`. Do this when someone other than the author should install it. Not the next coding increment.

---

## Looks — specific nits worth a polish pass

After onboarding + fonts, a short visual pass (one PR):

- **Brand wordmark** — the 8-layer 3D extrude is distinctive; in dusk it can glare. Add a quieter dusk treatment (fewer layers, softer fill).
- **Module sameness** — almost every card is title + meta + ghost buttons. Give Messages / Email / News a slightly denser row (avatar initial, unread pip, source mark) so the grid doesn’t read as twelve identical lists.
- **Notes** — inline title + textarea inside the card makes the module look like a form, not a journal. Move compose into the drawer; card shows the last 3 notes only.
- **Empty states** — PermissionCallout is good for TCC. Other empties are a single muted sentence. Add one illustration-or-glyph empty per unconfigured module (Home camera outline, Streaming ticket, etc.).
- **Favicon** — `index.html` still uses the Vite logo.
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`. Either follow the system when preference is Auto, or say “Sunset” in the button so the rule is honest.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login.
- **Tests** — only `emailConnectors.test.ts` on master (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.

---

## Suggested sequence for upcoming automations

1. Land or rebase **#15 Tasks**, close duplicate security PR.
2. **Onboarding overlay** (this document’s increment #1).
3. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together).
4. **YouTube thumbs** (#3).
5. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice.
6. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement onboarding instead.
