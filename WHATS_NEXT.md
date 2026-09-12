# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-12** (`476a28b`, still `0.1.0`). This supersedes the 11 Sep queue in [#38](https://github.com/theLaxerz/Mainstream/pull/38), plus [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make the **typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **thirteenth** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck. Close the older What’s Next drafts; do not treat another scan as new product work.
- **Security tip moved to #39.** The 11 Sep afternoon scan ([#39](https://github.com/theLaxerz/Mainstream/pull/39)) re-applies the 5–10 Sep IPC/SSRF set, then requires Health zip members to be named exactly `export.xml` (not `ends_with("export.xml")`) and rejects cross-host / HTTPS→HTTP redirects on Ring, Blink REST, OAuth token exchange, and TMDB so Bearer tokens stay on origin. #37 / #35 / #33 / #31 / #29 / #27 / #25 / #22 / #20 / #18 / #16 / #14 are older copies. Master is still Vite `7.3.2` / zip `2.2` / Tauri features `[]` / `allowedHosts` unset.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write.
- **Still true:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **Still true:** Email’s badge is `all.length` while the card renders `top` (`limit`). `list_all_important_emails` is already clamped to **500**. Health already re-parses the export zip on every Refresh all (`try_import_configured`, no mtime). Command-bar clock is `aria-live="polite"` on a 1s tick. Google Fonts `@import` is blocked by `connect-src` (not only `font-src`). YouTube thumbs need `i.ytimg.com` in `img-src`. Today briefing and ⌘K ignore layout. Locals are labeled `skipped` then reloaded via `REFRESH_EVENT`. `refresh.intervalMinutes` has a loader and no saver. Overlay title bar / tray need feature flags + capabilities beyond `core:default`. First-paint `defaultLayout()` mounts all 12 module effects before SQLite layout arrives. Today briefing returns `null` until pulse settles (and again if a later load throws). The interval timer never rebinds after a future saver. YouTube `@handle` is stored verbatim and the RSS URL 404s. `onRefresh` has no in-flight lock. Home mount is a live Ring+Blink fetch. News `bootstrap` seeds RSS without Refresh all. Email’s `PermissionCallout` is dead after `loadSettings` auto-opens the connect form. First paint reads Messages FDA twice and EventKit three times. ⌘K and every `DetailDrawer` are `aria-modal` without a focus trap; ⌘K + ⌘, can stack. Layout / ⌘, is not gated on `layoutReady`. `list_settings` is a live dedicated-key dump. Last-sync labels freeze at “just now.” News ignores Layout item counts. Notes All-drawer Edit writes the in-card form. Refresh all holds `DbState` through IMAP / Informed Delivery OCR / Health zip / Blink thumbs.
- **New finding — Layout persist is unqueued, and the Items field fights its own clamp.** `LayoutCustomize.persist()` is fire-and-forget (`void persist(...)`) against the current `draft`. Two quick toggles, or typing “12” in Items, start two writes from the same stale draft. `clampLimit` floors anything below 3 to 3, and the input is controlled by `entry.listLimit`, so the first keystroke of “12” persists **3**. If that write finishes last, it overwrites a successfully typed 12. `useEffect([open, layout])` also resets `draft` whenever the parent `setLayout` lands, which can wipe an in-progress customize — the same race as the ungated-`layoutReady` bug, even after the SQLite load. Serialize persists (or debounce the number field and keep a local string), and do not remount the draft from a parent write this drawer just sent.
- **New finding — News’ own 15-minute timer ignores `document.hidden`.** Dashboard Refresh all already skips when the window is hidden. `NewsSection`’s `NEWS_REFRESH_MS` interval does not — and still does not check `refreshing`. A backgrounded window keeps hitting RSS every 15 minutes, stacked on card Refresh, dashboard Refresh all, and a still-running `bootstrap`. Drop the News-only clock; one dashboard timer is enough. Until then, honor `document.hidden` and `refreshing`.
- **New finding — Home’s badge is the full live list; the card slices to `limit`.** `list_home_devices` has no limit. The card renders `devices.slice(0, limit)` but badges `devices.length`. Same class as Email’s `all.length` vs `top`. Changing Layout “items” only changes the slice; the count still reads as every camera. Badge the visible list, or label it “8 cameras.”
- **Sharper on `list_settings`:** besides email host/user, `weather.place`, `health.export_path`, and `news_last_refresh_at`, the dump also returns `email.google_client_id` / `email.microsoft_client_id`, Blink `hardware_id` / `account_id` / `tier` / `access_expires`. `home.blink_access_token` is filtered (the key contains `token`); the rest are not. Drop the command. Do not “fix” it by allow-listing those keys.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15), last updated 30 Aug), twelve stacked What’s Next docs, thirteen overlapping security PRs. **Twenty-six** open drafts.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts `@import` — blocked in Tauri; see fonts below), analog clock + digital readout + greeting, Open-Meteo weather (cached snapshot, refetch when stale), EventKit month grid with selected-day agenda
- Today briefing: greeting eyebrow, “At a glance” title, next event, unread chats, important email, steps, spend — chips scroll to the matching module. Section is unmounted until pulse settles, and unmounts again if a later load throws.
- Modules: Messages (grouped unread → Messages.app; FDA scan capped at 500, run twice on first paint), Calendar, Email (Google / Microsoft PKCE, Mail.app picker with timeouts, IMAP fallback; connect form auto-opens when disconnected), USPS Informed Delivery OCR thumbs, News (ranked RSS, auto-seeds `feeds.default.json` on first empty load; ignores later `listLimit` changes; polls RSS while hidden), Finance (local ledger + CSV + 14-day spend chart), Notes (inline compose; All-drawer Edit writes the hidden card form), Health (Apple Health zip, re-import on refresh, 7-day sparklines), Home (Ring + Blink stills — live fetch on mount; badge is full list, card is `slice(0, limit)`), YouTube RSS (text list, `UC…` / `youtube.com/channel/…` only), Streaming (TMDB Tonight / hot / new with posters; local list on mount), Shortcuts (compose lives in the drawer)
- Chrome: sticky command bar, ⌘K (jump / shortcuts / quick note), theme cycle, layout persist in SQLite (customize not gated on `layoutReady`; persist is unqueued), 15-minute quiet refresh, per-module last-sync (remote `ok`/`error` only; the “Xm ago” string does not tick)
- Hardening: Tauri CSP, `freezePrototype`, Keychain secrets, IPC validators. Rust unit tests exist in `security.rs`, email, finance, Mail.app, OAuth. Frontend tests are only `emailConnectors.test.ts`. `list_settings` remains a registered dump of non-secret dedicated keys.

