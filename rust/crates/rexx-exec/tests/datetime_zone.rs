//! `DATE` and `TIME` read the host's local zone, and the witness for that has
//! to pin the zone rather than the clock.
//!
//! # Why this is not a `corpus/builtin-probes.txt` row
//!
//! That file's own rule is that **nothing in it may read the clock**, and its
//! `DATE` and `TIME` rows are deliberately conversions of a *given* value
//! (`date('S','2026-08-04','I')`) for exactly that reason. A no-argument
//! reading cannot go there whatever it answers, so the containment argument in
//! `docs/superpowers/plans/phase-4-exclusions.txt` stays true and this file is
//! where the clock-reading half is compared instead.
//!
//! # What makes a wall-clock comparison deterministic anyway
//!
//! The two interpreters are launched seconds apart, so no reading of *now*
//! can be compared byte for byte -- `time('N')` never could be. `TIME('O')`
//! can: it answers the **zone offset**, not the instant, and that is constant
//! across a launch pair for every zone except during the one second a DST
//! transition lands between them. [`ZONES`] is therefore chosen so that most
//! of its entries have no DST at all, and the property under test -- that the
//! crate reads the same zone the oracle does -- is one every entry can show.
//!
//! `TZ` is set on both children. The oracle's `localtime` honours it and so
//! does this crate's own path, and that is the whole claim: two independent
//! implementations, one environment variable, one answer.
//!
//! # The two limbs, and why one probe cannot cover both
//!
//! `SystemInterpreter::getCurrentTime` fills a timestamp's calendar fields
//! *and* its offset; `BuiltinFunctions.cpp:1114` and `:1375` then port that
//! offset onto a timestamp parsed from an input value. Those are separate
//! code paths here and each needs its own line:
//!
//! * `time('O')` reads the clock's own offset -- the first limb.
//! * `time('O','12:34:56','N')` parses an input value, which starts from a
//!   cleared timestamp, and then reads *its* offset -- the second. Measured:
//!   with the port removed this answers `0` while the line above is
//!   unaffected, so a single-line probe would have missed it entirely.
//!
//! `time('O','3600000000','O')` is **not** a witness for the second limb and
//! is here to record that: its input style copies the current timestamp
//! rather than clearing one, so it never reaches the ported assignment and
//! answers `3600000000` either way.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use support::oracle::{Termination, locate};

/// The zones each probe is run under, with why each earns its place.
///
/// `UTC` is the control: it is the answer a crate that ignored the zone
/// entirely would give for every entry, so a run where only `UTC` passes is a
/// run that proves nothing. `Asia/Kolkata` has no DST and a half-hour offset,
/// which catches an implementation carrying whole hours. `Etc/GMT+12` is
/// negative -- the sign convention is the thing most easily inverted, and
/// `local_minus_utc` is positive east of Greenwich where the `Etc` names run
/// the other way. `America/New_York` observes DST, so its offset depends on
/// *when* the probe runs rather than only where.
const ZONES: &[&str] = &["UTC", "Asia/Kolkata", "Etc/GMT+12", "America/New_York"];

/// One line per probe, each a program and what it is the witness for.
const PROBES: &[(&str, &str)] = &[
    ("say time('O')", "the clock's own zone offset"),
    (
        "say time('O','12:34:56','N')",
        "the offset ported onto a parsed input value",
    ),
    (
        "say time('O','3600000000','O')",
        "an input style that copies the current timestamp, reaching neither limb",
    ),
    (
        "say date('S','2026-08-04','I') time('S','12:34:56','N')",
        "the deterministic conversions, which must not move with the zone",
    ),
];

fn probe_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("datetime-zone");
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// This crate's three descriptors for `path` under `zone`, as a subprocess.
///
/// In process would not do: `TZ` is read per process, so the executor would
/// answer under the test binary's own zone whatever this passed it.
fn run_crate(path: &Path, zone: &str) -> (Vec<u8>, Vec<u8>, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(path)
        .env("TZ", zone)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "cannot run this crate on {} under TZ={zone}: {e}",
                path.display()
            )
        });
    (out.stdout, out.stderr, out.status.code().unwrap_or(-1))
}

#[test]
fn date_and_time_read_the_same_zone_the_oracle_does() {
    let oracle = locate();
    let dir = probe_dir();
    let mut answers: Vec<(&str, &str, String)> = Vec::new();

    for (index, (source, subject)) in PROBES.iter().enumerate() {
        let program = dir.join(format!("probe{index}.rex"));
        fs::write(&program, format!("{source}\n"))
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", program.display()));

        for zone in ZONES {
            let cpp = oracle.run_in(&program, &dir, &[("TZ", zone)]);
            assert_eq!(
                cpp.termination,
                Termination::Exited(0),
                "the oracle did not run `{source}` under TZ={zone}, so there is nothing to \
                 compare against: {:?}",
                cpp.termination
            );
            let (stdout, stderr, exit_code) = run_crate(&program, zone);
            assert_eq!(
                (
                    String::from_utf8_lossy(&stdout).into_owned(),
                    String::from_utf8_lossy(&stderr).into_owned(),
                    exit_code
                ),
                (
                    String::from_utf8_lossy(&cpp.stdout).into_owned(),
                    String::from_utf8_lossy(&cpp.stderr).into_owned(),
                    0
                ),
                "`{source}` ({subject}) under TZ={zone}: this crate and the oracle differ"
            );
            answers.push((source, zone, String::from_utf8_lossy(&stdout).into_owned()));
        }
    }

    // **The comparison above passes for a crate that reads no zone at all**,
    // as long as the oracle reads none either -- and under `TZ=UTC` neither
    // does. So the run is only worth something if the answers actually moved
    // between zones, and that is asserted rather than assumed: without this a
    // machine with no time zone database would report every row matching and
    // look exactly like a machine where the feature works.
    let clock_offsets: Vec<&String> = answers
        .iter()
        .filter(|(source, _, _)| *source == PROBES[0].0)
        .map(|(_, _, answer)| answer)
        .collect();
    let distinct: std::collections::BTreeSet<&&String> = clock_offsets.iter().collect();
    assert!(
        distinct.len() > 1,
        "every zone answered the same offset {clock_offsets:?}, so this run cannot tell a crate \
         that reads the zone from one that ignores it"
    );

    // The deterministic conversions are the other half of the same argument:
    // they must be *insensitive* to the zone, or the fix reached further than
    // the clock and `corpus/builtin-probes.txt`'s two rows are no longer the
    // deterministic ones that file requires.
    let conversions: std::collections::BTreeSet<&String> = answers
        .iter()
        .filter(|(source, _, _)| *source == PROBES[3].0)
        .map(|(_, _, answer)| answer)
        .collect();
    assert_eq!(
        conversions.len(),
        1,
        "the fixed-input conversions moved with the zone: {conversions:?}. \
         `corpus/builtin-probes.txt` requires them to be deterministic"
    );
}
