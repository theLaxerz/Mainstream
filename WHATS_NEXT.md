# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-22** (`476a28b`, still `0.1.0`). This supersedes the 21 Sep queue in [#58](https://github.com/theLaxerz/Mainstream/pull/58), plus [#56](https://github.com/theLaxerz/Mainstream/pull/56) / [#54](https://github.com/theLaxerz/Mainstream/pull/54) / [#52](https://github.com/theLaxerz/Mainstream/pull/52) / [#50](https://github.com/theLaxerz/Mainstream/pull/50) / [#48](https://github.com/theLaxerz/Mainstream/pull/48) / [#46](https://github.com/theLaxerz/Mainstream/pull/46) / [#44](https://github.com/theLaxerz/Mainstream/pull/44) / [#42](https://github.com/theLaxerz/Mainstream/pull/42) / [#40](https://github.com/theLaxerz/Mainstream/pull/40) / [#38](https://github.com/theLaxerz/Mainstream/pull/38) / [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make **Mail.app Important and USPS mail use the same mailbox IMAP already does**, make **Layout “items” actually change what you see**, make **the typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **twenty-third** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck. Close the older What’s Next drafts; do not treat another scan as new product work.
- **Security tip moved to #59.** The 21 Sep afternoon scan ([#59](https://github.com/theLaxerz/Mainstream/pull/59)) hardens HTTP body caps and XML depth. #57 is the 20 Sep XML DTD / JWT `aud` set. #55 / #53 / #51 / #49 / #47 / #45 / #43 / #41 / #39 / #37 / #35 / #33 / #31 / #29 / #27 / #25 / #22 / #20 / #18 / #16 / #14 are older copies. Master is still Vite `7.3.2` / zip `2.2` / `quick-xml` `0.37` / Tauri features `[]` / `allowedHosts` unset.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write. The deny message even says “not writable” on a get. `saveThemePreference` / `loadThemePreference` swallow that error, so the button looks honest.
- **Still true:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **Still true from 21 Sep:** Mail.app `sync_mailapp_inbox` forges `to_addrs` from `settings.user`, hardcodes `preview=""`, `has_list_unsubscribe=false`, `is_newsletter=false`, and scores with mailbox `"INBOX"` while storing `mailapp:{account}`. Base 10 + forged +25 = 35 ≥ 15, so almost every unread is “Important” unless the sender looks like noreply / notifications / news / marketing. Health totals row still reuses `finance-totals` / `finance-total` / `finance-subhead`.
- **Still true from 20 Sep:** `greetingFor` (17:00 evening / 21:00 night / 05:00 morning) and `isDuskHour` (19:00–07:00) disagree. 17:00–19:00 is evening copy on light; 05:00–07:00 is “Good morning” on dusk. Auto theme interval is 60s. Visually-hidden `hourLabel` hardcodes `hour12: true` — four hour policies, not three. Weather Refresh-all status prints `{:.0}°` with no C/F unit.
- **Still true from 19 Sep:** the hero clock speaks three hardcoded times (12h visual, 12h hidden, 24h `aria-label`); weekday headers are English `Sun`–`Sat`; overnight Health sleep all credits `startDate`.
- **Still true from 18 Sep:** Health sleep sums every SleepAnalysis interval (InBed + stages + Awake). Finance `month_totals` / category chips / `list_spend_by_day` use SQLite UTC `now`. Window `minWidth` 960 vs stacked-hero CSS at 860px — that breakpoint never runs in the `.app`. Hero digital clock is hardcoded 12-hour; command bar uses locale `hour: numeric`.
- **Still true from 17 Sep:** LayoutCustomize persist is unqueued (`void persist`) against a stale `draft`; typing Items “12” first persists `clampLimit(1)=3`, and `useEffect([open, layout])` can reset the draft from the write the drawer just sent. News’ own 15-minute timer ignores `document.hidden` and `refreshing`. `list_settings` dumps email host/user, weather/health paths, OAuth client IDs, and Blink hardware/account/tier/expiry (`home.blink_access_token` is filtered; the rest are not). Command-bar clock is `aria-live="polite"` on a 1s tick. Google Fonts `@import` is blocked by `connect-src` (not only `font-src`). YouTube thumbs need `i.ytimg.com` in `img-src`. Today briefing and ⌘K ignore layout. Locals are labeled `skipped` then reloaded via `REFRESH_EVENT`. `refresh.intervalMinutes` has a loader and no saver. Overlay title bar / tray need feature flags + capabilities beyond `core:default`. First-paint `defaultLayout()` mounts all 12 module effects before SQLite layout arrives. Today briefing returns `null` until pulse settles (and again if a later load throws). The interval timer never rebinds after a future saver. YouTube `@handle` is stored verbatim and the RSS URL 404s. `onRefresh` has no in-flight lock (`⌘⇧R` and palette ignore the button `disabled`). Home mount is a live Ring+Blink fetch. News `bootstrap` seeds RSS without Refresh all (`run_refresh_news` always `seed_feeds_if_empty`). Email’s `PermissionCallout` is dead after `loadSettings` auto-opens the in-card connect form. First paint reads Messages FDA twice and EventKit three times (hero **200** / section `max(limit,50)` / pulse **24**). ⌘K and every `DetailDrawer` are `aria-modal` without a focus trap; ⌘K + ⌘, can stack. Layout / ⌘, is not gated on `layoutReady`. Last-sync labels freeze at “just now.” News ignores Layout item counts (bootstrap is mount-only; the “More stories” effect watches `showMore` only). Notes All-drawer Edit writes the in-card form. Refresh all holds `DbState` through IMAP / Informed Delivery OCR / Health zip / Blink thumbs. Health already re-parses the export on every Refresh all (`try_import_configured`, no mtime). Weather / Home wipe last-good on a failed reload. `DetailDrawer` overflow is not refcounted. Health / Streaming / YouTube settings saves are unhandled rejections. Home Refresh all copy counts every device as “online.” Shortcuts joins Email / Home / Messages / Calendar on badge-vs-slice. Finance summary hardcodes recent to 10 and Manage jumps the badge to 50. Home / Mail still thumbs and YouTube Del reject unhandled. Health sparkline captions stay “7d” while `listHealthDays(limit)` honors Layout items. YouTube / Streaming / Mail `open*` still fire without try/catch. Health hides “Recent days” once `history.length > 1`. Pulse calendar is 14d/24 vs the card’s 21d vs the hero’s 200. Finance briefing hides a $0 month. Shortcuts Edit (`updateShortcut`) is unused. Layout Items `max={50}` snaps back to 24 for shortcuts. Hero Calendar freezes `today` in `useMemo([], …)` and never flips `cancelled`. Open-Meteo humidity/wind never reach the UI (wind always mph; fahrenheit has no F). Email `openInternetAccounts` is uncaught. Hero month has no TCC empty state; selected-day agenda is a second `aria-live`. `calendarAccessStatus` / `getNote` / `updateAccount` / `listUnreadMessages` / `messagesAccessStatus` / `seedDefaultNewsFeeds` / `rerankNews` / `getNewsLastRefresh` are idle wrappers. Rust `update_transaction` and `list_categories` have no TS wrappers. YouTube / Streaming dates are raw ISO. Finance Add-transaction seeds UTC ISO and writes `T12:00:00Z` while the 14-day chart keys `localDayKey()`. Health Setup is an in-card form like Email auto-open Connect.
- **New finding — Mail.app only sees 14 days of unread, and USPS mail only looks in a mailbox named INBOX.** IMAP `sync_mailbox` selects the configured mailbox and walks UNSEEN with no date window. Mail.app `list_unread_script` does `whose date received > cutoff` with `cutoff = (current date) - 14 * days`, then stops at 200 rows. Older unread never enters Mainstream, so a week-old thread IMAP would still call Important is invisible on the featured “click an account already on this Mac” path. The same file is inconsistent about *which* mailbox: unread tries `mailbox "INBOX"` and falls back to `item 1 of (get mailboxes of acct)`; `list_informed_script` and `source_script` hardcode `mailbox "INBOX"` with no fallback. Exchange / IMAP accounts whose inbox is Inbox / INBOX/ or a localized name can list Important from the fallback box and then fail Informed Delivery + RFC822 source (the OCR path). Share the unread fallback (or the configured mailbox name) with Informed Delivery and `source`, and drop or widen the 14-day cutoff so Mail.app and IMAP show the same unread set.
- **New finding — the briefing email chip is hard-capped at 20.** `loadDashboardPulse` calls `listImportantEmails(20)`. The Email badge is `all.length` from `list_all_important_emails` (clamp 500). The card shows `top` (`limit`, default 10). After the 21 Sep Mail.app scoring flood, the eyebrow can say “20 important” while the badge says 147 and the list shows 10. Use the same limited fetch the card displays (or label the chip “20+”) — do not invent a fourth count. Widen pulse calendar to the section’s 21 days in the same pass.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15), last updated 30 Aug), twenty-two stacked What’s Next docs, twenty-three overlapping security PRs. **Forty-six** open drafts.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts `@import` — blocked in Tauri; see fonts below), analog clock + digital readout + greeting (live 12-hour, not locale; `aria-label` is 24h `HH:MM:SS`; visually-hidden is forced `hour12: true`), Open-Meteo weather (cached snapshot; humidity/wind fetched and unused; a failed reload clears it; no °C/°F toggle after pin; wind always mph; Refresh-all status prints `{:.0}°` with no unit), EventKit month grid with selected-day agenda (`today` frozen at mount; 200 events; stale dots if access later denies; weekday headers hardcoded `Sun`–`Sat`; no TCC empty state; agenda is `aria-live="polite"`; day keys are `toISOString()`).
- Sticky command bar: live locale clock (`aria-live="polite"` on a 1s tick), ⌘K / theme / Layout / Refresh all, last-sync status that freezes at “just now.” Compact mode after 220px of scroll. Theme cycle Auto / Dusk / Light — persist denied by the allowlist. Auto dusk is 19:00–07:00 on a 60s interval.
- Today briefing chips (next event, unread chats, important email **capped at 20**, steps, spend > $0). Greeting is snapshotted at pulse load; Clock ticks live. Returns `null` until the first settle, and again if a later load throws. Ignores hidden modules. Calendar window is 14d/24 vs the card’s 21d vs the hero’s 200.
- Twelve modules, layout persist (enable / order / width / item counts) — first paint is `defaultLayout()` (all 12 on) until SQLite arrives; Layout / ⌘, is not gated on `layoutReady`. Persist is unqueued; Items `max={50}` vs shortcuts clamp 24.
- Messages (FDA + `chat.db`, grouped by conversation). Badge is `groups.length`; card slices to `topCount`.
- Calendar section (EventKit, 21 days, fetch `max(limit, 50)`, badge `events.length`). Cleanup never flips `cancelled`.
- Email (Google / Microsoft PKCE or Mail.app or IMAP). Disconnected load auto-opens the in-card connect form and kills `PermissionCallout`. Badge is All (≤500). Mail.app Important forges `to_addrs`, skips newsletter / unsubscribe / preview, and only walks 14 days of unread. `openInternetAccounts` is uncaught. Quiet refresh re-lists Mail.app accounts whenever disconnected.
- Physical mail (USPS Informed Delivery + Vision OCR). Mail.app Informed Delivery + RFC822 source require a mailbox literally named `INBOX`. Still thumbs reject unhandled.
- News (RSS rank + feedback). Own 15-minute timer ignores `document.hidden` / `refreshing`. Bootstrap is mount-only (Layout items ignored after mount; “More stories” watches `showMore` only). Hidden News still seeds default feeds on Refresh all.
- Finance (local ledger + CSV). 14-day chart keys `localDayKey()`; Add-transaction seeds UTC ISO + `T12:00:00Z`. `month_totals` / category chips / `list_spend_by_day` use SQLite UTC `now`. Summary recent is hardcoded to 10; Manage jumps the badge to 50; accounts stay `slice(0, 4)`. `incomeThisMonth` unused.
- Notes (compose in-card; All-drawer Edit writes that form).
- Health (Apple Health zip/xml, 7-day sparklines). Sleep sums InBed + stages + Awake onto `startDate`. Setup is in-card. Badge is `today ? 1`. Day list only when `history.length === 1`. Captions stay “7d.” Totals row uses finance CSS classes. Refresh all re-parses the export with no mtime skip.
- Home (Ring token + Blink OAuth/2FA). Mount is a live fetch; error clears devices. Badge is `devices.length`. Refresh-all copy calls every device “online.” Camera + Connect drawers can stack. Still thumbs reject unhandled.
- YouTube (channel RSS). `@handle` stored verbatim (404). No posters (`videoId` is already on the type). Dates are ISO `slice(0, 10)`. `open*` and Del and `onAddChannel` are uncaught.
- Streaming (TMDB hot / new + Tonight). Limit applied to hot *and* new. Tonight poster is a `/w342` → `/w500` string replace. `open*` and settings save uncaught.
- Shortcuts (URL / app launch). Badge is the full list; card slices to `limit`. Edit via existing `updateShortcut` is unused.

