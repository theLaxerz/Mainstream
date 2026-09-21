# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-21** (`476a28b`, still `0.1.0`). This supersedes the 20 Sep queue in [#56](https://github.com/theLaxerz/Mainstream/pull/56), plus [#54](https://github.com/theLaxerz/Mainstream/pull/54) / [#52](https://github.com/theLaxerz/Mainstream/pull/52) / [#50](https://github.com/theLaxerz/Mainstream/pull/50) / [#48](https://github.com/theLaxerz/Mainstream/pull/48) / [#46](https://github.com/theLaxerz/Mainstream/pull/46) / [#44](https://github.com/theLaxerz/Mainstream/pull/44) / [#42](https://github.com/theLaxerz/Mainstream/pull/42) / [#40](https://github.com/theLaxerz/Mainstream/pull/40) / [#38](https://github.com/theLaxerz/Mainstream/pull/38) / [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make **Mail.app “Important” mean the same thing as IMAP**, make **Layout “items” actually change what you see**, make **the typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **twenty-second** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck. Close the older What’s Next drafts; do not treat another scan as new product work.
- **Security tip moved to #57.** The 20 Sep afternoon scan ([#57](https://github.com/theLaxerz/Mainstream/pull/57)) hardens XML DTD and JWT `aud`. #55 is the 19 Sep JWT `alg:none` / zip inflation set. #53 / #51 / #49 / #47 / #45 / #43 / #41 / #39 / #37 / #35 / #33 / #31 / #29 / #27 / #25 / #22 / #20 / #18 / #16 / #14 are older copies. Master is still Vite `7.3.2` / zip `2.2` / `quick-xml` `0.37` / Tauri features `[]` / `allowedHosts` unset.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write. The deny message even says “not writable” on a get. `saveThemePreference` / `loadThemePreference` swallow that error, so the button looks honest.
- **Still true:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **Still true from 20 Sep:** `greetingFor` (17:00 evening / 21:00 night / 05:00 morning) and `isDuskHour` (19:00–07:00) disagree. 17:00–19:00 is evening copy on light; 05:00–07:00 is “Good morning” on dusk. Auto theme interval is 60s. Visually-hidden `hourLabel` hardcodes `hour12: true` — four hour policies, not three. Weather Refresh-all status prints `{:.0}°` with no C/F unit.
- **Still true from 19 Sep:** the hero clock speaks three hardcoded times (12h visual, 12h hidden, 24h `aria-label`); weekday headers are English `Sun`–`Sat`; overnight Health sleep all credits `startDate`.
- **Still true from 18 Sep:** Health sleep sums every SleepAnalysis interval (InBed + stages + Awake). Finance `month_totals` / category chips / `list_spend_by_day` use SQLite UTC `now`. Window `minWidth` 960 vs stacked-hero CSS at 860px — that breakpoint never runs in the `.app`. Hero digital clock is hardcoded 12-hour; command bar uses locale `hour: numeric`.
- **Still true from 17 Sep:** LayoutCustomize persist is unqueued (`void persist`) against a stale `draft`; typing Items “12” first persists `clampLimit(1)=3`, and `useEffect([open, layout])` can reset the draft from the write the drawer just sent. News’ own 15-minute timer ignores `document.hidden` and `refreshing`. `list_settings` dumps email host/user, weather/health paths, OAuth client IDs, and Blink hardware/account/tier/expiry (`home.blink_access_token` is filtered; the rest are not). Command-bar clock is `aria-live="polite"` on a 1s tick. Google Fonts `@import` is blocked by `connect-src` (not only `font-src`). YouTube thumbs need `i.ytimg.com` in `img-src`. Today briefing and ⌘K ignore layout. Locals are labeled `skipped` then reloaded via `REFRESH_EVENT`. `refresh.intervalMinutes` has a loader and no saver. Overlay title bar / tray need feature flags + capabilities beyond `core:default`. First-paint `defaultLayout()` mounts all 12 module effects before SQLite layout arrives. Today briefing returns `null` until pulse settles (and again if a later load throws). The interval timer never rebinds after a future saver. YouTube `@handle` is stored verbatim and the RSS URL 404s. `onRefresh` has no in-flight lock (`⌘⇧R` and palette ignore the button `disabled`). Home mount is a live Ring+Blink fetch. News `bootstrap` seeds RSS without Refresh all (`run_refresh_news` always `seed_feeds_if_empty`). Email’s `PermissionCallout` is dead after `loadSettings` auto-opens the in-card connect form. First paint reads Messages FDA twice and EventKit three times (hero **200** / section `max(limit,50)` / pulse **24**). ⌘K and every `DetailDrawer` are `aria-modal` without a focus trap; ⌘K + ⌘, can stack. Layout / ⌘, is not gated on `layoutReady`. Last-sync labels freeze at “just now.” News ignores Layout item counts (bootstrap is mount-only; the “More stories” effect watches `showMore` only). Notes All-drawer Edit writes the in-card form. Refresh all holds `DbState` through IMAP / Informed Delivery OCR / Health zip / Blink thumbs. Health already re-parses the export on every Refresh all (`try_import_configured`, no mtime). Weather / Home wipe last-good on a failed reload. `DetailDrawer` overflow is not refcounted. Health / Streaming / YouTube settings saves are unhandled rejections. Home Refresh all copy counts every device as “online.” Shortcuts joins Email / Home / Messages / Calendar on badge-vs-slice. Finance summary hardcodes recent to 10 and Manage jumps the badge to 50. Home / Mail still thumbs and YouTube Del reject unhandled. Health sparkline captions stay “7d” while `listHealthDays(limit)` honors Layout items. YouTube / Streaming / Mail `open*` still fire without try/catch. Health hides “Recent days” once `history.length > 1`. Pulse calendar is 14d/24 vs the card’s 21d vs the hero’s 200. Finance briefing hides a $0 month. Shortcuts Edit (`updateShortcut`) is unused. Layout Items `max={50}` snaps back to 24 for shortcuts. Hero Calendar freezes `today` in `useMemo([], …)` and never flips `cancelled`. Open-Meteo humidity/wind never reach the UI (wind always mph; fahrenheit has no F). Email `openInternetAccounts` is uncaught. Hero month has no TCC empty state; selected-day agenda is a second `aria-live`. `calendarAccessStatus` / `getNote` / `updateAccount` / `listUnreadMessages` / `messagesAccessStatus` / `seedDefaultNewsFeeds` / `rerankNews` / `getNewsLastRefresh` are idle wrappers. Rust `update_transaction` and `list_categories` have no TS wrappers. YouTube / Streaming dates are raw ISO. Finance Add-transaction seeds UTC ISO and writes `T12:00:00Z` while the 14-day chart keys `localDayKey()`. Health Setup is an in-card form like Email auto-open Connect.
- **New finding — Mail.app “Important” is almost every unread.** IMAP / OAuth parse `To:`, `List-Unsubscribe`, newsletter hints, and a body preview (`email.rs` `ParsedHeaders`). The Mail.app path (`email_mailapp.rs` `sync_mailapp_inbox`) does none of that: `preview` is `""`, `has_list_unsubscribe` and `is_newsletter` are hardcoded `false`, and `to_addrs` is stuffed with `settings.user` (the connected account), not the message’s real recipients. `score_importance` then always awards the +25 “addressed to you” bump on top of the base 10 — **35 ≥ 15**, so the row is important unless the sender looks like `noreply` / `notifications@` / `news@` / `marketing@`. Scoring is also called with mailbox `"INBOX"` while the row is stored under `mailapp:{account}`, so junk-folder detection never runs. Mail.app is the featured “click an account already on this Mac” path; it currently fills Important with newsletters IMAP would have dropped, and the card has no preview snippet. Parse To: / List-Unsubscribe from Mail.app (or stop forging `to_addrs`) so both connectors share one filter.
- **New finding — Health is styled as Finance.** The Health card uses `finance-totals`, `finance-total`, and `finance-subhead` (`HealthSection.tsx`). Any finance polish restyles sleep/steps. Give Health its own totals row (or a shared `stat-totals` class) in the first-run / looks pass — do not treat this as a new Health feature.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15), last updated 30 Aug), twenty-one stacked What’s Next docs, twenty-two overlapping security PRs. **Forty-four** open drafts.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts `@import` — blocked in Tauri; see fonts below), analog clock + digital readout + greeting (live 12-hour, not locale; `aria-label` is 24h `HH:MM:SS`; visually-hidden is forced `hour12: true`), Open-Meteo weather (cached snapshot; humidity/wind fetched and unused; a failed reload clears it; no °C/°F toggle after pin; wind always mph; Refresh-all status prints `{:.0}°` with no unit), EventKit month grid with selected-day agenda (`today` frozen at mount; 200 events; stale dots if access later fails; `cancelled` never set; no permission empty state; agenda is `aria-live`; weekday headers hardcoded `Sun`–`Sat`; agenda times are locale)
- Today briefing: greeting eyebrow (frozen until pulse reload — can disagree with the live clock after a time-of-day boundary **and** with Auto-dusk at 17:00–19:00 / 05:00–07:00; `cancelled` never set), “At a glance” title, next event (14-day window, locale chip time), unread chats, important email, steps, spend (hidden at $0) — chips scroll to the matching module. Section is unmounted until pulse settles, and unmounts again if a later load throws.
- Modules: Messages (grouped unread → Messages.app; FDA scan is the full unread set via `listAllUnreadMessages`, run twice on first paint; unused `listUnreadMessages(limit)` / `messagesAccessStatus` exist; badge is all groups, card is `slice(0, limit)`), Calendar (section fetches ≥50 over 21 days, badges that, shows `limit`; unused `calendarAccessStatus` wrapper), Email (Google / Microsoft PKCE, Mail.app picker with timeouts, IMAP fallback; connect form auto-opens when disconnected; badge is All ≤500; `loadLists` always fetches the 500-row set; Internet Accounts open is uncaught; disconnected quiet refresh re-lists Mail.app; **Mail.app importance forges `to_addrs` and skips newsletter / unsubscribe / preview**), USPS Informed Delivery OCR thumbs (Mail “Open Email module” is `[data-module='email']` — silent if Email is hidden; “Email” / “Open digest” uncaught), News (ranked RSS; own 15-minute timer; Layout items ignored after mount; “More stories” in-card, hardcoded 40), Finance (14-day spend chart keyed with `localDayKey`; Add-transaction date is UTC + `T12:00:00Z`; **SQL month / category / spend-by-day cutoff are UTC too**; summary recent hardcoded 10; Manage jumps the badge to 50; `incomeThisMonth` unused; `updateAccount` idle; `update_transaction` / `list_categories` have no TS wrappers), Notes (compose lives in the card; All-drawer Edit writes that form; unused `getNote`), Health (Apple Health zip, re-import on refresh, sparklines labeled 7d regardless of `limit`; day list only when `history.length === 1`; heart is a today number with no sparkline; Setup is in-card and save is unhandled; **sleep sums InBed + stages + Awake and credits the start date**; **totals row reuses finance CSS classes**), Home (Ring + Blink stills — live fetch on mount; badge is full list; error wipes cameras; camera + Connect drawers can stack; thumbs have no catch), YouTube RSS (text list, `UC…` / `youtube.com/channel/…` only; dates are ISO `YYYY-MM-DD`; add-channel *and* Del *and* row-open are unhandled), Streaming (TMDB Tonight / hot / new; `limit` applied twice; featured poster is a `/w342` → `/w500` string replace; Sync disabled with no key even if rows exist), Shortcuts (badge is full dock; Edit unused)
- Command bar: live locale clock (`aria-live`, own 1s timer, separate from the hero’s 1s timer), theme cycle (auto / dusk / light — Auto is 19:00–07:00, 60s tick, not `prefers-color-scheme`; preference never persists), Layout customizer, Refresh all (15-minute quiet timer; `document.hidden` honored on the dashboard clock only; no in-flight lock)
- ⌘K palette: jump / shortcuts / quick note / theme / layout / refresh. “Go to” lists all 12 modules, including hidden ones. No focus trap.
- Persistence: SQLite `app.db`, Keychain for OAuth / IMAP / Blink refresh / Ring refresh. Blink *access* token still sits in settings (`home.blink_access_token`). Layout JSON is the only generic setting the IPC will touch.

