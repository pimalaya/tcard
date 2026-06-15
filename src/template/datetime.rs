//! vCard date conversions between calcard's [`PartialDateTime`], native TOML
//! date-times, and RFC 6350 basic ISO 8601 strings.
//!
//! `BDAY`/`ANNIVERSARY` project as a native TOML `date`/`datetime` when the
//! value is complete; a partial value (yearless `--0415`, year only) has no
//! native TOML form and falls back to a quoted RFC 6350 string.

use alloc::{format, string::String};

use calcard::common::PartialDateTime;
use toml_edit::{Date, Datetime, Offset, Time};

/// Build a native TOML value from a vCard date-time, or `None` when it is
/// partial (yearless or year only) and so has no native TOML form. An
/// all-day value becomes a local date, a UTC value an offset date-time,
/// anything else a local date-time.
pub fn toml_date(dt: &PartialDateTime) -> Option<Datetime> {
    let date = Date {
        year: dt.year?,
        month: dt.month?,
        day: dt.day?,
    };

    let Some((hour, minute)) = dt.hour.zip(dt.minute) else {
        return Some(Datetime {
            date: Some(date),
            time: None,
            offset: None,
        });
    };

    let time = Time {
        hour,
        minute,
        second: Some(dt.second.unwrap_or(0)),
        nanosecond: None,
    };
    let utc = matches!((dt.tz_hour, dt.tz_minute), (Some(0), Some(0)));

    Some(Datetime {
        date: Some(date),
        time: Some(time),
        offset: utc.then_some(Offset::Z),
    })
}

/// Build a vCard content line from a native TOML date-time, in RFC 6350
/// basic ISO 8601 form (`19960415`, with a `T..` time and trailing `Z` for
/// UTC).
pub fn toml_date_line(name: &str, dtm: &Datetime) -> String {
    let Some(date) = dtm.date else {
        return format!("{name}:{dtm}");
    };
    let mut value = format!("{:04}{:02}{:02}", date.year, date.month, date.day);

    if let Some(time) = dtm.time {
        value.push_str(&format!(
            "T{:02}{:02}{:02}",
            time.hour,
            time.minute,
            time.second.unwrap_or(0)
        ));
        if matches!(dtm.offset, Some(Offset::Z)) {
            value.push('Z');
        }
    }

    format!("{name}:{value}")
}

/// Render a vCard date-time as its RFC 6350 basic ISO 8601 string, covering
/// the partial forms TOML cannot hold natively (`--0415` for a yearless
/// birthday, `2009` for a year only), with a `T..` time when present.
pub fn vcard_date(dt: &PartialDateTime) -> String {
    let mut out = String::new();

    match (dt.year, dt.month, dt.day) {
        (Some(y), Some(m), Some(d)) => out.push_str(&format!("{y:04}{m:02}{d:02}")),
        (Some(y), Some(m), None) => out.push_str(&format!("{y:04}-{m:02}")),
        (Some(y), None, None) => out.push_str(&format!("{y:04}")),
        (None, Some(m), Some(d)) => out.push_str(&format!("--{m:02}{d:02}")),
        (None, Some(m), None) => out.push_str(&format!("--{m:02}")),
        (None, None, Some(d)) => out.push_str(&format!("---{d:02}")),
        _ => {}
    }

    if let Some(hour) = dt.hour {
        out.push_str(&format!("T{hour:02}"));
        if let Some(minute) = dt.minute {
            out.push_str(&format!("{minute:02}"));
            if let Some(second) = dt.second {
                out.push_str(&format!("{second:02}"));
            }
        }
    }

    out
}
