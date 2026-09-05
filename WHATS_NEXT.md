# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-05** (`476a28b`, still `0.1.0`). This supersedes the 4 Sep queue in [#24](https://github.com/theLaxerz/Mainstream/pull/24), plus [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make the **typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **sixth** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck.
- **Security PR #25 exists.** The 4 Sep afternoon scan succeeded ([#25](https://github.com/theLaxerz/Mainstream/pull/25)). It re-applies #14/#16/#18/#20/#22 plus obscured-IPv4 / DNS-rebinding checks, Vite `^7.3.6`, and zip `2.4`. Master is still Vite `7.3.2` / zip `2.2`. **#25 is now the newest hardening branch** — #22 is no longer the tip of that stack.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write.
- **New finding:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **New finding:** Email’s badge is `all.length` while the card renders `top` (`limit`). Health already re-parses the export zip on every Refresh all (`try_import_configured`) — yesterday’s “add a re-import button” was the wrong diagnosis.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15)), five stacked What’s Next docs, six overlapping security PRs.

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
| [#25](https://github.com/theLaxerz/Mainstream/pull/25) | IPC / zero-day hardening (4 Sep scan) | Review and land. Newest of the security set. |
| [#22](https://github.com/theLaxerz/Mainstream/pull/22) / [#20](https://github.com/theLaxerz/Mainstream/pull/20) / [#18](https://github.com/theLaxerz/Mainstream/pull/18) / [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | Older copies of the same hardening | Close as duplicates of #25. |
| [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17) | 4 Sep / 3 Sep / 2 Sep / 1 Sep / 31 Aug What's Next | Close when this document lands. |

Do **not** re-do: weather city picker, ⌘K skeleton, dusk theme tokens, health sparklines, Streaming Tonight card, today briefing, digital clock, hero agenda, finance spend chart / extra CSV banks, email browser sign-in, Blink OAuth stills, Mail.app timeouts, or a second Tasks module.

README “Later” still lists “Reminders / Tasks” as unbuilt. After #15 lands, strike that line.

---

## Priority queue

### 0. Unlock non-secret settings — **do this before onboarding**

`is_generic_setting_key` in `src-tauri/src/security.rs` only matches `dashboard.layout.v1`. Both get and set go through `require_generic_setting_key`. The frontend already writes:

- `appearance.theme` (`saveThemePreference` / `loadThemePreference`) — catch is empty, so the command-bar cycle looks like it worked until relaunch, and every launch reloads `"auto"`
- `refresh.intervalMinutes` (`loadRefreshIntervalMinutes`) — always falls back to 15

A future `onboarding.completed` flag would be denied the same way, so a skip-able overlay would greet every launch.

Allow-list **only** those three keys. Keep secrets on the existing denylist. Dedicated command keys that already live in the `settings` table must stay off the generic IPC:

- `weather.place` / `weather.snapshot` — weather commands
- `health.export_path` — health commands
- `streaming.tmdb_api_key` / `home.blink_email` — already secret-flagged

Extend `setting_keys_are_scoped` so theme / refresh / onboarding pass, `streaming.tmdb_api_key` still fails, and a weather/health key is not treated as generic. Tiny PR; blocks increment 1.

### 1. First-run: hide the empty wall — **next product increment**

Cold launch shows **twelve enabled modules**. Email does worse than “empty”: `loadSettings` calls `setShowSettings(true)` whenever `!connected`, and the connect UI is **in-card** (OAuth + Mail.app list + IMAP), not a drawer. Home, YouTube, Streaming, Mail, Health, and Finance are empty cards. News is the exception — it seeds default feeds *and* fetches RSS when the list is empty, so first paint is extra chatty. The hero and briefing cannot carry that.

Ship this as **one PR**, two layers (both are cheap; skip neither):

**A. Starter layout** — default `enabled: false` for Home, YouTube, Streaming, Mail, and Health until the user configures them (or until onboarding turns them on). Keep Messages, Calendar, Email, News, Finance, Notes, Shortcuts visible. Returning layouts in SQLite stay as-is (`normalizeLayout` already preserves stored enablement).

**B. 5-step overlay** (skip-able, resumable, `onboarding.completed` — requires increment 0):

1. **Permissions** — request Calendars from the app (required for the system prompt), deep-link Full Disk Access, live status chips.
2. **Place** — weather city search (reuse `searchWeatherPlaces`).
3. **Inbox** — Google / Microsoft / Mail.app cards. Mail.app is timeout-safe; lead with it. Move Email’s connect panel here (or into a drawer) so the card never inflates.
4. **Taste** — TMDB key + 2–3 services, or skip (enables Streaming).
5. **Done** — Refresh all, jump to Today briefing.

**Looks:** full-viewport glass sheet, Fraunces headlines, one primary action per step.

**Acceptance:** empty `app.db` never presents twelve “Connect / Add / Sync” cards. Returning users never see the overlay after a real quit/relaunch (not just in-session state). Email must not auto-open its in-card settings on a first paint that already has the overlay. News must not fire a full RSS pull until the user (or Refresh all) asks.

### 2. Ship the typeface (looks bug)

`theme.css` `@import`s Fraunces + Figtree from Google Fonts. **Both** production CSP and `devCsp` set `font-src 'self'` and do not allow `fonts.googleapis.com` / `fonts.gstatic.com`. Packaged *and* `tauri dev` fall back to Iowan / Avenir. Browser-only `npm run dev` is the only place the designed faces load. The wordmark’s 8-layer extrude and every module title are designed around those faces.

Self-host woff2 under `src/assets/fonts/`, `@font-face`, keep CSP tight. Biggest visual upgrade with zero new UI.

### 3. YouTube posters + @handles (coolness)

Streaming has TMDB posters and a Tonight hero. YouTube is a text list even though every `video_id` has a public thumb at `https://i.ytimg.com/vi/{id}/mqdefault.jpg`. The `youtube_items` table has no `thumbnail_url` column — derivation from `video_id` needs no schema.

- Card row: 16:9 thumb + title + relative time (same visual language as Streaming)
- Allow `https://i.ytimg.com` in `img-src` only
- `normalize_channel_id` already strips `youtube.com/channel/UC…`. `@handle` / `youtube.com/@…` is stored as-is and the RSS URL fails. Resolve handle → `UC…` once on add.
- Align the card eyebrow (`Subscriptions` in `YouTubeSection`) with layout meta (`Channels` in `MODULE_META`). Same class of mismatch: News card says **For you**, meta says **Tailored**.

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

News also keeps its **own** 15-minute timer (`NEWS_REFRESH_MS`) on top of the dashboard interval, so feeds can double-fetch. That timer does **not** pause when the window is hidden (the dashboard interval does).

- Parallelize independent modules (email → mail still ordered)
- Stream per-module status into the command bar (or a tiny sync popover)
- Drop the News-only timer; one clock is enough
- Surface `refresh.intervalMinutes` in Layout — but only after increment 0 actually persists it. LayoutCustomize today is toggle / items / column / reset only.

### 9. Health staleness (small — diagnosis corrected)

`HealthDay.importedAt` is already on the type and in SQLite. The card never shows it. The header count is hardcoded to `1` whenever today exists (`count={!loading && today ? 1 : null}`) — looks broken. Omit the badge, or show days imported / a streak.

**Refresh already re-imports.** `refresh_health_module` → `try_import_configured` re-parses the zip/xml on every Refresh all, with no mtime/hash short-circuit. Large exports make Refresh all feel stuck after email + mail + news. Skip the parse when the file has not changed; show “last imported” age; keep a manual Import for when the user dropped a new export.

Live HealthKit is a later extraction fight — don’t start it until onboarding + fonts + window chrome are done.

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
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`. Either follow the system when preference is Auto, or label the button **Sunset** so the rule is honest. Theme also does not survive relaunch until increment 0.
- **Email in-card settings** — `setShowSettings(true)` on every disconnected load fights the overlay *and* stretches the Email slot to a full connect form. Gate the auto-open; move the form to a drawer (Email “All” is already a drawer).
- **Email count vs list** — badge is `all.length` (`listAllImportantEmails`, unbounded) while the card shows `top` (`limit`). A mailbox with 40 important threads badges `40` over 10 rows. Badge the visible list, or label it “40 important”.
- **Wide-grid “Right column”** — at `min-width: 1180px` the grid is 3 columns. CSS `.placement-right` jumps to `grid-column: 3 / 4` (leaves a hole in column 2). The unused helper `placementGridStyle()` maps `"right"` to `2 / 3` — so if someone wires the helper later, “right” still would not mean the last column. Pick one: last column, or drop “right” until there is a real 3-slot placer.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login.
- **Disconnected Email re-lists Mail.app on every quiet refresh** — `onDashboardRefresh` → `refresh()` → `loadSettings()` → `loadAccounts()` whenever `!connected`. That is AppleScript on a 15-minute timer even after the timeout fix. List accounts only when the user opens Connect.
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email, health, and finance on every Refresh all, in parallel with each module’s own load (hero Calendar is a third EventKit read). Cheap locally, noisy on FDA / EventKit. Have modules publish into the briefing, or pass cached rows down.
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #25 became the security tip and because increment 0 / Health / Email-count needed tighter acceptance.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#25** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change. Do not start onboarding without it. Do not allow-list weather/health keys.
3. **Onboarding + starter layout** (increment #1).
4. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together).
5. **YouTube thumbs + @handles** (#3).
6. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice.
7. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement the settings allowlist and onboarding instead.
