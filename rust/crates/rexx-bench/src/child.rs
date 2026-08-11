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

//! The two interpreters this repository compares, and the single capped,
//! directory-pinned way of launching either of them.
//!
//! **One definition, every harness.** `src/bin/rexx-bench-suite.rs` produces
//! the committed baseline and `src/bin/rexx-bench-band.rs` produces the
//! between-run distribution the optimisation loop's accept rule is stated
//! against. When those two launched their children differently the two sets of
//! numbers were not interchangeable, and
//! `docs/superpowers/plans/phase-4d-attribution.md` records the consequence:
//! one document's base medians landed between 7.2% under and 0.2% over
//! another's on a byte-identical binary, with part of the movement attributed
//! to the harness difference rather than to the machine. A wrapper defined
//! once cannot drift between callers.

use std::path::{Path, PathBuf};

use crate::arms::Arm;
use crate::timing::{Capture, Completed, time_once};

/// Address-space ceiling applied to **both** sides on **every** axis, in KiB.
///
/// Not this project's usual 1 GiB. Measured 2026-08-08 on this tree: under
/// `ulimit -v 1048576` this crate aborts (SIGABRT, "memory allocation of N
/// bytes failed") on `varlookup`, `compound`, `strings`, `arith` and
/// `samples/rexxcps.rex`, and completes only `startup`. Uncapped, the same
/// four peak at 4.0 GB, 1.06 GB, 4.3 GB and 1.07 GB of resident memory
/// against the oracle's 20 MB on all four. The cap therefore cannot stay at
/// 1 GiB without the measurement covering one axis instead of five.
///
/// 8 GiB rather than no cap at all: the cap exists so a runaway interpreter
/// cannot take the machine's memory with it, which has ended a session here
/// before, and 8 GiB clears every axis on both sides with room to spare
/// (verified before this constant was chosen). Both sides get the same
/// number, so nothing about the comparison is asymmetric.
pub const ADDRESS_SPACE_LIMIT_KIB: u64 = 8 * 1024 * 1024;

/// Root of the built C++ oracle. Hardcoded for the reason
/// `rexx-exec/tests/support/oracle.rs` gives: the point of the comparison is
/// "against *this* build", and an env var would let a different one answer
/// for it with nothing to notice.
pub const ORACLE_ROOT: &str = "/home/moritz/dev/repos/ooRexx/build";

/// One interpreter, and what its child processes need in the environment.
pub struct Side {
    pub label: &'static str,
    pub binary: PathBuf,
    pub env: Vec<(String, String)>,
}

impl Side {
    /// The C++ oracle under [`ORACLE_ROOT`], with the library search path its
    /// launcher needs to find `librexx.so.4`.
    pub fn oracle() -> Side {
        Side {
            label: "oracle",
            binary: PathBuf::from(ORACLE_ROOT).join("bin/rexx"),
            env: vec![(
                "LD_LIBRARY_PATH".to_string(),
                PathBuf::from(ORACLE_ROOT).join("lib").display().to_string(),
            )],
        }
    }

    /// This crate's `rexx-run` at `binary`, on `arm`.
    ///
    /// **The arm is a parameter and there is no constructor without one**,
    /// because the version that had none inherited it. This function used to
    /// hand the child an empty environment, so `REXX_ENGINE` came from
    /// whatever launched the harness and, unset, from `rexx-run`'s own
    /// default -- which means the day that default moved, every binary built
    /// on this constructor changed which engine it measured, with nothing in
    /// its output saying so and a committed baseline to compare against that
    /// had been taken on the other one.
    ///
    /// **The label carries the arm too**, so a row or a message that names
    /// this side names the engine with it. A provenance block is something a
    /// caller has to remember to print; the label is in every line either
    /// way, and `rexx-bench-band`'s reducer branches on `oracle` rather than
    /// on this string, so widening it costs nothing there.
    pub fn rust(binary: PathBuf, arm: Arm) -> Side {
        Side {
            label: match arm {
                Arm::TreeWalker => "rust-tw",
                Arm::Ir => "rust-ir",
            },
            binary,
            env: vec![("REXX_ENGINE".to_string(), arm.engine().to_string())],
        }
    }
}

/// Which `perf stat` events wrap the child, if any.
///
/// **The variant names the events, and the same value both builds the `-e`
/// argument and reads the reply** ([`Counted::events`]). Asking for one pair
/// and parsing another is a reading attributed to the wrong instrument, which
/// nothing downstream could notice: a user-mode cycle count and a total one
/// differ by a few per cent on these axes, not by an order of magnitude.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Counted {
    /// No `perf stat` at all.
    #[default]
    Nothing,
    /// `cycles` and `instructions` over every privilege level.
    Total,
    /// `cycles:u` and `instructions:u`, user mode only.
    User,
}

