/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! `DATE` and `TIME`: the calendar, ported from `RexxDateTime`
//! (`classes/support/RexxDateTime.cpp`) rather than from either builtin's
//! documentation, because the two option lists documentation gives disagree
//! with the error inserts the oracle actually raises -- `DATE('C')` and
//! `DATE('J')` exist in other Rexx dialects and are rejected with 40.904 on
//! this build, measured.
//! ```text
//! first time('R')            0
//! time('E') after ~0.11s     0.109014
//! time('R') after another    0.219483   = the sum of BOTH burns
//! time('E') immediately      0.000023   so that R did reset
//! ```

use chrono::{Offset, TimeZone};

use rexx_core::ObjRef;
use rexx_num::Number;

use super::{Args, optional_string};
use crate::Interp;
use crate::error::{Failure, Raised};

// ---- the calendar, ported from `RexxDateTime` ----

/// Days in a `years`-year span of the proleptic Gregorian calendar, counting
/// from year 0 -- `RexxDateTime.hpp`'s `BASE_DAYS` macro. `years` is always a
/// multiple of 4, 100 or 400 at every call site below, but the formula does
/// not require that.
const fn base_days(years: i64) -> i64 {
    years * 365 + years / 4 - years / 100 + years / 400
}

const OLYMPIAD_DAYS: i64 = base_days(400);
const CENTURY_DAYS: i64 = base_days(100);
const LEAP_DAYS: i64 = base_days(4);

const SECONDS_IN_DAY: i64 = 86_400;
const MICROSECONDS: i64 = 1_000_000;
pub(crate) const MICROSECONDS_IN_DAY: i64 = SECONDS_IN_DAY * MICROSECONDS;

/// The largest basedate `DATE`/`TIME` accept: the days from 0001-01-01 to
/// 9999-12-31, both dates this crate can otherwise render. Measured against
/// the oracle -- `date('B', '31 Dec 9999', 'N')` -- rather than derived,
/// because it is the one constant [`Timestamp::set_base_date`] needs before
/// it has computed a single field to derive it *from*.
const MAX_BASE_DATE: i64 = 3_652_058;
/// The basetime at the last representable instant, `9999-12-31T23:59:59.999999`.
/// Computed rather than measured: [`MAX_BASE_DATE`] already fixes the day,
/// and the time-of-day component is the same arithmetic
/// [`Timestamp::base_time`] runs everywhere else.
const MAX_BASE_TIME: i64 =
    MAX_BASE_DATE * MICROSECONDS_IN_DAY + (23 * 3600 + 59 * 60 + 59) * MICROSECONDS + 999_999;
/// Days from 0001-01-01 to the Unix epoch, 1970-01-01. Measured against the
/// oracle (`date('B', '1 Jan 1970', 'N')`) and confirmed by [`base_days`]:
/// `base_days(1969) + 1 - 1 == 719_162`, the well-known .NET epoch constant,
/// since both systems count from the identical proleptic 0001-01-01.
const UNIX_BASE_DATE: i64 = 719_162;
pub(crate) const UNIX_BASE_TIME: i64 = UNIX_BASE_DATE * MICROSECONDS_IN_DAY;

const MONTH_NAMES: [&[u8]; 12] = [
    b"January",
    b"February",
    b"March",
    b"April",
    b"May",
    b"June",
    b"July",
    b"August",
    b"September",
    b"October",
    b"November",
    b"December",
];

/// `RexxDateTime::dayNames`, Monday first -- `getWeekDay`'s own `%7` is
/// indexed against this order, not against a locale's own week start.
const DAY_NAMES: [&[u8]; 7] = [
    b"Monday",
    b"Tuesday",
    b"Wednesday",
    b"Thursday",
    b"Friday",
    b"Saturday",
    b"Sunday",
];