`src/` is the React UI. `src-tauri/` is Rust + SQLite (`app.db`). `feeds.default.json` seeds News. Window is 1280×860, `minWidth` 960 / `minHeight` 700, no overlay chrome. Capabilities are `core:default` only. CSP `font-src 'self'`; `img-src` allows TMDB but not `i.ytimg.com`. The only generic setting the IPC will touch is `dashboard.layout.v1`.

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

Health Setup is the same in-card form — move it to a drawer with Email Connect. Keep last-good in briefing / Weather / Home / hero Calendar (access-deny currently leaves stale dots; a failed Weather/Home reload clears the card). **Roll hero `today` at midnight** (`useMemo([], …)` freezes `is-today`, the Today button, and the fetch window). **Flip `cancelled` on Calendar / briefing cleanup** (only Dashboard layout load does this today — hero Calendar’s cleanup is just `onDashboardRefresh` and never assigns `cancelled = true`, so a month-change race can paint the previous window). Honor enabled modules in Refresh all, ⌘K, and Today briefing. Hidden News must not seed RSS; hidden Home must not hit Ring/Blink.

Badge visible rows (Email / Home / Messages / Calendar / Shortcuts; Streaming applies `limit` twice; Finance recent is hardcoded to 10 and Manage jumps the badge to 50). Honor News / Health `listLimit` after mount (News watches `showMore` only; Health sparkline caption stays “7d”; day list vanishes when `history.length > 1`). One drawer at a time (Home camera + Connect; overflow is not refcounted). Catch Home / Mail still-thumb failures **and** YouTube / Streaming / Mail `open*` **and** Email `openInternetAccounts`. Pulse calendar 14d/24 vs section 21d vs hero 200 — use 21d for pulse. **Pulse email must not invent a 20-cap the card and badge do not share.** Finance chip hidden at $0; `incomeThisMonth` unused. Hero Calendar needs a TCC empty state **and locale weekday headers** (hardcoded `Sun`–`Sat` today).

