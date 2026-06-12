use time::{Month, OffsetDateTime, Weekday};

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