const MONTH_STARTS: [i64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const LEAP_MONTH_STARTS: [i64; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];
const MONTH_DAYS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// One calendar timestamp: a direct port of `RexxDateTime`'s own fields and
/// arithmetic, over `i64` throughout rather than mixing `int`/`int64_t` as
/// the C++ does, since nothing here needs the narrower width and a single
/// type is one fewer cast to get wrong.
#[derive(Clone, Copy, Debug)]
struct Timestamp {
    year: i64,
    month: i64,
    day: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
    microseconds: i64,
    /// Microseconds this timestamp's own fields are offset from UTC,
    /// positive east of Greenwich -- `RexxDateTime::timeZoneOffset`.
    time_zone_offset: i64,
}

impl Timestamp {
    /// `RexxDateTime::clear`'s own zero state, less one field: year, month
    /// and day start at `1/1/1` rather than `0/0/0` -- the module doc's own
    /// "a defined answer where the oracle's own is undefined" explains why a
    /// cleared timestamp here is never asked to render a zero calendar date.
    fn clear() -> Timestamp {
        Timestamp {
            year: 1,
            month: 1,
            day: 1,
            hours: 0,
            minutes: 0,
            seconds: 0,
            microseconds: 0,
            time_zone_offset: 0,
        }
    }

    fn is_leap_year(&self) -> bool {
        is_leap_year(self.year)
    }

    /// The day-of-year (`DATE('D')`), 1-based, leap day counted from March
    /// onward -- `RexxDateTime::getYearDay`.
    fn year_day(&self) -> i64 {
        let mut yearday = MONTH_STARTS
            .get((self.month - 1) as usize)
            .copied()
            .unwrap_or(0)
            + self.day;
        if self.month > 2 && self.is_leap_year() {
            yearday += 1;
        }
        yearday
    }

    /// Days since 0001-01-01 (`DATE('B')`) -- `RexxDateTime::getBaseDate`.
    fn base_date(&self) -> i64 {
        let temp_year = self.year - 1;
        let mut basedate = temp_year * 365 + temp_year / 4 - temp_year / 100 + temp_year / 400;
        basedate += self.year_day() - 1;
        basedate
    }

    fn time_seconds(&self) -> i64 {
        (self.hours * 60 + self.minutes) * 60 + self.seconds
    }

    /// Microseconds since 0001-01-01T00:00:00 (`DATE('F')`) --
    /// `RexxDateTime::getBaseTime`.
    fn base_time(&self) -> i64 {
        let mut time = self.base_date();
        time *= SECONDS_IN_DAY;
        time += self.time_seconds();
        time *= MICROSECONDS;
        time += self.microseconds;
        time
    }

    /// [`base_time`] adjusted by this timestamp's own [`time_zone_offset`]
    /// -- `RexxDateTime::getUTCBaseTime`, and `TIME('E')`/`TIME('R')`'s own
    /// unit.
    fn utc_base_time(&self) -> i64 {
        self.base_time() + self.time_zone_offset
    }

    /// Seconds since 1970-01-01T00:00:00, negative before it
    /// (`DATE('T')`/`TIME('T')`) -- `RexxDateTime::getUnixTime`.
    fn unix_time(&self) -> i64 {
        (self.base_time() - UNIX_BASE_TIME) / MICROSECONDS
    }

    /// Sets year/month/day from a day count since 0001-01-01, clearing every
    /// other field -- `RexxDateTime::setBaseDate`. `false` for a `basedays`
    /// outside `0..=MAX_BASE_DATE`.
    fn set_base_date(&mut self, basedays: i64) -> bool {
        if !(0..=MAX_BASE_DATE).contains(&basedays) {
            return false;
        }
        *self = Timestamp::clear();
        let mut remaining = basedays + 1;
        self.year = (remaining / OLYMPIAD_DAYS) * 400;
        remaining -= base_days(self.year);
        if remaining == 0 {
            remaining = 366;
        } else {
            self.year += (remaining / CENTURY_DAYS) * 100;
            remaining %= CENTURY_DAYS;
            if remaining == 0 {
                remaining = 365;
            } else {
                self.year += (remaining / LEAP_DAYS) * 4;
                remaining %= LEAP_DAYS;
                if remaining == 0 {
                    remaining = 366;
                } else {
                    self.year += remaining / 365;
                    remaining %= 365;
                    if remaining == 0 {
                        remaining = 365;
                    } else {
                        self.year += 1;
                    }
                }
            }
        }
        self.set_day(remaining);
        true
    }

    /// Sets month/day from a 1-based day-of-year, using this timestamp's own
    /// (already-set) year to choose the leap-aware month table --
    /// `RexxDateTime::setDay`.
    fn set_day(&mut self, basedays: i64) {
        if basedays < 1 {
            self.month = 0;
            self.day = basedays.max(0);
            return;
        }
        let table = if self.is_leap_year() {
            &LEAP_MONTH_STARTS
        } else {
            &MONTH_STARTS
        };
        for i in 1..table.len() {
            if table[i] >= basedays {
                self.month = i as i64;
                self.day = basedays - table[i - 1];
                return;
            }
        }
        // `basedays` is always at most one leap-aware year's own day count
        // at every call site (`set_base_date` reduces it to `1..=366` first,
        // and `DATE`'s own `'D'` input style validates its argument the same
        // way before calling `set_date`), so the table above always finds a
        // month before running out of entries.
        unreachable!("a valid day-of-year fits within its own year")
    }

    /// Sets year, then month/day from a day-of-year within it --
    /// `RexxDateTime::setDate`, `DATE('D')`'s own input style.
    fn set_date(&mut self, year: i64, day_of_year: i64) {
        self.year = year;
        self.set_day(day_of_year);
    }

    /// Sets every field from a microsecond count since 0001-01-01T00:00:00
    /// -- `RexxDateTime::setBaseTime`. `false` outside `0..=MAX_BASE_TIME`.
    fn set_base_time(&mut self, basetime: i64) -> bool {
        if !(0..=MAX_BASE_TIME).contains(&basetime) {
            return false;
        }
        let basedays = basetime / MICROSECONDS_IN_DAY;
        let mut remainder = basetime - basedays * MICROSECONDS_IN_DAY;
        // Setting the basedate clears every other field first, matching the
        // oracle's own note at the identical call site.
        self.set_base_date(basedays);
        self.microseconds = remainder % MICROSECONDS;
        remainder /= MICROSECONDS;
        self.hours = remainder / 3600;
        remainder %= 3600;
        self.minutes = remainder / 60;
        self.seconds = remainder % 60;
        true
    }

    /// `RexxDateTime::setUnixTime`: seconds since 1970-01-01T00:00:00,
    /// negative before it.
    fn set_unix_time(&mut self, seconds: i64) -> bool {
        match seconds
            .checked_mul(MICROSECONDS)
            .and_then(|micros| micros.checked_add(UNIX_BASE_TIME))
        {
            Some(basetime) => self.set_base_time(basetime),
            None => false,
        }
    }

    /// `RexxDateTime::setHours`: sets the hour, zeroing minutes/seconds/
    /// microseconds. `false` outside `0..24`.
    fn set_hours(&mut self, h: i64) -> bool {
        if !(0..24).contains(&h) {
            return false;
        }
        self.hours = h;
        self.minutes = 0;
        self.seconds = 0;
        self.microseconds = 0;
        true
    }

    /// `RexxDateTime::setMinutes`. `false` outside `0..1440`.
    fn set_minutes(&mut self, m: i64) -> bool {
        if !(0..1_440).contains(&m) {
            return false;
        }
        self.hours = m / 60;
        self.minutes = m % 60;
        self.seconds = 0;
        self.microseconds = 0;
        true
    }

    /// `RexxDateTime::setSeconds`. `false` outside `0..86400`.
    fn set_seconds(&mut self, s: i64) -> bool {
        if !(0..86_400).contains(&s) {
            return false;
        }
        self.hours = s / 3600;
        let rest = s % 3600;
        self.minutes = rest / 60;
        self.seconds = rest % 60;
        self.microseconds = 0;
        true
    }

    /// `RexxDateTime::adjustTimeZone` (`TIME('O')`'s own input style): keeps
    /// this timestamp's UTC instant fixed while moving its own zone offset
    /// to `offset` microseconds from UTC, which is what shifts the local
    /// wall-clock fields.
    fn adjust_time_zone(&mut self, offset: i64) -> bool {
        let base = self.utc_base_time();
        if let Some(shifted) = base.checked_sub(offset) {
            self.set_base_time(shifted);
        }
        self.time_zone_offset = offset;
        true
    }

    /// `DATE('M')`'s own month name (and `'L'`'s, through [`Timestamp::
    /// format_language`]) -- `RexxDateTime::getMonthName`, `monthNames
    /// [month - 1]`.
    fn month_name(&self) -> &'static [u8] {
        MONTH_NAMES
            .get((self.month - 1) as usize)
            .copied()
            .unwrap_or(b"")
    }

    fn week_day(&self) -> usize {
        self.base_date().rem_euclid(7) as usize
    }

    fn day_name(&self) -> &'static [u8] {
        DAY_NAMES[self.week_day()]
    }

    // ---- parsing an input conversion argument ----

    /// `RexxDateTime::parseDateTimeFormat`: `date` against a fixed template,
    /// `sep` substituted at each `/`. See the format alphabet at each of
    /// this method's own callers -- `m`/`d`/`D`/`H`/`i`/`s`/`u`/`y`/`Y`/`M`/
    /// `C`/`c`/`/`/`:`/`.`, the oracle's own thirteen -- ported field by
    /// field from the C++ source rather than from prose, because the
    /// variable-width fields (`D`, `c`, `u`) and the two-digit year's
    /// sliding century window are exactly the parts documentation never
    /// states precisely enough to reproduce.
    fn parse(&mut self, date: &[u8], format: &[u8], sep: &[u8], current_year: i64) -> bool {
        self.day = 1;
        self.month = 1;
        self.year = 1;
        if format.len() < date.len() {
            return false;
        }
        let mut input = 0usize;
        let mut fmt = 0usize;
        while fmt < format.len() {
            match format[fmt] {
                b'm' => {
                    let Some(v) = get_number_capped(date, input, 2, 12) else {
                        return false;
                    };
                    self.month = v;
                    input += 2;
                    fmt += 2;
                }
                b'd' => {
                    let Some(v) = get_number(date, input, 2) else {
                        return false;
                    };
                    self.day = v;
                    input += 2;
                    fmt += 2;
                }
                b'D' => {
                    let len = match date.get(input + 1) {
                        Some(b) if b.is_ascii_digit() => 2,
                        _ => 1,
                    };
                    let Some(v) = get_number(date, input, len) else {
                        return false;
                    };
                    self.day = v;
                    input += len;
                    fmt += 2;
                }
                b'H' => {
                    let Some(v) = get_number_capped(date, input, 2, 23) else {
                        return false;
                    };
                    self.hours = v;
                    input += 2;
                    fmt += 2;
                }
                b'i' => {
                    let Some(v) = get_number_capped(date, input, 2, 59) else {
                        return false;
                    };
                    self.minutes = v;
                    input += 2;
                    fmt += 2;
                }
                b's' => {
                    let Some(v) = get_number_capped(date, input, 2, 59) else {
                        return false;
                    };
                    self.seconds = v;
                    input += 2;
                    fmt += 2;
                }
                b'u' => {
                    let remaining = date.len() as i64 - input as i64;
                    let take = remaining.clamp(1, 6) as usize;
                    let Some(mut v) = get_number(date, input, take) else {
                        return false;
                    };
                    for _ in take..6 {
                        v *= 10;
                    }
                    self.microseconds = v;
                    input += take;
                    fmt += 6;
                }
                b'y' => {
                    let Some(mut year) = get_number(date, input, 2) else {
                        return false;
                    };
                    year += (current_year / 100) * 100;
                    if year < current_year {
                        if current_year - year > 50 {
                            year += 100;
                        }
                    } else if year - current_year > 49 {
                        year -= 100;
                    }
                    self.year = year;
                    input += 2;
                    fmt += 2;
                }
                b'Y' => {
                    let Some(v) = get_number(date, input, 4) else {
                        return false;
                    };
                    self.year = v;
                    input += 4;
                    fmt += 4;
                }
                b'M' => {
                    let Some(chunk) = date.get(input..input + 3) else {
                        return false;
                    };
                    let Some(month) = MONTH_NAMES.iter().position(|name| &name[..3] == chunk)
                    else {
                        return false;
                    };
                    self.month = month as i64 + 1;
                    input += 3;
                    fmt += 3;
                }
                b'C' => {
                    let Some(chunk) = date.get(input..input + 2) else {
                        return false;
                    };
                    if chunk == b"am" {
                        if self.hours == 12 {
                            self.hours = 0;
                        }
                    } else if chunk == b"pm" {
                        if self.hours != 12 {
                            self.hours += 12;
                        }
                    } else {
                        return false;
                    }
                    input += 2;
                    fmt += 2;
                }
                b'c' => {
                    let len = match date.get(input + 1) {
                        Some(b) if b.is_ascii_digit() => 2,
                        _ => 1,
                    };
                    let Some(v) = get_number_capped(date, input, len, 12) else {
                        return false;
                    };
                    self.hours = v;
                    input += len;
                    fmt += 2;
                }
                b'/' => {
                    if sep.is_empty() {
                        fmt += 1;
                    } else {
                        if date.get(input) != Some(&sep[0]) {
                            return false;
                        }
                        fmt += 1;
                        input += 1;
                    }
                }
                byte @ (b':' | b'.') => {
                    if date.get(input) != Some(&byte) {
                        return false;
                    }
                    fmt += 1;
                    input += 1;
                }
                _ => return false,
            }
        }
        if input != date.len() {
            return false;
        }
        if self.day == 0 || self.month == 0 || self.year == 0 {
            return false;
        }
        if self.month == 2 && self.is_leap_year() {
            self.day <= 29
        } else {
            self.day <= MONTH_DAYS[(self.month - 1) as usize]
        }
    }

    fn parse_normal_date(&mut self, date: &[u8], sep: Option<&[u8]>) -> bool {
        self.parse(date, b"DD/MMM/YYYY", sep.unwrap_or(b" "), 0)
    }

    fn parse_iso_date(&mut self, date: &[u8], sep: Option<&[u8]>) -> bool {
        self.parse(date, b"YYYY/mm/dd", sep.unwrap_or(b"-"), 0)
    }

    fn parse_standard_date(&mut self, date: &[u8], sep: Option<&[u8]>) -> bool {
        self.parse(date, b"YYYY/mm/dd", sep.unwrap_or(b""), 0)
    }

    fn parse_european_date(&mut self, date: &[u8], sep: Option<&[u8]>, current_year: i64) -> bool {
        self.parse(date, b"dd/mm/yy", sep.unwrap_or(b"/"), current_year)
    }

    fn parse_usa_date(&mut self, date: &[u8], sep: Option<&[u8]>, current_year: i64) -> bool {
        self.parse(date, b"mm/dd/yy", sep.unwrap_or(b"/"), current_year)
    }

    fn parse_ordered_date(&mut self, date: &[u8], sep: Option<&[u8]>, current_year: i64) -> bool {
        self.parse(date, b"yy/mm/dd", sep.unwrap_or(b"/"), current_year)
    }

    fn parse_normal_time(&mut self, time: &[u8]) -> bool {
        self.parse(time, b"HH:ii:ss", b"", 0)
    }

    fn parse_civil_time(&mut self, time: &[u8]) -> bool {
        self.parse(time, b"cc:iiCC", b"", 0)
    }

    fn parse_long_time(&mut self, time: &[u8]) -> bool {
        self.parse(time, b"HH:ii:ss.uuuuuu", b"", 0)
    }

    // ---- rendering an output style ----

    fn format_european(&self, sep: Option<&[u8]>) -> Vec<u8> {
        let sep = sep.unwrap_or(b"/");
        let mut out = Vec::new();
        push_decimal(&mut out, self.day, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.month, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.year % 100, 2);
        out
    }

    fn format_normal(&self, sep: Option<&[u8]>) -> Vec<u8> {
        let sep = sep.unwrap_or(b" ");
        let mut out = Vec::new();
        push_decimal(&mut out, self.day, 0);
        out.extend_from_slice(sep);
        out.extend_from_slice(&self.month_name()[..self.month_name().len().min(3)]);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.year, 4);
        out
    }

    fn format_ordered(&self, sep: Option<&[u8]>) -> Vec<u8> {
        let sep = sep.unwrap_or(b"/");
        let mut out = Vec::new();
        push_decimal(&mut out, self.year % 100, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.month, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.day, 2);
        out
    }

    fn format_iso(&self, sep: Option<&[u8]>) -> Vec<u8> {
        let sep = sep.unwrap_or(b"-");
        let mut out = Vec::new();
        push_decimal(&mut out, self.year, 4);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.month, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.day, 2);
        out
    }

    fn format_standard(&self, sep: Option<&[u8]>) -> Vec<u8> {
        let sep = sep.unwrap_or(b"");
        let mut out = Vec::new();
        push_decimal(&mut out, self.year, 4);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.month, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.day, 2);
        out
    }

    fn format_usa(&self, sep: Option<&[u8]>) -> Vec<u8> {
        let sep = sep.unwrap_or(b"/");
        let mut out = Vec::new();
        push_decimal(&mut out, self.month, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.day, 2);
        out.extend_from_slice(sep);
        push_decimal(&mut out, self.year % 100, 2);
        out
    }

    fn format_language(&self) -> Vec<u8> {
        let mut out = Vec::new();
        push_decimal(&mut out, self.day, 0);
        out.push(b' ');
        out.extend_from_slice(self.month_name());
        out.push(b' ');
        push_decimal(&mut out, self.year, 4);
        out
    }

    fn format_civil(&self) -> Vec<u8> {
        let mut hours = self.hours;
        if hours == 0 {
            hours = 12;
        } else if hours > 12 {
            hours -= 12;
        }
        let mut out = Vec::new();
        push_decimal(&mut out, hours, 0);
        out.push(b':');
        push_decimal(&mut out, self.minutes, 2);
        out.extend_from_slice(if self.hours >= 12 { b"pm" } else { b"am" });
        out
    }

    fn format_long(&self) -> Vec<u8> {
        let mut out = Vec::new();
        push_decimal(&mut out, self.hours, 2);
        out.push(b':');
        push_decimal(&mut out, self.minutes, 2);
        out.push(b':');
        push_decimal(&mut out, self.seconds, 2);
        out.push(b'.');
        push_decimal(&mut out, self.microseconds, 6);
        out
    }

    fn format_normal_time(&self) -> Vec<u8> {
        let mut out = Vec::new();
        push_decimal(&mut out, self.hours, 2);
        out.push(b':');
        push_decimal(&mut out, self.minutes, 2);
        out.push(b':');
        push_decimal(&mut out, self.seconds, 2);
        out
    }
}

