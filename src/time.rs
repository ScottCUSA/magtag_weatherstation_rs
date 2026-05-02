use time::{Date, Month, OffsetDateTime, Weekday};

/// Formats an ISO 8601 date string (`YYYY-MM-DD`) into a human-readable date.
///
/// The output looks like `"Monday January 1st, 2020"`. Returns `None` if `iso`
/// is not a valid `YYYY-MM-DD` date string.
pub fn format_date(iso: &str) -> Option<heapless::String<64>> {
    let year: i32 = iso.get(0..4)?.parse().ok()?;
    let month: u8 = iso.get(5..7)?.parse().ok()?;
    let day: u8 = iso.get(8..10)?.parse().ok()?;

    let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;

    let mut out = heapless::String::<64>::new();

    let weekday = match date.weekday() {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    };

    let month_name = match date.month() {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    };

    let _ = core::fmt::write(
        &mut out,
        format_args!(
            "{} {} {}{}, {}",
            weekday,
            month_name,
            day,
            ordinal(day),
            year
        ),
    );

    Some(out)
}

/// Extracts the `HH:MM` portion from an ISO 8601 datetime string (e.g. `"2020-01-01T14:15"`).
///
/// Returns `None` if the string is shorter than 16 characters or has no `T` separator at
/// index 10.
pub fn iso_8601_hh_mm(iso: &str) -> Option<&str> {
    if iso.len() < 16 || !iso.as_bytes()[10..11].eq(b"T") {
        return None;
    }
    iso.get(11..16)
}

/// Returns the abbreviated weekday name (`"SUN"`..`"SAT"`) for the given date using
/// Sakamoto's algorithm.
///
/// Returns `None` if `month` is not in `1..=12` or `day` is not in `1..=31`.
pub fn short_day_of_week_sakamoto(year: i32, month: i32, day: i32) -> Option<&'static str> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let mut y = year;
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if month < 3 {
        y -= 1;
    }
    let dow = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day) % 7;
    Some(match dow {
        0 => "SUN",
        1 => "MON",
        2 => "TUE",
        3 => "WED",
        4 => "THU",
        5 => "FRI",
        _ => "SAT",
    })
}

/// Formats a Unix timestamp as a human-readable date, adjusted for a UTC offset.
///
/// `utc_offset_seconds` is added to `ts` before formatting. The output looks like
/// `"Monday January 1st, 2020"`. Returns `None` if the timestamp is out of range.
pub fn format_date_unix(ts: i64, utc_offset_seconds: i32) -> Option<heapless::String<64>> {
    let local_ts = ts + utc_offset_seconds as i64;
    let dt = OffsetDateTime::from_unix_timestamp(local_ts).ok()?;

    let weekday = match dt.weekday() {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    };

    let month_name = match dt.month() {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    };

    let day = dt.day();
    let year = dt.year();
    let mut out = heapless::String::<64>::new();
    let _ = core::fmt::write(
        &mut out,
        format_args!(
            "{} {} {}{}, {}",
            weekday,
            month_name,
            day,
            ordinal(day),
            year
        ),
    );
    Some(out)
}

/// Formats a Unix timestamp as `HH:MM` in local time.
///
/// `utc_offset_seconds` is added to `ts` before formatting. Returns `None` if the
/// timestamp is out of range.
pub fn unix_hh_mm(ts: i64, utc_offset_seconds: i32) -> Option<heapless::String<6>> {
    let local_ts = ts + utc_offset_seconds as i64;
    let dt = OffsetDateTime::from_unix_timestamp(local_ts).ok()?;
    let mut out = heapless::String::<6>::new();
    let _ = core::fmt::write(
        &mut out,
        format_args!("{:02}:{:02}", dt.hour(), dt.minute()),
    );
    Some(out)
}

/// Returns the number of seconds until the next 6:00 AM in local time.
///
/// `utc_offset` is added to `current_time` to obtain local time. Targets today's
/// 6 AM if before it, otherwise tomorrow's.
pub fn secs_until_6am(current_time: i64, utc_offset: i32) -> u64 {
    let local_ts = current_time + utc_offset as i64;
    let secs_past_midnight = local_ts.rem_euclid(86400);
    let six_am = 6 * 3600_i64;
    if secs_past_midnight < six_am {
        (six_am - secs_past_midnight) as u64
    } else {
        (86400 + six_am - secs_past_midnight) as u64
    }
}

/// Returns the abbreviated weekday name (`"SUN"`..`"SAT"`) for a Unix timestamp.
///
/// `utc_offset_seconds` is added to `ts` to obtain local time. Returns `None` if
/// the timestamp is out of range.
pub fn short_dow_unix(ts: i64, utc_offset_seconds: i32) -> Option<&'static str> {
    let local_ts = ts + utc_offset_seconds as i64;
    let dt = OffsetDateTime::from_unix_timestamp(local_ts).ok()?;
    Some(match dt.weekday() {
        Weekday::Sunday => "SUN",
        Weekday::Monday => "MON",
        Weekday::Tuesday => "TUE",
        Weekday::Wednesday => "WED",
        Weekday::Thursday => "THU",
        Weekday::Friday => "FRI",
        Weekday::Saturday => "SAT",
    })
}

fn ordinal(n: u8) -> &'static str {
    match n {
        11..=13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    }
}
