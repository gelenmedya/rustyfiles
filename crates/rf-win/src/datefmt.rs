//! Dates and times written the way the user's Windows writes them: the short date and the short time of the regional format
//! (Settings > Time & language > Language & region > Regional format). Explorer's columns use the same two (issue 59: "date format
//! with Windows Explorer is 11/17/2026 9:30AM ... and 11.17.2026 09:30 with RustyFiles").

use chrono::{DateTime, Datelike, Local, Timelike};
use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::Globalization::{DATE_SHORTDATE, GetDateFormatEx, GetTimeFormatEx, TIME_NOSECONDS};
use windows::core::PCWSTR;

/// "11/17/2026 9:30 AM" (or whatever the regional format says); `None` when Windows cannot write it.
pub fn short_date_time(dt: &DateTime<Local>) -> Option<String> {
    let st = system_time(dt);
    let mut d = [0u16; 80];
    let mut t = [0u16; 80];
    // (a null locale name is the user's default locale)
    let dn = unsafe { GetDateFormatEx(PCWSTR::null(), DATE_SHORTDATE, Some(&st), PCWSTR::null(), Some(&mut d), PCWSTR::null()) };
    let tn = unsafe { GetTimeFormatEx(PCWSTR::null(), TIME_NOSECONDS, Some(&st), PCWSTR::null(), Some(&mut t)) };
    if dn <= 1 || tn <= 1 {
        return None;
    }
    let date = String::from_utf16_lossy(&d[..dn as usize - 1]);
    let time = String::from_utf16_lossy(&t[..tn as usize - 1]);
    Some(format!("{date} {time}"))
}

/// "11/17/2026" (or whatever the regional format says): the short date alone, for a day heading or a date with no time after it.
pub fn short_date(dt: &DateTime<Local>) -> Option<String> {
    let st = system_time(dt);
    let mut d = [0u16; 80];
    let dn = unsafe { GetDateFormatEx(PCWSTR::null(), DATE_SHORTDATE, Some(&st), PCWSTR::null(), Some(&mut d), PCWSTR::null()) };
    if dn <= 1 {
        return None;
    }
    Some(String::from_utf16_lossy(&d[..dn as usize - 1]))
}

/// The SYSTEMTIME of a local date, as both writers above want it.
fn system_time(dt: &DateTime<Local>) -> SYSTEMTIME {
    SYSTEMTIME {
        wYear: dt.year() as u16,
        wMonth: dt.month() as u16,
        wDayOfWeek: dt.weekday().num_days_from_sunday() as u16,
        wDay: dt.day() as u16,
        wHour: dt.hour() as u16,
        wMinute: dt.minute() as u16,
        wSecond: 0,
        wMilliseconds: 0,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    #[test]
    fn windows_writes_a_date_with_its_year_and_minutes() {
        let dt = chrono::Local.with_ymd_and_hms(2026, 11, 17, 9, 30, 0).unwrap();
        let s = super::short_date_time(&dt).expect("Windows writes a short date and time");
        assert!(s.contains("2026") || s.contains("26"), "the year is there: {s}");
        assert!(s.contains("30"), "the minutes are there: {s}");
    }

    /// The date alone is the date the full text begins with, and it holds no time (a heading, a "modified on" line).
    #[test]
    fn windows_writes_a_date_with_no_time() {
        let dt = chrono::Local.with_ymd_and_hms(2026, 11, 17, 9, 30, 0).unwrap();
        let day = super::short_date(&dt).expect("Windows writes a short date");
        assert!(day.contains("2026") || day.contains("26"), "the year is there: {day}");
        assert!(!day.contains('9') || !day.contains(':'), "no time in it: {day}");
        let full = super::short_date_time(&dt).expect("Windows writes a short date and time");
        assert!(full.starts_with(&day), "the full text begins with the day: {full:?} vs {day:?}");
    }
}