impl Counted {
    /// The two event names, cycles first, or `None` when nothing is counted.
    pub fn events(self) -> Option<[&'static str; 2]> {
        match self {
            Counted::Nothing => None,
            Counted::Total => Some(["cycles", "instructions"]),
            Counted::User => Some(["cycles:u", "instructions:u"]),
        }
    }
}

/// What sits between `/bin/sh` and the interpreter, beyond the address-space
/// cap and the working directory that every run gets.
///
/// Both fields default to off, so the committed baseline's wrapper is the
/// `Wrapper::default()` one and adding this type changed no number already on
/// the record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Wrapper {
    /// `taskset -c` CPU list. `Some("12,28")` confines the child to one
    /// physical core and its SMT sibling, so a migration between cores stops
    /// being part of the measurement.
    pub pin: Option<String>,
    /// Count retired cycles and instructions with `perf stat`.
    ///
    /// Cycles are the quantity frequency scaling does not touch, which
    /// matters here because this machine exposes no `cpufreq` interface to
    /// fix a governor with -- `/sys` carries no `devices/system/cpu` at all.
    pub counters: Counted,
}

/// Hardware counters `perf stat` reported for one run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counters {
    pub cycles: u64,
    pub instructions: u64,
}

/// One capped, directory-pinned invocation of one side.
///
/// The child's working directory is `workdir` and its address space is capped
/// at [`ADDRESS_SPACE_LIMIT_KIB`]. Both are shell builtins because
/// `std::process::Command` has no rlimit hook and this workspace forbids the
/// `unsafe` `pre_exec` that would give it one.
pub fn run(
    side: &Side,
    program: &Path,
    workdir: &Path,
    wrapper: &Wrapper,
) -> Result<Completed, String> {
    let args = shell_args(&side.binary, program, workdir, wrapper);
    time_once("/bin/sh", &args, &side.env, Capture::Collect)
        .map_err(|error| format!("{} could not be launched: {error}", side.label))
}

/// The argument vector [`run`] hands `/bin/sh`.
///
/// Separated from the spawn so the nesting can be asserted without a machine
/// that has `perf` installed deciding whether the assertion runs.
fn shell_args(binary: &Path, program: &Path, workdir: &Path, wrapper: &Wrapper) -> Vec<String> {
    // `cd` and `ulimit` are shell builtins and there is no rlimit hook on
    // `Command`; `"$@"` keeps the paths out of the shell string so nothing
    // needs quoting.
    let script = r#"cd "$1" || exit 111; ulimit -v "$2" || exit 112; shift 2; exec "$@""#;
    let mut args = vec![
        "-c".to_string(),
        script.to_string(),
        "rexx-bench".to_string(),
        workdir.display().to_string(),
        ADDRESS_SPACE_LIMIT_KIB.to_string(),
    ];
    // Order matters and only one order measures the intended thing:
    // `taskset` must set the affinity mask that `perf` and the interpreter
    // both inherit, and `perf` must be the immediate parent of the
    // interpreter so its counters cover that process and nothing else.
    if let Some(cpus) = &wrapper.pin {
        args.extend(["taskset".to_string(), "-c".to_string(), cpus.clone()]);
    }
    if let Some([cycles, instructions]) = wrapper.counters.events() {
        args.extend(
            ["perf", "stat", "-x,", "-e"]
                .into_iter()
                .map(str::to_string),
        );
        args.push(format!("{cycles},{instructions}"));
    }
    args.push(binary.display().to_string());
    args.push(program.display().to_string());
    args
}