Known v1 limits (README, still correct): macOS only, no in-app Messages reply, no Plaid, no multi-user sync, Blink unofficial API, Health is file import.

## The queue (do these, in this order)

### 0. Expand the settings allowlist (one-file, do this first)

`src-tauri/src/security.rs` `is_generic_setting_key` must allow:

- `dashboard.layout.v1` (already)
- `appearance.theme`
- `refresh.intervalMinutes`
- `onboarding.completed` (needed the moment increment 1 writes it)

Do **not** add `weather.place`, `weather.snapshot`, `health.export_path`, or any `email.*` / `home.*` / `streaming.*` key. Dedicated commands already persist those.

Also drop `list_settings` (Rust command + `listSettings` wrapper). Nothing in the UI calls it. It is a live dump of email host/user, OAuth client IDs, Blink hardware/account/tier/expiry, health export path, and the full layout JSON. `is_secret_setting_key` only filters `*password*` / `*token*` / `*secret*` / `*api_key*`.

`refresh.intervalMinutes` has `loadRefreshIntervalMinutes` and no saver. Add `saveRefreshIntervalMinutes` when the allowlist opens — and keep the minutes in React state so the interval rebinds. Today the timer is created once from a mount-time promise.

### 1. First-run that does not lie

Wait for `layoutReady` **before mounting cards** (stops default-12 IPC, not just flicker). Gate Layout / ⌘, on `layoutReady` so persist-before-load cannot overwrite a saved layout. Serialize Layout persist (clamp-to-3 can overwrite a typed 12; Items `max={50}` vs shortcuts clamp 24). Ship a starter layout (Messages, Calendar, Email, News, Finance, Notes — hide Mail / Health / Home / YouTube / Streaming / Shortcuts until the user turns them on) plus a 5-step overlay:

