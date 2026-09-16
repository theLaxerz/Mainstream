# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-16** (`476a28b`, still `0.1.0`). This supersedes the 15 Sep queue in [#46](https://github.com/theLaxerz/Mainstream/pull/46), plus [#44](https://github.com/theLaxerz/Mainstream/pull/44) / [#42](https://github.com/theLaxerz/Mainstream/pull/42) / [#40](https://github.com/theLaxerz/Mainstream/pull/40) / [#38](https://github.com/theLaxerz/Mainstream/pull/38) / [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make **Layout “items” actually change what you see**, make the **typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **seventeenth** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck. Close the older What’s Next drafts; do not treat another scan as new product work.
- **Security tip moved to #47.** The 15 Sep afternoon scan ([#47](https://github.com/theLaxerz/Mainstream/pull/47)) hardens rustls TLS 1.3 alignment and zip file types. #45 is the 14 Sep `open` argv / Health zip-size set. #43 is the 13 Sep cache path-join set. #41 / #39 / #37 / #35 / #33 / #31 / #29 / #27 / #25 / #22 / #20 / #18 / #16 / #14 are older copies. Master is still Vite `7.3.2` / zip `2.2` / `quick-xml` `0.37` / Tauri features `[]` / `allowedHosts` unset.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write. The deny message even says “not writable” on a get. `saveThemePreference` / `loadThemePreference` swallow that error, so the button looks honest.
- **Still true:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **Still true:** LayoutCustomize persist is unqueued (`void persist`) against a stale `draft`; typing Items “12” first persists `clampLimit(1)=3`, and `useEffect([open, layout])` can reset the draft from the write the drawer just sent. News’ own 15-minute timer ignores `document.hidden` and `refreshing`. `list_settings` dumps email host/user, weather/health paths, OAuth client IDs, and Blink hardware/account/tier/expiry (`home.blink_access_token` is filtered; the rest are not). Command-bar clock is `aria-live="polite"` on a 1s tick. Google Fonts `@import` is blocked by `connect-src` (not only `font-src`). YouTube thumbs need `i.ytimg.com` in `img-src`. Today briefing and ⌘K ignore layout. Locals are labeled `skipped` then reloaded via `REFRESH_EVENT`. `refresh.intervalMinutes` has a loader and no saver. Overlay title bar / tray need feature flags + capabilities beyond `core:default`. First-paint `defaultLayout()` mounts all 12 module effects before SQLite layout arrives. Today briefing returns `null` until pulse settles (and again if a later load throws). The interval timer never rebinds after a future saver. YouTube `@handle` is stored verbatim and the RSS URL 404s. `onRefresh` has no in-flight lock. Home mount is a live Ring+Blink fetch. News `bootstrap` seeds RSS without Refresh all (`run_refresh_news` always `seed_feeds_if_empty`). Email’s `PermissionCallout` is dead after `loadSettings` auto-opens the in-card connect form. First paint reads Messages FDA twice and EventKit three times (hero **200** / section `max(limit,50)` / pulse **24**). ⌘K and every `DetailDrawer` are `aria-modal` without a focus trap; ⌘K + ⌘, can stack. Layout / ⌘, is not gated on `layoutReady`. Last-sync labels freeze at “just now.” News ignores Layout item counts (bootstrap is mount-only; the “More stories” effect watches `showMore` only). Notes All-drawer Edit writes the in-card form. Refresh all holds `DbState` through IMAP / Informed Delivery OCR / Health zip / Blink thumbs. Health already re-parses the export on every Refresh all (`try_import_configured`, no mtime). Weather / Home wipe last-good on a failed reload. `DetailDrawer` overflow is not refcounted. Health / Streaming / YouTube settings saves are unhandled rejections. Home Refresh all copy counts every device as “online.” Shortcuts joins Email / Home / Messages / Calendar on badge-vs-slice. Finance summary hardcodes recent to 10 and Manage jumps the badge to 50. Home / Mail still thumbs and YouTube Del reject unhandled. Health sparkline captions stay “7d” while `listHealthDays(limit)` honors Layout items. YouTube / Streaming / Mail `open*` still fire without try/catch. Health hides “Recent days” once `history.length > 1`. Pulse calendar is 14d/24 vs the card’s 21d vs the hero’s 200. Finance briefing hides a $0 month. Shortcuts Edit (`updateShortcut`) is unused. Layout Items `max={50}` snaps back to 24 for shortcuts.
- **New finding — the hero calendar never rolls past midnight.** `Calendar` freezes `today` in `useMemo(..., [])`. Clock, command bar, and greetings tick every second; the month grid’s `is-today`, **Today** button, selected-day default, and `daysBack`/`daysAhead` fetch window stay on the launch date for as long as the window stays open. A Life OS left overnight highlights yesterday. Roll `today` off a shared midnight tick (the command-bar clock already has a 1s interval). Distinct from yesterday’s access-deny stale-dots bug: that path keeps old events; this path keeps the wrong *day*.
- **New finding — Open-Meteo already returns humidity and wind, and the hero throws them away.** `WeatherSnapshot` has `humidity` and `windSpeed`. `fetch_forecast` requests `relative_humidity_2m,wind_speed_10m`. The panel only renders glyph + temp + condition + H/L. A quiet `62% · 8 mph` line is free coolness. Catch: the URL hardcodes `wind_speed_unit=mph` even when the place is celsius, and fahrenheit temps render as `72°` with no F while celsius gets `22°C`. Pair the readout with the missing °F/°C toggle; do not add a second weather API.
- **New finding — Email “Internet Accounts…” is another uncaught `open`.** Email / Messages / Calendar / News / Shortcuts wrap `open*`. YouTube rows, Streaming Tonight + rails, and Mail “Email” / “Open digest” still do not (15 Sep). `EmailSection` also fires `void openInternetAccounts()` with no catch — System Settings failing is an unhandled rejection on the Connect form, the first surface a new user hits.
- **New finding — the hero month has no TCC empty state, and its agenda is live.** Calendar *section* has `PermissionCallout`. The hero grid is a blank month that says “Nothing scheduled” when EventKit is denied (or when `today` is stale). The selected-day agenda is `aria-live="polite"`, so picking days stacks VoiceOver chatter on top of the command-bar 1s clock. Show a one-line “Calendar access needed” on the hero; `aria-live` the agenda only when the selected day actually changes, or not at all.
- **New finding — more unused wrappers in the same family as `listUnreadMessages`.** `calendarAccessStatus`, `getNote`, and `updateAccount` are wrapped and never called. Notes never opens a single note by id; Finance never edits an account in place (delete + re-add). Rust also registers `update_transaction` with **no** TypeScript wrapper. Drop the idle wrappers, or wire Edit on those drawers in increment 1 next to Shortcuts Edit. YouTube / Streaming dates are raw ISO (`publishedAt.slice(0, 10)`, `releaseDate` as stored) while News / Messages use `Intl` — same polish pass as eyebrow drift.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15), last updated 30 Aug), sixteen stacked What’s Next docs, seventeen overlapping security PRs. **Thirty-four** open drafts.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts `@import` — blocked in Tauri; see fonts below), analog clock + digital readout + greeting (live), Open-Meteo weather (cached snapshot; humidity/wind fetched and unused; a failed reload clears it; no °C/°F toggle after pin; wind always mph), EventKit month grid with selected-day agenda (`today` frozen at mount; 200 events; stale dots if access later fails; no permission empty state; agenda is `aria-live`)
- Today briefing: greeting eyebrow (frozen until pulse reload — can disagree with the live clock after a time-of-day boundary), “At a glance” title, next event (14-day window), unread chats, important email, steps, spend (hidden at $0) — chips scroll to the matching module. Section is unmounted until pulse settles, and unmounts again if a later load throws.
- Modules: Messages (grouped unread → Messages.app; FDA scan is the full unread set via `listAllUnreadMessages`, run twice on first paint; unused `listUnreadMessages(limit)` exists; badge is all groups, card is `slice(0, limit)`), Calendar (section fetches ≥50 over 21 days, badges that, shows `limit`; unused `calendarAccessStatus` wrapper), Email (Google / Microsoft PKCE, Mail.app picker with timeouts, IMAP fallback; connect form auto-opens when disconnected; badge is All ≤500; `loadLists` always fetches the 500-row set; Internet Accounts open is uncaught), USPS Informed Delivery OCR thumbs (base64 `.then` has no catch; “Email” open has no catch), News (ranked RSS, auto-seeds `feeds.default.json` on first empty load; ignores later `listLimit` changes; polls RSS while hidden; leftover `seedDefaultNewsFeeds` / `rerankNews` / `getNewsLastRefresh` wrappers are unused), Finance (local ledger + CSV + 14-day spend chart; summary recent is hardcoded to 10; Manage rewrites the card list to 50; accounts clamp to 4; `incomeThisMonth` unused; `updateAccount` unused; `update_transaction` has no TS wrapper), Notes (inline compose; All-drawer Edit writes the hidden card form; unused `getNote`), Health (Apple Health zip, re-import on refresh, sparklines labeled 7d regardless of `limit`; day list only when `history.length === 1`; heart is a today number with no sparkline; Setup save is unhandled), Home (Ring + Blink stills — live fetch on mount; badge is full list; error wipes cameras; camera + Connect drawers can stack; thumbs have no catch), YouTube RSS (text list, `UC…` / `youtube.com/channel/…` only; dates are ISO `YYYY-MM-DD`; add-channel *and* Del *and* row-open are unhandled), Streaming (TMDB Tonight / hot / new with posters; local list on mount; `limit` applied to *both* rails; dates are raw; Services save *and* poster/row open are unhandled), Shortcuts (compose lives in the drawer; no Edit despite `updateShortcut`; badge is the full list, card is `slice(0, limit)`)
- Chrome: sticky command bar, ⌘K (jump / shortcuts / quick note), theme cycle, layout persist in SQLite (customize not gated on `layoutReady`; persist is unqueued; Items max is 50 even for shortcuts), 15-minute quiet refresh, per-module last-sync (remote `ok`/`error` only; the “Xm ago” string does not tick)
- Hardening: Tauri CSP, `freezePrototype`, Keychain secrets, IPC validators. Rust unit tests exist in `security.rs`, email, finance, Mail.app, OAuth. Frontend tests are only `emailConnectors.test.ts`. `list_settings` remains a registered dump of non-secret dedicated keys.

