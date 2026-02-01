use time::{Date, Month, Weekday};

/// Format an ISO 8601 date string (`YYYY-MM-DD`) into a readable form.
///
/// Returns `Some` like "Monday January 1st, 2020" on success or `None` for invalid input.
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

/// Extract `HH:MM` from an ISO 8601 datetime (`...T14:15`) if present.
///
/// Returns `Some("HH:MM")` or `None` for malformed or too-short strings.
pub fn iso_8601_hh_mm(iso: &str) -> Option<&str> {
    if iso.len() < 16 || !iso.as_bytes()[10..11].eq(b"T") {
        return None;
    }
    iso.get(11..16)
}

/// Return a short uppercase weekday (`SUN`..`SAT`) using Sakamoto's algorithm.
///
/// Inputs are `year`, `month` (1-12) and `day` (1-31).
/// Returns `Some("SUN".."SAT")` for valid inputs or `None` for invalid month/day.
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