1. Grant Calendar + Full Disk Access (Messages)
2. Connect Email (Google / Microsoft / Mail.app) — **do not auto-open the in-card form**; `PermissionCallout` is dead after `loadSettings`
3. Pin weather
4. Optional: Health export, Home, YouTube, Streaming
5. Customize layout

Health Setup is the same in-card form — move it to a drawer with Email Connect. Keep last-good in briefing / Weather / Home / hero Calendar (access-deny currently leaves stale dots; a failed Weather/Home reload clears the card). **Roll hero `today` at midnight** (`useMemo([], …)` freezes `is-today`, the Today button, and the fetch window). **Flip `cancelled` on Calendar / briefing cleanup** (only Dashboard layout load does this today). Honor enabled modules in Refresh all, ⌘K, and Today briefing. Hidden News must not seed RSS; hidden Home must not hit Ring/Blink.

Badge visible rows (Email / Home / Messages / Calendar / Shortcuts; Streaming applies `limit` twice; Finance recent is hardcoded to 10 and Manage jumps the badge to 50). Honor News / Health `listLimit` after mount (News watches `showMore` only; Health sparkline caption stays “7d”; day list vanishes when `history.length > 1`). One drawer at a time (Home camera + Connect; overflow is not refcounted). Catch Home / Mail still-thumb failures **and** YouTube / Streaming / Mail `open*` **and** Email `openInternetAccounts`. Pulse calendar 14d/24 vs section 21d vs hero 200 — use 21d for pulse. Finance chip hidden at $0; `incomeThisMonth` unused. Hero Calendar needs a TCC empty state **and locale weekday headers** (hardcoded `Sun`–`Sat` today).