/// `n`, zero-padded to at least `width` digits. Every field these builtins
/// render is non-negative (a year, a month, a day, a two-digit year
/// remainder, a clock field), so this never has a sign to place.
fn push_decimal(out: &mut Vec<u8>, n: i64, width: usize) {
    let digits = n.to_string();
    for _ in digits.len()..width {
        out.push(b'0');
    }
    out.extend_from_slice(digits.as_bytes());
}

/// `date[start..start+len]` parsed as an unsigned decimal integer, or `None`
/// if any of those bytes is missing or is not an ASCII digit --
/// `RexxDateTime::getNumber`'s first overload.
fn get_number(date: &[u8], start: usize, len: usize) -> Option<i64> {
    let slice = date.get(start..start + len)?;
    let mut value = 0i64;
    for &byte in slice {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value * 10 + i64::from(byte - b'0');
    }
    Some(value)
}

/// [`get_number`], rejected if the parsed value exceeds `max` --
/// `RexxDateTime::getNumber`'s second overload.
fn get_number_capped(date: &[u8], start: usize, len: usize, max: i64) -> Option<i64> {
    get_number(date, start, len).filter(|&value| value <= max)
}

/// A Rexx number's own whole-number conversion, under `Numerics::
/// ARGUMENT_DIGITS` -- the same precision `whole_number` (`builtin.rs`)
/// converts a typed builtin argument under, reused here because `DATE`'s
/// `B`/`F`/`T`/`D` and `TIME`'s `H`/`S`/`M`/`F`/`T`/`O` input styles convert
/// their own `indate`/`intime` the identical way -- `RexxString::
/// numberValue`/`Numerics::objectToInt64`, both of which the oracle also
/// caps rather than reading unbounded digits.
fn whole_number_of(text: &[u8]) -> Option<i64> {
    std::str::from_utf8(text)
        .ok()
        .and_then(Number::parse)?
        .whole_value(rexx_num::ARGUMENT_DIGITS)
}

