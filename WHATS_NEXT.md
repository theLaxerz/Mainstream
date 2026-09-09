# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-09** (`476a28b`, still `0.1.0`). This supersedes the 8 Sep queue in [#32](https://github.com/theLaxerz/Mainstream/pull/32), plus [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make the **typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **tenth** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck. Close the older What’s Next drafts; do not treat another scan as new product work.
- **Security tip moved to #33.** The 8 Sep afternoon scan ([#33](https://github.com/theLaxerz/Mainstream/pull/33)) re-applies the 5–7 Sep IPC/SSRF set, then adds Azure WireServer `168.63.129.16` (plus NAT64 wrap), `kubernetes.default` / Docker helper hostnames, a Vite `allowedHosts` pin (never `true`), `quick-xml` 0.41, and a lockfile poison test. #31 / #29 / #27 / #25 / #22 / #20 / #18 / #16 / #14 are older copies. Master is still Vite `7.3.2` / zip `2.2` / Tauri features `[]` / `allowedHosts` unset.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write.
- **Still true:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **Still true:** Email’s badge is `all.length` while the card renders `top` (`limit`). `list_all_important_emails` is already clamped to **500**. Health already re-parses the export zip on every Refresh all (`try_import_configured`, no mtime). Command-bar clock is `aria-live="polite"` on a 1s tick. `list_settings` is unused and dumps dedicated keys. Google Fonts `@import` is blocked by `connect-src` (not only `font-src`). YouTube thumbs need `i.ytimg.com` in `img-src`. Today briefing and ⌘K ignore layout. Locals are labeled `skipped` then reloaded via `REFRESH_EVENT`. `refresh.intervalMinutes` has a loader and no saver. Overlay title bar / tray need feature flags + capabilities beyond `core:default`.
- **New finding — first-paint `defaultLayout()` is not just flicker.** `layoutReady` only flips the “Loading layout…” meta string. The grid renders immediately from `useState(defaultLayout)` — all 12 modules enabled. Each card’s `useEffect` fires **before** SQLite layout arrives:
  - Email: `loadSettings` → `setShowSettings(true)` + `list_mail_accounts` (AppleScript) when disconnected
  - News: `bootstrap` calls `refreshNews()` when the story table is empty (seeds `feeds.default.json` and hits the network)
  - Home / Streaming / YouTube / Health / Messages / Calendar all start their IPC
  Then the saved layout unmounts the hidden ones. Waiting for `layoutReady` is a functional fix (wasted TCC, Mail.app, RSS, cameras), not a polish nit.
- **New finding — Today briefing collapses the page.** `TodayBriefing` returns `null` until `loadDashboardPulse` settles (EventKit + all unread Messages + 20 important emails + Health + Finance). The modules-head + grid jump down after that. Combined with the default-12 first paint, first launch has two layout shifts. Reserve a briefing-height skeleton (or keep the section mounted with empty chips) as part of increment 1.
- **New finding — Refresh all can stack.** The 15-minute interval calls `void onRefresh()` without checking `refreshing`. The button disables; the timer does not. A slow Health zip + IMAP + Informed Delivery + TMDB can still be running when the next tick fires. Guard with an in-flight flag (increment 8).
- **New finding — even after a saver exists, the interval never rebinds.** `loadRefreshIntervalMinutes` runs once in a `useEffect` keyed only on `[onRefresh]`. Adding `saveRefreshIntervalMinutes` + a Layout control is not enough — store minutes in state and restart the timer when they change.
- **New finding — YouTube `@handle` is a silent store-garbage path.** `normalize_channel_id` accepts `UC…` and `youtube.com/channel/…`. Anything else (`@mkbhd`, `youtube.com/@name`) is stored verbatim and the RSS URL 404s. Do not persist a handle as a channel id. Resolve once to `UC…`, or reject with a clear error. Public RSS is channel-id only.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15), last updated 30 Aug), nine stacked What’s Next docs, ten overlapping security PRs. **Twenty** open drafts.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts `@import` — blocked in Tauri; see fonts below), analog clock + digital readout + greeting, Open-Meteo weather, EventKit month grid with selected-day agenda
- Today briefing: greeting eyebrow, “At a glance” title, next event, unread chats, important email, steps, spend — chips scroll to the matching module. Section is unmounted until pulse settles.
- Modules: Messages (grouped unread → Messages.app), Calendar, Email (Google / Microsoft PKCE, Mail.app picker with timeouts, IMAP fallback), USPS Informed Delivery OCR thumbs, News (ranked RSS, auto-seeds `feeds.default.json` on first refresh), Finance (local ledger + CSV + 14-day spend chart), Notes, Health (Apple Health zip, re-import on refresh, 7-day sparklines), Home (Ring + Blink stills), YouTube RSS (text list, `UC…` / `youtube.com/channel/…` only), Streaming (TMDB Tonight / hot / new with posters), Shortcuts (compose lives in the drawer)
- Chrome: sticky command bar, ⌘K (jump / shortcuts / quick note), theme cycle, layout persist in SQLite, 15-minute quiet refresh, per-module last-sync (remote `ok`/`error` only)
- Hardening: Tauri CSP, `freezePrototype`, Keychain secrets, IPC validators. Rust unit tests exist in `security.rs`, email, finance, Mail.app, OAuth. Frontend tests are only `emailConnectors.test.ts`.