**Finance Add-transaction uses `localDayKey`, not `toISOString().slice(0, 10)` + `T12:00:00Z` — and `month_totals` / category chips / `list_spend_by_day` must stop using UTC `now`.** **Health sleep = Asleep* only (do not sum InBed + stages + Awake) and credit overnight minutes to the wake morning (`record_day` prefers `startDate`).** Share one locale clock formatter for hero digital, aria-label (drop 24h `HH:MM:SS`), visually-hidden (drop hardcoded `hour12: true`), command bar, briefing chips, and agenda. **Share one hour table for `greetingFor` (17/21/05) and `isDuskHour` (19–07).** Raise the stacked-hero breakpoint (CSS 860 vs window `minWidth` 960).

**Mail.app must score like IMAP.** Stop forging `to_addrs` from `settings.user`. Parse To: and `List-Unsubscribe` from the AppleScript / source you already fetch for Informed Delivery, or accept that Mail.app Important will stay a noisy unread dump. Empty `preview` is why Mail.app rows have no snippet. This belongs in the first-run Email step — connecting Mail.app should not flood the briefing chip.

Notes compose / Edit belongs in the drawer; Shortcuts Edit via existing `updateShortcut`.

### 2. Self-host fonts

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

`refresh_dashboard` ignores layout, so hidden Streaming / YouTube / Home / News still hit remote APIs every 15 minutes. News has **no** configured check — `run_refresh_news` always seeds defaults, then fetches. Status counts only `ok` (“Synced 1 module(s)” on first-run looks broken). Locals are labeled `skipped` then the UI reloads them via `REFRESH_EVENT`, so they never show “Synced Xm ago.” `formatSyncedAgo` runs once when `buildSyncState` lands — no tick. `onRefresh` has no in-flight guard (only the button `disabled={refreshing}`); the interval, `⌘⇧R`, and palette “Refresh all” can stack. The interval `useEffect` depends only on `[onRefresh]` and never reads `refreshing`. News has a second 15-minute timer that ignores `document.hidden` and `refreshing`, then paints “Background refresh complete” on the card. Home Refresh all reports `"{n} device(s) online"` for `devices.len()` — offline cameras and the Blink error placeholder count as online. Weather’s status line is `"{condition} · {temp}° in {city}"` with no C/F.