// ---- the per-clause clock and the elapsed-time anchor ----

/// Wall-clock time right now, in [`Timestamp::base_time`]'s own unit --
/// UTC, with no time zone applied (the module doc's own divergence).
fn real_clock_base_time() -> i64 {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    UNIX_BASE_TIME + since_epoch.as_micros() as i64
}

/// The clock reading in force for the clause the *currently executing
/// activation* is stepping -- cached on that activation, and read fresh
/// only once [`Activation::clock_stale`] says so. See its own doc for why
/// the cache lives on the activation rather than on `Interp`.
fn now_base_time(interp: &mut Interp) -> i64 {
    if !interp.activation().clock_stale {
        return interp
            .activation()
            .cached_clock
            .expect("a clock that is not stale was read at least once");
    }
    if interp.pending_elapsed_reset {
        if let Some(stale) = interp.activation().cached_clock {
            interp.elapsed_anchor = Some(stale);
        }
        interp.pending_elapsed_reset = false;
    }
    let micros = real_clock_base_time();
    let activation = interp.activation_mut();
    activation.cached_clock = Some(micros);
    activation.clock_stale = false;
    micros
}

/// Microseconds the host's local zone is ahead of UTC at `utc_micros`,
/// which is [`UNIX_BASE_TIME`]-based like every other reading here.
///
/// `.File`'s timestamps read through this too: a file's stamp is encoded at
/// the offset in force **at that stamp**, not at the offset in force now.
/// The civil fields one base time names, for a caller outside this module
/// that renders a layout of its own.
pub(crate) struct Calendar {
    pub(crate) year: i64,
    pub(crate) month: i64,
    pub(crate) day: i64,
    pub(crate) hours: i64,
    pub(crate) minutes: i64,
    pub(crate) seconds: i64,
}

