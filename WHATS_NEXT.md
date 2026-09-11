# What's Next — Mainstream

Product plan from a full scan of `master` on **2026-09-11** (`476a28b`, still `0.1.0`). This supersedes the 10 Sep queue in [#36](https://github.com/theLaxerz/Mainstream/pull/36), plus [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17).

The dashboard already feels like a Life OS. The next work should **unblock prefs that silently fail**, make **first launch obvious**, make the **typeface actually ship in the `.app`**, and add one or two moments that feel magical — not another empty module.

## What changed overnight

- **Master still has not moved.** Tip is `476a28b` (Mail.app AppleScript timeout, 29 Aug). This is the **twelfth** planning day stacked on the same commit. Discovery is done. Shipping is the bottleneck. Close the older What’s Next drafts; do not treat another scan as new product work.
- **Security tip moved to #37.** The 10 Sep afternoon scan ([#37](https://github.com/theLaxerz/Mainstream/pull/37)) re-applies the 5–9 Sep IPC/SSRF set, then adds `mailto:` rejection of `attach` / `attachment` / `attachments` plus nested `javascript:` / `data:` / `file://` (Mail.app local-file disclosure class), and a Cargo.lock poison test for `aligned_box` 0.3.0. #35 / #33 / #31 / #29 / #27 / #25 / #22 / #20 / #18 / #16 / #14 are older copies. Master is still Vite `7.3.2` / zip `2.2` / Tauri features `[]` / `allowedHosts` unset.
- **Settings allowlist is still the product blocker.** `is_generic_setting_key` only allows `dashboard.layout.v1`. Both `get_setting_cmd` and `set_setting_cmd` deny everything else, so theme never *loads* either — it is not just a silent write.
- **Still true:** do **not** allow-list `weather.place`, `weather.snapshot`, or `health.export_path`. Those already persist through dedicated commands. A kitchen-sink allowlist would expose them on the generic IPC.
- **Still true:** Email’s badge is `all.length` while the card renders `top` (`limit`). `list_all_important_emails` is already clamped to **500**. Health already re-parses the export zip on every Refresh all (`try_import_configured`, no mtime). Command-bar clock is `aria-live="polite"` on a 1s tick. Google Fonts `@import` is blocked by `connect-src` (not only `font-src`). YouTube thumbs need `i.ytimg.com` in `img-src`. Today briefing and ⌘K ignore layout. Locals are labeled `skipped` then reloaded via `REFRESH_EVENT`. `refresh.intervalMinutes` has a loader and no saver. Overlay title bar / tray need feature flags + capabilities beyond `core:default`. First-paint `defaultLayout()` mounts all 12 module effects before SQLite layout arrives. Today briefing returns `null` until pulse settles. The interval timer never rebinds after a future saver. YouTube `@handle` is stored verbatim and the RSS URL 404s. `onRefresh` has no in-flight lock. Home mount is a live Ring+Blink fetch. News `bootstrap` seeds RSS without Refresh all. Email’s `PermissionCallout` is dead after `loadSettings` auto-opens the connect form. First paint reads Messages FDA twice and EventKit three times. ⌘K is `aria-modal` without a focus trap.
- **New finding — Layout / ⌘, is not gated on `layoutReady`.** Command bar opens customize immediately. `LayoutCustomize` drafts whatever is in React state, which starts as `defaultLayout()` (all 12 enabled). A toggle before the SQLite load returns persists that default and can overwrite a saved layout; a late `setLayout` from the in-flight `get_setting` can also clobber an in-progress customize. Gate ⌘, / Layout until `layoutReady`, and do not mount the drawer against the first-paint default.
- **New finding — `list_settings` is a live ungated dump, not just dead code.** Registered in `lib.rs`, wrapped in `api.ts`, never called by the UI. It selects every settings row and only filters `is_secret_setting_key`. That still returns email host/user, `weather.place`, `health.export_path`, Blink account/hardware/tier, `news_last_refresh_at`. Drop the command. Do not “fix” it by allow-listing those keys.
- **New finding — last-sync labels freeze at “just now”.** `formatSyncedAgo` runs once when `buildSyncState` lands. Modules do not tick, so “Synced just now” stays until the next Refresh all or a local setState. After increment 8, either tick the label or store an epoch and derive the string on a shared clock.
- **New finding — News ignores Layout’s item count after mount.** `bootstrap` is `[]` (mount-only). YouTube / Streaming / Health / Notes / Email all re-run when `limit` changes. Changing “items” in Layout does nothing for News until “More stories” or a remount. The News-only 15-minute timer also does not check `refreshing`, so it can overlap the card Refresh button *and* dashboard Refresh all *and* a still-running bootstrap.
- **New finding — Notes “Edit” in the All drawer writes the in-card form behind the overlay.** `startEdit` fills the title/textarea that live in the card. The drawer lists notes but has no compose surface, so the user cannot see what they are editing until they hit Close. This is why increment 1’s “Notes compose belongs in the drawer” is not just looks — the All path is broken.
- **New finding — every `DetailDrawer` is `aria-modal` without a focus trap.** Same class as ⌘K (10 Sep), but Layout, Email All, Notes All, YouTube channels, Home connect, Streaming services, and Mail detail all share `DetailDrawer`. No initial focus; Tab escapes to the dashboard. ⌘K and ⌘, also do not dismiss each other, so two `aria-modal` dialogs can stack. Fold drawers + stacked-dialog dismiss into increment 7.
- **New finding — Refresh-all mutex is not uniform.** Yesterday said “holds the SQLite mutex around each module.” Sharper: News / YouTube / Streaming / Weather / Ring already drop the lock during HTTP. **Email IMAP, Informed Delivery + Vision OCR, Health zip parse, and Blink homescreen + thumbnail downloads hold `DbState` for the entire I/O.** The webview cannot list notes or save layout until those finish. Parallelize *and* do the heavy work off the mutex.
- **New finding — a failed pulse reload unmounts the briefing.** `TodayBriefing` `catch` sets `pulse` to `null`, which `return null`s. A Refresh-all glitch after a good first load collapses the section and shoves the grid up. Keep the slot mounted on error, same as the first-paint skeleton fix.
- **Still not on master:** Tasks ([#15](https://github.com/theLaxerz/Mainstream/pull/15), last updated 30 Aug), eleven stacked What’s Next docs, twelve overlapping security PRs. **Twenty-four** open drafts.

## Snapshot (what's on master)

Shipped and working:

- Hero: Fraunces wordmark (Google Fonts `@import` — blocked in Tauri; see fonts below), analog clock + digital readout + greeting, Open-Meteo weather (cached snapshot, refetch when stale), EventKit month grid with selected-day agenda
- Today briefing: greeting eyebrow, “At a glance” title, next event, unread chats, important email, steps, spend — chips scroll to the matching module. Section is unmounted until pulse settles, and unmounts again if a later load throws.
- Modules: Messages (grouped unread → Messages.app; FDA scan capped at 500, run twice on first paint), Calendar, Email (Google / Microsoft PKCE, Mail.app picker with timeouts, IMAP fallback; connect form auto-opens when disconnected), USPS Informed Delivery OCR thumbs, News (ranked RSS, auto-seeds `feeds.default.json` on first empty load; ignores later `listLimit` changes), Finance (local ledger + CSV + 14-day spend chart), Notes (inline compose; All-drawer Edit writes the hidden card form), Health (Apple Health zip, re-import on refresh, 7-day sparklines), Home (Ring + Blink stills — live fetch on mount), YouTube RSS (text list, `UC…` / `youtube.com/channel/…` only), Streaming (TMDB Tonight / hot / new with posters; local list on mount), Shortcuts (compose lives in the drawer)
- Chrome: sticky command bar, ⌘K (jump / shortcuts / quick note), theme cycle, layout persist in SQLite (customize not gated on `layoutReady`), 15-minute quiet refresh, per-module last-sync (remote `ok`/`error` only; the “Xm ago” string does not tick)
- Hardening: Tauri CSP, `freezePrototype`, Keychain secrets, IPC validators. Rust unit tests exist in `security.rs`, email, finance, Mail.app, OAuth. Frontend tests are only `emailConnectors.test.ts`. `list_settings` remains a registered dump of non-secret dedicated keys.

Open PRs — **do not rebuild these on another branch:**

| PR | What | Action |
| --- | --- | --- |
| [#15](https://github.com/theLaxerz/Mainstream/pull/15) | Local Tasks (due buckets, command-palette `Task: …`, briefing chip) | **Merge first.** Highest-value missing module. Last touched 30 Aug. |
| [#37](https://github.com/theLaxerz/Mainstream/pull/37) | mailto `attach=` block, `aligned_box` poison, Chromium KEV review (10 Sep scan) | Review and land. Newest of the security set. |
| [#35](https://github.com/theLaxerz/Mainstream/pull/35) / [#33](https://github.com/theLaxerz/Mainstream/pull/33) / [#31](https://github.com/theLaxerz/Mainstream/pull/31) / [#29](https://github.com/theLaxerz/Mainstream/pull/29) / [#27](https://github.com/theLaxerz/Mainstream/pull/27) / [#25](https://github.com/theLaxerz/Mainstream/pull/25) / [#22](https://github.com/theLaxerz/Mainstream/pull/22) / [#20](https://github.com/theLaxerz/Mainstream/pull/20) / [#18](https://github.com/theLaxerz/Mainstream/pull/18) / [#16](https://github.com/theLaxerz/Mainstream/pull/16) / [#14](https://github.com/theLaxerz/Mainstream/pull/14) | Older security copies | Close as superseded by #37. |
| [#36](https://github.com/theLaxerz/Mainstream/pull/36) / [#34](https://github.com/theLaxerz/Mainstream/pull/34) / [#32](https://github.com/theLaxerz/Mainstream/pull/32) / [#30](https://github.com/theLaxerz/Mainstream/pull/30) / [#28](https://github.com/theLaxerz/Mainstream/pull/28) / [#26](https://github.com/theLaxerz/Mainstream/pull/26) / [#24](https://github.com/theLaxerz/Mainstream/pull/24) / [#23](https://github.com/theLaxerz/Mainstream/pull/23) / [#21](https://github.com/theLaxerz/Mainstream/pull/21) / [#19](https://github.com/theLaxerz/Mainstream/pull/19) / [#17](https://github.com/theLaxerz/Mainstream/pull/17) | Prior What’s Next docs | Close when this lands. |

---

## Next increments (in this order)

### 0. Settings allowlist (one-file unblock)

Expand `is_generic_setting_key` to:

- `dashboard.layout.v1` (already allowed)
- `appearance.theme`
- `refresh.intervalMinutes`
- `onboarding.completed`

That is the full list. Dedicated commands already persist weather, health, streaming, email, and home. Adding those keys here would put them on the generic get/set IPC.

Also **delete** `list_settings` (Rust command + `api.ts` wrapper). It is unused and dumps mailbox identity, weather coordinates, the Health export path, and Blink account metadata.

`refresh.intervalMinutes` still has no saver — allowlisting it does not create a Layout control. Increment 8 adds `saveRefreshIntervalMinutes` and a state-backed timer. Until then the key is writeable and unread.

### 1. First-run: wait for layout, then onboard

Do not mount module cards until `layoutReady`. Today `useState(defaultLayout)` paints all 12 `useEffect`s (Messages FDA, EventKit ×2, email settings ×2, News RSS seed, Home Ring+Blink, Mail thumbs, …) before SQLite can hide anything. Waiting only stops the flicker if the cards are already talking to the OS.

Then:

1. **Welcome** — one sentence: local-first, data stays on this Mac.
2. **Permissions** — Full Disk Access (Messages) and Calendars, with the existing deep links. Skip is fine.
3. **Connect** — Email (browser sign-in or Mail.app). Weather city. TMDB key if they want Streaming.
4. **Starter layout** — Messages, Calendar, Email, News, Notes, Shortcuts on. Mail / Finance / Health / Home / YouTube / Streaming off until configured. Persist that; do not leave first-run on the all-12 default.
5. **Done** — “Refresh all” once. Mark `onboarding.completed`.

Honor enabled modules from day one:

- Gate **Layout / ⌘,** until `layoutReady`. Never persist `defaultLayout()` over a layout that has not been read yet.
- `refresh_dashboard` must take enabled IDs (or skip hidden *and* unconfigured). Hidden Streaming / YouTube / Home must not hit TMDB / RSS / cameras every 15 minutes. Hidden News must not seed + fetch RSS on mount.
- ⌘K “Go to” and Today briefing chips must use `enabledModules(layout)`, not `MODULE_IDS` / every pulse source. `CommandPalette` currently does not receive `layout` at all. Mail’s “Open Email module” (`[data-module='email']`) is the same silent no-op when Email is hidden — use `#module-email` and honor enabled.
- Keep the briefing section mounted (skeleton, empty chips, or last-good pulse) so the grid does not jump when pulse arrives **or** when a later load throws.
- After layout is known, do not re-query Messages / EventKit / email settings from pulse *and* each card on the same tick — publish once or pass rows down.
- Email must not auto-open in-card settings (`loadSettings` → `setShowSettings(true)`). Keep `PermissionCallout` as the empty state.
- News must not `refreshNews()` on an empty table if the module is hidden. Home must not `list_home_devices` on a hidden card.
- Notes compose (and Edit) belongs in the drawer, matching Shortcuts. The All-drawer Edit path is the proof.

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

⌘K already jumps, launches, and captures a note. Matching is `includes`, not fuzzy. The command-bar button still says **Search**. “Go to” currently lists all twelve modules even when hidden. Palette does not take `layout` as a prop. It is `role="dialog" aria-modal="true"` but Tab leaves the panel, and arrows do not wrap.

`DetailDrawer` is the same dialog class without a trap (Layout, Email All, Notes All, YouTube, Home, Streaming, Mail). ⌘K and ⌘, do not dismiss each other.

After Tasks: `Task: …`, natural-time notes, and a “pin city / connect email” setup action when those are empty. Fuzzy match. Keyboard hint row. Rename the button to **⌘K**. Hide or disable “Go to” for modules the layout has turned off. Trap focus while open; wrap arrow selection. Apply the same trap to `DetailDrawer`. Opening one overlay should close the other.

### 8. Refresh that feels alive

`refresh_dashboard` runs modules **sequentially**. News / YouTube / Streaming / Weather / Ring already drop `DbState` during HTTP. Email IMAP, Informed Delivery + OCR, Health zip parse, and Blink homescreen + thumbnail downloads **do not** — they hold the mutex for the whole I/O, so notes / layout / palette IPC stall. Status is a single truncated string that only counts `ok`. Hidden modules still sync. Locals are labeled `skipped` even though `REFRESH_EVENT` reloads them. Per-module “Synced just now” never advances.

News also keeps its **own** 15-minute timer (`NEWS_REFRESH_MS`) on top of the dashboard interval, so feeds can double-fetch. That timer does **not** pause when the window is hidden (the dashboard interval does) and does **not** check `refreshing`.

`onRefresh` has no in-flight flag (10 Sep). The interval, `⌘⇧R`, and palette “Refresh all” can all stack a still-running run. The button disable is cosmetic. The timer is also keyed only on `[onRefresh]`, so it will not restart if the user later changes minutes in Layout.

- Skip modules the layout has disabled (and still skip unconfigured)
- Parallelize independent modules (email → mail still ordered)
- Do IMAP / OCR / Health zip / Blink HTTP **off** the SQLite mutex (News/YouTube already show the pattern)
- Stream per-module status into the command bar (or a tiny sync popover); say “News synced · 6 skipped” for *remote* skips only — or mark locals `ok` (“Reloaded”)
- Tick “Synced Xm ago” (shared clock or store epoch + derive)
- Drop the News-only timer; one clock is enough. If News stays on its own timer for a while, honor `document.hidden` and `refreshing`
- Honor `limit` on News the way YouTube does (`[limit]` in the effect)
- Guard overlapping Refresh all **inside `onRefresh`** (early-return / in-flight ref) so the interval, shortcut, and palette share one lock
- Surface `refresh.intervalMinutes` in Layout — add `saveRefreshIntervalMinutes` **and** keep minutes in React state so the timer rebinds. LayoutCustomize today is toggle / items / column / reset only.

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
- Another planning-only PR while `476a28b` is still tip — close the older What’s Next PRs instead. This document exists because #37 became the security tip and because Layout/⌘, can overwrite a saved layout before `layoutReady`, `list_settings` is a live dedicated-key dump, last-sync labels freeze, News ignores `listLimit` and overlaps its own timer, Notes All-drawer Edit is behind the overlay, every `DetailDrawer` shares ⌘K’s missing focus trap, Refresh all holds `DbState` through IMAP/OCR/Health/Blink I/O, and a failed pulse unmounts the briefing.

---

## Suggested sequence for upcoming automations

1. Human: land **#15 Tasks**, land **#37** (or close it), close #14 / #16 / #17 / #18 / #19 / #20 / #21 / #22 / #23 / #24 / #25 / #26 / #27 / #28 / #29 / #30 / #31 / #32 / #33 / #34 / #35 / #36.
2. **Settings allowlist** (this document’s increment #0) — one-file Rust change, plus drop unused `list_settings`. Do not start onboarding without it. Do not allow-list weather/health keys.
3. **Onboarding + starter layout + wait for `layoutReady` before mounting cards** (increment #1). Gate Layout/⌘, on `layoutReady`. Honor enabled modules in Refresh all, ⌘K, and the Today briefing from day one. Keep the briefing slot mounted (including pulse errors). Do not auto-open Email settings. Do not let hidden News/Home hit the network on mount. Move Notes compose/Edit into the drawer.
4. **Self-hosted fonts + overlay title bar** (increments #2 and #4 — can ship together; include capability grants). Self-host; do not open CSP to Google Fonts.
5. **YouTube thumbs + @handles** (#3) — add `i.ytimg.com` to `img-src`; derive posters from `videoId`; resolve handles to `UC…` (do not store `@name`).
6. **Menu-bar pulse** (#5) *or* palette/refresh polish (#7–8), whichever is a one-PR slice. Refresh polish must add a saver **and** state-backed timer for `refresh.intervalMinutes`, put the in-flight lock on `onRefresh` (interval + `⌘⇧R` + palette), drop the mutex around IMAP/OCR/Health/Blink I/O, tick last-sync labels, honor News `limit`, and stop labeling local reloads as skipped. Palette polish must trap focus on ⌘K **and** `DetailDrawer`, and prevent stacked dialogs.
7. Reminders sync only after Tasks has lived on master.

One increment per PR. If a run is tempted to add a new module, it should implement the settings allowlist and onboarding instead.