Open PRs — **do not rebuild these on another branch:**

| PR | What | Action |
| --- | --- | --- |
| [#15](https://github.com/theLaxerz/Mainstream/pull/15) | Local Tasks (due buckets, ⌘K `Task: …`, briefing chip) | **Merge first.** Highest-value missing module. Last touched 30 Aug. |
| [#33](https://github.com/theLaxerz/Mainstream/pull/33) | IPC / SSRF / Vite host pin / quick-xml 0.41 (8 Sep scan) | Review and land. Newest of the security set. |
| [#31](https://github.com/theLaxerz/Mainstream/pull/31) / [#29](https://github.com/theLaxerz/Mainstream/pull/29) / [#27](https://github.com/theLaxerz/Mainstream/pull/27) / [#25](https://github.com/theLaxerz/Mainstream/pull/25) / [#22](https://github.com/theLaxerz/Mainstream/pull/22) / [#20](https://github.com/theLaxerz/Mainstream/pull/20) / [#18](https://github.com/theLaxerz/Mainstream/pull/18) / [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | Older copies of the same hardening | Close as duplicates of #33. |
| [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17) | 8 Sep / 7 Sep / 6 Sep / 5 Sep / 4 Sep / 3 Sep / 2 Sep / 1 Sep / 31 Aug What's Next | Close when this document lands. |

Do **not** re-do: weather city picker, ⌘K skeleton, dusk theme tokens, health sparklines, Streaming Tonight card, today briefing, digital clock, hero agenda, finance spend chart / extra CSV banks, email browser sign-in, Blink OAuth stills, Mail.app timeouts, or a second Tasks module.

README “Later” still lists “Reminders / Tasks” as unbuilt. After #15 lands, strike that line.

---

## Priority queue

### 0. Unlock non-secret settings — **do this before onboarding**

`is_generic_setting_key` in `src-tauri/src/security.rs` only matches `dashboard.layout.v1`. Both get and set go through `require_generic_setting_key`. The frontend already writes:

- `appearance.theme` (`saveThemePreference` / `loadThemePreference`) — catch is empty, so the command-bar cycle looks like it worked until relaunch, and every launch reloads `"auto"`
- `refresh.intervalMinutes` (`loadRefreshIntervalMinutes`) — always falls back to 15. **There is no `saveRefreshIntervalMinutes`.** Unlocking the key does not persist a custom interval until Layout grows a control (increment 8), and the timer will not pick up a new value until it is stored in React state (9 Sep).

A future `onboarding.completed` flag would be denied the same way, so a skip-able overlay would greet every launch.

Allow-list **only** those three keys. Keep secrets on the existing denylist. Dedicated command keys that already live in the `settings` table must stay off the generic IPC:

- `weather.place` / `weather.snapshot` — weather commands
- `health.export_path` — health commands
- `streaming.tmdb_api_key` / `home.blink_email` — already secret-flagged

Do **not** allow-list anything just because `list_settings` can read it. That command is unused; if you touch this file, delete `list_settings` (and the `api.ts` wrapper) so dedicated keys are not dumped through a second door.

Extend `setting_keys_are_scoped` so the three new keys pass and a dedicated key still fails.

### 1. First-run: wait for layout, starter grid, onboarding overlay

Three stacked problems, one increment:

1. **Do not render the module grid until `layoutReady`.** Today the meta line says “Loading layout…” while all 12 cards mount and start IPC. Show the hero + a briefing-height placeholder; keep the grid empty (or a single skeleton) until SQLite answers. This stops the default-12 flicker **and** the Mail.app / News-seed / camera side effects.
2. **Starter layout** for a brand-new `app.db`: Messages, Calendar, Email, News, Notes, Shortcuts. Hide Mail, Finance, Health, Home, YouTube, Streaming until the user turns them on (or onboarding enables them). Returning users keep their saved layout.
3. **Five-step overlay** (`onboarding.completed`), skip-able:

   1. **Permissions** — request Calendars from the app (required for the system prompt), deep-link Full Disk Access, live status chips.
   2. **Place** — weather city search (reuse `searchWeatherPlaces`).
   3. **Inbox** — Google / Microsoft / Mail.app cards. Mail.app is timeout-safe; lead with it. **Move Email’s connect panel into this step (or a drawer).** Do not leave `setShowSettings(true)` on every disconnected `loadSettings` — that fights the overlay and stretches the Email slot into a full form. The in-card “Connect” empty state + `PermissionCallout` is enough until the user asks.
   4. **Watch** — optional: TMDB key + 2–3 services, or 1–2 YouTube channels (`@handle` once increment 3 exists; `UC…` is fine for v1).
   5. **Done** — “Refresh all” once. Mark `onboarding.completed`.

Honor enabled modules from day one:

- `refresh_dashboard` must take enabled IDs (or skip hidden *and* unconfigured). Hidden Streaming / YouTube / Home must not hit TMDB / RSS / cameras every 15 minutes.
- ⌘K “Go to” and Today briefing chips must use `enabledModules(layout)`, not `MODULE_IDS` / every pulse source. `CommandPalette` currently does not receive `layout` at all.
- Keep the briefing section mounted (skeleton or empty chips) so the grid does not jump when pulse arrives.

Skip is a first-class outcome. Do not auto-open Email settings after skip.

### 2. Self-host Fraunces + Figtree

`font-src 'self'` is set in **both** prod CSP and `devCsp`. The `@import` in `theme.css` is also a `connect-src` violation in Tauri. Browser `vite` without the WebView CSP can still load Google Fonts; `tauri dev` and the packaged `.app` cannot.

Vendor the OFL woff2 files under `src/assets/fonts/` (or `@fontsource/fraunces` + `@fontsource/figtree`) and switch to `@font-face`. Do **not** add `fonts.googleapis.com` / `fonts.gstatic.com` to CSP. `index.html` has no font `<link>` — the only loader is the CSS `@import`.

This is the highest-leverage looks change. The 3D wordmark and clock readout are designed for Fraunces; without it they fall back to Iowan / Palatino and the page feels generic.

### 3. YouTube posters + @handles (coolness)

The card is a text list: title + `channelTitle · date`. Streaming already has posters (`image.tmdb.org` is in `img-src`). YouTube should match.

- Derive `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg` from the existing `videoId`. No new SQLite column.
- Add `https://i.ytimg.com` to `img-src` (prod + `devCsp`). Optionally `https://i9.ytimg.com`.
- Accept `@handle` / `youtube.com/@name` in `normalize_channel_id`. **Resolve handle → `UC…` once** (channel page / public RSS), then store `UC…`. Today a pasted `@handle` is written through as the channel id and the feed 404s. Do not persist a handle.
- Eyebrow is `"Subscriptions"` in the card and `"Channels"` in `MODULE_META`. Pick one.

Small, visible, and it makes the grid feel less like twelve identical lists.

### 4. Overlay title bar (looks)

`tauri.conf.json` windows still use the default decorated title bar (`title` / `width` / `height` / min size only). The sticky command bar sits *under* traffic lights.

This increment is **not** “set `titleBarStyle: Overlay` and ship”:

- `capabilities/default.json` is `core:default` only — drag-region / window controls need `core:window:*` (or the specific window permissions)
- `Cargo.toml` has `tauri = { version = "2", features = [] }` — no `macos-private-api`, no tray
- `tauri-plugin-window-state` is not a dependency

Then:

- `titleBarStyle: "Overlay"`, `hiddenTitle: true`, `transparent` if you want vibrancy
- `data-tauri-drag-region` on the command bar (leave extra left padding for the lights)
- Optional vibrancy behind the page gradient so dusk blends with the desktop
- Remember window size / position (`tauri-plugin-window-state`)
- Enable the Tauri features / capabilities those APIs need — `core:default` alone will reject drag-region and window-state calls

Small config change **plus** capability grants, large “this belongs on my Mac” lift. Pair with fonts in one PR if both stay small.

### 5. Menu-bar pulse (coolness)

A menu extra that shows next event + unread chat count, with “Open Mainstream” and “Refresh.” Data is already in `loadDashboardPulse`. Makes the app useful when the window is closed — the gap between “dashboard” and “command center.”

`tray-icon` is in `Cargo.lock` via Tauri but **not** in `Cargo.toml` features and **not** wired in `tauri.conf.json`. This increment enables the feature, adds tray permissions, then the extra.

Honor the layout here too: if Messages is hidden, do not show an unread count that jumps to a missing card.

### 6. After #15: EventKit Reminders sync

Do not start until Tasks is on master. Then optionally read/write Apple Reminders when Reminders TCC is granted. Local SQLite stays source of truth if access is denied. Keep the due-bucket UI; add a “Reminders” account chip, not a second module. `Info.plist` will need a Reminders usage string when that ships.

### 7. Command palette, round 2

⌘K already jumps, launches, and captures a note. Matching is `includes`, not fuzzy. The command-bar button still says **Search**. “Go to” currently lists all twelve modules even when hidden. Palette does not take `layout` as a prop.

After Tasks: `Task: …`, natural-time notes, and a “pin city / connect email” setup action when those are empty. Fuzzy match. Keyboard hint row. Rename the button to **⌘K**. Hide or disable “Go to” for modules the layout has turned off.

### 8. Refresh that feels alive

`refresh_dashboard` runs modules **sequentially** while holding the SQLite mutex around each one. Email + Informed Delivery + news + YouTube + TMDB + Health + Home + weather can stall the primary button. Status is a single truncated string that only counts `ok`. Hidden modules still sync. Locals are labeled `skipped` even though `REFRESH_EVENT` reloads them.

News also keeps its **own** 15-minute timer (`NEWS_REFRESH_MS`) on top of the dashboard interval, so feeds can double-fetch. That timer does **not** pause when the window is hidden (the dashboard interval does).

The dashboard interval itself can overlap a still-running refresh (9 Sep) and will not restart if the user later changes minutes in Layout.

- Skip modules the layout has disabled (and still skip unconfigured)
- Parallelize independent modules (email → mail still ordered)
- Stream per-module status into the command bar (or a tiny sync popover); say “News synced · 6 skipped” for *remote* skips only — or mark locals `ok` (“Reloaded”)
- Drop the News-only timer; one clock is enough
- Guard overlapping Refresh all (`refreshing` / in-flight)
- Surface `refresh.intervalMinutes` in Layout — add `saveRefreshIntervalMinutes` **and** keep minutes in React state so the timer rebinds. LayoutCustomize today is toggle / items / column / reset only.

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
- **Three clocks, three timers** — command bar live time + analog face + digital hero, each with its own 1s `setInterval`. Compact mode already repeats the hero. Once overlay chrome lands, drop seconds from the command bar and share one tick.
- **Command-bar `aria-live`** — VoiceOver should not hear the clock every second. The hero clock already does this right (`aria-hidden` analog + a visually-hidden time).
- **Unused skeleton** — `ModuleSkeleton` + shimmer CSS already exist. Every module still prints “Loading…”. Wire the skeleton on first paint, and use it for the briefing slot while pulse loads.
- **Module sameness** — almost every card is title + meta + ghost buttons. Give Messages / Email / News a denser row (avatar initial, unread pip, source mark) so the grid doesn’t read as twelve identical lists.
- **Notes** — inline title + textarea inside the card makes the module look like a form, not a journal. Shortcuts already compose in the drawer; Notes should match. The card lists up to `listLimit` (default 10) *under* the compose form, so the form eats the slot.
- **Empty states** — `PermissionCallout` is good for TCC. Other empties are a single muted sentence. One glyph empty per unconfigured module.
- **Favicon** — `index.html` still uses the Vite logo (`/vite.svg`).
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`. Either follow the system when preference is Auto, or label the button **Sunset** so the rule is honest. Theme also does not survive relaunch until increment 0.
- **Email in-card settings** — `setShowSettings(true)` on every disconnected load fights the overlay *and* stretches the Email slot to a full connect form. Gate the auto-open; move the form to a drawer (Email “All” is already a drawer).
- **Email count vs list** — badge is `all.length` (`listAllImportantEmails`, clamp 500) while the card shows `top` (`limit`). Badge the visible list, or label it “40 important”.
- **Wide-grid “Right column”** — at `min-width: 1180px` the grid is 3 columns. CSS `.placement-right` jumps to `grid-column: 3 / 4` (leaves a hole in column 2). The unused helper `placementGridStyle()` maps `"right"` to `2 / 3` — so if someone wires the helper later, “right” still would not mean the last column. Pick one: last column, or drop “right” until there is a real 3-slot placer.
- **Eyebrow drift** — card copy and `MODULE_META` disagree on YouTube (`Subscriptions` vs `Channels`), News (`For you` vs `Tailored`), Finance (`Snapshot` vs `Ledger`), Home (`Ring & Blink` vs `Cameras`), Streaming (`What's on` vs `Watch`), and Mail (`Informed Delivery` vs `Physical`). Pick one voice per module.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login.
- **Disconnected Email re-lists Mail.app on every quiet refresh** — `onDashboardRefresh` → `refresh()` → `loadSettings()` → `loadAccounts()` whenever `!connected`. That is AppleScript on a 15-minute timer even after the timeout fix. List accounts only when the user opens Connect.
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email, health, and finance on every Refresh all, in parallel with each module’s own load (hero Calendar is a third EventKit read). Cheap locally, noisy on FDA / EventKit. Have modules publish into the briefing, or pass cached rows down. After increment 1, also skip sources the layout has hidden.
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #33 became the security tip and because first-paint module mounts, briefing CLS, overlapping refresh, the rebound-less interval timer, and YouTube handle fallthrough needed tighter acceptance.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#33** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24 / #25 / #26 / #27 / #28 / #29 / #30 / #31 / #32.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change, plus drop unused `list_settings`. Do not start onboarding without it. Do not allow-list weather/health keys.
3. **Onboarding + starter layout + wait for `layoutReady` before mounting cards** (increment #1). Honor enabled modules in Refresh all, ⌘K, and the Today briefing from day one. Keep the briefing slot mounted.
4. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together; include capability grants). Self-host; do not open CSP to Google Fonts.
5. **YouTube thumbs + @handles** (#3) — add `i.ytimg.com` to `img-src`; derive posters from `videoId`; resolve handles to `UC…` (do not store `@name`).
6. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice. Refresh polish must add a saver **and** state-backed timer for `refresh.intervalMinutes`, skip overlapping runs, and stop labeling local reloads as skipped.
7. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement the settings allowlist and onboarding instead.