/// [`Calendar`] for `base_time`, which is a local reading in the caller's
/// hands already.
pub(crate) fn calendar_of(base_time: i64) -> Calendar {
    let mut stamp = Timestamp::clear();
    stamp.set_base_time(base_time);
    Calendar {
        year: stamp.year,
        month: stamp.month,
        day: stamp.day,
        hours: stamp.hours,
        minutes: stamp.minutes,
        seconds: stamp.seconds,
    }
}

pub(crate) fn local_offset_micros(utc_micros: i64) -> i64 {
    let unix_micros = utc_micros - UNIX_BASE_TIME;
    let secs = unix_micros.div_euclid(MICROSECONDS);
    let nanos = (unix_micros.rem_euclid(MICROSECONDS) * 1_000) as u32;
    match chrono::Local.timestamp_opt(secs, nanos) {
        chrono::offset::LocalResult::Single(at) => {
            i64::from(at.offset().fix().local_minus_utc()) * MICROSECONDS
        }
        // An instant the host's zone maps to zero or two local readings --
        // the hour a DST jump skips, and the hour it repeats. `Ambiguous`
        // carries both; the earlier is what `localtime` answers for a
        // repeated hour, and `None` cannot arise from a real clock sample
        // because a skipped hour never occurs. Neither is a reason to fail
        // a `DATE` call, so both fall back to the zone's offset now.
        chrono::offset::LocalResult::Ambiguous(earlier, _) => {
            i64::from(earlier.offset().fix().local_minus_utc()) * MICROSECONDS
        }
        chrono::offset::LocalResult::None => 0,
    }
}

