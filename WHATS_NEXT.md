# What's next

Scan of master on **2026-10-04**. Tip is still `476a28b` (*Stop Mail.app AppleScript from freezing the UI when adding Outlook.*, 29 Aug) — **thirty-fifth** planning day on the same commit.

Newest security PR is **#82**. Settings allowlist is still the product blocker — get *and* set deny everything except `dashboard.layout.v1`. Do not allow-list weather/health dedicated keys.

**New this scan:** the hero talks to itself. `Clock` ticks `greetingFor(now)` every second (`Clock.tsx`). `TodayBriefing` stores `pulse.greeting` from `loadDashboardPulse()` and never re-ticks. Leave the window open from morning and by 17:00 the analog face says **Good evening** while Today still says **Good morning**. `formatChipTime` also snapshots `new Date()` at pulse-load, so “same day” after midnight still prints only a time. Hero `Calendar` freezes `today` in `useMemo([], …)` at mount, so the highlighted cell and **Today** button stay on launch day. Auto-dusk uses a *third* clock: `isDuskHour` is `>= 19 || < 7`, while greetings cut at 5 / 12 / 17 / 21. One hour table should drive greeting, briefing eyebrow, and theme.

Same first paint: `refresh_dashboard` **skip-stamps** Messages, Calendar, Finance, Notes, and Shortcuts (`refresh.rs` 309–321). Those cards never get a last-synced line from ⌘⇧R / the 15-minute timer — Refresh All is a half-dashboard. Ring device `id` is `Value::to_string()`, so a JSON string becomes `ring-"123"` with quotes. Opening Finance Manage re-runs the full `get_finance_summary` because `useEffect` depends on `manageOpen`. Notes **All** edits write into the card composer (`startEdit` shares `title` / `body` / `editingId`). There is still no `saveRefreshIntervalMinutes` — even after the allowlist grows, nothing in the UI writes `refresh.intervalMinutes`.

Supersedes #81 / #79 / #77 / #75 / #73 / #71 / #69 / #67 / #65 / #64 / #62 / #60 / #58 / #56 / #54 / #52 / #50 / #48 / #46 / #44 / #42 / #40 / #38 / #36 / #34 / #32 / #30 / #28 / #26 / #24 / #23 / #21 / #19 / #17. Does not re-implement Tasks (#15) or the security set (#82).

Sixty-nine open drafts before this increment.

---

## Merge hygiene (do first)

1. Land **PR #15** (Tasks) — do not re-implement.
2. Land **PR #82** (3 Oct IPC / crate harden) or close it in favor of the next security scan; close older security drafts including #80 / #78.
3. Close duplicate planning PRs **#14, #16–#81** once this plan (or an implementation of it) lands.

Master has not moved since 29 Aug. Shipping any one of the items below beats another planning rewrite.

---

## 0. Unlock settings (blocks theme, refresh, onboarding)

`is_generic_setting_key` only allows `dashboard.layout.v1`. Both get and set are gated, so `appearance.theme` and `refresh.intervalMinutes` never persist. `saveThemePreference` / `loadThemePreference` swallow the deny. Allow-list only:

- `appearance.theme`
- `refresh.intervalMinutes`
- `onboarding.completed`

Do not allow-list weather/health dedicated keys (those have their own commands). Drop unused `list_settings`. Add a real `saveRefreshIntervalMinutes` (today there is only a loader). Even after a saver exists, the refresh timer must live in React state so it rebinds. Boot a default `data-theme` on `<html>` before React so dusk does not flash light (`index.html` has none; first paint hits EventKit three times with no `refreshDashboard`).

---

## 1. First-run that does not lie

Wait for `layoutReady` **before mounting cards**. Gate Layout / ⌘, on `layoutReady`. Serialize Layout persist. Ship a starter layout + a short onboarding overlay (Calendar TCC, Full Disk Access, pin a city, connect email). Honor enabled modules in Pulse, ⌘K “Go to”, and Refresh All. All twelve sections currently fetch on mount while the layout is still the default.

**Time (new 4 Oct)**

