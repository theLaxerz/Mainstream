import EventKit
import Foundation

struct CalendarEventOut: Codable {
    let id: String
    let title: String
    let start: String
    let end: String
    let isAllDay: Bool
    let location: String?
    let calendarName: String?
}

struct CalendarErrorOut: Codable {
    let error: String
    let status: String
}

func printError(_ message: String, status: String = "error") -> Never {
    let payload = CalendarErrorOut(error: message, status: status)
    let data = try! JSONEncoder().encode(payload)
    print(String(data: data, encoding: .utf8)!)
    exit(1)
}

let daysBack = Int(CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "0") ?? 0
let daysAhead = Int(CommandLine.arguments.count > 2 ? CommandLine.arguments[2] : "14") ?? 14

let store = EKEventStore()
let semaphore = DispatchSemaphore(value: 0)
var granted = false
var accessError: String?

if #available(macOS 14.0, *) {
    store.requestFullAccessToEvents { ok, err in
        granted = ok
        if let err {
            accessError = err.localizedDescription
        }
        semaphore.signal()
    }
} else {
    store.requestAccess(to: .event) { ok, err in
        granted = ok
        if let err {
            accessError = err.localizedDescription
        }
        semaphore.signal()
    }
}
semaphore.wait()

if !granted {
    printError(accessError ?? "Calendar access denied", status: "needs_permission")
}

let cal = Calendar.current
let now = Date()
let start = cal.startOfDay(
    for: cal.date(byAdding: .day, value: -daysBack, to: now) ?? now
)
let end = cal.date(byAdding: .day, value: daysAhead + 1, to: cal.startOfDay(for: now)) ?? now

let predicate = store.predicateForEvents(withStart: start, end: end, calendars: nil)
let events = store.events(matching: predicate).sorted { $0.startDate < $1.startDate }

let formatter = ISO8601DateFormatter()
formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]

let out: [CalendarEventOut] = events.map { event in
    CalendarEventOut(
        id: event.eventIdentifier ?? UUID().uuidString,
        title: event.title?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false
            ? event.title!
            : "(No title)",
        start: formatter.string(from: event.startDate),
        end: formatter.string(from: event.endDate),
        isAllDay: event.isAllDay,
        location: event.location?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false
            ? event.location
            : nil,
        calendarName: event.calendar?.title
    )
}

let data = try JSONEncoder().encode(out)
print(String(data: data, encoding: .utf8)!)