/// The clause's own clock reading, decomposed into calendar fields.
fn now(interp: &mut Interp) -> Timestamp {
    let mut timestamp = Timestamp::clear();
    let utc = now_base_time(interp);
    let offset = local_offset_micros(utc);
    // A real wall-clock reading is always within `0..=MAX_BASE_TIME` (the
    // year 1 to 9999 range this crate's own calendar covers), so this
    // always succeeds; the fallback is an inert cleared timestamp for the
    // one host whose own clock is set outside that range.
    let _ = timestamp.set_base_time(utc + offset);
    timestamp.time_zone_offset = offset;
    timestamp
}

/// `TIME('E')`/`TIME('R')`'s own reading: elapsed microseconds, formatted,
/// since [`Interp::elapsed_anchor`] -- lazily anchored to `reading` on the
/// very first call. A reset (`reset` set, or the clock read backward) does
/// **not** move the anchor here -- it only arms [`Interp::
/// pending_elapsed_reset`], which [`now_base_time`]'s own cache-miss path
/// is what actually consumes, matching the oracle's own lazy order.
fn elapsed_reading(interp: &mut Interp, reading: i64, reset: bool) -> Vec<u8> {
    let anchor = *interp.elapsed_anchor.get_or_insert(reading);
    let threshold = reading - anchor;
    let text = match threshold {
        negative if negative < 0 => {
            interp.pending_elapsed_reset = true;
            b"0".to_vec()
        }
        0 => b"0".to_vec(),
        positive => {
            format!("{}.{:06}", positive / MICROSECONDS, positive % MICROSECONDS).into_bytes()
        }
    };
    if reset {
        interp.pending_elapsed_reset = true;
    }
    text
}

// ---- option validation ----

/// `DATE`'s output-style letters, exactly as its own 40.904 message spells
/// them (`Error_Incorrect_call_list`'s `"BDEFILMNOSTUW"` literal,
/// `expression/BuiltinFunctions.cpp`).
const DATE_OPTIONS1: &str = "BDEFILMNOSTUW";
/// `DATE`'s input-style letters -- ten, not thirteen: no `L`, `M` or `W`,
/// since a month or weekday *name* is not itself a parseable date.
const DATE_OPTIONS2: &str = "BDEFINOSTU";
const TIME_OPTIONS1: &str = "CEFHLMNORST";
const TIME_OPTIONS2: &str = "CFHLMNOST";

/// Reads the option-letter argument at `position`, upcasing its first byte.
/// `default` when the argument is omitted; a present-but-empty argument is
/// always 40.904, never treated as omitted -- measured, `date('')` and
/// `time('')` both report `found ""`, and this raises directly rather than
/// falling through to the letter switch, matching the oracle's own separate
/// early check for that case.
fn style_byte(
    interp: &mut Interp,
    args: Args<'_>,
    position: usize,
    name: &[u8],
    valid: &str,
    default: u8,
) -> Result<u8, Failure> {
    match optional_string(interp, args, position) {
        None => Ok(default),
        Some(text) if text.is_empty() => {
            Err(Raised::argument_not_in_list(name, position, valid, b"").into())
        }
        Some(text) => Ok(text[0].to_ascii_uppercase()),
    }
}

/// `expression/BuiltinFunctions.hpp`'s own `ALPHANUM` macro, spelled out so
/// [`check_separator`] can run the identical `strchr`-based membership test
/// the oracle does, the `0`-byte quirk included -- see [`strchr_matches`].
const ALPHANUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Membership as the oracle's own `strchr(set, byte)` computes it, its
/// C-string quirk included: `strchr` finds the byte `0` in **any** string,
/// because a C string's own terminator is a `0` byte and `strchr` treats
/// that as ordinary membership, checked or not against `set`'s own
/// contents. Every `strchr`-based membership test `BUILTIN(DATE)`/
/// `BUILTIN(TIME)` runs therefore reports "found" for a `0` byte against
/// *every* set, including one that itself carries no zero -- measured,
/// three call sites at once, all rc 216 and none of them what
/// `set.contains(&0)` alone would answer since none of [`ALPHANUM`],
/// `"EINOSU"` or `"BDFLMTW"` contains a literal zero:
/// ```text
/// date('S','20070922','S','00'x)     40.43   (the alphanumeric-set osep check)
/// date('00'x,,,'-')                  40.904  (the EINOSU osep-compatibility check;
///                                              0 counts as "found", so as
///                                              "compatible", so parsing falls
///                                              through to the style switch, which
///                                              then rejects the 0 byte as a style)
/// date(,'20070922','00'x,,'-')       40.44   (the BDFLMTW isep-compatibility check;
///                                              0 counts as "found", so as
///                                              "incompatible")
/// ```
fn strchr_matches(set: &[u8], byte: u8) -> bool {
    byte == 0 || set.contains(&byte)
}