Open PRs — **do not rebuild these on another branch:**

| PR | What | Action |
| --- | --- | --- |
| [#15](https://github.com/theLaxerz/Mainstream/pull/15) | Local Tasks (due buckets, command-palette `Task: …`, briefing chip) | **Merge first.** Highest-value missing module. Last touched 30 Aug. |
| [#47](https://github.com/theLaxerz/Mainstream/pull/47) | rustls TLS 1.3 alignment + zip file types after 15 Sep scan | Review and land. Newest of the security set. |
| [#45](https://github.com/theLaxerz/Mainstream/pull/45) | `open` argv `--` + Health zip size caps after HexStrike CVE-2026-90690 (14 Sep scan) | Close after #47 lands (or land if #47 is a subset). |
| [#43](https://github.com/theLaxerz/Mainstream/pull/43) | Cache path-join confinement after rustypaste CVE-2026-90774 (13 Sep scan) | Close after #47 lands. |
| [#41](https://github.com/theLaxerz/Mainstream/pull/41) / [#39](https://github.com/theLaxerz/Mainstream/pull/39) / [#37](https://github.com/theLaxerz/Mainstream/pull/37) / [#35](https://github.com/theLaxerz/Mainstream/pull/35) / [#33](https://github.com/theLaxerz/Mainstream/pull/33) / [#31](https://github.com/theLaxerz/Mainstream/pull/31) / [#29](https://github.com/theLaxerz/Mainstream/pull/29) / [#27](https://github.com/theLaxerz/Mainstream/pull/27) / [#25](https://github.com/theLaxerz/Mainstream/pull/25) / [#22](https://github.com/theLaxerz/Mainstream/pull/22) / [#20](https://github.com/theLaxerz/Mainstream/pull/20) / [#18](https://github.com/theLaxerz/Mainstream/pull/18) / [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | Older security copies | Close after #47 lands. |
| [#46](https://github.com/theLaxerz/Mainstream/pull/46) / [#44](https://github.com/theLaxerz/Mainstream/pull/44) / [#42](https://github.com/theLaxerz/Mainstream/pull/42) / [#40](https://github.com/theLaxerz/Mainstream/pull/40) / [#38](https://github.com/theLaxerz/Mainstream/pull/38) / [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17) | Older What’s Next docs | Close when this lands. |

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

`layoutReady` exists and only drives the “Loading layout…” caption. The grid first-paints `defaultLayout()` (all 12 enabled). That mounts every module `useEffect` before SQLite answers — Messages FDA ×2 (full unread set, not `limit`; `listUnreadMessages(limit)` is unused), EventKit ×3 (hero 200 / section ≥50 over 21d / pulse 24 over 14d), `getEmailSettings` ×2, plus News RSS seed and a live Ring+Blink fetch if those cards are in the default twelve.

Do this as one increment:

1. **Do not mount module cards until `layoutReady`.** Show the existing `ModuleSkeleton` (already in `ModuleSection.tsx`, unused) in the grid slots. This is the actual IPC fix, not a flicker tweak.
2. **Gate Layout / ⌘, on `layoutReady`.** Command bar and palette open customize immediately. The drawer drafts whatever is in React state, which starts as `defaultLayout()`. A toggle before the load returns persists that default over a saved layout; a late `setLayout` can also clobber an in-progress customize. Keep the drawer closed (or read-only) until the SQLite layout is in, **and** serialize `persist()` so typing Items / double-toggles cannot lose the later write. Debounce the number field (keep a local string) so clamp-to-3 cannot overwrite a typed 12. Set the Items `<input max>` from `clampLimit` (24 for shortcuts, 50 otherwise).
3. **Starter layout** — Messages, Calendar, Email, Notes, Shortcuts on. Everything else off until the user turns it on (or onboarding does). Reset defaults should restore *that* starter, not all 12 on.
4. **Five-step overlay** (Email, Messages FDA, Calendar, city, one more: Health export *or* a news feed). Persist `onboarding.completed`. Skip if already set. **Do not** let Email auto-open the in-card connect form (`loadSettings` → `setShowSettings(true)`). That hides `PermissionCallout` after the first load and fights the overlay. Wrap Health / Streaming / YouTube settings saves *and* YouTube Del *and* YouTube / Streaming / Mail `open*` *and* Email `openInternetAccounts` so a failed IPC is not an unhandled rejection.
5. **Honor enabled modules everywhere from day one.** `refresh_dashboard` should take the enabled set (or skip hidden News / YouTube / Streaming / Home). ⌘K “Go to” should list enabled modules only — `scrollIntoView` on a hidden `#module-*` is a no-op. `loadDashboardPulse` should skip sources the layout has hidden. Mail’s “Open Email module” (`[data-module='email']`) is the same silent no-op when Email is hidden — jump to onboarding / Layout instead.
6. **Keep last-good in every slot — and roll the hero’s `today`.** `TodayBriefing` `return null`s until pulse settles *and* if a later reload throws. Weather clears the city with no error. Home clears every camera. Hero Calendar keeps stale month dots when EventKit access is not `ok`, and keeps *yesterday’s* highlight when the window stays open past midnight (`useMemo([], …)`). Reserve the briefing space with the skeleton; on error keep last-good + a quiet retry. Same for Weather, Home, and the hero month grid. Share one clock tick so `is-today` / **Today** / the fetch window move at local midnight. If EventKit is denied, the hero needs the same `PermissionCallout` the section already has — a blank month that says “Nothing scheduled” looks like an empty calendar.
7. **Badge the visible list — and make Layout items real.** Email / Home / Messages / Calendar / **Shortcuts** all count the un-sliced fetch. Streaming spends `limit` on hot *and* on new. Finance summary ignores `limit` (hardcoded 10; Manage jumps the badge to 50). News bootstrap is mount-only (changing Items later does nothing until “More stories”). Health badges `1`, labels sparklines “7d”, and hides the day list once `history.length > 1`. Either badge `shown.length`, or label the total (“8 of 24 chats”). Finance recent must take `limit`; keep the Manage 50 in drawer state; catch still-thumb failures.
8. **Notes compose/Edit belongs in the drawer.** The All-drawer Edit button calls `startEdit`, which fills the title/textarea that live in the card behind the overlay. Shortcuts already compose in the drawer — match that, **and** use the existing `updateShortcut` so a typo is Edit, not delete. The card should be a list. Same family: `updateAccount` / `getNote` / `calendarAccessStatus` are wrapped and idle; `update_transaction` has no TS wrapper — wire Edit or drop them.
9. **One drawer at a time.** Home camera detail + Connect can stack; their overflow cleanup fights. ⌘K and ⌘, already stack. Dismiss the other overlay when one opens; refcount body overflow (or only mount one `DetailDrawer`).
10. **One calendar window for pulse + card.** Pulse should use the section’s 21-day fetch (not 14d/24). Show the finance chip whenever a ledger exists, including $0 months. Align briefing greeting with the live clock (or drop the briefing eyebrow — see Looks).

Without (1) + (5), onboarding that hides modules still pays the default-12 tax on the first paint *and* every 15-minute Refresh all.

### 2. Self-host Fraunces + Figtree

`theme.css` `@import`s Google Fonts. Production **and** `devCsp` set `font-src 'self'`. The `@import` also needs `connect-src` to `fonts.googleapis.com` / `fonts.gstatic.com`, which are not in the IPC-only `connect-src`. Fallback stack (`Iowan` / `Avenir Next`) is what actually renders in the `.app`.

Vendor `.woff2` under `src/assets/fonts/`, `@font-face` locally, keep CSP as-is. Do not punch a `font-src` / `connect-src` hole.

### 3. YouTube posters + @handles

`YoutubeItem` already has `videoId`. Derive `https://i.ytimg.com/vi/{id}/hqdefault.jpg` in the React card (Streaming already does this with TMDB). Add `https://i.ytimg.com` to `img-src` (prod + `devCsp`). No schema change.

`normalize_channel_id` only understands `UC…` and `youtube.com/channel/…`. A pasted `@handle` or `youtube.com/@name` is stored verbatim and `feeds/videos.xml?channel_id=@name` 404s. Resolve to `UC…` (or reject with “paste the channel ID”) — do not persist a handle. Wrap `onAddChannel` **and** Del **and** row-open so a bad paste / failed delete / blocked `open` is a field error, not an unhandled rejection. Format `publishedAt` with `Intl` the way News already does (ISO `slice(0, 10)` is the current card).

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
- ⌘K and ⌘, must dismiss each other (two `aria-modal` dialogs can stack; both listen for Escape on `window`, so Escape closes both at once instead of the top one). Home’s two drawers are the same bug.
- `aria-activedescendant` on the highlighted row
- Hero agenda should not be `aria-live="polite"` on every selected-day render while the command-bar clock already is

### 8. Refresh all should feel honest

`refresh_dashboard` ignores layout, so hidden Streaming / YouTube / Home / News still hit remote APIs every 15 minutes. News has **no** configured check — `run_refresh_news` always seeds defaults, then fetches. Status counts only `ok` (“Synced 1 module(s)” on first-run looks broken). Locals are labeled `skipped` then the UI reloads them via `REFRESH_EVENT`, so they never show “Synced Xm ago.” `formatSyncedAgo` runs once when `buildSyncState` lands — no tick. `onRefresh` has no in-flight guard (only the button `disabled={refreshing}`); the interval, `⌘⇧R`, and palette “Refresh all” can stack. The interval `useEffect` depends only on `[onRefresh]` and never reads `refreshing`. News has a second 15-minute timer that ignores `document.hidden` and `refreshing`, then paints “Background refresh complete” on the card. Home Refresh all reports `"{n} device(s) online"` for `devices.len()` — offline cameras and the Blink error placeholder count as online.

News / YouTube / Streaming / Weather / Ring already drop `DbState` during HTTP. **Email IMAP, Informed Delivery + Vision OCR, Health zip parse, and Blink homescreen + thumbnail downloads hold the mutex for the entire I/O.** Notes / layout saves wait.

Fix:

- Pass enabled modules (or skip hidden remote ones). Do not seed News feeds for a hidden card.
- Stream per-module status into the command bar (or a tiny sync popover); say “News synced · 6 skipped” for *remote* skips only — or mark locals `ok` (“Reloaded”)
- Tick “Synced Xm ago” (shared clock or store epoch + derive)
- Drop the News-only timer; one clock is enough. If News stays on its own timer for a while, honor `document.hidden` and `refreshing`, honor `limit` the way YouTube does (`[limit]` in the load effect, not just the timer), and drop the leftover `seedDefaultNewsFeeds` / `rerankNews` / `getNewsLastRefresh` wrappers (or use `getNewsLastRefresh` as the card’s last-sync)
- Guard overlapping Refresh all **inside `onRefresh`** (early-return / in-flight ref) so the interval, shortcut, and palette share one lock. Home’s card Refresh has no busy flag either — it can stack with Refresh all and then wipe cameras on the losing call.
- Surface `refresh.intervalMinutes` in Layout — add `saveRefreshIntervalMinutes` **and** keep minutes in React state so the timer rebinds. LayoutCustomize today is toggle / items / column / reset only. Serialize or debounce Items so clamp-to-3 cannot overwrite “12.”
- Home status should say “8 cameras” (or count `status == online`), not “8 device(s) online” for the mixed list.

### 9. Health staleness (small — diagnosis corrected)

`HealthDay.importedAt` is already on the type and in SQLite. The card never shows it. The header count is hardcoded to `1` whenever today exists (`count={!loading && today ? 1 : null}`) — looks broken. Omit the badge, or show days imported / a streak. Sparkline captions must follow `history.length`, not a baked-in “7d”. Keep the recent-days list visible when sparklines show — right now it vanishes at `history.length > 1`. Today already shows heart bpm; `avgHeartRate` is on every history row and never charted — a third sparkline is data you already import, not a new Health feature.

**Refresh already re-imports.** `refresh_health_module` → `try_import_configured` re-parses the zip/xml on every Refresh all, with no mtime/hash short-circuit, **while holding `DbState`**. Large exports make Refresh all feel stuck after email + mail + news *and* freeze other IPC. Skip the parse when the file has not changed; show “last imported” age; keep a manual Import for when the user dropped a new export; drop the lock before the parse. Wrap `onSavePath` so a bad path is a field error.

Live HealthKit is a later extraction fight — don’t start it until onboarding + fonts + window chrome are done.

### 10. Signing, notarization, auto-update

Still `0.1.0`, no updater plugin, README already documents unsigned local `.app`. Do this when someone other than the author should install it. Not the next coding increment.

---

## Looks — specific nits worth a polish pass

After onboarding + fonts, a short visual pass (one PR):

- **Brand wordmark** — the 8-layer 3D extrude is distinctive; in dusk it can glare. Quieter dusk treatment (fewer layers, softer fill).
- **Double greeting** — `Clock` and `TodayBriefing` both call `greetingFor`. The clock ticks live; the briefing greeting is captured at pulse load and only updates on Refresh all / the 15-minute timer. Across noon they can disagree for up to 15 minutes. Keep it on the clock readout; let the briefing lead with “At a glance” only (drop the greeting eyebrow).
- **Three clocks, three timers** — command bar live time + analog face + digital hero, each with its own 1s `setInterval`. Compact mode already repeats the hero. Once overlay chrome lands, drop seconds from the command bar and share one tick. That same tick should roll hero `today` at midnight (see increment 1).
- **Command-bar `aria-live`** — VoiceOver should not hear the clock every second. The hero clock analog is `aria-hidden` and the visually-hidden time is hour:minute, but the parent `aria-label` still includes seconds (`HH:MM:SS`) and updates every tick. Drop seconds from the label; keep `aria-live` off the hero. The selected-day agenda is a second live region — don’t announce every day click.
- **Unused skeleton** — `ModuleSkeleton` + shimmer CSS already exist. Every module still prints “Loading…”. Wire the skeleton on first paint, and use it for the briefing slot while pulse loads (and on pulse error).
- **Module sameness** — almost every card is title + meta + ghost buttons. Give Messages / Email / News a denser row (avatar initial, unread pip, source mark) so the grid doesn’t read as twelve identical lists. News’ four Like / Hide / Follow / Mute chips are the worst offender. News “More stories” expands *in-card* (not a drawer) and the button shows even with zero stories — match Email / Messages All. YouTube / Streaming dates should use `Intl` like News, not raw ISO.
- **Notes** — inline title + textarea inside the card makes the module look like a form, not a journal. Shortcuts already compose in the drawer; Notes should match. The All-drawer Edit path currently writes that hidden form. The card lists up to `listLimit` (default 10) *under* the compose form, so the form eats the slot.
- **Empty states** — `PermissionCallout` is good for TCC. Other empties are a single muted sentence. One glyph empty per unconfigured module. Hero Calendar is the loudest miss: denied access looks like a free month.
- **Favicon** — `index.html` still uses the Vite logo (`/vite.svg`).
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`. Either follow the system when preference is Auto, or label the button **Sunset** so the rule is honest. Theme also does not survive relaunch until increment 0.
- **Email in-card settings** — `setShowSettings(true)` on every disconnected load fights the overlay *and* stretches the Email slot to a full connect form *and* hides the `PermissionCallout`. Gate the auto-open; move the form to a drawer (Email “All” is already a drawer). Catch `openInternetAccounts`.
- **Count vs list** — Email badges All (≤500) while the card shows `top`. Home badges every camera while the card slices to `limit`. Messages badges every unread chat while the card slices to `topCount`. Calendar fetches ≥50, badges that, shows `limit`. **Shortcuts badges the full dock, shows `limit`.** Streaming badges hot+new (2× `limit`). Finance badges summary-10 (or Manage-50) and slices to `limit`; accounts stay at 4. Health badges `1`. Mail / YouTube / Notes already badge the limited fetch — copy that. Badge the visible list, or label the totals.
- **Wide-grid “Right column”** — at `min-width: 1180px` the grid is 3 columns. CSS `.placement-right` jumps to `grid-column: 3 / 4` (leaves a hole in column 2). The unused helper `placementGridStyle()` maps `"right"` to `2 / 3` — so if someone wires the helper later, “right” still would not mean the last column. Pick one: last column, or drop “right” until there is a real 3-slot placer.
- **Eyebrow drift** — card copy and `MODULE_META` disagree on YouTube (`Subscriptions` vs `Channels`), News (`For you` vs `Tailored`), Finance (`Snapshot` vs `Ledger`), Home (`Ring & Blink` vs `Cameras`), Streaming (`What's on` vs `Watch`), and Mail (`Informed Delivery` vs `Physical`). Pick one voice per module.
- **Weather readout** — humidity and wind are already on the snapshot. Show them. Label fahrenheit with °F. If units are celsius, request `wind_speed_unit=kmh` (or convert) so the line does not mix 22°C with mph.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs. `incomeThisMonth` is already in the summary — surface it on the card before inventing envelopes. `updateAccount` is already wrapped if inline edit comes first; `update_transaction` would need a TS wrapper.
- **Weather units** — snapshot stores `celsius` / `fahrenheit`, but after pin there is no toggle; re-picking a city keeps the last units (default °F). A ghost °F/°C on the hero is enough. Same control should pick the wind unit (Open-Meteo is hardcoded to mph today). Humidity / wind are already in the snapshot — render them when the toggle lands.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login. `list_home_devices` should grow a local cache so mount / Refresh all are not the same live Ring+Blink round-trip. Closing Connect while `pinRequired` clears the PIN field and the flag — the user has to restart Blink login. Keep the pending verify across drawer close. Catch `homeDeviceImageBase64` / `physicalMailImageBase64` failures so a missing still is an empty tile, not an unhandled rejection.
- **Disconnected Email re-lists Mail.app on every quiet refresh** — `onDashboardRefresh` → `refresh()` → `loadSettings()` → `loadAccounts()` whenever `!connected`. That is AppleScript on a 15-minute timer even after the timeout fix. List accounts only when the user opens Connect.
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email, health, and finance on every Refresh all, in parallel with each module’s own load (hero Calendar is a third EventKit read). On first paint this is Messages ×2 and EventKit ×3 before layout is known. Have modules publish into the briefing, or pass cached rows down. Use `listUnreadMessages(limit)` for the card and keep the full scan for the All drawer / pulse chat count. After increment 1, also skip sources the layout has hidden. Widen pulse calendar to the section’s 21 days (or share the section rows).
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #47 became the security tip and because the hero calendar still freezes `today` at launch, Open-Meteo humidity/wind never reach the UI, Email’s Internet Accounts button still rejects unhandled, and Layout “items” still does not mean what the cards show.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#47** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24 / #25 / #26 / #27 / #28 / #29 / #30 / #31 / #32 / #33 / #34 / #35 / #36 / #37 / #38 / #39 / #40 / #41 / #42 / #43 / #44 / #45 / #46.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change, plus drop unused `list_settings`. Do not start onboarding without it. Do not allow-list weather/health keys.
3. **Onboarding + starter layout + wait for `layoutReady` before mounting cards** (increment #1). Gate Layout/⌘, on `layoutReady`. Serialize Layout persist (debounce Items; don’t let clamp-to-3 win the race; per-module Items max). Honor enabled modules in Refresh all, ⌘K, and the Today briefing from day one. Keep last-good in briefing / Weather / Home / hero Calendar (including pulse errors, EventKit access-deny, **and a midnight roll of hero `today`**). Badge visible rows (Email, Home, Messages, Calendar, **Shortcuts**; Streaming once; Finance recent must take `limit` and stay out of Manage state). Honor News / Health `listLimit` after mount (News load effect should watch `dashboardLimit`, not only `showMore`; Health sparkline label follows `history.length`; keep the day list when sparklines show). Do not auto-open Email settings. Do not let hidden News/Home hit the network on mount. Move Notes compose/Edit into the drawer; wire Shortcuts Edit. One drawer at a time (refcount overflow). Catch Home / Mail still-thumb failures **and** YouTube / Streaming / Mail `open*` **and** Email `openInternetAccounts`. Pulse calendar = section 21d; finance chip at $0. Hero Calendar needs a permission empty state.
4. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together; include capability grants). Self-host; do not open CSP to Google Fonts.
5. **YouTube thumbs + @handles** (#3) — add `i.ytimg.com` to `img-src`; derive posters from `videoId`; resolve handles to `UC…` (do not store `@name`). Intl the dates.
6. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice. Refresh polish must add a saver **and** state-backed timer for `refresh.intervalMinutes`, put the in-flight lock on `onRefresh` (interval + `⌘⇧R` + palette), drop the mutex around IMAP/OCR/Health/Blink I/O, tick last-sync labels, honor News `limit`, drop the News-only timer (or at least honor `document.hidden`), stop labeling local reloads as skipped, and stop calling every Home device “online.” Palette polish must trap focus on ⌘K **and** `DetailDrawer`, prevent stacked dialogs, and take `aria-live` off the hero agenda. Weather humidity/wind + °F/°C can ride a small polish PR after onboarding.
7. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement the settings allowlist and onboarding instead.