- Share **one** ticking `now` (or a midnight + day-part timer) across Clock, briefing eyebrow, hero Calendar `today`, and Auto-dusk.
- Recompute `greetingFor` in the briefing; do not store it on the pulse snapshot.
- `formatChipTime` must ask “is this local today?” at render time, not at pulse-load.
- One hour table: greeting cuts *and* `isDuskHour` (today 5/12/17/21 vs 19/7).

**Refresh All (new 4 Oct + 3 Oct)**

- Stop skip-stamping Messages / Calendar / Finance / Notes / Shortcuts. Those modules are local, but they still have last-synced UI. Emit `ok` after a cheap re-read (or a dedicated “local reload” status the card understands).
- Pass `app_data_dir` into `refresh_home_module` so Blink stills download on ⌘⇧R / the quiet timer, not only when the card re-lists.
- Skip Health re-import when the export path’s mtime / size / hash matches the last successful ingest. Report `Health up to date` instead of `Imported N day(s)` for an unchanged zip. Delete `health_daily` days absent from the latest export (today only upserts).
- Count Home “online” from `status`, not `devices.len()` (includes Blink error placeholders).

**Health (still true)**

- Deduplicate steps **by source per day** (Watch + iPhone currently stack; `sourceName` is never read). Heart average blends every sample from every device.
- Sleep = `Asleep*` only, credited to the **wake** morning (`record_day` uses `startDate`, so an 11pm–7am night lands on yesterday and InBed + stages + Awake all add).

**Home (still true + new 4 Oct)**

- Persist Ring’s rotated `refresh_token` the same way Blink already does. Send a Ring `client_id` on the token POST. Read `battery_life` as string *or* number.
- Parse Ring `id` as string *or* number — `Value::to_string()` quotes string IDs (`ring-"123"`). Ring “alert” today means “`alerts` is any JSON object” (empty `{}` counts). Blink Snap is `device_type == "camera"` only. Blink OAuth pending state dies if the process restarts mid-login.

**Finance (still true + new 4 Oct)**

- `get_finance_summary` must honor the card’s `listLimit` (today always 10). Layout’s number input goes to 50; the card `slice`s a 10-row payload.
- Date picker must default to `localDayKey`, not `toISOString().slice(0, 10)`. After ~17:00 US Pacific the picker is tomorrow and the row misses `fillSpendDays`’ 14 local bars. SQL month/day filters must be local (`strftime('%Y-%m', 'now')` is UTC; spend-chart query is 30 UTC days vs 14 local bars). Txn `formatMoney` must use account currency. CSV parse failures must count. **Show** the Spent chip at $0 (today it hides, so a quiet month looks disconnected).
- Opening Manage must not re-run `get_finance_summary`; only fetch the extra `listTransactions(50)` when the drawer opens.

**News (still true from 2 Oct)**

- Seed defaults **once**, on first launch, not whenever `news_prefs` is empty. Deleting every feed is a clean slate, not a request to restore NPR.
- Kill the card-local 15-minute `refreshNews` timer. One dashboard interval, paused while `document.hidden`, shared with YouTube / Streaming / Email.
- News Prefs Add must fetch (YouTube Add already syncs). News / YouTube Delete must hide child rows. Hidden News must not run `run_refresh_news` at all.

**Messages**

- Group chats **before** the 500-row cap. Pulse and the card both call `listAllUnreadMessages` (SQL `LIMIT 500`, newest first) and group afterward, so a busy group thread hides every other unread conversation. Each also stages its own `chat.db` + WAL. `list_unread_messages` is unused.
- Decode `attributedBody` as UTF-16/UTF-8. `preview_text` prefers `m.text` (often NULL on modern iMessage). `extract_from_attributed_body` reads ASCII until NUL, so UTF-16LE `Hello` becomes `"H"` and a leading emoji becomes “Attachment or unavailable preview.” `apple_date_to_iso` then substitutes `Utc::now()` for a bad Cocoa timestamp, pinning that broken row as newest.

**Calendar**