/// A separator argument (`DATE`'s `osep`/`isep`): exactly one non-
/// alphanumeric byte, or the null string. A `0x00` byte counts as
/// alphanumeric here -- not because it is one, but because [`strchr_matches`]
/// is what the oracle's own check actually computes.
fn check_separator(name: &[u8], position: usize, sep: &[u8]) -> Result<(), Failure> {
    let bad = sep.len() > 1
        || sep
            .first()
            .is_some_and(|&byte| strchr_matches(ALPHANUM, byte));
    if bad {
        Err(Raised::separator_not_a_char(name, position, sep).into())
    } else {
        Ok(())
    }
}

// ---- DATE ----

/// `DATE()` / `DATE(option, indate, option2, osep, isep)`: today's date, in
/// one of thirteen output styles, optionally converted from an explicit
/// `indate` given in one of ten input styles.
pub(crate) fn date(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let style = style_byte(interp, args, 1, name, DATE_OPTIONS1, b'N')?;
    let indate = optional_string(interp, args, 2);
    let option2 = optional_string(interp, args, 3);
    let osep = optional_string(interp, args, 4);
    let isep = optional_string(interp, args, 5);

    if indate.is_none() && (option2.is_some() || isep.is_some()) {
        return Err(Raised::missing_argument(name, 2).into());
    }

    let mut style2 = b'N';
    if let Some(text) = &option2 {
        if text.is_empty() {
            return Err(Raised::argument_not_in_list(name, 3, DATE_OPTIONS2, b"").into());
        }
        style2 = text[0].to_ascii_uppercase();
    }

    let output_sep: Option<&[u8]> = match &osep {
        None => None,
        Some(sep) => {
            if !strchr_matches(b"EINOSU", style) {
                return Err(Raised::format_incompatible_separator(name, 1, &[style], 4).into());
            }
            check_separator(name, 4, sep)?;
            Some(sep.as_slice())
        }
    };

    let current = now(interp);
    let mut timestamp = current;
    if let Some(indate_bytes) = &indate {
        let input_sep: Option<&[u8]> = match &isep {
            None => None,
            Some(sep) => {
                if strchr_matches(b"BDFLMTW", style2) {
                    return Err(Raised::format_incompatible_separator(name, 3, &[style2], 5).into());
                }
                check_separator(name, 5, sep)?;
                Some(sep.as_slice())
            }
        };
        timestamp = Timestamp::clear();
        // `BuiltinFunctions.cpp:1114`, immediately after its own `clear()`:
        // "everything is done using the current timezone offset". Without
        // this the parsed timestamp carries offset `0` and `TIME('O')`'s
        // input style answers the wrong zone -- invisible while every "now"
        // was fixed at UTC, and a silent wrong answer the moment it is not.
        timestamp.time_zone_offset = current.time_zone_offset;
        let valid = match style2 {
            b'N' => timestamp.parse_normal_date(indate_bytes, input_sep),
            b'B' => whole_number_of(indate_bytes).is_some_and(|n| timestamp.set_base_date(n)),
            b'F' => whole_number_of(indate_bytes).is_some_and(|n| timestamp.set_base_time(n)),
            b'T' => whole_number_of(indate_bytes).is_some_and(|n| timestamp.set_unix_time(n)),
            b'D' => whole_number_of(indate_bytes).is_some_and(|n| {
                let in_range = n >= 0 && (n <= 365 || (n == 366 && is_leap_year(current.year)));
                if in_range {
                    timestamp.set_date(current.year, n);
                }
                in_range
            }),
            b'E' => timestamp.parse_european_date(indate_bytes, input_sep, current.year),
            b'O' => timestamp.parse_ordered_date(indate_bytes, input_sep, current.year),
            b'I' => timestamp.parse_iso_date(indate_bytes, input_sep),
            b'S' => timestamp.parse_standard_date(indate_bytes, input_sep),
            b'U' => timestamp.parse_usa_date(indate_bytes, input_sep, current.year),
            _ => {
                return Err(Raised::argument_not_in_list(name, 3, DATE_OPTIONS2, &[style2]).into());
            }
        };
        if !valid {
            if isep.is_some() {
                return Err(Raised::format_incompatible_separator(name, 2, indate_bytes, 5).into());
            }
            return Err(Raised::date_format_invalid(name, indate_bytes, style2).into());
        }
    }

    match style {
        b'B' => Ok(interp.text(timestamp.base_date().to_string().as_bytes())),
        b'F' => Ok(interp.text(timestamp.base_time().to_string().as_bytes())),
        b'T' => Ok(interp.text(timestamp.unix_time().to_string().as_bytes())),
        b'D' => Ok(interp.text(timestamp.year_day().to_string().as_bytes())),
        b'E' => Ok(interp.text(&timestamp.format_european(output_sep))),
        b'L' => Ok(interp.text(&timestamp.format_language())),
        b'M' => Ok(interp.text(timestamp.month_name())),
        b'N' => Ok(interp.text(&timestamp.format_normal(output_sep))),
        b'O' => Ok(interp.text(&timestamp.format_ordered(output_sep))),
        b'I' => Ok(interp.text(&timestamp.format_iso(output_sep))),
        b'S' => Ok(interp.text(&timestamp.format_standard(output_sep))),
        b'U' => Ok(interp.text(&timestamp.format_usa(output_sep))),
        b'W' => Ok(interp.text(timestamp.day_name())),
        _ => Err(Raised::argument_not_in_list(name, 1, DATE_OPTIONS1, &[style]).into()),
    }
}