News / YouTube / Streaming / Weather / Ring already drop `DbState` during HTTP. **Email IMAP, Informed Delivery + Vision OCR, Health zip parse, and Blink homescreen + thumbnail downloads hold the mutex for the entire I/O.** Notes / layout saves wait.

Fix:

- Pass enabled modules (or skip hidden remote ones). Do not seed News feeds for a hidden card.
- Stream per-module status into the command bar (or a tiny sync popover); say “News synced · 6 skipped” for *remote* skips only — or mark locals `ok` (“Reloaded”)
- Tick “Synced Xm ago” (shared clock or store epoch + derive)
- Drop the News-only timer; one clock is enough. If News stays on its own timer for a while, honor `document.hidden` and `refreshing`, honor `limit` the way YouTube does (`[limit]` in the load effect, not just the timer), and drop the leftover `seedDefaultNewsFeeds` / `rerankNews` / `getNewsLastRefresh` wrappers (or use `getNewsLastRefresh` as the card’s last-sync)
- Guard overlapping Refresh all **inside `onRefresh`** (early-return / in-flight ref) so the interval, shortcut, and palette share one lock. Home’s card Refresh has no busy flag either — it can stack with Refresh all and then wipe cameras on the losing call.
- Surface `refresh.intervalMinutes` in Layout — add `saveRefreshIntervalMinutes` **and** keep minutes in React state so the timer rebinds. LayoutCustomize today is toggle / items / column / reset only. Serialize or debounce Items so clamp-to-3 cannot overwrite “12.”
- Home status should say “8 cameras” (or count `status == online`), not “8 device(s) online” for the mixed list. Weather status should reuse `formatTemp`.

### 9. Health staleness (small — diagnosis corrected)

`HealthDay.importedAt` is already on the type and in SQLite. The card never shows it. The header count is hardcoded to `1` whenever today exists (`count={!loading && today ? 1 : null}`) — looks broken. Omit the badge, or show days imported / a streak. Sparkline captions must follow `history.length`, not a baked-in “7d”. Keep the recent-days list visible when sparklines show — right now it vanishes at `history.length > 1`. Today already shows heart bpm; `avgHeartRate` is on every history row and never charted — a third sparkline is data you already import, not a new Health feature.

**Sleep hours are wrong before they are stale.** Filter SleepAnalysis to Asleep* only (`value` is parsed and ignored), then credit overnight intervals to the wake morning (or split at midnight) — see 18–20 Sep findings. Then skip the zip parse when the file has not changed.

