# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-03** (`476a28b`, still `0.1.0`). This supersedes the 2 Sep queue in [#21](https://github.com/theLaxerz/Mainstream/pull/21), the 1 Sep queue in [#19](https://github.com/theLaxerz/Mainstream/pull/19), and the 31 Aug queue in [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should make **first launch obvious**, make the **visual system actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master did not move.** Tip is still `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **fourth** planning day stacked on the same commit. The queue is stable; shipping it is the bottleneck, not more discovery.
- **New open PR:** [#22](https://github.com/theLaxerz/Mainstream/pull/22) (2 Sep afternoon) is now the newest IPC / zero-day hardening branch. It re-applies the unmerged SSRF/input checks and adds IMAP host/port/CRLF guards, XOAUTH2 token charset, Blink region/thumbnail checks, YouTube URL allow-list, and RSS URL guards.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15)), yesterday’s plan (#21), and five overlapping security PRs.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts only in *browser* Vite — see fonts below), analog clock + digital readout + greeting, Open-Meteo weather, EventKit month grid with selected-day agenda
- Today briefing: greeting eyebrow, “At a glance” title, next event, unread chats, important email, steps, spend — chips scroll to the matching module
- Modules: Messages (grouped unread → Messages.app), Calendar, Email (Google / Microsoft PKCE, Mail.app picker with timeouts, IMAP fallback), USPS Informed Delivery OCR thumbs, News (ranked RSS, auto-seeds `feeds.default.json` on first refresh), Finance (local ledger + CSV + 14-day spend chart), Notes, Health (Apple Health zip, re-import on refresh, 7-day sparklines), Home (Ring + Blink stills), YouTube RSS (text list), Streaming (TMDB Tonight / hot / new with posters), Shortcuts (compose lives in the drawer)
- Chrome: sticky command bar, ⌘K (jump / shortcuts / quick note), theme cycle, layout persist in SQLite, 15-minute quiet refresh, per-module last-sync
- Hardening: Tauri CSP, `freezePrototype`, Keychain secrets, IPC validators. Rust unit tests exist in `security.rs`, email, finance, Mail.app, OAuth. Frontend tests are only `emailConnectors.test.ts`.

Open PRs — **do not rebuild these on another branch:**

| PR | What | Action |
| --- | --- | --- |
| [#15](https://github.com/theLaxerz/Mainstream/pull/15) | Local Tasks (due buckets, ⌘K `Task: …`, briefing chip) | **Merge first.** Highest-value missing module. |
| [#22](https://github.com/theLaxerz/Mainstream/pull/22) | IPC / zero-day hardening (2 Sep scan) | Review and land. Newest of the security set. |
| [#20](https://github.com/theLaxerz/Mainstream/pull/20) / [#18](https://github.com/theLaxerz/Mainstream/pull/18) / [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | Older copies of the same hardening | Close as duplicates of #22. |
| [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17) | 2 Sep / 1 Sep / 31 Aug What's Next | Close when this document lands. |

Do **not** re-do: weather city picker, ⌘K skeleton, dusk theme tokens, health sparklines, Streaming Tonight card, today briefing, digital clock, hero agenda, finance spend chart / extra CSV banks, email browser sign-in, Blink OAuth stills, Mail.app timeouts, or a second Tasks module.

README “Later” still lists “Reminders / Tasks” as unbuilt. After #15 lands, strike that line.

---

## Priority queue

### 1. First-run: hide the empty wall — **next coding increment**

Cold launch shows **twelve enabled modules**. Email does worse than “empty”: `loadSettings` calls `setShowSettings(true)` whenever `!connected`, and the connect UI is **in-card** (OAuth + Mail.app list + IMAP), not a drawer. Home, YouTube, Streaming, Mail, Health, and Finance are empty cards. News is the exception — it seeds default feeds *and* fetches RSS when the list is empty, so first paint is extra chatty. The hero and briefing cannot carry that.

Ship this as **one PR**, two layers (both are cheap; skip neither):

**A. Starter layout** — default `enabled: false` for Home, YouTube, Streaming, Mail, and Health until the user configures them (or until onboarding turns them on). Keep Messages, Calendar, Email, News, Finance, Notes, Shortcuts visible. Returning layouts in SQLite stay as-is (`normalizeLayout` already preserves stored enablement).

**B. 5-step overlay** (skip-able, resumable, `onboarding.completed` in settings):

1. **Permissions** — request Calendars from the app (required for the system prompt), deep-link Full Disk Access, live status chips.
2. **Place** — weather city search (reuse `searchWeatherPlaces`).
3. **Inbox** — Google / Microsoft / Mail.app cards. Mail.app is timeout-safe; lead with it. Move Email’s connect panel here (or into a drawer) so the card never inflates.
4. **Taste** — TMDB key + 2–3 services, or skip (enables Streaming).
5. **Done** — Refresh all, jump to Today briefing.

**Looks:** full-viewport glass sheet, Fraunces headlines, one primary action per step.

**Acceptance:** empty `app.db` never presents twelve “Connect / Add / Sync” cards. Returning users never see the overlay. Email must not auto-open its in-card settings on a first paint that already has the overlay. News must not fire a full RSS pull until the user (or Refresh all) asks.

### 2. Ship the typeface (looks bug)

`theme.css` `@import`s Fraunces + Figtree from Google Fonts. **Both** production CSP and `devCsp` set `font-src 'self'` and do not allow `fonts.googleapis.com` / `fonts.gstatic.com`. Packaged *and* `tauri dev` fall back to Iowan / Avenir. Browser-only `npm run dev` is the only place the designed faces load. The wordmark’s 8-layer extrude and every module title are designed around those faces.

Self-host woff2 under `src/assets/fonts/`, `@font-face`, keep CSP tight. Biggest visual upgrade with zero new UI.

### 3. YouTube posters + @handles (coolness)

Streaming has TMDB posters and a Tonight hero. YouTube is a text list even though every `video_id` has a public thumb at `https://i.ytimg.com/vi/{id}/mqdefault.jpg`. The `youtube_items` table has no `thumbnail_url` column — derivation from `video_id` needs no schema.

- Card row: 16:9 thumb + title + relative time (same visual language as Streaming)
- Allow `https://i.ytimg.com` in `img-src` only
- `normalize_channel_id` already strips `youtube.com/channel/UC…`. `@handle` / `youtube.com/@…` is stored as-is and the RSS URL fails. Resolve handle → `UC…` once on add.
- Align the card eyebrow (`Subscriptions` in `YouTubeSection`) with layout meta (`Channels` in `MODULE_META`).

### 4. Native window chrome (looks + coolness)

The window is a generic 1280×860 chrome box (`tauri.conf.json` has no `titleBarStyle`). Life OS should feel like a Mac app.

- Hidden title bar + traffic lights inset (`titleBarStyle: "overlay"`)
- Drag region on the command bar (leave extra left padding for the lights)
- Optional vibrancy behind the page gradient so dusk blends with the desktop
- Remember window size / position (`tauri-plugin-window-state`)

Small Tauri config change, large “this belongs on my Mac” lift. Pair with fonts in one PR if both stay small.

### 5. Menu-bar pulse (coolness)

A menu extra that shows next event + unread chat count, with “Open Mainstream” and “Refresh.” Data is already in `loadDashboardPulse`. Makes the app useful when the window is closed — the gap between “dashboard” and “command center.”

### 6. After #15: EventKit Reminders sync

Do not start until Tasks is on master. Then optionally read/write Apple Reminders when Reminders TCC is granted. Local SQLite stays source of truth if access is denied. Keep the due-bucket UI; add a “Reminders” account chip, not a second module. `Info.plist` will need a Reminders usage string when that ships.

### 7. Command palette, round 2

⌘K already jumps, launches, and captures a note. Matching is `includes`, not fuzzy. The command-bar button still says **Search**.

After Tasks: `Task: …`, natural-time notes, and a “pin city / connect email” setup action when those are empty. Fuzzy match. Keyboard hint row. Rename the button to **⌘K**.

### 8. Refresh that feels alive

`refresh_dashboard` runs modules **sequentially** while holding the SQLite mutex around each one. Email + Informed Delivery + news + YouTube + TMDB + Health + Home + weather can stall the primary button. Status is a single truncated string in the command bar.

News also keeps its **own** 15-minute timer (`NEWS_REFRESH_MS`) on top of the dashboard interval, so feeds can double-fetch.

- Parallelize independent modules (email → mail still ordered)
- Stream per-module status into the command bar (or a tiny sync popover)
- Drop the News-only timer; one clock is enough
- Surface `refresh.intervalMinutes` in Layout (already persisted in `appearance.ts`, never exposed — LayoutCustomize has toggle / items / column / reset only)

### 9. Health staleness + count badge (small)

`HealthDay.importedAt` is already on the type and in SQLite. The card never shows it. The header count is hardcoded to `1` whenever today exists (`count={!loading && today ? 1 : null}`) — looks broken. Omit the badge, or show days imported / a streak.

Add “last imported” age and a one-click re-import when the zip is older than a day. Live HealthKit is a later entitlement fight — don’t start it until onboarding + fonts + window chrome are done.

### 10. Signing, notarization, auto-update

Still `0.1.0`, no updater plugin, README already documents unsigned local `.app`. Do this when someone other than the author should install it. Not the next coding increment.

---

## Looks — specific nits worth a polish pass

After onboarding + fonts, a short visual pass (one PR):

- **Brand wordmark** — the 8-layer 3D extrude is distinctive; in dusk it can glare. Quieter dusk treatment (fewer layers, softer fill).
- **Double greeting** — `Clock` and `TodayBriefing` both call `greetingFor`. Keep it on the clock readout; let the briefing lead with “At a glance” only (drop the greeting eyebrow).
- **Three clocks** — command bar live time + analog face + digital hero. Compact mode already repeats the hero. Once overlay chrome lands, drop seconds from the command bar.
- **Unused skeleton** — `ModuleSkeleton` + shimmer CSS already exist. Every module still prints “Loading…”. Wire the skeleton on first paint.
- **Module sameness** — almost every card is title + meta + ghost buttons. Give Messages / Email / News a denser row (avatar initial, unread pip, source mark) so the grid doesn’t read as twelve identical lists.
- **Notes** — inline title + textarea inside the card makes the module look like a form, not a journal. Shortcuts already compose in the drawer; Notes should match. Card shows the last 3 notes only.
- **Empty states** — `PermissionCallout` is good for TCC. Other empties are a single muted sentence. One glyph empty per unconfigured module.
- **Favicon** — `index.html` still uses the Vite logo (`/vite.svg`).
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`. Either follow the system when preference is Auto, or label the button **Sunset** so the rule is honest.
- **Email in-card settings** — `setShowSettings(true)` on every disconnected load fights the overlay *and* stretches the Email slot to a full connect form. Gate the auto-open; move the form to a drawer (Email “All” is already a drawer).
- **Wide-grid “Right column”** — at `min-width: 1180px` the grid is 3 columns, but `.placement-right` jumps to `grid-column: 3 / 4`, leaving a hole in column 2. Map “right” to the last column honestly, or add a real third placement.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login.
- **Disconnected Email re-lists Mail.app on every quiet refresh** — `refresh()` → `loadSettings()` → `loadAccounts()` whenever `!connected`. That is AppleScript on a 15-minute timer even after the timeout fix. List accounts only when the user opens Connect.
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email, health, and finance on every Refresh all, in parallel with each module’s own load. Cheap locally, noisy on FDA / EventKit. Have modules publish into the briefing, or pass cached rows down.
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#22** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21.
2. **Onboarding + starter layout** (this document’s increment #1).
3. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together).
4. **YouTube thumbs + @handles** (#3).
5. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice.
6. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement onboarding instead.
