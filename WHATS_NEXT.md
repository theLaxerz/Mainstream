# What's next

Scan of master on **2026-10-03**. Tip is still `476a28b` (*Stop Mail.app AppleScript from freezing the UI when adding Outlook.*, 29 Aug) — **thirty-fourth** planning day on the same commit.

Newest security PR is **#80**. Settings allowlist is still the product blocker — get *and* set deny everything except `dashboard.layout.v1`. Do not allow-list weather/health dedicated keys.

**New this scan:** Health re-imports the zip on every Refresh All. `refresh_health_module` always calls `try_import_configured`, which opens the saved `export.zip` / `export.xml` and re-parses every `Record` with **no mtime, size, or hash gate**. A typical Apple Health archive is huge; ⌘⇧R and the 15-minute timer both re-sum Watch + iPhone steps, re-stack InBed/Asleep/Awake onto `startDate`, and rewrite every `health_daily` row. The sync badge then says `Imported N day(s)` even when the file has not changed. The parser still never reads `sourceName`, so the expensive pass is the same 2×-steps bug as before.

Same Refresh All path: Home is called as `fetch_home_devices(state, None)`. Blink only writes camera stills when `data_dir` is `Some`, so the orchestrator never updates `blink-thumbs`. The card then re-lists with an app-data dir (stills arrive late) and the badge still reports `devices.len()` as “online”, including the Blink error placeholder. Finance `get_finance_summary` hardcodes `list_recent_transactions(..., 10)` — Layout items above 10 is a no-op on the card (`slice(0, limit)` of a 10-row payload). `ModuleSkeleton` is already styled and never mounted; the briefing returns `null` until pulse resolves, so first paint has a hole where Today should be.

Supersedes #79 / #77 / #75 / #73 / #71 / #69 / #67 / #65 / #64 / #62 / #60 / #58 / #56 / #54 / #52 / #50 / #48 / #46 / #44 / #42 / #40 / #38 / #36 / #34 / #32 / #30 / #28 / #26 / #24 / #23 / #21 / #19 / #17. Does not re-implement Tasks (#15) or the security set (#80).

Sixty-seven open drafts before this increment.

---

## Merge hygiene (do first)

1. Land **PR #15** (Tasks) — do not re-implement.
2. Land **PR #80** (2 Oct IPC harden) or close it in favor of the next security scan; close older security drafts including #78.
3. Close duplicate planning PRs **#14, #16–#79** once this plan (or an implementation of it) lands.

Master has not moved since 29 Aug. Shipping any one of the items below beats another planning rewrite.

---

## 0. Unlock settings (blocks theme, refresh, onboarding)

`is_generic_setting_key` only allows `dashboard.layout.v1`. Both get and set are gated, so `appearance.theme` and `refresh.intervalMinutes` never persist. `saveThemePreference` / `loadThemePreference` swallow the deny. Allow-list only:

- `appearance.theme`
- `refresh.intervalMinutes`
- `onboarding.completed`

Do not allow-list weather/health dedicated keys (those have their own commands). Drop unused `list_settings`. Even after a saver exists, the refresh timer must live in React state so it rebinds. Boot a default `data-theme` on `<html>` before React so dusk does not flash light (`index.html` has none; first paint hits EventKit three times with no `refreshDashboard`).

---

## 1. First-run that does not lie

Wait for `layoutReady` **before mounting cards**. Gate Layout / ⌘, on `layoutReady`. Serialize Layout persist. Ship a starter layout + a short onboarding overlay (Calendar TCC, Full Disk Access, pin a city, connect email). Honor enabled modules in Pulse, ⌘K “Go to”, and Refresh All.

**Health (new 3 Oct)**

- Skip re-import when the export path’s mtime / size / hash matches the last successful ingest. Refresh All should report `Health up to date` instead of `Imported N day(s)` for a file that has not changed.
- Deduplicate steps **by source per day** (Watch + iPhone currently stack; `sourceName` is never read). Heart average blends every sample from every device.
- Sleep = `Asleep*` only, credited to the **wake** morning (`record_day` uses `startDate`, so an 11pm–7am night lands on yesterday and InBed + stages + Awake all add).

**Home / Refresh All (new 3 Oct)**

- Pass `app_data_dir` into `refresh_home_module` so Blink stills download on ⌘⇧R / the quiet timer, not only when the card re-lists.
- Persist Ring’s rotated `refresh_token` the same way Blink already does. Send a Ring `client_id` on the token POST. Read `battery_life` as string *or* number. Count “online” from `status`, not `devices.len()`. Ring “alert” today means “`alerts` is any JSON object” (empty `{}` counts); Blink Snap is `device_type == "camera"` only.

**Finance (new 3 Oct + still true)**