**Finance Add-transaction uses `localDayKey`, not `toISOString().slice(0, 10)` + `T12:00:00Z` — and `month_totals` / category chips / `list_spend_by_day` must stop using UTC `now`.** **Health sleep = Asleep* only (do not sum InBed + stages + Awake) and credit overnight minutes to the wake morning (`record_day` prefers `startDate`).** Share one locale clock formatter for hero digital, aria-label (drop 24h `HH:MM:SS`), visually-hidden (drop hardcoded `hour12: true`), command bar, briefing chips, and agenda. **Share one hour table for `greetingFor` (17/21/05) and `isDuskHour` (19–07).** Keep greeting on the clock; let the briefing lead with “At a glance” only (the eyebrow is a pulse snapshot and drifts from the live clock across noon / 17:00 / 21:00). Raise the stacked-hero breakpoint (CSS 860 vs window `minWidth` 960). **At ≥1180px, `.placement-right` is `grid-column: 3 / 4` while left stays `1 / 2` — Left+Right leaves a hole in column 2.** Unused `placementGridStyle()` still maps right to `2 / 3`. Pick last-column or drop “right” until there is a real 3-slot placer.

**Mail.app must score like IMAP *and* see the same mailbox.** Stop forging `to_addrs` from `settings.user`. Parse To: and `List-Unsubscribe` from the AppleScript / source you already fetch for Informed Delivery, or accept that Mail.app Important will stay a noisy unread dump. Empty `preview` is why Mail.app rows have no snippet. **Drop or widen the 14-day `date received` cutoff** (IMAP has none). **Reuse the unread mailbox fallback in `list_informed_script` and `source_script`** — a mailbox that is not literally `"INBOX"` currently lists Important and then fails USPS OCR. This belongs in the first-run Email step — connecting Mail.app should not flood the briefing chip or silently skip Informed Delivery.

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
- **Count vs list** — Email badges All (≤500) while the card shows `top`. Home badges every camera while the card slices to `limit`. Messages badges every unread chat while the card slices to `topCount`. Calendar fetches ≥50, badges that, shows `limit`. **Shortcuts badges the full dock, shows `limit`.** Streaming badges hot+new (2× `limit`). Finance badges summary-10 (or Manage-50) and slices to `limit`; accounts stay at 4. Health badges `1`. Mail / YouTube / Notes already badge the limited fetch — copy that. Badge the visible list, or label the totals. **Briefing email is a third number (`listImportantEmails(20)`).**
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
- **Pulse duplicate fetches** — `loadDashboardPulse` re-queries calendar, all unread messages, important email (20), health, and finance on every Refresh all, in parallel with each module’s own load (hero Calendar is a third EventKit read). On first paint this is Messages ×2 and EventKit ×3 before layout is known. Have modules publish into the briefing, or pass cached rows down. Use `listUnreadMessages(limit)` for the card and keep the full scan for the All drawer / pulse chat count. After increment 1, also skip sources the layout has hidden. Widen pulse calendar to the section’s 21 days (or share the section rows). Stop the 20-email cap from disagreeing with the badge.
- **Email always loads the All list** — `loadLists` calls `listAllImportantEmails()` even when the All drawer is closed. Fetch that on open.
- **Tests** — only `emailConnectors.test.ts` on the frontend (#15 adds `tasks.test.ts`). Add pulse / layout / news-rank / `localDayKey` / SleepAnalysis-filter / locale-weekday / greeting-vs-dusk / Mail.app-header-score / Mail.app-mailbox-fallback unit tests when those files next change. Don’t start a test-only PR.

## Explicitly not next

- New vendor modules (Spotify, Slack, GitHub, Stocks) until onboarding exists — more empty cards make first-run worse.
- Cloud sync / multi-user / non-macOS.
- Replacing the local-first story with hosted accounts.
- Re-implementing Tasks, weather, ⌘K, dusk, health sparklines, Streaming Tonight, briefing, clock, hero agenda, finance chart, email OAuth, or Blink stills.
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #59 became the security tip, because Mail.app Important still forges `to_addrs` *and* only walks 14 days of one mailbox named INBOX, because the briefing email chip is a third count (20), because greeting and Auto-dusk still use different hour tables, because the hero still forces three hour formats while the rest of the chrome follows the Mac, weekday headers are still English-only, Health overnight sleep still lands on yesterday, Finance SQL is still UTC, and Layout “items” still does not mean what the cards show.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#59** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24 / #25 / #26 / #27 / #28 / #29 / #30 / #31 / #32 / #33 / #34 / #35 / #36 / #37 / #38 / #39 / #40 / #41 / #42 / #43 / #44 / #45 / #46 / #47 / #48 / #49 / #50 / #51 / #52 / #53 / #54 / #55 / #56 / #57 / #58.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change, plus drop unused `list_settings`. Do not start onboarding without it. Do not allow-list weather/health keys.
3. First-run increment #1 (wait for `layoutReady` before mounting cards + gate Layout/⌘, + serialize persist + starter layout + overlay). Email must not auto-open in-card settings; Health Setup belongs in a drawer. Keep last-good in briefing / Weather / Home / hero Calendar. **Roll hero `today` at midnight. Flip `cancelled` on Calendar / briefing cleanup. Finance Add-transaction uses `localDayKey` and SQL month/day filters must be local too. Health sleep = Asleep* only and credit the wake morning. Share one locale clock (visual + aria-label + visually-hidden + command bar + chips + agenda). Share one hour table for greeting and Auto-dusk. Locale weekday headers. Raise stacked-hero breakpoint (or drop minWidth 960). Fix 3-col Left+Right hole. Mail.app must parse To: / List-Unsubscribe (stop forging `to_addrs`), drop the 14-day unread cutoff, and share the inbox fallback with Informed Delivery / source.** Badge visible rows (briefing email must not be a third 20-cap). Honor News/Health `listLimit` after mount; keep Health day list when sparklines show. Honor enabled modules in Refresh all, ⌘K, and Today briefing. Catch Home/Mail still-thumb failures and YouTube/Streaming/Mail `open*` **and** Email `openInternetAccounts`. Pulse calendar = section 21d; finance chip at $0. Hero Calendar permission empty state.
4. Self-host fonts.
5. Overlay title bar + capabilities + YouTube thumbs/@handles (`i.ytimg.com` in img-src; resolve handles to UC) + menu-bar pulse.
6. Refresh polish: in-flight lock on `onRefresh`, saver + state-backed timer, drop mutex around IMAP/OCR/Health/Blink I/O, tick last-sync labels, honor News `limit`, drop News-only timer (or honor `document.hidden`), stop calling every Home device “online”, weather status uses `formatTemp`.
