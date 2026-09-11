//! `DATE` and `TIME` read the host's local zone, and the witness for that has
//! to pin the zone rather than the clock.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use support::oracle::{Termination, locate};

/// The zones each probe is run under, with why each earns its place.
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