- `get_finance_summary` must honor the card’s `listLimit` (today always 10). Layout’s number input goes to 50; the card `slice`s a 10-row payload.
- Date picker must default to `localDayKey`, not `toISOString().slice(0, 10)`. After ~17:00 US Pacific the picker is tomorrow and the row misses `fillSpendDays`’ 14 local bars. SQL month/day filters must be local (`strftime('%Y-%m', 'now')` is UTC; spend-chart query is 30 UTC days vs 14 local bars). Txn `formatMoney` must use account currency. CSV parse failures must count. Hide the Spent chip at $0.

**News (still true from 2 Oct)**

- Seed defaults **once**, on first launch, not whenever `news_prefs` is empty. Deleting every feed is a clean slate, not a request to restore NPR.
- Kill the card-local 15-minute `refreshNews` timer. One dashboard interval, paused while `document.hidden`, shared with YouTube / Streaming / Email.
- News Prefs Add must fetch (YouTube Add already syncs). News / YouTube Delete must hide child rows. Hidden News must not run `run_refresh_news` at all.

**Messages**

- Group chats **before** the 500-row cap. Pulse and the card both call `listAllUnreadMessages` (SQL `LIMIT 500`, newest first) and group afterward, so a busy group thread hides every other unread conversation. Each also stages its own `chat.db` + WAL. `list_unread_messages` is unused.
- Decode `attributedBody` as UTF-16/UTF-8. `preview_text` prefers `m.text` (often NULL on modern iMessage). `extract_from_attributed_body` reads ASCII until NUL, so UTF-16LE `Hello` becomes `"H"` and a leading emoji becomes “Attachment or unavailable preview.” `apple_date_to_iso` then substitutes `Utc::now()` for a bad Cocoa timestamp, pinning that broken row as newest.

**Calendar**

- Filter ended events **before** `truncate(limit)`. Pulse `listCalendarEvents(24, 14, 0)` starts at midnight and keeps the earliest 24 starts; a busy morning prints **Nothing upcoming** while this afternoon is still in EventKit. Calendar Upcoming still shows finished meetings (`nextUpcomingEvent` already filters `end >= now`).
- All-day EventKit as a **local date**. `NSISO8601DateFormatter` emits GMT with no timeZone, so a Monday holiday paints Sunday and `calshow:` jumps there. `eventCoversLocalDay` uses `dayStart + 24h` ms (DST leak/drop).
- Cap `request_full_access` (today `DISPATCH_TIME_FOREVER`). Stable fallback ids (missing `eventIdentifier` is a new UUID every fetch). Page EventKit from the **viewed month** (`daysBack`/`daysAhead` clamp to 90 from today). Drop `EKEventStatusCanceled`. Flip `cancelled` on hero Calendar / briefing cleanup. Roll hero `today` at midnight (`useMemo([], …)` freezes launch day). Locale weekday headers (hardcoded `Sun`–`Sat`). Hero Calendar TCC empty state. Share Pulse’s 21-day window with the section.

**Email / Mail**

- Do not auto-open in-card Connect. Run `start_email_oauth` off the command thread.
- **Do not open live `chat.db` from `load_known_contacts`.** Handles are phones vs email From, so the +20 almost never fires. Strip IMAP `BODY.PEEK[TEXT]` HTML soup. Scope Important mail (`list_important` is unscoped; Pulse is `listImportantEmails(20)` vs badge `all.length` (500) vs card `top`). Wipe `emails` / `mail_pieces` on disconnect; clear `mail_pieces` on upsert conflict (`ON CONFLICT` keeps `emails.id`).
- Stop marking leftover UNSEEN as read after IMAP `truncate(200)` or Mail.app `fetch_unread(200)`. Mail.app: parse To:/List-Unsubscribe, drop the 14-day cutoff, share the inbox fallback, stop `orphan_images.pop()`. Informed Delivery SEARCH is From-only and hardcodes `INBOX`.

**Weather**

- Cache-hit must match **lat + lon + units** (today latitude only). Show humidity / wind Open-Meteo already returns. Wind unit is hardcoded mph.

**UI honesty**

- Keep last-good data on refresh error. Catch still-thumb and `open*` failures (YouTube / Streaming / Mail / Internet Accounts). One drawer at a time. Badge visible rows. Filter Streaming / YouTube lists to enabled prefs. Notes All uses 200 (backend max 500). Briefing subtitle must not say “Nothing urgent” next to unread chips. Share one locale clock and one hour table for greeting + Auto-dusk. Raise the stacked-hero breakpoint (860px never runs under `.app` `minWidth` 960). At ≥1180px, Left+Right leaves a hole in column 2.
- Mount `ModuleSkeleton` (already in `ModuleSection.tsx`) instead of a blank briefing / “Loading …” paragraphs.

---

## 2. Looks

- **Self-host Fraunces / Figtree.** CSP `font-src` is `'self'`; the Google Fonts `@import` also fails `connect-src`, so production falls back to Iowan / Avenir and the display wordmark looks thinner than design.
- Overlay title bar + window vibrancy. Capabilities stay `core:default` (no `core:window` for a hidden title bar yet).

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