**Refresh already re-imports.** `refresh_health_module` → `try_import_configured` re-parses the zip/xml on every Refresh all, with no mtime/hash short-circuit, **while holding `DbState`**. Large exports make Refresh all feel stuck after email + mail + news *and* freeze other IPC. Skip the parse when the file has not changed; show “last imported” age; keep a manual Import for when the user dropped a new export; drop the lock before the parse. Wrap `onSavePath` so a bad path is a field error. Move Setup out of the card (see increment 1). Restyle the totals row off `finance-*` classes (21 Sep).

Live HealthKit is a later extraction fight — don’t start it until onboarding + fonts + window chrome are done.

### 10. Signing, notarization, auto-update

Still `0.1.0`, no updater plugin, README already documents unsigned local `.app`. Do this when someone other than the author should install it. Not the next coding increment.

---

## Looks — specific nits worth a polish pass

After onboarding + fonts, a short visual pass (one PR):

- **Brand wordmark** — the 8-layer 3D extrude is distinctive; in dusk it can glare. Quieter dusk treatment (fewer layers, softer fill).
- **Double greeting, two hour tables** — `Clock` and `TodayBriefing` both call `greetingFor`. The clock ticks live; the briefing greeting is captured at pulse load and only updates on Refresh all / the 15-minute timer. Across noon they can disagree for up to 15 minutes. Across 17:00 they can also disagree with Auto-dusk (evening copy, light chrome) until 19:00. Keep greeting on the clock readout; let the briefing lead with “At a glance” only (drop the greeting eyebrow). If Auto stays clock-based, use the same cutoffs as the greeting — or label the button **Sunset**.
- **Four hour policies** — command bar live time + analog face + digital hero, each with its own 1s `setInterval`. Compact mode already repeats the hero. Hero digital and the visually-hidden span are forced 12-hour (`hour12: true`). The hero `aria-label` is 24-hour with seconds. The bar, briefing chips, hero agenda, and Calendar section follow the Mac. Once overlay chrome lands, drop seconds from the command bar, share one tick, **and share one `Intl` time for every string**. That same tick should roll hero `today` at midnight (see increment 1).
- **Command-bar `aria-live`** — VoiceOver should not hear the clock every second. The hero clock analog is `aria-hidden` and the visually-hidden time is hour:minute, but the parent `aria-label` still includes seconds (`HH:MM:SS`, 24-hour) and updates every tick. Drop seconds from the label; keep `aria-live` off the hero. The selected-day agenda is a second live region — don’t announce every day click.
- **Hero weekday headers** — locale `weekday: "short"` instead of hardcoded `Sun`–`Sat`. Cheap, and it makes the month feel native next to the localized heading.
- **Unused skeleton** — `ModuleSkeleton` + shimmer CSS already exist. Every module still prints “Loading…”. Wire the skeleton on first paint, and use it for the briefing slot while pulse loads (and on pulse error).
- **Module sameness** — almost every card is title + meta + ghost buttons. Give Messages / Email / News a denser row (avatar initial, unread pip, source mark) so the grid doesn’t read as twelve identical lists. News’ four Like / Hide / Follow / Mute chips are the worst offender. News “More stories” expands *in-card* (not a drawer) and the button shows even with zero stories — match Email / Messages All. YouTube / Streaming dates should use `Intl` like News, not raw ISO. Email / Shortcuts still ship a redundant **Open** next to a clickable row.
- **Notes** — inline title + textarea inside the card makes the module look like a form, not a journal. Shortcuts already compose in the drawer; Notes should match. The All-drawer Edit path currently writes that hidden form. The card lists up to `listLimit` (default 10) *under* the compose form, so the form eats the slot. Health Setup is the same in-card form.
- **Health as Finance** — `finance-totals` / `finance-total` / `finance-subhead` on the Health card. Shared stat chrome is fine; finance-named classes are not.
- **Empty states** — `PermissionCallout` is good for TCC. Other empties are a single muted sentence. One glyph empty per unconfigured module. Hero Calendar is the loudest miss: denied access looks like a free month. `PermissionCallout` keys list items by step text — duplicate steps warn in dev.
- **Favicon** — `index.html` still uses the Vite logo (`/vite.svg`).
- **Auto theme** — dusk/light is clock-based (`19:00–07:00`), not `prefers-color-scheme`, and the auto interval is 60s so dusk can lag the live clock by a minute. Either follow the system when preference is Auto, or label the button **Sunset** so the rule is honest — **and align it with `greetingFor`**. Theme also does not survive relaunch until increment 0.
- **Email in-card settings** — `setShowSettings(true)` on every disconnected load fights the overlay *and* stretches the Email slot to a full connect form *and* hides the `PermissionCallout`. Gate the auto-open; move the form to a drawer (Email “All” is already a drawer). Catch `openInternetAccounts`. Same move for Health Setup.
- **Count vs list** — Email badges All (≤500) while the card shows `top`. Home badges every camera while the card slices to `limit`. Messages badges every unread chat while the card slices to `topCount`. Calendar fetches ≥50, badges that, shows `limit`. **Shortcuts badges the full dock, shows `limit`.** Streaming badges hot+new (2× `limit`). Finance badges summary-10 (or Manage-50) and slices to `limit`; accounts stay at 4. Health badges `1`. Mail / YouTube / Notes already badge the limited fetch — copy that. Badge the visible list, or label the totals.
- **Wide-grid “Right column”** — at `min-width: 1180px` the grid is 3 columns. CSS `.placement-right` jumps to `grid-column: 3 / 4` (leaves a hole in column 2). The unused helper `placementGridStyle()` maps `"right"` to `2 / 3` — so if someone wires the helper later, “right” still would not mean the last column. Pick one: last column, or drop “right” until there is a real 3-slot placer.
- **Dead mobile stack** — `@media (max-width: 860px)` never runs in the `.app` (`minWidth: 960`). Raise the breakpoint or lower the window min so clock / weather / calendar can stack before they wrap.
- **Eyebrow drift** — card copy and `MODULE_META` disagree on YouTube (`Subscriptions` vs `Channels`), News (`For you` vs `Tailored`), Finance (`Snapshot` vs `Ledger`), Home (`Ring & Blink` vs `Cameras`), Streaming (`What's on` vs `Watch`), and Mail (`Informed Delivery` vs `Physical`). Pick one voice per module.
- **Weather readout** — humidity and wind are already on the snapshot. Show them. Label fahrenheit with °F. If units are celsius, request `wind_speed_unit=kmh` (or convert) so the line does not mix 22°C with mph. Reuse the same `formatTemp` in the Refresh-all status (today it is bare `°`).
- **Finance date field** — the Add-transaction date should be today’s *local* day (`localDayKey` already exists). UTC ISO + noon-Z is a looks bug (wrong default in the picker) and a ledger bug (spend bars miss the row). The SQL month/day cutoffs have to move with it.
- **Health sleep** — a 7h night that reads as 14h on yesterday and 0h today is a looks bug as much as a parse bug. Filter Asleep* and credit the wake morning before restyling the sparkline.
- **Streaming Tonight poster** — `posterPath.replace("/w342", "/w500")` assumes the Rust helper always emits `w342` (`streaming.rs`). A TMDB path change silently keeps the small card. Build the size from `file_path`, don’t string-replace.