// ---- TIME ----

/// `TIME()` / `TIME(option, intime, option2)`: the current time of day, in
/// one of eleven output styles, optionally converted from an explicit
/// `intime` given in one of nine input styles; `E`/`R` are the elapsed-time
/// pair the module doc's own transcript pins.
pub(crate) fn time(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let style = style_byte(interp, args, 1, name, TIME_OPTIONS1, b'N')?;
    let intime = optional_string(interp, args, 2);
    let option2 = optional_string(interp, args, 3);

    let mut style2 = b'N';
    if let Some(text) = &option2 {
        if intime.is_none() {
            return Err(Raised::missing_argument(name, 2).into());
        }
        if text.is_empty() {
            return Err(Raised::argument_not_in_list(name, 3, TIME_OPTIONS2, b"").into());
        }
        style2 = text[0].to_ascii_uppercase();
    }

    let current = now(interp);
    let mut timestamp = current;
    if let Some(intime_bytes) = &intime {
        if style == b'R' || style == b'E' {
            return Err(Raised::invalid_conversion(name, style).into());
        }
        timestamp = Timestamp::clear();
        // `BuiltinFunctions.cpp:1375`, the same port as `DATE`'s above.
        timestamp.time_zone_offset = current.time_zone_offset;
        let valid = match style2 {
            b'N' => timestamp.parse_normal_time(intime_bytes),
            b'C' => timestamp.parse_civil_time(intime_bytes),
            b'L' => timestamp.parse_long_time(intime_bytes),
            b'H' => whole_number_of(intime_bytes).is_some_and(|n| timestamp.set_hours(n)),
            b'S' => whole_number_of(intime_bytes).is_some_and(|n| timestamp.set_seconds(n)),
            b'M' => whole_number_of(intime_bytes).is_some_and(|n| timestamp.set_minutes(n)),
            b'F' => whole_number_of(intime_bytes).is_some_and(|n| timestamp.set_base_time(n)),
            b'T' => whole_number_of(intime_bytes).is_some_and(|n| timestamp.set_unix_time(n)),
            b'O' => {
                timestamp = current;
                whole_number_of(intime_bytes).is_some_and(|n| timestamp.adjust_time_zone(n))
            }
            _ => {
                return Err(Raised::argument_not_in_list(name, 3, TIME_OPTIONS2, &[style2]).into());
            }
        };
        if !valid {
            return Err(Raised::date_format_invalid(name, intime_bytes, style2).into());
        }
    }

    match style {
        b'E' | b'R' => {
            // **The raw clock, not [`Timestamp::utc_base_time`].** The anchor
            // this is differenced against is [`Activation::cached_clock`],
            // which is what [`real_clock_base_time`] returned -- so the
            // reading has to be the same quantity or the difference carries
            // the zone. `utc_base_time` is `base_time() + time_zone_offset`,
            // faithful to `RexxDateTime::getUTCBaseTime`, and with the local
            // calendar fields [`now`] now builds that is the UTC instant plus
            // *twice* the offset. The oracle cancels it by differencing two
            // values of its own formula; this cancels it by differencing two
            // raw readings. Measured before the fix: `TIME('R')` in a callee
            // leaked 14400.000001 seconds into the caller at UTC+2, which is
            // two offsets, not one.
            let reading = now_base_time(interp);
            let text = elapsed_reading(interp, reading, style == b'R');
            Ok(interp.text(&text))
        }
        b'C' => Ok(interp.text(&timestamp.format_civil())),
        b'H' => Ok(interp.text(timestamp.hours.to_string().as_bytes())),
        b'L' => Ok(interp.text(&timestamp.format_long())),
        b'M' => Ok(interp.text(
            (timestamp.hours * 60 + timestamp.minutes)
                .to_string()
                .as_bytes(),
        )),
        b'N' => Ok(interp.text(&timestamp.format_normal_time())),
        b'S' => Ok(interp.text(
            ((timestamp.hours * 60 + timestamp.minutes) * 60 + timestamp.seconds)
                .to_string()
                .as_bytes(),
        )),
        b'F' => Ok(interp.text(timestamp.base_time().to_string().as_bytes())),
        b'T' => Ok(interp.text(timestamp.unix_time().to_string().as_bytes())),
        b'O' => Ok(interp.text(timestamp.time_zone_offset.to_string().as_bytes())),
        _ => Err(Raised::argument_not_in_list(name, 1, TIME_OPTIONS1, &[style]).into()),
    }
}

#[cfg(test)]
mod tests;