/// The counters `perf stat -x,` wrote to standard error, or `None` when the
/// lines are absent or unreadable.
///
/// The format is one line per event, `value,unit,event,run-time,percent,,`,
/// with `<not counted>` in the value field when the event was multiplexed out.
/// A missing or unparsable value returns `None` for the whole reading rather
/// than a zero: a zero cycle count would be a very fast run.
///
/// **`events` is the pair that was asked for, and the match is exact.** A
/// caller that requested `cycles:u` and received `cycles` has been answered by
/// a different instrument, and reporting that as the reading it asked for is
/// the one error here that no downstream check could see. Callers get the
/// array from [`Counted::events`] rather than writing the names again.
///
/// **A counter that was not enabled for the whole run is refused, and that is
/// not defensiveness.** The fifth field is the percentage of the run the event
/// was scheduled for, and when it is below 100 `perf` reports the count
/// *scaled up* to what it would have been -- an estimate, printed in the same
/// column and the same format as an exact count. On a quantity that is
/// otherwise deterministic to eight significant figures, an estimate is
/// indistinguishable from a real movement of a few per cent, which is larger
/// than most of what this phase measures.
pub fn parse_counters(stderr: &[u8], events: [&str; 2]) -> Option<Counters> {
    let text = String::from_utf8_lossy(stderr);
    let [wanted_cycles, wanted_instructions] = events;
    let mut cycles = None;
    let mut instructions = None;
    for line in text.lines() {
        let mut fields = line.split(',');
        let value = fields.next()?;
        let _unit = fields.next();
        let Some(event) = fields.next() else { continue };
        let Ok(count) = value.trim().parse::<u64>() else {
            continue;
        };
        let event = event.trim();
        if event != wanted_cycles && event != wanted_instructions {
            continue;
        }
        let _run_time = fields.next();
        // Absent rather than below 100 is accepted: `perf` omits the column
        // for a single-event run on some versions, and there is no scaling to
        // hide there. A column that is present and short is refused.
        if let Some(enabled) = fields.next()
            && !enabled.trim().is_empty()
        {
            let Ok(percent) = enabled.trim().parse::<f64>() else {
                return None;
            };
            if percent < 99.995 {
                return None;
            }
        }
        if event == wanted_cycles {
            cycles = Some(count);
        } else {
            instructions = Some(count);
        }
    }
    Some(Counters {
        cycles: cycles?,
        instructions: instructions?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wrapper composes `taskset` outside `perf` outside the interpreter.
    ///
    /// Asserted on the argument vector rather than described, because the
    /// order is the whole content of the claim: `perf` inside `taskset`
    /// inherits the affinity mask and counts only the interpreter, while the
    /// other order would count `taskset` as well and set the mask after the
    /// counters were already attached.
    #[test]
    fn the_wrapper_nests_taskset_outside_perf_outside_the_interpreter() {
        let args = shell_args(
            Path::new("/bin/rexx"),
            Path::new("/programs/arith.rex"),
            Path::new("/work"),
            &Wrapper {
                pin: Some("3,19".to_string()),
                counters: Counted::Total,
            },
        );
        let tail: Vec<&str> = args[5..].iter().map(String::as_str).collect();
        assert_eq!(
            tail,
            [
                "taskset",
                "-c",
                "3,19",
                "perf",
                "stat",
                "-x,",
                "-e",
                "cycles,instructions",
                "/bin/rexx",
                "/programs/arith.rex"
            ]
        );
    }

    /// The events named on the command line are the events [`Counted`] says
    /// it counts, for every variant that counts anything.
    ///
    /// Asserted rather than read off the two sites, because the whole value of
    /// routing both through `Counted::events` is that they cannot drift; a
    /// spelling changed in one place and not the other produces a run counted
    /// on one instrument and parsed as another, and every number downstream
    /// still looks like a number.
    #[test]
    fn the_perf_argument_names_the_events_the_reading_is_parsed_by() {
        for counted in [Counted::Total, Counted::User] {
            let events = counted.events().expect("this variant counts something");
            let args = shell_args(
                Path::new("/bin/rexx"),
                Path::new("/programs/arith.rex"),
                Path::new("/work"),
                &Wrapper {
                    pin: None,
                    counters: counted,
                },
            );
            assert!(
                args.contains(&format!("{},{}", events[0], events[1])),
                "{counted:?} asks perf for something other than {events:?}: {args:?}"
            );
        }
        assert_eq!(Counted::Nothing.events(), None);
    }

    /// A reading is attributed to the instrument that produced it, so
    /// `perf stat -e cycles` cannot answer a request for `cycles:u`.
    ///
    /// The two differ by a few per cent on this crate's axes rather than by an
    /// order of magnitude, so a silent substitution would read as a plausible
    /// measurement rather than as a fault.
    #[test]
    fn a_total_counter_does_not_answer_for_a_user_one() {
        let total = b"200088,,cycles,471330,100.00,,\n143200,,instructions,471330,100.00,,\n";
        assert_eq!(
            parse_counters(total, Counted::Total.events().unwrap()),
            Some(Counters {
                cycles: 200_088,
                instructions: 143_200
            })
        );
        assert_eq!(parse_counters(total, Counted::User.events().unwrap()), None);
        let user = b"200088,,cycles:u,471330,100.00,,\n143200,,instructions:u,471330,100.00,,\n";
        assert_eq!(parse_counters(user, Counted::Total.events().unwrap()), None);
        assert!(parse_counters(user, Counted::User.events().unwrap()).is_some());
    }

    /// A default wrapper adds nothing between the shell and the interpreter,
    /// and this is what keeps the committed baseline's harness unchanged: the
    /// suite's argument vector is byte for byte the one it built before this
    /// module existed.
    #[test]
    fn the_default_wrapper_is_the_bare_one() {
        assert_eq!(
            Wrapper::default(),
            Wrapper {
                pin: None,
                counters: Counted::Nothing
            }
        );
        let args = shell_args(
            Path::new("/bin/rexx"),
            Path::new("/programs/arith.rex"),
            Path::new("/work"),
            &Wrapper::default(),
        );
        assert_eq!(
            args,
            [
                "-c",
                r#"cd "$1" || exit 111; ulimit -v "$2" || exit 112; shift 2; exec "$@""#,
                "rexx-bench",
                "/work",
                "8388608",
                "/bin/rexx",
                "/programs/arith.rex"
            ]
        );
    }

    /// Every arm this crate can measure is named in the child's environment
    /// and in the side's own label, and the two agree.
    ///
    /// **Red if `Side::rust` ever inherits again.** The version that did
    /// passed every test in this crate: an inherited `REXX_ENGINE` is a
    /// correct-looking run of whichever engine `rexx-run` defaults to, and
    /// the day that default moved, `rexx-bench-suite` and `rexx-bench-band`
    /// changed which arm they measured against baselines taken on the other
    /// one. Nothing in either binary's output would have said so, which is
    /// why this asserts the environment rather than the report.
    #[test]
    fn a_rust_side_names_its_arm_in_the_environment_and_in_its_label() {
        for arm in Arm::BOTH {
            let side = Side::rust(PathBuf::from("/bin/true"), arm);
            assert_eq!(
                side.env,
                vec![("REXX_ENGINE".to_string(), arm.engine().to_string())],
                "{} left the engine to whatever launched the harness",
                arm.label()
            );
            assert!(
                side.label.ends_with(arm.label()),
                "the {} arm labels itself `{}`, so a row naming this side does \
                 not name the engine that produced it",
                arm.label(),
                side.label
            );
        }
        assert_ne!(
            Side::rust(PathBuf::from("/bin/true"), Arm::Ir).label,
            Side::rust(PathBuf::from("/bin/true"), Arm::TreeWalker).label,
            "one label for both arms tells a reducer nothing"
        );
    }

    /// The shell really does apply the cap and the directory, and the
    /// interpreter really is the innermost command. The argument-vector pins
    /// above describe an intent; this one observes it.
    #[test]
    fn the_bare_wrapper_runs_the_command_in_the_working_directory() {
        let dir = std::env::temp_dir();
        let completed = run(
            &Side::rust(PathBuf::from("/bin/pwd"), Arm::Ir),
            Path::new("--"),
            &dir,
            &Wrapper::default(),
        )
        .expect("/bin/pwd runs");
        assert!(completed.succeeded(), "{completed:?}");
        let printed = String::from_utf8_lossy(&completed.stdout)
            .trim()
            .to_string();
        assert_eq!(printed, dir.canonicalize().unwrap().display().to_string());
    }

    #[test]
    fn perf_counter_lines_parse() {
        let stderr = b"6665338787,,cycles,2238865636,100.00,,\n\
                       16831183509,,instructions,2238865636,100.00,,\n";
        assert_eq!(
            parse_counters(stderr, Counted::Total.events().unwrap()),
            Some(Counters {
                cycles: 6_665_338_787,
                instructions: 16_831_183_509
            })
        );
    }

    /// A reading missing either event is no reading at all. Without this a
    /// multiplexed-out counter would be reported as a run that took zero
    /// cycles, which reads as the fastest run in the set.
    /// A count `perf` scaled up because the event was not scheduled for the
    /// whole run is not a count.
    ///
    /// The failure this rules out is silent by construction: the scaled figure
    /// is printed in the same column and the same format as an exact one, and
    /// a few per cent of scaling on an instruction count reads exactly like a
    /// change to the program. Observed here as one round in ten reading 2.85%
    /// high on a body whose instruction count is otherwise stable to eight
    /// significant figures.
    #[test]
    fn a_scaled_counter_is_not_a_count() {
        let user = Counted::User.events().unwrap();
        let full = b"200088,,cycles:u,471330,100.00,,\n143200,,instructions:u,471330,100.00,,\n";
        assert!(parse_counters(full, user).is_some());
        let half = b"400176,,cycles:u,235665,50.00,,\n143200,,instructions:u,471330,100.00,,\n";
        assert_eq!(parse_counters(half, user), None);
        let nearly = b"200088,,cycles:u,471330,99.31,,\n143200,,instructions:u,471330,100.00,,\n";
        assert_eq!(parse_counters(nearly, user), None);
        // No percentage column at all is not a scaled reading, so it stays a
        // reading: there is nothing there for `perf` to have scaled by.
        let bare = b"200088,,cycles:u\n143200,,instructions:u\n";
        assert!(parse_counters(bare, user).is_some());
    }

    #[test]
    fn a_missing_counter_is_not_a_zero() {
        let total = Counted::Total.events().unwrap();
        assert_eq!(
            parse_counters(b"6665338787,,cycles,1,100.00,,\n", total),
            None
        );
        assert_eq!(
            parse_counters(
                b"<not counted>,,cycles,,,,\n<not counted>,,instructions,,,,\n",
                total
            ),
            None
        );
        assert_eq!(parse_counters(b"", total), None);
    }
}