## Functionality — later, harder, still real

Keep these behind the queue above:

- **In-app Messages reply** — AppleScript / private Messages APIs; easy to break; keep as a dedicated later slice.
- **Email send / reply** — SMTP or Graph/Gmail send. Needs a compose drawer and much more auth surface.
- **Calendar create** — EventKit write + a quick-add from ⌘K (`Lunch Friday 1pm`).
- **Notes ↔ Apple Notes** — tempting; permission + sync conflict. Skip until Reminders sync proves the EventKit pattern.
- **Finance live banks** — README correctly refuses Plaid. Category rules and a monthly envelope view are the useful local next step, not APIs. `incomeThisMonth` is already in the summary — surface it on the card before inventing envelopes. `updateAccount` is already wrapped if inline edit comes first; `update_transaction` / `list_categories` would need TS wrappers. Fix the UTC picker **and** the UTC SQL month/day filters before any of that, or evening US spend never hits the 14-day bars *or* “Spent · month.”
- **Weather units** — snapshot stores `celsius` / `fahrenheit`, but after pin there is no toggle; re-picking a city keeps the last units (default °F). A ghost °F/°C on the hero is enough. Same control should pick the wind unit (Open-Meteo is hardcoded to mph today). Humidity / wind are already in the snapshot — render them when the toggle lands.
- **Home** — Ring still wants a pasted refresh token. A guided “copy from existing token” is enough; don’t chase unofficial login. Ring’s token refresh posts `grant_type` + `refresh_token` with **no `client_id`** (`home.rs`) — if Ring tightens the endpoint, Home goes dark. `list_home_devices` should grow a local cache so mount / Refresh all are not the same live Ring+Blink round-trip. Closing Connect while `pinRequired` clears the PIN field and the flag — the user has to restart Blink login. Keep the pending verify across drawer close. Catch `homeDeviceImageBase64` / `physicalMailImageBase64` failures so a missing still is an empty tile, not an unhandled rejection. Blink access tokens belong in Keychain with the refresh token, not `settings`.
- **Disconnected Email re-lists Mail.app on every quiet refresh** — `onDashboardRefresh` → `refresh()` → `loadSettings()` → `loadAccounts()` whenever `!connected`. That is AppleScript on a 15-minute timer even after the timeout fix. List accounts only when the user opens Connect.
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email, health, and finance on every Refresh all, in parallel with each module’s own load (hero Calendar is a third EventKit read). On first paint this is Messages ×2 and EventKit ×3 before layout is known. Have modules publish into the briefing, or pass cached rows down. Use `listUnreadMessages(limit)` for the card and keep the full scan for the All drawer / pulse chat count. After increment 1, also skip sources the layout has hidden. Widen pulse calendar to the section’s 21 days (or share the section rows).
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank / `localDayKey` / SleepAnalysis-filter / locale-weekday / greeting-vs-dusk / Mail.app-header-score unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #57 became the security tip, because Mail.app Important still forges `to_addrs`, because greeting and Auto-dusk still use different hour tables, because the hero still forces three hour formats while the rest of the chrome follows the Mac, weekday headers are still English-only, Health overnight sleep still lands on yesterday, Finance SQL is still UTC, and Layout “items” still does not mean what the cards show.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#57** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24 / #25 / #26 / #27 / #28 / #29 / #30 / #31 / #32 / #33 / #34 / #35 / #36 / #37 / #38 / #39 / #40 / #41 / #42 / #43 / #44 / #45 / #46 / #47 / #48 / #49 / #50 / #51 / #52 / #53 / #54 / #55 / #56.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change, plus drop unused `list_settings`. Do not start onboarding without it. Do not allow-list weather/health keys.
3. First-run increment #1 (wait for `layoutReady` before mounting cards + gate Layout/⌘, + serialize persist + starter layout + overlay). Email must not auto-open in-card settings; Health Setup belongs in a drawer. Keep last-good in briefing / Weather / Home / hero Calendar. **Roll hero `today` at midnight. Flip `cancelled` on Calendar / briefing cleanup. Finance Add-transaction uses `localDayKey` and SQL month/day filters must be local too. Health sleep = Asleep* only and credit the wake morning. Share one locale clock (visual + aria-label + visually-hidden + command bar + chips + agenda). Share one hour table for greeting and Auto-dusk. Locale weekday headers. Raise stacked-hero breakpoint (or drop minWidth 960). Mail.app must parse To: / List-Unsubscribe (stop forging `to_addrs`).** Badge visible rows. Honor News/Health `listLimit` after mount; keep Health day list when sparklines show. Honor enabled modules in Refresh all, ⌘K, and Today briefing. Catch Home/Mail still-thumb failures and YouTube/Streaming/Mail `open*` **and** Email `openInternetAccounts`. Pulse calendar = section 21d; finance chip at $0. Hero Calendar permission empty state.
4. Self-host fonts.
5. Overlay title bar + capabilities + YouTube thumbs/@handles (`i.ytimg.com` in img-src; resolve handles to UC) + menu-bar pulse.
6. Refresh polish: in-flight lock on `onRefresh`, saver + state-backed timer, drop mutex around IMAP/OCR/Health/Blink I/O, tick last-sync labels, honor News `limit`, drop News-only timer (or honor `document.hidden`), stop calling every Home device “online”, weather status uses `formatTemp`.
