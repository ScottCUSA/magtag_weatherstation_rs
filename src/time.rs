use time::{Date, Month, Weekday};

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

pub fn get_iso_8601_hh_mm(iso: &str) -> Option<&str> {
    if iso.len() < 16 || !iso.as_bytes()[10..11].eq(b"T") {
        return None;
    }
    iso.get(11..16)
}
