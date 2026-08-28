#import <EventKit/EventKit.h>
#import <Foundation/Foundation.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

enum {
    MAINSTREAM_CAL_OK = 0,
    MAINSTREAM_CAL_NEEDS_PERMISSION = 1,
    MAINSTREAM_CAL_ERROR = 2,
};

static char *copy_utf8(NSString *string) {
    if (string == nil) {
        return NULL;
    }
    const char *utf8 = [string UTF8String];
    if (utf8 == NULL) {
        return NULL;
    }
    return strdup(utf8);
}

static BOOL status_is_full_access(EKAuthorizationStatus status) {
    return status == EKAuthorizationStatusFullAccess;
}

static BOOL request_full_access(EKEventStore *store, NSError **outError) {
    EKAuthorizationStatus status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
    if (status_is_full_access(status)) {
        return YES;
    }
    if (status == EKAuthorizationStatusDenied || status == EKAuthorizationStatusRestricted) {
        return NO;
    }

    dispatch_semaphore_t sem = dispatch_semaphore_create(0);
    __block BOOL granted = NO;
    __block NSError *requestError = nil;

    if (@available(macOS 14.0, *)) {
        [store requestFullAccessToEventsWithCompletion:^(BOOL ok, NSError *error) {
            granted = ok;
            requestError = error;
            dispatch_semaphore_signal(sem);
        }];
    } else {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
        [store requestAccessToEntityType:EKEntityTypeEvent
                              completion:^(BOOL ok, NSError *error) {
                                  granted = ok;
                                  requestError = error;
                                  dispatch_semaphore_signal(sem);
                              }];
#pragma clang diagnostic pop
    }

    dispatch_semaphore_wait(sem, DISPATCH_TIME_FOREVER);
    if (outError != NULL) {
        *outError = requestError;
    }
    return granted && status_is_full_access(
        [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent]
    );
}

int mainstream_calendar_events(int64_t days_ahead, char **json_out, char **error_out) {
    @autoreleasepool {
        if (json_out != NULL) {
            *json_out = NULL;
        }
        if (error_out != NULL) {
            *error_out = NULL;
        }

        EKEventStore *store = [[EKEventStore alloc] init];
        NSError *accessError = nil;
        if (!request_full_access(store, &accessError)) {
            if (error_out != NULL) {
                NSString *message = accessError.localizedDescription ?: @"Calendar access denied";
                *error_out = copy_utf8(message);
            }
            return MAINSTREAM_CAL_NEEDS_PERMISSION;
        }

        NSCalendar *cal = [NSCalendar currentCalendar];
        NSDate *now = [NSDate date];
        NSDate *start = [cal startOfDayForDate:now];
        NSDate *end = [cal dateByAddingUnit:NSCalendarUnitDay
                                      value:(NSInteger)days_ahead + 1
                                     toDate:start
                                    options:0];
        if (end == nil) {
            end = now;
        }

        NSPredicate *predicate = [store predicateForEventsWithStartDate:start
                                                                endDate:end
                                                              calendars:nil];
        NSArray<EKEvent *> *events = [[store eventsMatchingPredicate:predicate]
            sortedArrayUsingComparator:^NSComparisonResult(EKEvent *a, EKEvent *b) {
                return [a.startDate compare:b.startDate];
            }];

        NSISO8601DateFormatter *formatter = [[NSISO8601DateFormatter alloc] init];
        formatter.formatOptions =
            NSISO8601DateFormatWithInternetDateTime | NSISO8601DateFormatWithFractionalSeconds;

        NSMutableArray *payload = [NSMutableArray arrayWithCapacity:events.count];
        for (EKEvent *event in events) {
            NSString *title = [event.title stringByTrimmingCharactersInSet:
                [NSCharacterSet whitespaceAndNewlineCharacterSet]];
            if (title.length == 0) {
                title = @"(No title)";
            }

            id location = [NSNull null];
            NSString *rawLocation = [event.location stringByTrimmingCharactersInSet:
                [NSCharacterSet whitespaceAndNewlineCharacterSet]];
            if (rawLocation.length > 0) {
                location = rawLocation;
            }

            id calendarName = [NSNull null];
            if (event.calendar.title.length > 0) {
                calendarName = event.calendar.title;
            }

            [payload addObject:@{
                @"id": event.eventIdentifier ?: [[NSUUID UUID] UUIDString],
                @"title": title,
                @"start": [formatter stringFromDate:event.startDate] ?: @"",
                @"end": [formatter stringFromDate:event.endDate] ?: @"",
                @"isAllDay": @(event.allDay),
                @"location": location,
                @"calendarName": calendarName,
            }];
        }

        NSError *jsonError = nil;
        NSData *data = [NSJSONSerialization dataWithJSONObject:payload
                                                       options:0
                                                         error:&jsonError];
        if (data == nil) {
            if (error_out != NULL) {
                NSString *message = jsonError.localizedDescription ?: @"Failed to encode calendar events";
                *error_out = copy_utf8(message);
            }
            return MAINSTREAM_CAL_ERROR;
        }

        if (json_out != NULL) {
            NSString *json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
            *json_out = copy_utf8(json);
        }
        return MAINSTREAM_CAL_OK;
    }
}

void mainstream_calendar_string_free(char *s) {
    free(s);
}
