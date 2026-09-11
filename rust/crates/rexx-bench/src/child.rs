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

use std::path::{Path, PathBuf};

use crate::arms::Arm;
use crate::timing::{Capture, Completed, time_once};

/// Address-space ceiling applied to **both** sides on **every** axis, in KiB.
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
    pub fn rust(binary: PathBuf, arm: Arm) -> Side {
        Side {
            label: match arm {
                Arm::Ir => "rust-ir",
            },
            binary,
            // **No environment.** The arm used to be selected per child
            // through `REXX_ENGINE`; `rexx-run` reads no such variable now,
            // so setting one would be a row labelled with an arm nobody
            // chose.
            env: Vec::new(),
        }
    }
}

/// Which `perf stat` events wrap the child, if any.
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Wrapper {
    /// `taskset -c` CPU list. `Some("12,28")` confines the child to one
    /// physical core and its SMT sibling, so a migration between cores stops
    /// being part of the measurement.
    pub pin: Option<String>,
    /// Count retired cycles and instructions with `perf stat`.
    pub counters: Counted,
}

/// Hardware counters `perf stat` reported for one run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counters {
    pub cycles: u64,
    pub instructions: u64,
}

/// One capped, directory-pinned invocation of one side.
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
    /// A rust side sets **no** environment, and says which arm it is in its
    /// label.
    #[test]
    fn a_rust_side_sets_no_engine_environment_and_names_its_arm() {
        for arm in Arm::BOTH {
            let side = Side::rust(PathBuf::from("/bin/true"), arm);
            assert!(
                side.env.is_empty(),
                "a rust side set {:?}, which selects an engine that no longer \
                 exists and would label the row with an arm nobody ran",
                side.env
            );
            assert!(
                side.label.ends_with(arm.label()),
                "the {} arm labels itself `{}`, so a row naming this side does \
                 not name the engine that produced it",
                arm.label(),
                side.label
            );
        }
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
