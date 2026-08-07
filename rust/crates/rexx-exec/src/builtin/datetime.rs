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
//!
//! # Neither builtin may appear in a differential corpus program (D11)
//!
//! Both read the real wall clock for their no-argument forms, so nothing
//! here can be pinned to a byte-exact value the way every other family in
//! this phase is. What *is* deterministic, and what every test below pins,
//! is the **conversion form** -- an explicit `indate`/`intime` argument
//! converted between two named styles, which depends on nothing but its own
//! bytes. `DATE`'s thirteen output letters (`BDEFILMNOSTUW`) and ten input
//! letters (`BDEFINOSTU`) are each deterministic once an input is supplied,
//! including `L`/`M`/`W` (month and weekday names): the oracle's own
//! `monthNames`/`dayNames` tables (`RexxDateTime.cpp`) are hardcoded English
//! text, not a locale lookup, so a fixed calendar date names its month and
//! weekday exactly as pinned here regardless of when or where this runs.
//! Only a **no-argument** call -- any of the thirteen letters, not a
//! distinguished subset -- reads "today" and cannot be pinned.
//!
//! # The clock is cached once per clause, and `TIME('R')` is state that survives across clauses
//!
//! Measured against the oracle: two `TIME('L')` calls inside one clause
//! answer identically across a real CPU burn placed *between* them, where
//! the same burn placed between two separate clauses' calls changes the
//! answer. `RexxActivation::getTime` (`execution/RexxActivation.cpp:3390`)
//! is why: it caches a timestamp until `settings.timeStamp.valid` is
//! cleared, and that happens exactly once per instruction, right after
//! `nextInst->execute()` returns (`:647`). `Activation::cached_clock`
//! reproduces this, invalidated from the identical one place every
//! instruction passes through (`run.rs`'s `step_in_temps_frame`), on
//! whichever activation is executing at the time -- which is also why it
//! lives on the activation and not on the interpreter: a nested internal
//! `CALL`'s own instructions invalidate only *its own* cache, never the
//! caller's, matching the oracle's own per-activation `timeStamp` and
//! measured directly against the shape above (`burn()` is a real internal
//! routine, not scaffolding).
//!
//! `TIME('R')` resets an elapsed-time clock a later `TIME('E')`/`TIME('R')`
//! measures from -- [`Interp::elapsed_anchor`]'s own doc has the reset
//! semantics and the deliberate simplification this crate takes of the
//! oracle's own lazily-applied reset. The transcript that pins it:
//!
//! ```text
//! first time('R')            0
//! time('E') after ~0.11s     0.109014
//! time('R') after another    0.219483   = the sum of BOTH burns
//! time('E') immediately      0.000023   so that R did reset
//! ```
//!
//! # A few no-argument styles are also host-dependent absolute clocks
//!
//! Given a fixed input, `T`/`F` (a signed integer count of seconds or
//! microseconds since a fixed epoch) are exactly as deterministic as every
//! other letter -- `DATE.testGroup`'s own `date('T', date, format)`
//! transcripts pin them. Only their *no-argument* reading is a raw clock
//! sample, no more and no less host-dependent than a no-argument `B`/`D`
//! reading "today"'s own basedate or day-of-year would be.
//!
//! This crate reads that clock in UTC, with the time zone offset fixed at
//! zero. The oracle's own no-argument reading uses the host's local zone;
//! that is a real, acknowledged divergence for the no-argument forms only
//! (D11 bars them from every differential comparison this crate has), not
//! a claim that this crate's "now" is otherwise wrong.
//!
//! # A defined answer where the oracle's own is undefined
//!
//! `TIME`'s time-only input styles (`N`/`C`/`L`/`H`/`S`/`M`) never touch a
//! `year`/`month`/`day` field, and the oracle's own `RexxDateTime::clear`
//! leaves those at `0` rather than at a real calendar date. Asking such a
//! result for a *date*-shaped output (`F`/`T`/`O`) then indexes
//! `monthStarts[month - 1]` at `month == 0`, i.e. one element before the
//! array -- measured, `time('F', '12:34:56', 'N')` returns a value that
//! depends on whatever byte happens to sit there, not on any Rexx
//! specification. This crate starts every cleared [`Timestamp`] at
//! `1/1/1`, never `0/0/0`, so the identical call is merely a very old date
//! (`00010101`-shaped) rather than a read of undefined memory -- observably
//! different from the oracle on this one unspecified cross of input and
//! output style, and not attempted to be reproduced byte for byte.

use rexx_core::ObjRef;
use rexx_num::Number;