Open PRs — **do not rebuild these on another branch:**

| PR | What | Action |
| --- | --- | --- |
| [#15](https://github.com/theLaxerz/Mainstream/pull/15) | Local Tasks (due buckets, command-palette `Task: …`, briefing chip) | **Merge first.** Highest-value missing module. Last touched 30 Aug. |
| [#39](https://github.com/theLaxerz/Mainstream/pull/39) | Health zip exact `export.xml` basename + authenticated-client redirect lock (11 Sep scan) | Review and land. Newest of the security set. |
| [#37](https://github.com/theLaxerz/Mainstream/pull/37) / [#35](https://github.com/theLaxerz/Mainstream/pull/35) / [#33](https://github.com/theLaxerz/Mainstream/pull/33) / [#31](https://github.com/theLaxerz/Mainstream/pull/31) / [#29](https://github.com/theLaxerz/Mainstream/pull/29) / [#27](https://github.com/theLaxerz/Mainstream/pull/27) / [#25](https://github.com/theLaxerz/Mainstream/pull/25) / [#22](https://github.com/theLaxerz/Mainstream/pull/22) / [#20](https://github.com/theLaxerz/Mainstream/pull/20) / [#18](https://github.com/theLaxerz/Mainstream/pull/18) / [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | Older security copies | Close after #39 lands. |
| [#38](https://github.com/theLaxerz/Mainstream/pull/38) / [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17) | Older What’s Next docs | Close when this lands. |

---

## Do this next (priority)

### 0. Expand the settings allowlist (tiny, blocks everything else)

`src-tauri/src/security.rs` → `is_generic_setting_key` only allows `dashboard.layout.v1`. Theme (`appearance.theme`), refresh interval (`refresh.intervalMinutes`), and the onboarding flag (`onboarding.completed`) all 404 on get *and* set. The theme button looks like it works (`applyTheme` is local) and then resets on relaunch.

Allow only:

- `dashboard.layout.v1`
- `appearance.theme`
- `refresh.intervalMinutes`
- `onboarding.completed`

Do **not** add `weather.place`, `weather.snapshot`, or `health.export_path`. Those have dedicated commands. Drop unused `list_settings` (and the `listSettings` wrapper) in the same PR — it is a live dump of email / weather / health / Blink metadata and OAuth client IDs. `refresh.intervalMinutes` still needs a saver + a React-state timer before a Layout control does anything.

### 1. First-run: wait for layout, then onboard

`layoutReady` exists and only drives the “Loading layout…” caption. The grid first-paints `defaultLayout()` (all 12 enabled). That mounts every module `useEffect` before SQLite answers — Messages FDA ×2, EventKit ×3 (hero 200 / section 50 / pulse 24), `getEmailSettings` ×2, plus News RSS seed and a live Ring+Blink fetch if those cards are in the default twelve.

Do this as one increment:

1. **Do not mount module cards until `layoutReady`.** Show the existing `ModuleSkeleton` (already in `ModuleSection.tsx`, unused) in the grid slots. This is the actual IPC fix, not a flicker tweak.
2. **Gate Layout / ⌘, on `layoutReady`.** Command bar and palette open customize immediately. The drawer drafts whatever is in React state, which starts as `defaultLayout()`. A toggle before the load returns persists that default over a saved layout; a late `setLayout` can also clobber an in-progress customize. Keep the drawer closed (or read-only) until the SQLite layout is in, **and** serialize `persist()` so typing Items / double-toggles cannot lose the later write.
3. **Starter layout** — Messages, Calendar, Email, Notes, Shortcuts on. Everything else off until the user turns it on (or onboarding does).
4. **Five-step overlay** (Email, Messages FDA, Calendar, city, one more: Health export *or* a news feed). Persist `onboarding.completed`. Skip if already set. **Do not** let Email auto-open the in-card connect form (`loadSettings` → `setShowSettings(true)`). That hides `PermissionCallout` after the first load and fights the overlay.
5. **Honor enabled modules everywhere from day one.** `refresh_dashboard` should take the enabled set (or skip hidden News / YouTube / Streaming / Home). ⌘K “Go to” should list enabled modules only — `scrollIntoView` on a hidden `#module-*` is a no-op. `loadDashboardPulse` should skip sources the layout has hidden. Mail’s “Open Email module” (`[data-module='email']`) is the same silent no-op when Email is hidden — jump to onboarding / Layout instead.
6. **Keep the briefing slot mounted.** `TodayBriefing` `return null`s until pulse settles *and* if a later reload throws (`catch` → `setPulse(null)`). That is two layout shifts. Reserve the space with the skeleton; on error keep last-good + a quiet retry.
7. **Notes compose/Edit belongs in the drawer.** The All-drawer Edit button calls `startEdit`, which fills the title/textarea that live in the card behind the overlay. Shortcuts already compose in the drawer — match that. The card should be a list.

Without (1) + (5), onboarding that hides modules still pays the default-12 tax on the first paint *and* every 15-minute Refresh all.

### 2. Self-host Fraunces + Figtree

`theme.css` `@import`s Google Fonts. Production **and** `devCsp` set `font-src 'self'`. The `@import` also needs `connect-src` to `fonts.googleapis.com` / `fonts.gstatic.com`, which are not in the IPC-only `connect-src`. Fallback stack (`Iowan` / `Avenir Next`) is what actually renders in the `.app`.

Vendor `.woff2` under `src/assets/fonts/`, `@font-face` locally, keep CSP as-is. Do not punch a `font-src` / `connect-src` hole.

### 3. YouTube posters + @handles

`YoutubeItem` already has `videoId`. Derive `https://i.ytimg.com/vi/{id}/hqdefault.jpg` in the React card (Streaming already does this with TMDB). Add `https://i.ytimg.com` to `img-src` (prod + `devCsp`). No schema change.

`normalize_channel_id` only understands `UC…` and `youtube.com/channel/…`. A pasted `@handle` or `youtube.com/@name` is stored verbatim and `feeds/videos.xml?channel_id=@name` 404s. Resolve to `UC…` (or reject with “paste the channel ID”) — do not persist a handle.

### 4. Overlay title bar + liquid glass

`tauri.conf.json` windows have no `decorations: false`, `titleBarStyle`, `transparent`, or `vibrancy`. `Cargo.toml` `tauri` features are `[]`. Capabilities are `core:default` only.

Needs, in one PR: `macos-private-api` (and whatever overlay/tray features the chosen API requires), Info.plist, capability grants (`core:window:*` as needed), and a drag region on the command bar. Do not ship the CSS chrome until the capabilities exist or the window is undeclared-undraggable.

### 5. Menu-bar pulse (the cool slice)

Once overlay chrome works, a tray extra: next event + unread count, click to focus. Needs `tray-icon` / `macos-private-api` plus a `core:tray:*` capability — same feature-flag problem as #4, so this can ride along or follow immediately. This is the “it lives on my Mac” moment.

### 6. Reminders sync (only after Tasks is on master)

[#15](https://github.com/theLaxerz/Mainstream/pull/15) is the local Tasks module. After it has lived on master, EventKit Reminders as a second source on that same card — not a new module. Calendar already has the EventKit permission pattern.

### 7. Command palette + drawer polish

⌘K is already the jump / shortcut / quick-note bar. After Tasks lands, `Task: …` from #15 belongs here. Then:

- Filter “Go to” to **enabled** modules (see increment 1)
- Wrap arrow keys; **focus trap** so Tab cannot escape the dialog (`aria-modal` is a lie today)
- Same trap on `DetailDrawer` — Layout, Email All, Notes All, YouTube channels, Home connect, Streaming services, and Mail detail all share it. No initial focus.
- ⌘K and ⌘, must dismiss each other (two `aria-modal` dialogs can stack; both listen for Escape on `window`, so Escape closes both at once instead of the top one)
- `aria-activedescendant` on the highlighted row

### 8. Refresh all should feel honest

`refresh_dashboard` ignores layout, so hidden Streaming / YouTube / Home / News still hit remote APIs every 15 minutes. Status counts only `ok` (“Synced 1 module(s)” on first-run looks broken). Locals are labeled `skipped` then the UI reloads them via `REFRESH_EVENT`, so they never show “Synced Xm ago.” `formatSyncedAgo` runs once when `buildSyncState` lands — no tick. `onRefresh` has no in-flight guard (only the button `disabled={refreshing}`); the interval, `⌘⇧R`, and palette “Refresh all” can stack. The interval `useEffect` depends only on `[onRefresh]` and never reads `refreshing`. News has a second 15-minute timer that ignores `document.hidden` and `refreshing`.

News / YouTube / Streaming / Weather / Ring already drop `DbState` during HTTP. **Email IMAP, Informed Delivery + Vision OCR, Health zip parse, and Blink homescreen + thumbnail downloads hold the mutex for the entire I/O.** Notes / layout saves wait.

Fix:

- Pass enabled modules (or skip hidden remote ones)
- Stream per-module status into the command bar (or a tiny sync popover); say “News synced · 6 skipped” for *remote* skips only — or mark locals `ok` (“Reloaded”)
- Tick “Synced Xm ago” (shared clock or store epoch + derive)
- Drop the News-only timer; one clock is enough. If News stays on its own timer for a while, honor `document.hidden` and `refreshing`, and honor `limit` the way YouTube does (`[limit]` in the effect)
- Guard overlapping Refresh all **inside `onRefresh`** (early-return / in-flight ref) so the interval, shortcut, and palette share one lock
- Surface `refresh.intervalMinutes` in Layout — add `saveRefreshIntervalMinutes` **and** keep minutes in React state so the timer rebinds. LayoutCustomize today is toggle / items / column / reset only. Serialize or debounce Items so clamp-to-3 cannot overwrite “12.”

### 9. Health staleness (small — diagnosis corrected)

`HealthDay.importedAt` is already on the type and in SQLite. The card never shows it. The header count is hardcoded to `1` whenever today exists (`count={!loading && today ? 1 : null}`) — looks broken. Omit the badge, or show days imported / a streak.

**Refresh already re-imports.** `refresh_health_module` → `try_import_configured` re-parses the zip/xml on every Refresh all, with no mtime/hash short-circuit, **while holding `DbState`**. Large exports make Refresh all feel stuck after email + mail + news *and* freeze other IPC. Skip the parse when the file has not changed; show “last imported” age; keep a manual Import for when the user dropped a new export; drop the lock before the parse.

Live HealthKit is a later extraction fight — don’t start it until onboarding + fonts + window chrome are done.

### 10. Signing, notarization, auto-update

Still `0.1.0`, no updater plugin, README already documents unsigned local `.app`. Do this when someone other than the author should install it. Not the next coding increment.

---

## Looks — specific nits worth a polish pass

After onboarding + fonts, a short visual pass (one PR):

- **Brand wordmark** — the 8-layer 3D extrude is distinctive; in dusk it can glare. Quieter dusk treatment (fewer layers, softer fill).
- **Double greeting** — `Clock` and `TodayBriefing` both call `greetingFor`. Keep it on the clock readout; let the briefing lead with “At a glance” only (drop the greeting eyebrow).
- **Three clocks, three timers** — command bar live time + analog face + digital hero, each with its own 1s `setInterval`. Compact mode already repeats the hero. Once overlay chrome lands, drop seconds from the command bar and share one tick.
- **Command-bar `aria-live`** — VoiceOver should not hear the clock every second. The hero clock analog is `aria-hidden` and the visually-hidden time is hour:minute, but the parent `aria-label` still includes seconds (`HH:MM:SS`) and updates every tick. Drop seconds from the label; keep `aria-live` off the hero.
- **Unused skeleton** — `ModuleSkeleton` + shimmer CSS already exist. Every module still prints “Loading…”. Wire the skeleton on first paint, and use it for the briefing slot while pulse loads (and on pulse error).
- **Module sameness** — almost every card is title + meta + ghost buttons. Give Messages / Email / News a denser row (avatar initial, unread pip, source mark) so the grid doesn’t read as twelve identical lists. News’ four Like / Hide / Follow / Mute chips are the worst offender.
- **Notes** — inline title + textarea inside the card makes the module look like a form, not a journal. Shortcuts already compose in the drawer; Notes should match. The All-drawer Edit path currently writes that hidden form. The card lists up to `listLimit` (default 10) *under* the compose form, so the form eats the slot.
- **Empty states** — `PermissionCallout` is good for TCC. Other empties are a single muted sentence. One glyph empty per unconfigured module.
- **Favicon** — `index.html` still uses the Vite logo (`/vite.svg`).
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`. Either follow the system when preference is Auto, or label the button **Sunset** so the rule is honest. Theme also does not survive relaunch until increment 0.
- **Email in-card settings** — `setShowSettings(true)` on every disconnected load fights the overlay *and* stretches the Email slot to a full connect form *and* hides the `PermissionCallout`. Gate the auto-open; move the form to a drawer (Email “All” is already a drawer).
- **Email / Home count vs list** — Email badges `all.length` (clamp 500) while the card shows `top`. Home badges `devices.length` while the card shows `slice(0, limit)`. Badge the visible list, or label the totals.
- **Wide-grid “Right column”** — at `min-width: 1180px` the grid is 3 columns. CSS `.placement-right` jumps to `grid-column: 3 / 4` (leaves a hole in column 2). The unused helper `placementGridStyle()` maps `"right"` to `2 / 3` — so if someone wires the helper later, “right” still would not mean the last column. Pick one: last column, or drop “right” until there is a real 3-slot placer.
- **Eyebrow drift** — card copy and `MODULE_META` disagree on YouTube (`Subscriptions` vs `Channels`), News (`For you` vs `Tailored`), Finance (`Snapshot` vs `Ledger`), Home (`Ring & Blink` vs `Cameras`), Streaming (`What's on` vs `Watch`), and Mail (`Informed Delivery` vs `Physical`). Pick one voice per module.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login. `list_home_devices` should grow a local cache so mount / Refresh all are not the same live Ring+Blink round-trip.
- **Disconnected Email re-lists Mail.app on every quiet refresh** — `onDashboardRefresh` → `refresh()` → `loadSettings()` → `loadAccounts()` whenever `!connected`. That is AppleScript on a 15-minute timer even after the timeout fix. List accounts only when the user opens Connect.
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email, health, and finance on every Refresh all, in parallel with each module’s own load (hero Calendar is a third EventKit read). On first paint this is Messages ×2 and EventKit ×3 before layout is known. Have modules publish into the briefing, or pass cached rows down. After increment 1, also skip sources the layout has hidden.
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #39 became the security tip and because Layout persist can lose a typed item count (and overwrite a saved layout before `layoutReady`), News polls RSS while the window is hidden, Home’s badge ignores `limit`, and `list_settings` still dumps dedicated keys including OAuth client IDs and Blink hardware/account metadata.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#39** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24 / #25 / #26 / #27 / #28 / #29 / #30 / #31 / #32 / #33 / #34 / #35 / #36 / #37 / #38.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change, plus drop unused `list_settings`. Do not start onboarding without it. Do not allow-list weather/health keys.
3. **Onboarding + starter layout + wait for `layoutReady` before mounting cards** (increment #1). Gate Layout/⌘, on `layoutReady`. Serialize Layout persist (debounce Items; don’t let clamp-to-3 win the race). Honor enabled modules in Refresh all, ⌘K, and the Today briefing from day one. Keep the briefing slot mounted (including pulse errors). Do not auto-open Email settings. Do not let hidden News/Home hit the network on mount. Move Notes compose/Edit into the drawer.
4. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together; include capability grants). Self-host; do not open CSP to Google Fonts.
5. **YouTube thumbs + @handles** (#3) — add `i.ytimg.com` to `img-src`; derive posters from `videoId`; resolve handles to `UC…` (do not store `@name`).
6. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice. Refresh polish must add a saver **and** state-backed timer for `refresh.intervalMinutes`, put the in-flight lock on `onRefresh` (interval + `⌘⇧R` + palette), drop the mutex around IMAP/OCR/Health/Blink I/O, tick last-sync labels, honor News `limit`, drop the News-only timer (or at least honor `document.hidden`), and stop labeling local reloads as skipped. Palette polish must trap focus on ⌘K **and** `DetailDrawer`, and prevent stacked dialogs.
7. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement the settings allowlist and onboarding instead.