- Filter ended events **before** `truncate(limit)`. Pulse `listCalendarEvents(24, 14, 0)` starts at midnight and keeps the earliest 24 starts; a busy morning prints **Nothing upcoming** while this afternoon is still in EventKit. Calendar Upcoming still shows finished meetings (`nextUpcomingEvent` already filters `end >= now`).
- All-day EventKit as a **local date**. `NSISO8601DateFormatter` emits GMT with no timeZone, so a Monday holiday paints Sunday and `calshow:` jumps there (`open_calendar_event` subtracts Apple epoch from a UTC `timestamp()`, which can skew vs Calendar.app). `eventCoversLocalDay` uses `dayStart + 24h` ms (DST leak/drop).
- Cap `request_full_access` (today `DISPATCH_TIME_FOREVER`). Stable fallback ids (missing `eventIdentifier` is a new UUID every fetch). Page EventKit from the **viewed month** (`daysBack`/`daysAhead` clamp to 90 from today). Drop `EKEventStatusCanceled`. Flip `cancelled` on hero Calendar / briefing cleanup. Locale weekday headers (hardcoded `Sun`–`Sat`). Hero Calendar TCC empty state. Share Pulse’s 21-day window with the section.

**Email / Mail**

- Do not auto-open in-card Connect. Run `start_email_oauth` off the command thread.
- **Do not open live `chat.db` from `load_known_contacts`.** Handles are phones vs email From, so the +20 almost never fires. Strip IMAP `BODY.PEEK[TEXT]` HTML soup. Scope Important mail (`list_important` is unscoped; Pulse is `listImportantEmails(20)` vs badge `all.length` (500) vs card `top`). Wipe `emails` / `mail_pieces` on disconnect; clear `mail_pieces` on upsert conflict (`ON CONFLICT` keeps `emails.id`).
- Stop marking leftover UNSEEN as read after IMAP `truncate(200)` or Mail.app `fetch_unread(200)`. Mail.app: parse To:/List-Unsubscribe, drop the 14-day cutoff, share the inbox fallback, stop `orphan_images.pop()`. Informed Delivery SEARCH is From-only and hardcodes `INBOX`.

**Weather**

- Cache-hit must match **lat + lon + units** (today latitude only). Show humidity / wind Open-Meteo already returns. Wind unit is hardcoded mph.

**UI honesty**

- Keep last-good data on refresh error. Catch still-thumb and `open*` failures (YouTube / Streaming / Mail / Internet Accounts). One drawer at a time (Layout + module drawers + ⌘K can all be `aria-modal` with no focus trap). Badge visible rows. Filter Streaming / YouTube lists to enabled prefs. Notes All uses **200** (backend max 500) — raise the drawer to 500, and do not let drawer **Edit** hijack the card composer. Briefing subtitle must not say “Nothing urgent” next to unread chips. Raise the stacked-hero breakpoint (860px never runs under `.app` `minWidth` 960). At ≥1180px, Left+Right leaves a hole in column 2.
- Mount `ModuleSkeleton` (already in `ModuleSection.tsx`) instead of a blank briefing / “Loading …” paragraphs.

---

## 2. Looks

- **Self-host Fraunces / Figtree.** CSP `font-src` is `'self'`; the Google Fonts `@import` also fails `connect-src`, so production falls back to Iowan / Avenir and the display wordmark looks thinner than design.
- Overlay title bar + window vibrancy. Capabilities stay `core:default` (no `core:window` for a hidden title bar yet).
- Kind-colored 3px accents on briefing chips (same language as `.module::before`). Deepen dusk `.clock-face` glow so clock / weather / calendar share `--hero-card-*`.

---

## 3. Coolness

- YouTube posters from `videoId` (`i.ytimg.com` is **not** in `img-src` yet — only TMDB is). Resolve `@handles` to `UC…` channel IDs. `video_id_from_url` misses `youtu.be/` and Atom `yt:video:` ids, then stores the whole URL as `video_id`.
- Menu-bar pulse: next event, unread **chat** count (not the 500-message cap), important mail.
- Catch-up Refresh on mount (`refreshDashboard` is never called until ⌘⇧R / the timer). One staged `chat.db` shared by Pulse + Messages.

---

## 4. Later

- EventKit Reminders sync **only after** Tasks is on master.
- In-app Messages reply.
- Signing, notarization, and auto-update.

---

## Do not re-do

Weather module, ⌘K skeleton, dusk tokens, health sparklines, Streaming Tonight (the card), today briefing, digital clock, hero agenda, finance spend chart / extra CSV formats, email browser sign-in, Blink OAuth stills, Mail.app AppleScript timeouts, or Tasks.