use super::optional_string;
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
const MICROSECONDS_IN_DAY: i64 = SECONDS_IN_DAY * MICROSECONDS;

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
const UNIX_BASE_TIME: i64 = UNIX_BASE_DATE * MICROSECONDS_IN_DAY;

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
    /// Microseconds this timestamp's own fields are offset from UTC.
    /// Always `0` for a clock reading in this crate (the module doc's own
    /// UTC-only divergence); non-zero only after [`Timestamp::adjust_time_zone`].
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
        let mut yearday = MONTH_STARTS[(self.month - 1) as usize] + self.day;
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
    ///
    /// [`base_time`]: Timestamp::base_time
    /// [`time_zone_offset`]: Timestamp::time_zone_offset
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
    ///
    /// A day-of-year at or below `0` is the module doc's own defined-instead-
    /// of-undefined case: set to month `0`, day `0` (clamped, never
    /// negative) rather than indexing the month table at `-1`.
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
/// ARGUMENT_DIGITS` -- the same precision `whole_number` (`builtin/mod.rs`)
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
/// only once that cache has been invalidated. See [`Activation::
/// cached_clock`]'s own doc for where the invalidation happens, why it
/// lives on the activation rather than on `Interp`, and what it
/// reproduces.
///
/// [`Activation::cached_clock`]: crate::activation::Activation::cached_clock
fn now_base_time(interp: &mut Interp) -> i64 {
    if let Some(cached) = interp.activation().cached_clock {
        return cached;
    }
    let micros = real_clock_base_time();
    interp.activation_mut().cached_clock = Some(micros);
    micros
}

/// The clause's own clock reading, decomposed into calendar fields.
fn now(interp: &mut Interp) -> Timestamp {
    let mut timestamp = Timestamp::clear();
    // A real wall-clock reading is always within `0..=MAX_BASE_TIME` (the
    // year 1 to 9999 range this crate's own calendar covers), so this
    // always succeeds; the fallback is an inert cleared timestamp for the
    // one host whose own clock is set outside that range.
    let _ = timestamp.set_base_time(now_base_time(interp));
    timestamp
}

