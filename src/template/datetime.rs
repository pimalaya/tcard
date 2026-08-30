//! # Dates
//!
//! Conversions between a card's RFC 6350 basic ISO 8601 dates and the native
//! TOML `date` / `datetime` the projection writes them as.
//!
//! `BDAY` and `ANNIVERSARY` project as a native TOML value when the value is
//! complete; a partial one (yearless `--0415`, year only) has no native TOML
//! form and falls back to a quoted string as the card wrote it.

use alloc::{
    format,
    string::{String, ToString},
};

use toml_edit::{Date, Datetime, Offset, Time};

use crate::template::toml::toml_str;

/// A date as the projection writes one.
///
/// Native where the value is complete, else the quoted string the card wrote.
pub fn date_rhs(value: &str) -> String {
    match toml_datetime(value) {
        Some(native) => native.to_string(),
        None => toml_str(value),
    }
}

/// Read an RFC 6350 basic ISO 8601 date-time into a native TOML value.
///
/// `None` where it carries no complete date and so has no native form. The
/// extended form a real card sometimes writes (`1996-04-15`) is read too, and
/// a non-UTC offset is dropped rather than refused.
pub fn toml_datetime(value: &str) -> Option<Datetime> {
    let (date, time) = match value.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (value, None),
    };

    let date = date.replace('-', "");

    if date.len() != 8 {
        return None;
    }

    let date = Date {
        year: date[..4].parse().ok()?,
        month: date[4..6].parse().ok()?,
        day: date[6..].parse().ok()?,
    };

    let Some(time) = time else {
        return Some(Datetime {
            date: Some(date),
            time: None,
            offset: None,
        });
    };

    let utc = time.ends_with('Z');
    let time = time.trim_end_matches('Z');
    let time = match time.find(['+', '-']) {
        Some(at) => &time[..at],
        None => time,
    };

    if !matches!(time.len(), 4 | 6) {
        return None;
    }

    let time = Time {
        hour: time[..2].parse().ok()?,
        minute: time[2..4].parse().ok()?,
        second: Some(time.get(4..6).map_or(Ok(0), str::parse).ok()?),
        nanosecond: None,
    };

    Some(Datetime {
        date: Some(date),
        time: Some(time),
        offset: utc.then_some(Offset::Z),
    })
}

/// Build a vCard value from a native TOML date-time.
///
/// RFC 6350 basic ISO 8601 form: `19960415`, with a `T..` time and a trailing
/// `Z` for UTC.
pub fn toml_date_value(dtm: &Datetime) -> String {
    let Some(date) = dtm.date else {
        return dtm.to_string();
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

    value
}