/// `TIME('E')`/`TIME('R')`'s own reading: elapsed microseconds, formatted,
/// since [`Interp::elapsed_anchor`] -- lazily anchored to `reading` on the
/// very first call, and reset to `reading` when `reset` is set or the clock
/// read backward. See [`Interp::elapsed_anchor`]'s own doc for the
/// transcript this reproduces and the simplification it names.
fn elapsed_reading(interp: &mut Interp, reading: i64, reset: bool) -> Vec<u8> {
    let anchor = *interp.elapsed_anchor.get_or_insert(reading);
    let threshold = reading - anchor;
    let text = match threshold {
        negative if negative < 0 => {
            interp.elapsed_anchor = Some(reading);
            b"0".to_vec()
        }
        0 => b"0".to_vec(),
        positive => {
            format!("{}.{:06}", positive / MICROSECONDS, positive % MICROSECONDS).into_bytes()
        }
    };
    if reset {
        interp.elapsed_anchor = Some(reading);
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
    args: &[Option<ObjRef>],
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

/// A separator argument (`DATE`'s `osep`/`isep`): exactly one non-
/// alphanumeric byte, or the null string.
fn check_separator(name: &[u8], position: usize, sep: &[u8]) -> Result<(), Failure> {
    let bad = sep.len() > 1 || sep.first().is_some_and(u8::is_ascii_alphanumeric);
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
///
/// Argument order and every validation this reproduces follow `BUILTIN
/// (DATE)` (`expression/BuiltinFunctions.cpp:1001`) line for line, because
/// the ordering between checks is itself observable -- measured,
/// `date(,'20070922','w',,'-')` raises 40.44 (the input separator is
/// incompatible with style `W`) rather than 40.904 (`W` is not a valid
/// input style at all), so the separator-compatibility check runs before
/// the style-letter switch that would otherwise catch it first.
pub(crate) fn date(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
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
            if !b"EINOSU".contains(&style) {
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
                if b"BDFLMTW".contains(&style2) {
                    return Err(Raised::format_incompatible_separator(name, 3, &[style2], 5).into());
                }
                check_separator(name, 5, sep)?;
                Some(sep.as_slice())
            }
        };
        timestamp = Timestamp::clear();
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
///
/// Ordering, as `BUILTIN(TIME)` (`expression/BuiltinFunctions.cpp:1313`)
/// applies it: `option2`'s own presence-requires-`intime` check
/// (measured, `time(,,'n')` is 40.5 naming argument 2) runs *before*
/// `option2`'s own emptiness check, the reverse of the order `DATE`'s
/// analogous pair runs in -- each is read directly from its own builtin
/// rather than assumed to match the other's.
pub(crate) fn time(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
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
            let text = elapsed_reading(interp, current.utc_base_time(), style == b'R');
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
mod tests {
    use super::*;
    use crate::plan::{BodyKey, ProgramId};
    use crate::{Activation, Interp, Invocation, run_program};
    use std::rc::Rc;

    /// Pushes a fresh top-level activation for `program`, the minimal setup
    /// `Interp::run` does. Copied rather than shared, matching every other
    /// test module in this crate (`queue.rs`'s own copy has the same note)
    /// -- needed here, rather than reached through `run_program`, only by
    /// the `TIME('R')` tests that manipulate [`Interp::elapsed_anchor`]
    /// directly between two calls on one live `Interp`, which `run_program`
    /// hands back only after the whole run has ended.
    fn activate(interp: &mut Interp, program: rexx_parse::Program) -> Rc<rexx_parse::Program> {
        let program = Rc::new(program);
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp
            .activations
            .push(Activation::new(id, Rc::clone(&program), plan, frame));
        program
    }

    /// A live `Interp` with one pushed top-level activation, empty source --
    /// enough for [`now`] to have somewhere to cache a reading, and nothing
    /// more.
    fn live_interp() -> Interp {
        let mut interp = Interp::new();
        let program =
            rexx_parse::parse_program(b"nop\n".to_vec()).expect("a trivial program parses");
        activate(&mut interp, program);
        interp
    }

    /// Runs `source` as a whole program and hands back the stdout it
    /// produced, having first insisted the run ended cleanly.
    fn output(source: &[u8]) -> String {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        String::from_utf8(outcome.stdout).expect("ASCII output")
    }

    fn failure(source: &[u8]) -> (i32, String) {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        (
            outcome.exit_code,
            String::from_utf8(outcome.stderr).expect("ASCII stderr"),
        )
    }

    /// [`failure`], without the ASCII assumption -- for the one test whose
    /// own alphabet includes a byte `>= 0x80`, which `String::from_utf8`
    /// would reject outright rather than mangle, so `failure`'s own
    /// `.expect` cannot be reused for it.
    fn failure_bytes(source: &[u8]) -> (i32, Vec<u8>) {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        (outcome.exit_code, outcome.stderr)
    }

    // ---- Step 0: the clock is cached per clause ----

    /// A real CPU burn **inside one clause**, between two `TIME('L')` calls,
    /// changes nothing -- both calls answer identically. This is the
    /// discriminating shape the brief's own first attempt at this probe
    /// missed: a burn placed *between two statements* cannot tell a cached
    /// read apart from a live one, because the clause boundary alone would
    /// already change the answer. Putting both reads inside one `PARSE
    /// VALUE` clause is what a burn between them actually tests.
    #[test]
    fn two_clock_reads_in_one_clause_answer_identically_across_a_real_burn() {
        assert_eq!(
            output(
                b"parse value time('L') burn() time('L') with n1 . n2\nsay (n1 = n2)\nexit\nburn: procedure\n  do i = 1 to 20000\n    j = i * i\n  end\n  return j\n"
            ),
            "1\n"
        );
    }

    /// The adjacent success proving the burn above is real and the harness
    /// can see it: the identical burn between two **separate** clauses'
    /// clock reads does change the answer, because each clause gets its own
    /// fresh reading.
    #[test]
    fn the_same_burn_between_two_clauses_changes_the_answer() {
        assert_eq!(
            output(
                b"n1 = time('L')\nzz = burn()\nn2 = time('L')\nsay (n1 = n2)\nexit\nburn: procedure\n  do i = 1 to 20000\n    j = i * i\n  end\n  return j\n"
            ),
            "0\n"
        );
    }

    /// `DATE('T')` is cached the same way `TIME('L')` is -- the module doc's
    /// own claim that the caching is one mechanism shared by both builtins,
    /// not a `TIME`-only property.
    #[test]
    fn dates_absolute_clock_is_cached_the_same_way() {
        assert_eq!(
            output(
                b"parse value date('T') burn() date('T') with n1 . n2\nsay (n1 = n2)\nexit\nburn: procedure\n  do i = 1 to 20000\n    j = i * i\n  end\n  return j\n"
            ),
            "1\n"
        );
    }

    // ---- TIME('R')'s reset semantics ----

    /// The first `TIME('R')` a program runs answers `0`; a later one answers
    /// elapsed time **since the last reset**, not since the first call --
    /// pinned by driving the interpreter's own state (a fabricated anchor
    /// well in the past) rather than the wall clock, since two clock reads
    /// close together cannot tell a live read from a stale one apart
    /// (`Interp::elapsed_anchor`'s own doc has the transcript this
    /// reproduces).
    #[test]
    fn time_r_resets_relative_to_the_last_reset_not_program_start() {
        let mut interp = live_interp();
        // The first read is always `0`, wherever the wall clock actually is
        // -- `elapsed_anchor` starts `None` and self-anchors on first use.
        let first = dispatch_time_r(&mut interp);
        assert_eq!(first, b"0".to_vec());

        // Move the anchor five seconds into the past without touching the
        // real clock, so the *next* read has a known, large elapsed value
        // to report -- this is "since program start" and "since the last
        // reset" agreeing, because there has been only one reset so far.
        interp.elapsed_anchor = interp.elapsed_anchor.map(|anchor| anchor - 5_000_000);
        let since_first_reset = dispatch_time_r(&mut interp);
        let since_first_reset = std::str::from_utf8(&since_first_reset)
            .expect("ASCII")
            .parse::<f64>()
            .unwrap();
        assert!(
            (4.9..5.5).contains(&since_first_reset),
            "expected about 5 seconds since the first reset, got {since_first_reset}"
        );

        // That same `TIME('R')` call reset the anchor to *now*, so an
        // immediate follow-up reports elapsed-since-that-reset, not
        // elapsed-since-program-start (which would again read about 5s).
        let since_second_reset = dispatch_time_r(&mut interp);
        let since_second_reset = std::str::from_utf8(&since_second_reset)
            .expect("ASCII")
            .parse::<f64>()
            .unwrap();
        assert!(
            since_second_reset < 0.5,
            "expected well under a second since the second reset, got {since_second_reset}"
        );
    }

    fn dispatch_time_r(interp: &mut Interp) -> Vec<u8> {
        let r = interp.text(b"R");
        let result = super::super::dispatch(interp, b"TIME", &[Some(r)])
            .expect("a builtin name")
            .expect("the call succeeds");
        interp.to_text(result).into_owned()
    }

    /// `TIME('E')` reads the same anchor without resetting it -- the pair
    /// that tells "E reads the state" apart from "E happens to also be the
    /// thing that establishes it".
    #[test]
    fn time_e_does_not_reset_the_anchor_time_r_does() {
        let mut interp = live_interp();
        let e = interp.text(b"E");
        let read = |interp: &mut Interp| {
            let result = super::super::dispatch(interp, b"TIME", &[Some(e)])
                .expect("a builtin name")
                .expect("the call succeeds");
            interp.to_text(result).into_owned()
        };
        let first = read(&mut interp);
        assert_eq!(first, b"0".to_vec());
        // A second `E`, with no clock advance an assertion can see, still
        // reads `0` -- unlike `R`, nothing here moved the anchor.
        let second = read(&mut interp);
        assert_eq!(second, b"0".to_vec());
    }

    // ---- DATE's option letters: the deterministic conversion form ----

    /// Every one of `DATE`'s thirteen output letters, converted from one
    /// fixed input -- `DATE.testGroup`'s own `test_standard` transcript,
    /// unabridged.
    #[test]
    fn date_every_output_letter_from_a_fixed_standard_input() {
        for (probe, expected) in [
            (&b"date('B','20070922','S')"[..], "732940"),
            (b"date('D','20070922','S')", "265"),
            (b"date('E','20070922','S')", "22/09/07"),
            (b"date('E','20070922','S','')", "220907"),
            (b"date('E','20070922','S','-')", "22-09-07"),
            (b"date('L','20070922','S')", "22 September 2007"),
            (b"date('M','20070922','S')", "September"),
            (b"date('N','20070922','S')", "22 Sep 2007"),
            (b"date('N','20070922','S','')", "22Sep2007"),
            (b"date('N','20070922','S','-')", "22-Sep-2007"),
            (b"date('O','20070922','S')", "07/09/22"),
            (b"date('O','20070922','S','')", "070922"),
            (b"date('O','20070922','S','-')", "07-09-22"),
            (b"date('S','20070922','S')", "20070922"),
            (b"date('S','20070922','S','')", "20070922"),
            (b"date('S','20070922','S','-')", "2007-09-22"),
            (b"date('U','20070922','S')", "09/22/07"),
            (b"date('U','20070922','S','')", "092207"),
            (b"date('U','20070922','S','-')", "09-22-07"),
            (b"date('W','20070922','S')", "Saturday"),
            (b"date('I','20070922','S')", "2007-09-22"),
            (b"date('I','20070922','S','')", "20070922"),
            (b"date('I','20070922','S','/')", "2007/09/22"),
        ] {
            assert_eq!(
                output(&[b"say ".as_slice(), probe, b"\n".as_slice()].concat()),
                format!("{expected}\n"),
                "{}",
                String::from_utf8_lossy(probe)
            );
        }
    }

    /// The reverse conversion for every one of the ten **input** letters --
    /// `S` output makes each of them independently checkable, and `date('B',
    /// '1 Jan 0001')` (the zero basedate) and `date('B', '31 Dec 9999')`
    /// (the maximum) pin the two ends of the range [`Timestamp::
    /// set_base_date`] accepts, both measured against the oracle.
    #[test]
    fn date_every_input_letter_round_trips_to_standard() {
        for (probe, expected) in [
            (&b"date('S','732940','B')"[..], "20070922"),
            (b"date('S','265','D')", "20260922"),
            (b"date('S','22/09/07','E')", "20070922"),
            (b"date('S','63326016000000000','F')", "20070922"),
            (b"date('S','2007-09-22','I')", "20070922"),
            (b"date('S','07/09/22','O')", "20070922"),
            (b"date('S','20070922','S')", "20070922"),
            (b"date('S','1190419200','T')", "20070922"),
            (b"date('S','09/22/07','U')", "20070922"),
            (b"date('B','1 Jan 0001')", "0"),
            (b"date('B','31 Dec 9999')", "3652058"),
        ] {
            assert_eq!(
                output(&[b"say ".as_slice(), probe, b"\n".as_slice()].concat()),
                format!("{expected}\n"),
                "{}",
                String::from_utf8_lossy(probe)
            );
        }
    }

    /// `'D'` (day-of-year) is deterministic given a fixed *reference* year,
    /// even though its own default reads "today"'s year: the round trip
    /// D -> D holds identically whichever year "today" is, and `265` on a
    /// day 265 that is neither the first nor the last day of any year
    /// pins the conversion itself rather than depending on which year ran
    /// it.
    #[test]
    fn dates_day_of_year_style_round_trips_regardless_of_the_current_year() {
        assert_eq!(output(b"say date('D','265','D')\n"), "265\n");
        assert_eq!(output(b"say date('D','1','D')\n"), "1\n");
    }

    /// A malformed or out-of-range conversion input is 40.19, never a wrong
    /// answer -- measured against the oracle in every one of these shapes:
    /// an empty input, a day-of-year past even a leap year's own 366, a
    /// basedate one past the maximum, and a basetime that never parses as a
    /// number at all. `367` rather than `366`: whether `366` itself is
    /// valid depends on the real calendar year this test happens to run
    /// in, which [`date_366_is_valid_only_in_a_leap_year`] checks
    /// separately and correctly instead of assuming one answer here.
    ///
    /// [`date_366_is_valid_only_in_a_leap_year`]: date_366_is_valid_only_in_a_leap_year
    #[test]
    fn date_conversion_errors_are_40_19() {
        for (source, expected_style) in [
            (&b"call date , '', 'f'\n"[..], "F"),
            (b"call date 'D', '367', 'D'\n", "D"),
            (b"call date 'B', '3652059', 'B'\n", "B"),
            (b"call date 'B', 'not a number', 'B'\n", "B"),
        ] {
            let (code, stderr) = failure(source);
            assert_eq!(code, 216, "{stderr}");
            assert!(
                stderr.contains(&format!(
                    "is not in the format described by argument 3, \"{expected_style}\""
                )),
                "{stderr}"
            );
        }
    }

    /// `365` is always a valid day-of-year, leap or not -- paired with
    /// [`date_366_is_valid_only_in_a_leap_year`], which is the one that
    /// actually depends on which year "today" is.
    ///
    /// [`date_366_is_valid_only_in_a_leap_year`]: date_366_is_valid_only_in_a_leap_year
    #[test]
    fn date_365_is_always_a_valid_day_of_year() {
        assert_eq!(output(b"say date('D','365','D')\n"), "365\n");
    }

    /// `366` is a valid day-of-year exactly when the real calendar year
    /// this test happens to run in is a leap year -- computed here from the
    /// wall clock rather than assumed, since `DATE('D')`'s own input style
    /// is defined in terms of "today"'s year and this crate cannot pin a
    /// fixed one. The leap-year rule is written out again independently
    /// rather than imported from [`is_leap_year`], because importing it
    /// would only check that this test's expectation matches whatever this
    /// crate happens to compute, not that the boundary is the calendar's
    /// own.
    #[test]
    fn date_366_is_valid_only_in_a_leap_year() {
        let year: i64 = output(b"say date('S')\n")[..4]
            .parse()
            .expect("a 4-digit year");
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        if leap {
            assert_eq!(output(b"say date('D','366','D')\n"), "366\n");
        } else {
            let (code, stderr) = failure(b"say date('D','366','D')\n");
            assert_eq!(code, 216, "{stderr}");
            assert!(
                stderr.contains("is not in the format described by argument 3, \"D\"."),
                "{stderr}"
            );
        }
    }

    /// `DATE`'s thirteen option letters and `TIME`'s eleven, taken from the
    /// error insert rather than from documentation -- measured against the
    /// oracle directly, and the two lists this crate raises with match byte
    /// for byte.
    #[test]
    fn date_and_times_option_lists_match_the_oracles_own_error_insert() {
        let (code, stderr) = failure(b"say date('J')\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("DATE argument 1 must be one of BDEFILMNOSTUW; found \"J\"."),
            "{stderr}"
        );
        let (code, stderr) = failure(b"say time('Z')\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("TIME argument 1 must be one of CEFHLMNORST; found \"Z\"."),
            "{stderr}"
        );
    }

    /// Every one of the thirteen `DATE` output letters is accepted -- Step
    /// 1's "probe every one" -- against a fixed conversion input so none of
    /// the thirteen answers can be the wrong kind of non-deterministic.
    /// Each runs and returns 0; a letter this crate does not implement
    /// would instead raise 40.904.
    #[test]
    fn every_date_output_letter_is_accepted() {
        for letter in DATE_OPTIONS1.bytes() {
            let probe = format!("call date '{}','20070922','S'\n", letter as char);
            let (code, stderr) = failure(probe.as_bytes());
            assert_eq!(code, 0, "letter {}: {stderr}", letter as char);
        }
    }

    /// Every one of the ten `DATE` **input** letters is accepted, each with
    /// a fixed input built for its own format -- the sibling probe to
    /// [`every_date_output_letter_is_accepted`], since a single shared
    /// input string cannot exercise every input style at once.
    #[test]
    fn every_date_input_letter_is_accepted() {
        for (letter, indate) in [
            ('B', "0"),
            ('D', "1"),
            ('E', "01/01/07"),
            ('F', "0"),
            ('I', "2007-01-01"),
            ('N', "1 Jan 2007"),
            ('O', "07/01/01"),
            ('S', "20070101"),
            ('T', "0"),
            ('U', "01/01/07"),
        ] {
            let probe = format!("call date 'N', '{indate}', '{letter}'\n");
            let (code, stderr) = failure(probe.as_bytes());
            assert_eq!(code, 0, "letter {letter}: {stderr}");
        }
    }

    /// The empty option renders as `found ""`, not `found "?"` -- the
    /// spelling `datatype`'s own 93.915 uses for the identical empty-first-
    /// byte situation. The two error families disagree about which
    /// placeholder an empty option gets, and this pins `DATE`/`TIME`'s own
    /// answer rather than assuming datatype's.
    #[test]
    fn the_empty_option_renders_as_the_empty_string_not_a_placeholder() {
        let (code, stderr) = failure(b"say date('')\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("DATE argument 1 must be one of BDEFILMNOSTUW; found \"\"."),
            "{stderr}"
        );
        let (code, stderr) = failure(b"say time('')\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("TIME argument 1 must be one of CEFHLMNORST; found \"\"."),
            "{stderr}"
        );
    }

    // ---- TIME's option letters: the deterministic conversion form ----

    /// `TIME`'s full output alphabet from one fixed input, unabridged
    /// against the brief's own `time('S','12:34:56','N')` probe plus the
    /// remaining eight this brief left unlisted.
    #[test]
    fn time_every_output_letter_from_a_fixed_normal_input() {
        for (probe, expected) in [
            (&b"time('N','12:34:56','N')"[..], "12:34:56"),
            (b"time('S','12:34:56','N')", "45296"),
            (b"time('H','12:34:56','N')", "12"),
            (b"time('M','12:34:56','N')", "754"),
            (b"time('C','12:34:56','N')", "12:34pm"),
            (b"time('C','00:05:00','N')", "12:05am"),
            (b"time('C','13:00:00','N')", "1:00pm"),
        ] {
            assert_eq!(
                output(&[b"say ".as_slice(), probe, b"\n".as_slice()].concat()),
                format!("{expected}\n"),
                "{}",
                String::from_utf8_lossy(probe)
            );
        }
        assert_eq!(
            output(b"say time('L','01:02:03.456789','L')\n"),
            "01:02:03.456789\n"
        );
    }

    /// `TIME`'s remaining input/output letters -- `F` and `T` -- each round
    /// trip through the identical style on both sides: a basetime of
    /// exactly one day's worth of microseconds, and a Unix time of `0`
    /// (the epoch itself).
    #[test]
    fn time_f_and_t_round_trip_through_their_own_style() {
        assert_eq!(
            output(b"say time('F','86400000000','F')\n"),
            "86400000000\n"
        );
        assert_eq!(output(b"say time('T','0','T')\n"), "0\n");
    }

    /// `TIME('O')`'s own input style adjusts the current instant's time
    /// zone offset rather than reading a bare number back -- measured, a
    /// zero offset and a one-hour offset both round-trip to themselves.
    #[test]
    fn time_o_offset_round_trips() {
        assert_eq!(output(b"say time('O','0','O')\n"), "0\n");
        assert_eq!(output(b"say time('O','3600000000','O')\n"), "3600000000\n");
    }

    /// `time('Z')`/`date('C')` and `date('J')`: letters that exist in other
    /// Rexx dialects or documentation but that this build's own error
    /// insert excludes, each raised as an ordinary invalid option rather
    /// than silently accepted.
    #[test]
    fn letters_from_other_dialects_are_rejected_here_too() {
        for (source, expected) in [
            (&b"say date('C')\n"[..], "BDEFILMNOSTUW"),
            (b"say date('J')\n", "BDEFILMNOSTUW"),
        ] {
            let (code, stderr) = failure(source);
            assert_eq!(code, 216, "{stderr}");
            assert!(stderr.contains(expected), "{stderr}");
        }
    }

    // ---- TIME's conversion-form errors ----

    /// A malformed `intime` is 40.19, and `TIME` never supplies a fifth
    /// argument to disagree with `DATE`'s own "argument 2"/"argument 3"
    /// fixed text.
    #[test]
    fn time_conversion_errors_are_40_19() {
        let (code, stderr) = failure(b"call time , '99', 'h'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains(
                "TIME argument 2, \"99\", is not in the format described by argument 3, \"H\"."
            ),
            "{stderr}"
        );
    }

    /// Supplying an `intime` while asking for the elapsed-time output
    /// styles is refused outright, before the input is even parsed --
    /// 40.29, naming the **output** style.
    #[test]
    fn time_r_or_e_with_an_intime_is_refused_up_front() {
        let (code, stderr) = failure(b"call time 'r', '12:00:00'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("TIME conversion to format \"R\" is not allowed."),
            "{stderr}"
        );
        let (code, stderr) = failure(b"call time 'e', '12:00:00'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("TIME conversion to format \"E\" is not allowed."),
            "{stderr}"
        );
    }

    /// `option2` present without `intime` is 40.5 naming argument 2, the
    /// same sub-code `DATE`'s analogous check raises -- paired with the
    /// adjacent success, `option2` present *with* `intime`.
    #[test]
    fn times_option2_requires_intime() {
        let (code, stderr) = failure(b"call time , , 'n'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("Missing argument in invocation of TIME; argument 2 is required."),
            "{stderr}"
        );
        assert_eq!(output(b"say time('S', '00:00:01', 'N')\n"), "1\n");
    }

    // ---- DATE's arity quirk: an interior omission can still be required ----

    /// `date('S',,'S')` -- position 2 omitted, position 3 supplied -- is
    /// 40.5, because supplying `option2` without `indate` is meaningless;
    /// `check_arity`'s own `(min, max)` model cannot express this, so
    /// `date` checks its own positions (`builtin/mod.rs`'s own module doc
    /// names this exact probe). Paired with the adjacent success: `date()`
    /// and `date('S')` both succeed with **no** second argument at all.
    #[test]
    fn dates_option2_without_indate_is_missing_not_omitted() {
        let (code, stderr) = failure(b"call date 'S', , 'S'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("Missing argument in invocation of DATE; argument 2 is required."),
            "{stderr}"
        );
        assert!(output(b"say date()\n").ends_with('\n'));
        assert!(output(b"say date('S')\n").ends_with('\n'));
    }

    // ---- separators ----

    /// A separator argument that is not a single non-alphanumeric byte (nor
    /// the null string) is 40.43, and a style incompatible with any output
    /// separator at all is 40.44 -- the two ways `osep` can be rejected,
    /// each measured against the oracle.
    #[test]
    fn separator_errors_are_40_43_and_40_44() {
        let (code, stderr) = failure(b"call date , , , 'a'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains(
                "DATE argument 4 must be a single non-alphanumeric character or the null string; found \"a\"."
            ),
            "{stderr}"
        );
        let (code, stderr) = failure(b"call date 'b', , , ''\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains(
                "DATE argument 1, \"B\", is a format incompatible with the separator specified in argument 4."
            ),
            "{stderr}"
        );
    }

    /// An input separator incompatible with the input style (`isep` with
    /// `style2` in `BDFLMTW`) is 40.44 naming argument 3 and argument 5 --
    /// the sibling check on the *input* side of the same pair `osep`'s own
    /// test above exercises on the output side.
    #[test]
    fn an_input_separator_incompatible_with_the_input_style_is_40_44() {
        let (code, stderr) = failure(b"call date , '20070922', 'w', , '-'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains(
                "DATE argument 3, \"W\", is a format incompatible with the separator specified in argument 5."
            ),
            "{stderr}"
        );
    }

    /// A parse failure with an input separator supplied reports the
    /// *incompatible-format* shape (40.44, naming the whole `indate` value)
    /// rather than the plain conversion error (40.19) the identical
    /// malformed input gets with no separator at all -- the pair that tells
    /// the two error paths apart.
    #[test]
    fn a_parse_failure_with_an_input_separator_is_40_44_not_40_19() {
        let (code, stderr) = failure(b"call date , '1 May 2022', , , '-'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains(
                "DATE argument 2, \"1 May 2022\", is a format incompatible with the separator specified in argument 5."
            ),
            "{stderr}"
        );
        // The adjacent success is not that same input again: `'1 May
        // 2022'` disagrees only with the *explicit* `-` separator above --
        // under `N`'s own default separator (a blank) it is a
        // perfectly good normal-format date, which is the reason the
        // separator-incompatible path exists at all rather than every
        // wrong separator falling straight through to 40.19. A date that
        // is malformed regardless of separator is what pins "no isep at
        // all is the plain 40.19" instead.
        let (code, stderr) = failure(b"call date , '1 Zzz 2022'\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("is not in the format described by argument 3, \"N\"."),
            "{stderr}"
        );
    }

    // ---- Rexx strings are byte strings ----

    /// A high byte in an option argument round-trips into the error's
    /// `found` field raw -- never through a lossy UTF-8 conversion (the
    /// shared brief's own Task 3 defect, checked directly here rather than
    /// trusted). A control byte does **not** round-trip the same way: the
    /// whole report line, this substitution included, passes through
    /// `error.rs`'s own `displayable` before it is written, which turns
    /// every byte below `0x20` (other than tab/newline/CR) into `?` --
    /// measured directly against the oracle, `call date "01"x` reports
    /// `found "?"`, not the raw byte.
    #[test]
    fn a_high_byte_round_trips_raw_and_a_control_byte_becomes_a_placeholder() {
        let (code, stderr) = failure_bytes(b"call date \"ff\"x\n");
        assert_eq!(code, 216, "{:?}", String::from_utf8_lossy(&stderr));
        assert!(
            stderr.windows(10).any(|w| w == b"found \"\xff\"."),
            "{:?}",
            String::from_utf8_lossy(&stderr)
        );
        let (code, stderr) = failure(b"call date \"01\"x\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(stderr.contains("found \"?\"."), "{stderr}");
    }

    // The null string in an option argument is its own third case -- `found
    // ""`, already checked by
    // `the_empty_option_renders_as_the_empty_string_not_a_placeholder`,
    // above -- completing this alphabet's three required members (a byte
    // `>= 0x80`, a control byte, the null string) across the two tests.

    // ---- D15-adjacent: allocation and rooting ----

    /// A `DATE` result assigned to a variable survives every allocation a
    /// later clause makes, under `run_program_collect_every_alloc` (the
    /// gate's own collect-on-every-allocation mode, `lib.rs`).
    ///
    /// **Not evidence of a defect this task had to guard against.** Every
    /// arm of [`date`] and [`time`] allocates its result exactly once, as
    /// the very last thing it does before returning -- there is no second
    /// allocation in the same call for an earlier one to be swept across,
    /// unlike the compound-`VALUE` shape the shared brief's own rooting
    /// warning names. This test is the general property (a variable's
    /// value stays reachable across further allocations) run against this
    /// module's own code, not a probe for a hazard this module's own shape
    /// does not have.
    #[test]
    fn a_dates_result_survives_the_next_calls_allocations() {
        let outcome = crate::run_program_collect_every_alloc(
            "/t.rex",
            b"n1 = date('S','20070922','S')\nn2 = date('S','20070922','S')\nsay n1 n2\n".to_vec(),
            Invocation::none(),
        );
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"20070922 20070922\n");
    }
}
