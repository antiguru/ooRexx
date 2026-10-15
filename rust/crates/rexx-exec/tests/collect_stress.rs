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

//! The 4a exit gate's criterion 4: the named L0 subset -- the union of every
//! `rust/corpus/phase-*.txt` file, which is what [`read_subset`]'s caller
//! reads -- passes again under collect-on-every-allocation, and the mode is
//! proved to do something before its pass is believed.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{run_program, run_program_collect_every_alloc};
use support::sidecar::{prepare_run_directory, resolved_environment, sidecar_for};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program. Duplicated from `corpus.rs`/`coverage.rs` rather
/// than shared: see either file's own module doc for why an integration
/// test cannot `mod` another test binary.
fn read_subset(list_paths: &[&Path]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut union = Vec::new();
    for list_path in list_paths {
        let text = fs::read_to_string(list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if seen.insert(line.to_string()) {
                union.push(line.to_string());
            }
        }
    }
    union
}

/// The subset files this harness reads, in union order.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
    "phase-5b.txt",
    "phase-5c.txt",
    "phase-5d.txt",
    "phase-5j.txt",
    "phase-6-1.txt",
    "phase-6.txt",
    "phase-7.txt",
    "phase-8.txt",
    "phase-9.txt",
];

/// The directory one corpus program runs in: its own, under the target
/// directory, named by the program's corpus path with `/` spelled `__`. That
/// spelling collapses the path to a single component, so no entry can name a
/// directory outside this root.
fn run_directory(rel_path: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("collect-stress-run")
        .join(rel_path.replace('/', "__"))
}

/// The programs that allocate nothing under the stress mode, so that one losing
/// an allocation, or gaining one, is a decision rather than a silent change.
/// Re-derive it by running this test binary: the assertion below prints the set
/// it observed beside this list whenever the two differ, and a program added to
/// the corpus that collects nothing belongs here.
const NO_ALLOCATION_PROGRAMS: &[&str] = &[
    "lang/call_trace_nested.rex",
    "lang/comparison_families.rex",
    "lang/comparison_operators_remaining.rex",
    "lang/deep_nested_expr.rex",
    "lang/do_counter_ctrl.rex",
    "lang/do_counter_forever.rex",
    "lang/do_counter_rep.rex",
    "lang/do_counter_until.rex",
    "lang/do_counter_while.rex",
    "lang/do_ctrl_class.rex",
    "lang/do_loop_forms.rex",
    "lang/do_to_class.rex",
    "lang/exit_no_value.rex",
    "lang/exit_with_value.rex",
    "lang/if_else_chain.rex",
    "lang/iterate_from_select.rex",
    "lang/loop_counter.rex",
    "lang/loop_label_counter.rex",
    "lang/message_assignment_form.rex",
    "lang/message_instruction.rex",
    "lang/message_send_scope_override.rex",
    "lang/message_send_unknown_method.rex",
    "lang/message_send_unknown_method_on_a_number.rex",
    "lang/message_send_unknown_method_on_nil.rex",
    "lang/mutation_controlled_order.rex",
    "lang/no_trailing_newline.rex",
    "lang/options.rex",
    "lang/scope_case_text_absorbed.rex",
    "lang/scope_case_text_absorbed_other.rex",
    "lang/select_when.rex",
    "lang/select_when_absorption.rex",
    "lang/select_when_bodies.rex",
    "lang/string_caseless.rex",
    "lang/string_compare.rex",
    "lang/trace_numeric_request.rex",
    "lang/trace_output.rex",
    "lang/trace_results.rex",
    "num/comparison.rex",
];

/// The programs whose stdout depends on when a collection happens, which is
/// not an observable the oracle defines (ruled 2026-09-01): each loads a
/// package nothing holds, which the requires cache holds weakly, and loads it
/// again. Measured on the oracle with a loop of `copies('y', 100000)` between
/// the two loads: `library_package_retried.rex` prints `its class The K3
/// class` where it stands at `its class .K3`, and
/// `requires_manager_rename.rex` prints `second load 1 4 events` where it
/// stands at `2 events`. Each still exits and writes stderr as its plain run
/// does.
const COLLECTION_TIMED_PROGRAMS: &[&str] = &[
    "lang/library_package_retried.rex",
    "lang/requires_manager_rename.rex",
];

/// The phase subset files that exist in the corpus directory, sorted.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = corpus_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    names.sort();
    names
}

/// The stress run reads **every** phase subset file the corpus has.
#[test]
fn the_stress_subset_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the collect-on-every-allocation run does not read every phase subset \
         file in rust/corpus/. A file missing from SUBSET_FILES is a phase \
         whose programs never reach this harness, and nothing else in the \
         workspace can see that -- the run passes over whatever it was given"
    );
}

#[test]
fn the_l0_subset_passes_again_under_collect_on_every_allocation() {
    let corpus_dir = corpus_dir();
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    assert!(
        !subset.is_empty(),
        "the phase subset files named no programs -- that is a corpus defect, \
         not an empty pass"
    );

    for timed in COLLECTION_TIMED_PROGRAMS {
        assert!(
            subset.iter().any(|path| path == timed),
            "{timed} is not in the subset"
        );
    }

    let mut total_collections: u64 = 0;
    let mut mismatches = Vec::new();
    let mut zero_collection_programs = Vec::new();

    for rel_path in &subset {
        let abs = corpus_dir.join(rel_path);
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let path_str = abs
            .to_str()
            .unwrap_or_else(|| panic!("corpus path {} is not valid UTF-8", abs.display()));

        // A directory of the program's own, emptied between the two runs: a
        // program that writes files must not have the second run read what
        // the first left, or its own output differs for a reason that has
        // nothing to do with collecting on every allocation. Each run gets the
        // fixtures, environment and input the corpus differential gives it.
        let dir = run_directory(rel_path);
        let sidecar = sidecar_for(&corpus_dir, rel_path);
        let overrides = resolved_environment(&sidecar, &dir);
        let stdin = sidecar.stdin.as_deref();
        let cwd = prepare_run_directory(&dir, &sidecar);
        let plain = run_program(
            path_str,
            text.clone(),
            support::sidecar::invocation(&cwd, &overrides, stdin)
                .with_deadline(support::oracle::RUN_DEADLINE),
        );
        let cwd = prepare_run_directory(&dir, &sidecar);
        let stress = run_program_collect_every_alloc(
            path_str,
            text,
            support::sidecar::invocation(&cwd, &overrides, stdin)
                .with_deadline(support::oracle::RUN_DEADLINE * 10),
        );

        if stress.collections == 0 {
            zero_collection_programs.push(rel_path.clone());
        }
        total_collections += stress.collections;

        let timed = COLLECTION_TIMED_PROGRAMS.contains(&rel_path.as_str());
        if plain.exit_code != stress.exit_code
            || (plain.stdout != stress.stdout && !timed)
            || plain.stderr != stress.stderr
        {
            mismatches.push(format!(
                "{rel_path}: plain exit={} stdout={:?} stderr={:?}; \
                 stress exit={} stdout={:?} stderr={:?}",
                plain.exit_code,
                String::from_utf8_lossy(&plain.stdout),
                String::from_utf8_lossy(&plain.stderr),
                stress.exit_code,
                String::from_utf8_lossy(&stress.stdout),
                String::from_utf8_lossy(&stress.stderr),
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "the subset diverges under collect-on-every-allocation:\n{}",
        mismatches.join("\n")
    );

    // Anti-vacuity device 1: a mode that never collects would pass the
    // check above by construction (nothing to sweep, nothing to diverge),
    // exactly the defect this criterion was rewritten to close. Checked
    // per program, not only in aggregate, so one silent program cannot
    // hide behind the rest of the subset's counts.
    let mut observed: Vec<&str> = zero_collection_programs
        .iter()
        .map(String::as_str)
        .collect();
    observed.sort_unstable();
    let mut expected: Vec<&str> = NO_ALLOCATION_PROGRAMS.to_vec();
    expected.sort_unstable();
    assert_eq!(
        observed, expected,
        "the set of programs performing zero collections under the stress \
         mode has drifted from the committed list; a program that gained an \
         allocation and one that lost one are both worth deciding about"
    );
    assert!(
        total_collections > 0,
        "collect-on-every-allocation performed zero collections across the \
         whole subset -- the mode is a no-op and this criterion cannot tell \
         a real pass from a vacuous one"
    );
}

/// **Fix round 4's NEW-4.** The value a clause resolves to is rooted across
/// the `CALL ON` handler its boundary runs, and *this* is what fails when
/// that rooting goes: the corpus subset above does not contain the shape,
/// which is why deleting the `push_temp` left 970 tests and this whole file
/// green while panicking on `a live value` under a hand-run program.
#[test]
fn a_clause_value_survives_the_handler_its_boundary_runs() {
    let rows: [(&str, &str, &str); 3] = [
        (
            "return-at-an-activation-boundary",
            "call on user foo name uh\nzmark = 'NOMARK'\nzr = aa()\nsay zr zmark\nexit\naa:\nreturn bb() || 'TAIL'\nbb:\nraise user foo return 'HEAD'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            "HEADTAIL HANDLER-AT 7\n",
        ),
        (
            "the same inside a DO body",
            "call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to 1\nzr = aa()\nend\nsay zr zmark\nexit\naa:\nreturn bb() || 'TAIL'\nbb:\nraise user foo return 'HEAD'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            "HEADTAIL HANDLER-AT 9\n",
        ),
        (
            "EXIT rather than RETURN",
            "call on user foo name uh\nzmark = 'NOMARK'\nsay 'go'\nexit bb() || 'TAIL'\nbb:\nraise user foo return 'HEAD'\nuh:\nzmark = 'HANDLER-AT' sigl\nsay 'handler' zmark\nreturn\n",
            "go\nhandler HANDLER-AT 4\n",
        ),
    ];
    let mut total_collections: u64 = 0;
    for (name, text, expected) in rows {
        let path = format!("<clause-value-rooting: {name}>");
        let stress = run_program_collect_every_alloc(
            &path,
            text.as_bytes().to_vec(),
            rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            expected,
            "{name}, under collect-on-every-allocation"
        );
        // Anti-vacuity: a program that never collects cannot observe a
        // dropped root, so a green row with zero collections would prove
        // nothing at all.
        assert!(
            stress.collections > 0,
            "{name} performed zero collections, so it cannot see a dropped root"
        );
        total_collections += stress.collections;
    }
    assert!(total_collections > 0);
}

/// A loop driven by message keeps its `TO` and `BY` across the passes after
/// its control became an object, while each `+` and comparison allocates.
#[test]
fn a_message_driven_loops_to_and_by_survive_collect_on_every_allocation() {
    let path = "lang/do_object_bounds_rooted.rex";
    let text = std::fs::read(corpus_dir().join(path)).expect("the witness is in the corpus");
    let stress = run_program_collect_every_alloc(
        path,
        text,
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        "end k10 7\ni k1\ni k3.5\ni k6.0\ni k8.5\nend k11.0\n",
        "stderr {:?}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert!(
        stress.collections > 0,
        "a run that never collects cannot see a dropped root"
    );
}

/// The command line's argument string survives every allocation the program
/// makes.
#[test]
fn a_command_line_argument_survives_collect_on_every_allocation() {
    let text = "do i = 1 to 20\n  zj = 'filler' i\nend\nparse arg zp\nsay '['zp']'\n";
    let stress = run_program_collect_every_alloc(
        "<argument-rooting>",
        text.as_bytes().to_vec(),
        rexx_exec::Invocation::with_argument(b"the-argument".to_vec()),
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        "[the-argument]\n",
        "the argument string did not survive collect-on-every-allocation"
    );
    // Anti-vacuity: a run that never collected cannot observe a dropped root.
    assert!(
        stress.collections > 0,
        "zero collections, so this row cannot see a dropped root"
    );
}

/// `VALUE`'s compound-write path (`builtin/datatype.rs`) holds the *old*
/// value across the allocation its own write performs.
#[test]
fn values_compound_write_roots_the_old_value_before_the_stems_first_allocation() {
    let rows: [(&str, &str, &str); 4] = [
        (
            "a direct compound assignment, no VALUE involved",
            "j=3\na.j='new'\nsay a.3\n",
            "new\n",
        ),
        (
            "a stem already carrying a default, read directly",
            "s.='d'\nsay s.9\n",
            "d\n",
        ),
        // **The stem name is long on purpose.** These two rows need the
        // derived compound name to be a value the heap actually holds, and a
        // string short enough travels in the handle with no slot and no
        // allocation -- which leaves the stress mode nothing to collect on
        // and the row unable to see the defect it exists for.
        (
            "VALUE's read-only form on the same never-touched compound",
            "j=3\nsay value('abcdefgh.j')\n",
            "ABCDEFGH.3\n",
        ),
        (
            "the failing shape: VALUE's write on a never-touched compound",
            "j=3\nsay value('abcdefgh.j','new')\n",
            "ABCDEFGH.3\n",
        ),
    ];
    let mut total_collections: u64 = 0;
    for (name, text, expected) in rows {
        let path = format!("<value-compound-write-rooting: {name}>");
        let stress = run_program_collect_every_alloc(
            &path,
            text.as_bytes().to_vec(),
            rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            expected,
            "{name}, under collect-on-every-allocation"
        );
        assert!(
            stress.collections > 0,
            "{name} performed zero collections, so it cannot see a dropped root"
        );
        total_collections += stress.collections;
    }
    assert!(total_collections > 0);
}

/// `VALUE(name, new, '')` (`builtin/platform.rs`) holds the *old* value across
/// the allocation its write into `.environment`'s store performs.
#[test]
fn values_environment_write_roots_the_old_value_before_the_stores_allocation() {
    // The name is long for the compound row's reason: the old value of an
    // absent name is its `.NAME` text, which has to be a heap string for a
    // collection to be able to free it.
    let rows: [(&str, &str, &str); 5] = [
        (
            "VALUE's read-only form on an absent name",
            "say value('ABCDEFGHIJKLMNOP', , '')\n",
            ".ABCDEFGHIJKLMNOP\n",
        ),
        (
            "a write to a name .environment already holds",
            ".environment['ABCDEFGHIJKLMNOP'] = 'one'\nsay value('ABCDEFGHIJKLMNOP', 'two', '')\n",
            "one\n",
        ),
        (
            "the failing shape: VALUE's write on an absent name",
            "say value('ABCDEFGHIJKLMNOP', 'new', '')\nsay value('ABCDEFGHIJKLMNOP', , '')\n",
            ".ABCDEFGHIJKLMNOP\nnew\n",
        ),
        (
            "a new value that is itself a heap string built in the call",
            "say value('ABCDEFGHIJKLMNOP', copies('NEWVALUE', 3), '')\nsay value('ABCDEFGHIJKLMNOP', , '')\n",
            ".ABCDEFGHIJKLMNOP\nNEWVALUENEWVALUENEWVALUE\n",
        ),
        (
            "a .local entry written by a mint-free send, then read as .NAME",
            ".local~put(copies('LOCALVALUE', 3), 'ABCDEFGHIJKLMNOP')\nsay .abcdefghijklmnop\n",
            "LOCALVALUELOCALVALUELOCALVALUE\n",
        ),
    ];
    for (name, text, expected) in rows {
        let path = format!("<value-environment-write-rooting: {name}>");
        let stress = run_program_collect_every_alloc(
            &path,
            text.as_bytes().to_vec(),
            rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            expected,
            "{name}, under collect-on-every-allocation"
        );
        assert!(
            stress.collections > 0,
            "{name} performed zero collections, so it cannot see a dropped root"
        );
    }
}

/// A loop's own per-pass roots survive the pass they belong to.
#[test]
fn a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop() {
    /// The fourth column is whether the row allocates at all. A counted loop
    /// over small integers does not -- `Interp::literal` and the tagged
    /// control value keep every one of its values off the heap -- so it can
    /// have no rooting defect and asserting it collects would be asserting
    /// something false. Stated per row and in both directions, like
    /// [`NO_ALLOCATION_PROGRAMS`] above: a row that stops allocating has had
    /// its subject taken away, and a row that starts has gained one.
    struct Row {
        name: &'static str,
        program: &'static str,
        stdout: &'static str,
        allocates: bool,
    }
    let rows = [
        Row {
            name: "a counted loop over small integers, which allocates nothing per pass",
            program: "do i = 1 to 3\n  say i\nend\n",
            stdout: "1\n2\n3\n",
            allocates: false,
        },
        Row {
            name: "a WHILE test whose value is a heap object, with an allocating body",
            // Wide enough to need a slot: a shorter concatenation is carried
            // in the handle, and then the body this row is named for does not
            // allocate at all.
            program: "k = 0\ndo while k < 3\n  k = k + 1\n  zz = 'xxxxxxxx' || k\nend\nsay zz k\n",
            stdout: "xxxxxxxx3 3\n",
            allocates: true,
        },
        Row {
            name: "the same for UNTIL",
            // Wide for the reason the WHILE row above gives.
            program: "n = 0\ndo until n >= 2\n  n = n + 1\n  q = n || 'pppppppp'\nend\nsay q n\n",
            stdout: "2pppppppp 2\n",
            allocates: true,
        },
        Row {
            name: "a compound control whose values are still small integers",
            program: "j = 2\ndo cv.j = 1 to 3\n  say cv.j\nend\n",
            stdout: "1\n2\n3\n",
            allocates: true,
        },
        Row {
            name: "the failing shape: a compound control stepped by a fraction",
            program: "j = 7\ndo cv.j = 1.5 to 2.5 by 0.5\n  say cv.j\nend\nsay cv.7\n",
            stdout: "1.5\n2.0\n2.5\n3.0\n",
            allocates: true,
        },
    ];
    for row in rows {
        let path = format!("<loop-pass-rooting: {}>", row.name);
        let stress = run_program_collect_every_alloc(
            &path,
            row.program.as_bytes().to_vec(),
            rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            row.stdout,
            "{}, under collect-on-every-allocation",
            row.name
        );
        assert_eq!(
            stress.collections > 0,
            row.allocates,
            "{} collected {} times, against the row's own claim that it \
             allocates: {}",
            row.name,
            stress.collections,
            row.allocates
        );
    }
}

/// The rest of a method body a `REPLY` moved to a new activity still reads
/// its own variables and arguments after the routine that sent it has
/// returned, with a collection at every allocation and switch in between.
#[test]
fn a_parked_reply_keeps_its_variables_across_a_collection() {
    let program = concat!(
        "call send\n",
        "sender = 'bbbbbbbbbbbbbbbb' || 'cccccccccccccccc'\n",
        "do until .local~done == 1\n",
        "  call SysSleep 0.01\n",
        "end\n",
        "say sender\n",
        "exit\n",
        "send: procedure\n",
        "  say .K~m('aaaaaaaa' || 'aaaaaaaa')\n",
        "  return\n",
        "::class K\n",
        "::method m class\n",
        "  expose held\n",
        "  local = 'dddddddddddddddd' || 'eeeeeeeeeeeeeeee'\n",
        "  held = 'ffffffffffffffff' || 'gggggggggggggggg'\n",
        "  reply 'replied'\n",
        "  call SysSleep 0.05\n",
        "  say local\n",
        "  say held\n",
        "  say arg(1)\n",
        "  .local~done = 1\n",
        "  return\n",
    );
    let expected = concat!(
        "replied\n",
        "ddddddddddddddddeeeeeeeeeeeeeeee\n",
        "ffffffffffffffffgggggggggggggggg\n",
        "aaaaaaaaaaaaaaaa\n",
        "bbbbbbbbbbbbbbbbcccccccccccccccc\n",
    );
    let stress = run_program_collect_every_alloc(
        "<parked-reply-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(stress.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        expected,
        "a moved REPLY continuation lost a value under collect-on-every-allocation"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A `REPLY` continuation that has yet to run keeps the values its moved
/// registers hold, here the array a `DO OVER` walks, while its sender
/// allocates with a collection at every allocation.
#[test]
fn a_pending_reply_keeps_its_registers_across_a_collection() {
    let program = concat!(
        "say .K~m\n",
        "x = ''\n",
        "do i = 1 to 20\n",
        "  x = x || 'zzzzzzzzzzzzzzzz' || i\n",
        "end\n",
        "do until .local~done == 1\n",
        "  call SysSleep 0.01\n",
        "end\n",
        "exit\n",
        "::class K\n",
        "::method m class\n",
        "  do w over ('alpha' || 'alphaalpha beta' || 'betabetabeta')~makeArray(' ')\n",
        "    if w~left(1) == 'a' then reply 'replied'\n",
        "    say w\n",
        "  end\n",
        "  .local~done = 1\n",
    );
    let stress = run_program_collect_every_alloc(
        "<pending-reply-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        stress.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        "replied\nalphaalphaalpha\nbetabetabetabeta\n",
        "a pending REPLY continuation lost a register value under collect-on-every-allocation"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// The stream builtins keep the stream and the argument they send to it
/// while the send's body takes a pinned yield, here on a `REPLY`
/// continuation under the switch mode with a collection at every allocation.
#[test]
fn a_stream_builtins_values_survive_a_pinned_yield() {
    let program = concat!(
        "say .k~new~m\n",
        "call SysSleep 0.1\n",
        "say 'main end'\n",
        "::class k\n",
        "::method m\n",
        "  reply 'r'\n",
        "  x = linein('/nonexistent/dir/file')\n",
        "  say 'after' length(x)\n",
        "  y = stream('/nonexistent/dir/file', 'c', 'query exists')\n",
        "  say 'q' length(y) lines('/nonexistent/dir/file', 'C')\n",
        "  return\n",
    );
    let stress = run_program_collect_every_alloc(
        "<stream-pinned-yield>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none()
            .with_deadline(support::oracle::RUN_DEADLINE * 10)
            .with_switch_mode(rexx_exec::SwitchMode::EveryOpportunity),
    );
    assert_eq!(
        stress.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        "r\nafter 0\nq 0 0\nmain end\n"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A started activity parked on `~result` keeps the values its registers
/// hold, in the level that parked and in the level below it, while the
/// activity that completes the message allocates, with a collection at every
/// allocation.
#[test]
fn a_parked_activitys_registers_survive_the_activity_that_wakes_it() {
    let program = concat!(
        "m = .message~new(.j~new, 'SECOND')\n",
        "a = .k~new~start('FIRST', m)\n",
        "m~start\n",
        "say a~result\n",
        "::class k\n",
        "::method first\n",
        "  use arg m\n",
        "  p = 'aaaaaaaaaaaaaaaa'\n",
        "  q = 'bbbbbbbbbbbbbbbb'\n",
        "  return (p || q) || self~inner(m)\n",
        "::method inner\n",
        "  use arg m\n",
        "  p = 'cccccccccccccccc'\n",
        "  q = 'dddddddddddddddd'\n",
        "  return (p || q) || m~result\n",
        "::class j\n",
        "::method second\n",
        "  s = ''\n",
        "  do i = 1 to 20\n",
        "    s = s || 'eeeeeeeeeeeeeeee'\n",
        "  end\n",
        "  return length(s)\n",
    );
    let expected = "aaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbccccccccccccccccdddddddddddddddd320\n";
    let stress = run_program_collect_every_alloc(
        "<parked-activity-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        stress.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        expected,
        "a parked activity lost a value under collect-on-every-allocation"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A pinned wait sets its activity aside while the activity it waits on runs
/// on the same Rust stack: the values the pinned activity's registers hold
/// survive that activity's allocations, with a collection at every
/// allocation.
#[test]
fn a_pinned_activitys_registers_survive_the_activity_its_wait_runs() {
    let program = concat!(
        "m = .message~new(.j~new, 'SECOND')\n",
        "m~start\n",
        "a = .array~of(2, 1)\n",
        "a~sortWith(.c~new(m))\n",
        "say a~toString('L', ',')\n",
        "::class c\n",
        "::method init\n",
        "  expose m\n",
        "  use arg m\n",
        "::method compare\n",
        "  expose m\n",
        "  use arg l, r\n",
        "  p = 'aaaaaaaaaaaaaaaa'\n",
        "  q = 'bbbbbbbbbbbbbbbb'\n",
        "  say (p || q) || m~result\n",
        "  return l - r\n",
        "::class j\n",
        "::method second\n",
        "  s = ''\n",
        "  do i = 1 to 20\n",
        "    s = s || 'eeeeeeeeeeeeeeee'\n",
        "  end\n",
        "  return length(s)\n",
    );
    let expected = "aaaaaaaaaaaaaaaabbbbbbbbbbbbbbbb320\n1,2\n";
    let stress = run_program_collect_every_alloc(
        "<pinned-activity-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        stress.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        expected,
        "a pinned activity lost a value under collect-on-every-allocation"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A running activation's `RexxContext` survives a collection, with one at
/// every allocation between the two sends that reach it.
#[test]
fn a_running_activation_keeps_its_context_object_across_a_collection() {
    let program = concat!(
        "a = .context~objectName\n",
        "say a\n",
        "b = 'xxxxxxxxxxxxxxxx' || 'yyyyyyyyyyyyyyyy'\n",
        "say b\n",
        "say .context~objectName\n",
    );
    let expected = concat!(
        "a RexxContext\n",
        "xxxxxxxxxxxxxxxxyyyyyyyyyyyyyyyy\n",
        "a RexxContext\n",
    );
    let stress = run_program_collect_every_alloc(
        "<running-context-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(stress.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        expected,
        "a running activation lost its context object under \
         collect-on-every-allocation"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A method body that assigns over `SELF` still reads its exposed variables
/// back, with a collection at every allocation in between.
#[test]
fn a_method_that_assigns_over_self_keeps_its_exposed_variables() {
    let program = concat!(
        "o = .K~new\n",
        "say 'a' o~go\n",
        "::CLASS K\n",
        "::METHOD init\n",
        "  expose kept\n",
        "  self = 'clobbered-in-init'\n",
        "  kept = 'kept-' || copies('yz', 30)\n",
        "::METHOD go\n",
        "  expose kept\n",
        "  self = 'clobbered'\n",
        "  t = ''\n",
        "  do i = 1 to 100\n",
        "    t = t || i\n",
        "  end\n",
        "  return kept length(t) self\n",
    );
    let expected = concat!(
        "a kept-",
        "yzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyzyz",
        " 192 clobbered\n",
    );
    let stress = run_program_collect_every_alloc(
        "<self-reassigned-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(stress.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        expected,
        "an instance lost its variable pools under collect-on-every-allocation"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A weak reference does not keep its referent alive, and keeps answering one
/// that something else does.
#[test]
fn a_weak_reference_clears_only_when_its_referent_becomes_unreachable() {
    let program = concat!(
        "dropped = .Object~new\n",
        "wd = .WeakReference~new(dropped)\n",
        "held = .Object~new\n",
        "wh = .WeakReference~new(held)\n",
        "drop dropped\n",
        "do i = 1 to 20\n",
        "  zj = 'filler' i\n",
        "end\n",
        "say 'dropped' (wd~value == .nil)\n",
        "say 'held' (wh~value == held) wh~value~class~id\n",
        // The referent arrives as an argument with no other handle on it, and
        // building the cell allocates -- so the collect this line's own
        // allocations run is between the referent's creation and the read.
        "say 'temp' (.WeakReference~new(.Object~new)~value == .nil)\n",
    );
    let stress = run_program_collect_every_alloc(
        "<weak-reference-clearing>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(stress.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        "dropped 1\nheld 1 Object\ntemp 0\n"
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A package copy nothing holds is collected and its weak reference clears,
/// while a held copy stays, as on the oracle (L7's copy half): measured,
/// oracle rc 0, `dropped 1` and `held 1` for the same program with a 100-pass
/// loop of `copies('y', 100000)` in place of the filler.
#[test]
fn a_dropped_package_copy_is_collected() {
    let program = concat!(
        "w = .WeakReference~new(.context~package~copy)\n",
        "h = .context~package~copy\n",
        "v = .WeakReference~new(h)\n",
        "do i = 1 to 20\n",
        "  zj = 'filler' i\n",
        "end\n",
        "say 'dropped' (w~value == .nil)\n",
        "say 'held' (v~value == h)\n",
    );
    let stress = run_program_collect_every_alloc(
        "<package-copy-collected>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        (
            stress.exit_code,
            String::from_utf8_lossy(&stress.stdout).as_ref()
        ),
        (0, "dropped 1\nheld 1\n"),
        "stderr: {}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// Runs `program` collecting at every allocation and asserts it exited 0
/// having printed `expected`, and that it collected.
fn assert_stress_prints(label: &str, program: &str, expected: &str) {
    assert_stress_run_prints(label, program, rexx_exec::Invocation::none(), expected);
}

/// [`assert_stress_prints`] for a program that calls external routines: each
/// of `files` is written into a directory of the label's own, which the
/// program runs in.
fn assert_stress_prints_beside(label: &str, files: &[(&str, &str)], program: &str, expected: &str) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("collect-stress-beside")
        .join(label.trim_matches(['<', '>']));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap_or_else(|e| panic!("cannot empty {}: {e}", dir.display()));
    }
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot make {}: {e}", dir.display()));
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap_or_else(|e| panic!("cannot write {name}: {e}"));
    }
    let path = dir.join("main.rex");
    assert_stress_run_prints(
        path.to_str().expect("a UTF-8 target directory"),
        program,
        rexx_exec::Invocation::none().with_directory(dir.clone()),
        expected,
    );
}

/// Runs `program` as `path` under `invocation`, collecting at every
/// allocation, and asserts it exited 0 having printed `expected`, and that
/// it collected.
fn assert_stress_run_prints(
    path: &str,
    program: &str,
    invocation: rexx_exec::Invocation,
    expected: &str,
) {
    let stress = run_program_collect_every_alloc(
        path,
        program.as_bytes().to_vec(),
        invocation.with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        (
            stress.exit_code,
            String::from_utf8_lossy(&stress.stdout).as_ref()
        ),
        (0, expected),
        "stderr: {}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A REXX package copy's `~addRoutine` of a library row's `Routine` object
/// keeps it while the copy holds it. Measured, oracle rc 0, the same lines
/// with a 3000-pass allocating loop in place of the filler.
#[test]
fn a_rexx_package_copys_added_routine_survives() {
    assert_stress_prints(
        "<rexx-copy-added-routine>",
        concat!(
            "c = .array~package~copy\n",
            "u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')\n",
            "c~addRoutine('CC', u~copy)\n",
            "w = .WeakReference~new(c~routines['CC'])\n",
            "do i = 1 to 20\n",
            "  zj = 'filler' i\n",
            "end\n",
            "say 'weak' (w~value \\== .nil)\n",
            "f = c~findRoutine('CC')\n",
            "say 'found' f~class~id (f \\== u) f~call(0)\n",
        ),
        "weak 1\nfound Routine 1 0\n",
    );
}

/// A package copy that only another package imports survives as that
/// import, under a namespace too. Measured, oracle rc 0, the same lines with
/// a 3000-pass allocating loop in place of the filler.
#[test]
fn a_package_copy_held_as_an_import_survives() {
    assert_stress_prints(
        "<package-copy-import>",
        concat!(
            "p = .context~package\n",
            "h = p~copy\n",
            "i = p~copy\n",
            "i~addRoutine('IMARK', .routine~new('im', 'return \"imark\"'))\n",
            "wi = .WeakReference~new(i)\n",
            "h~addPackage(i)\n",
            "drop i\n",
            "h2 = p~copy\n",
            "i2 = p~copy\n",
            "wn = .WeakReference~new(i2)\n",
            "h2~addPackage(i2, 'NSI2')\n",
            "drop i2\n",
            "do k = 1 to 20\n",
            "  zj = 'filler' k\n",
            "end\n",
            "say 'weak' (wi~value \\== .nil) (wn~value \\== .nil)\n",
            "say 'import' h~importedPackages~items h~importedPackages[1]~routines~hasIndex('IMARK')\n",
            "say 'ns' (h2~findNamespace('NSI2') \\== .nil)\n",
        ),
        "weak 1 1\nimport 1 1\nns 1\n",
    );
}

/// A package copy that is only another package's parent survives as that
/// parent, and the routine it holds answers through the child. Measured,
/// oracle rc 0, the same lines with a 3000-pass allocating loop in place of
/// the filler.
#[test]
fn a_package_copy_held_as_a_parent_survives() {
    assert_stress_prints(
        "<package-copy-parent>",
        concat!(
            "src = .package~new('js', .array~of('::routine jmark', 'return \"jmark\"'))\n",
            "j = src~copy\n",
            "wj = .WeakReference~new(j)\n",
            "n = .package~new('n', .array~of('return 1'), j)\n",
            "drop j\n",
            "do k = 1 to 20\n",
            "  zj = 'filler' k\n",
            "end\n",
            "say 'weak' (wj~value \\== .nil)\n",
            "say 'parent' n~findRoutine('JMARK')~call\n",
        ),
        "weak 1\nparent jmark\n",
    );
}

/// Runs `program` collecting at every allocation and asserts it printed
/// `0 Routine 0`: a copy of a row's `Routine` object, held only by a routine
/// table once the package's `.ROUTINES` entry stops naming it, still answers.
fn assert_added_routine_survives(label: &str, program: &str) {
    let stress = run_program_collect_every_alloc(
        label,
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        (
            stress.exit_code,
            String::from_utf8_lossy(&stress.stdout).as_ref()
        ),
        (0, "0 Routine 0\n"),
        "stderr: {}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// The copy is in an importer's merged table, and its own package replaced
/// it.
#[test]
fn an_added_routine_object_survives_in_its_importer() {
    assert_added_routine_survives(
        "<added-routine-importer>",
        concat!(
            "p = .context~package\n",
            "u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')\n",
            "r = .Package~new('r', 'nop')\n",
            "p~addPublicRoutine('C2', u~copy)\n",
            "r~addPackage(p)\n",
            "p~addPublicRoutine('C2', u)\n",
            "do i = 1 to 20\n",
            "  zj = 'filler' i\n",
            "end\n",
            "f = r~findRoutine('C2')\n",
            "say (f == u) f~class~id f~call(0)\n",
        ),
    );
}

/// The copy is in its package's public routines alone once a program
/// replaced its `.ROUTINES` entry, and an importer merges it after a
/// collection. Measured, oracle rc 0, `0 Routine 0` for the same program
/// without the weak reference's line.
#[test]
fn an_added_public_routine_object_survives_its_table_entry() {
    assert_added_routine_survives(
        "<added-routine-public>",
        concat!(
            "p = .context~package\n",
            "u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')\n",
            "d = u~copy\n",
            "w = .WeakReference~new(d)\n",
            "p~addPublicRoutine('C3', d)\n",
            "drop d\n",
            ".routines['C3'] = u\n",
            "r = .Package~new('r', 'nop')\n",
            "do i = 1 to 20\n",
            "  zj = 'filler' i\n",
            "end\n",
            "if w~value == .nil then say 'collected'\n",
            "r~addPackage(p)\n",
            "f = r~findRoutine('C3')\n",
            "say (f == u) f~class~id f~call(0)\n",
        ),
    );
}

/// The objects a package holds and nothing else does survive collections:
/// its local directory's entry, a written `.ROUTINES` table's entry, its
/// prolog `Routine` object, its security manager, and a `::CONSTANT`
/// expression's value in a package a routine built, whose install temps went
/// with the routine's frame. Measured, oracle rc 0, the same lines for the
/// program with a 3000-pass allocating loop in place of the filler.
#[test]
fn the_objects_a_package_holds_survive_collect_on_every_allocation() {
    let program = concat!(
        "p = .context~package\n",
        "p~local~put(.array~of('local'), 'PKGHELD')\n",
        ".routines['ADDED'] = .routine~new('a', 'return \"table\"')\n",
        "w = .WeakReference~new(p~prolog)\n",
        "p~setSecurityManager(.sm~new)\n",
        "c = mk()\n",
        "do i = 1 to 20\n",
        "  zj = 'filler' i\n",
        "end\n",
        "say .pkgheld[1]\n",
        "say added()\n",
        "say (w~value == p~prolog) (w~value \\== .nil)\n",
        "say .k~v[1] c~v[1]\n",
        "x = .local\n",
        "exit\n",
        "::routine mk\n",
        "  p = .package~new('cp', .array~of('::class k public', \"::constant v (.array~of('built'))\"))\n",
        "  return p~findClass('K')\n",
        "::routine r1\n",
        "  return 1\n",
        "::class k\n",
        "::constant v (.array~of('const'))\n",
        "::class sm\n",
        "::method local\n",
        "  say 'mgr local'\n",
        "  return 0\n",
        "::method unknown\n",
        "  return 0\n",
    );
    let stress = run_program_collect_every_alloc(
        "<package-held-objects>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
    );
    assert_eq!(
        (
            stress.exit_code,
            String::from_utf8_lossy(&stress.stdout).as_ref()
        ),
        (0, "local\ntable\n1 1\nconst built\nmgr local\n"),
        "stderr: {}",
        String::from_utf8_lossy(&stress.stderr)
    );
    assert!(
        stress.collections > 0,
        "the stress mode did not collect, so this proves nothing"
    );
}

/// A trapped condition's own object survives collect-on-every-allocation.
///
/// **Two root paths the exhaustive destructures do not reach**: the object
/// hangs off `TrappedCondition`, which `Activation::object_roots` reached
/// through a `condition: _` wildcard until this landed, and off a queued
/// `PendingTrap` inside a `VecDeque` that `Interp::object_roots` wildcarded
/// the same way. A field added inside either is invisible to `E0027`, so
/// nothing but a collecting run says whether they were named correctly.
/// The handler allocates before it reads, so a collect falls between the
/// object's creation and the read.
#[test]
fn a_trapped_conditions_object_survives_collect_on_every_allocation() {
    for (label, program) in [
        (
            "signal on",
            "signal on error name h\n\"sh -c 'exit 3'\"\nexit\nh:\n  o = condition('O')\n  \
             do i = 1 to 20\n    zj = 'filler' i\n  end\n  \
             say o~at('CONDITION') o~at('RC') o~at('STACKFRAMES')~items\n",
        ),
        (
            "call on",
            "call on error name h\n\"sh -c 'exit 3'\"\ncall off error\nexit\nh:\n  \
             o = condition('O')\n  do i = 1 to 20\n    zj = 'filler' i\n  end\n  \
             say o~at('CONDITION') o~at('RC') o~at('STACKFRAMES')~items\n  return\n",
        ),
    ] {
        let stress = run_program_collect_every_alloc(
            "<condition-object-rooting>",
            program.as_bytes().to_vec(),
            rexx_exec::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE * 10),
        );
        assert_eq!(
            stress.exit_code,
            0,
            "{label}: {}",
            String::from_utf8_lossy(&stress.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            "ERROR 3 1\n",
            "{label}: the condition object did not survive the collector"
        );
        assert!(
            stress.collections > 0,
            "{label}: the stress mode did not collect, so this proves nothing"
        );
    }
}

/// A package is never collected while something reaches it: an activation
/// of its code, running or waiting on another activity, a variable holding
/// it, a class it defines, a `Routine` or `Method` object compiled into it, a
/// subclass instance of either, or a `Method` whose parent it is. Its
/// annotation table lives as long as it does. Measured, oracle rc 0, the same
/// lines for each program with a 100-pass loop of `copies('y', 100000)` in
/// place of the filler.
#[test]
fn a_package_in_use_is_never_collected() {
    for (label, program, expected) in [
        (
            "<package-of-a-running-activation>",
            concat!(
                "w = .WeakReference~new(.context~package)\n",
                "h = .context~package~identityHash\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "say 'running' (w~value \\== .nil) (.context~package~identityHash == h)\n",
            ),
            "running 1 1\n",
        ),
        (
            "<package-of-a-waiting-activation>",
            concat!(
                "w = .WeakReference~new(.context~package)\n",
                "h = .context~package~identityHash\n",
                "k = .Package~new('w', ('::class worker public', '::method churn', ",
                "'  do i = 1 to 20', '    zj = \"filler\" i', '  end', '  return 1'))",
                "~findClass('WORKER')\n",
                "m = k~new~start('CHURN')\n",
                "say 'idle' m~result (w~value \\== .nil) (.context~package~identityHash == h)\n",
            ),
            "idle 1 1 1\n",
        ),
        (
            "<package-in-a-variable>",
            concat!(
                "p = .Routine~new('r', 'return .context~package')~call\n",
                "p~addRoutine('X7', .routine~new('x', 'return 7'))\n",
                "w = .WeakReference~new(p)\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "say 'variable' (w~value == p) p~findRoutine('X7')~call\n",
            ),
            "variable 1 7\n",
        ),
        (
            "<package-of-a-class>",
            concat!(
                "k = .Package~new('p', ('::class k public', '::method m', ",
                "'  return .context~package~identityHash'))~findClass('K')\n",
                "h = k~package~identityHash\n",
                "w = .WeakReference~new(k~package)\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "say 'class' (w~value \\== .nil) (k~new~m == h) (k~package~identityHash == h)\n",
            ),
            "class 1 1 1\n",
        ),
        (
            "<package-of-a-routine>",
            concat!(
                "r = .Package~new('p', ('::routine r public', ",
                "'  return .context~package~identityHash'))~findRoutine('R')\n",
                "h = r~package~identityHash\n",
                "w = .WeakReference~new(r~package)\n",
                "s = .Routine~new('s', 'return .context~package~identityHash')\n",
                "g = s~package~identityHash\n",
                "v = .WeakReference~new(s~package)\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "say 'directive routine' (w~value \\== .nil) (r~call == h) (r~package~identityHash == h)\n",
                "say 'source routine' (v~value \\== .nil) (s~call == g) (s~package~identityHash == g)\n",
            ),
            "directive routine 1 1 1\nsource routine 1 1 1\n",
        ),
        (
            "<package-of-a-method>",
            concat!(
                "m = .Method~new('m', 'return .context~package~identityHash')\n",
                "h = m~package~identityHash\n",
                "w = .WeakReference~new(m~package)\n",
                "c = .context~package~copy\n",
                "c~addRoutine('Q9', .routine~new('q', 'return 9'))\n",
                "n = .Method~new('n', 'return q9()', c)\n",
                "drop c\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "o = .runner~new\n",
                "say 'method' (w~value \\== .nil) (o~go(m) == h) (m~package~identityHash == h) o~go(n)\n",
                "::class runner\n",
                "::method go\n",
                "  use arg m\n",
                "  return self~run(m)\n",
            ),
            "method 1 1 1 9\n",
        ),
        (
            "<package-of-an-executable-subclass-instance>",
            concat!(
                "s = .mm~new('m', 'return .context~package~identityHash')\n",
                "h = s~package~identityHash\n",
                "w = .WeakReference~new(s~package)\n",
                "t = .rr~new('r', 'return .context~package~identityHash')\n",
                "g = t~package~identityHash\n",
                "v = .WeakReference~new(t~package)\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "say 'subclass' (w~value \\== .nil) (s~package~identityHash == h) ",
                "(v~value \\== .nil) (t~call == g) (t~package~identityHash == g)\n",
                "::class mm subclass method\n",
                "::class rr subclass routine\n",
            ),
            "subclass 1 1 1 1 1\n",
        ),
        (
            "<annotations-of-a-package>",
            concat!(
                "p = .Package~new('p', ('::annotate package colour \"blue\"', ",
                "'::routine r public', '  return 1'))\n",
                "w = .WeakReference~new(p~annotations)\n",
                "do i = 1 to 20\n",
                "  zj = 'filler' i\n",
                "end\n",
                "say 'annotation' p~annotation('colour') (w~value == p~annotations)\n",
            ),
            "annotation blue 1\n",
        ),
    ] {
        assert_stress_prints(label, program, expected);
    }
    // An external routine parked on another activity keeps its package,
    // which an internal routine built. Oracle: `p10 1 1 7`, rc 0.
    assert_stress_prints_beside(
        "<package-of-a-parked-frame>",
        &[(
            "ext10.rex",
            concat!(
                "use arg sem, ready\n",
                "h = pk()\n",
                "ready~post\n",
                "sem~wait\n",
                "return (.local~w10~value \\== .nil) (.context~package~identityHash == h) ",
                ".context~package~findRoutine('Z7')~call\n",
                "pk:\n",
                "  .local~w10 = .WeakReference~new(.context~package)\n",
                "  .context~package~addRoutine('Z7', .routine~new('z', 'return 7'))\n",
                "  return .context~package~identityHash\n",
            ),
        )],
        concat!(
            "sem = .EventSemaphore~new\n",
            "ready = .EventSemaphore~new\n",
            "msg = .starter~new~start('go', sem, ready)\n",
            "ready~wait\n",
            "do i = 1 to 20\n",
            "  zj = 'filler' i\n",
            "end\n",
            "sem~post\n",
            "say 'p10' msg~result\n",
            "::class starter\n",
            "::method go\n",
            "  use arg sem, ready\n",
            "  return ext10(sem, ready)\n",
        ),
        "p10 1 1 7\n",
    );
}

/// The filler of the external-routine tests below, in place of the oracle's
/// 3000-pass loop of `copies('y', 100000)`.
const EXTERNAL_FILLER: &str = "do zi = 1 to 3\n  zz = 'f' zi\nend\n";

/// An external routine's package outlives its return until the caller has
/// merged what it made public, here a public class it added at run time,
/// whether or not the caller's own package existed before the call.
/// Measured, oracle rc 0, the same lines with the oracle's filler.
#[test]
fn an_external_routines_added_public_class_is_merged() {
    let ext9e = format!(
        ".context~package~addPublicClass('DK', .object~subclass('dk'))\n{EXTERNAL_FILLER}return 'v' || 1\n"
    );
    assert_stress_prints_beside(
        "<external-public-class>",
        &[("ext9e.rex", &ext9e)],
        "v = ext9e()\nsay 'p09e' v .dk~id\n",
        "p09e v1 dk\n",
    );
}

/// The same for a routine an external routine added to its own package,
/// called twice. Measured, oracle rc 0, the same lines with the oracle's
/// filler.
#[test]
fn an_external_routines_package_survives_two_calls() {
    let ext1 = format!(
        "h = .context~package~identityHash\n\
         w = .WeakReference~new(.context~package)\n\
         .context~package~addRoutine('Z7', .routine~new('z', 'return 7'))\n\
         {EXTERNAL_FILLER}\
         return (w~value \\== .nil) (.context~package~identityHash == h) \
         .context~package~findRoutine('Z7')~call\n"
    );
    assert_stress_prints_beside(
        "<external-added-routine>",
        &[("ext1.rex", &ext1)],
        "say 'p01' ext1()\nsay 'p01' ext1()\n",
        "p01 1 1 7\np01 1 1 7\n",
    );
}

/// The same through INTERPRET. Measured, oracle rc 0, the same lines with
/// the oracle's filler.
#[test]
fn an_external_routines_package_survives_interpret() {
    let ext6 = format!(
        "interpret \"h = .context~package~identityHash\"\n\
         interpret \".context~package~addRoutine('Z7', .routine~new('z', 'return 7'))\"\n\
         {EXTERNAL_FILLER}\
         interpret \"r = (.context~package~identityHash == h) .context~package~findRoutine('Z7')~call\"\n\
         return r\n"
    );
    assert_stress_prints_beside(
        "<external-interpret>",
        &[("ext6.rex", &ext6)],
        "say 'p06' ext6()\n",
        "p06 1 7\n",
    );
}

/// A package nothing reaches is collected and its weak reference clears,
/// whether `Package~new` built it or a `Routine` or `Method` compiled from
/// source text did (L7). Measured, oracle rc 0, `dropped 1 1 1 1` for the same
/// lines with a 100-pass loop of `copies('y', 100000)` in place of the filler.
#[test]
fn a_dropped_package_is_collected() {
    assert_stress_prints(
        "<package-collected>",
        concat!(
            "w = .WeakReference~new(.Package~new('src', 'nop'))\n",
            "v = .WeakReference~new(.Package~new('q', ('::routine r public', '  return 1')))\n",
            "u = .WeakReference~new(.Routine~new('r', 'return 1')~package)\n",
            "t = .WeakReference~new(.Method~new('m', 'return 1')~package)\n",
            "do i = 1 to 20\n",
            "  zj = 'filler' i\n",
            "end\n",
            "say 'dropped' (w~value == .nil) (v~value == .nil) (u~value == .nil) (t~value == .nil)\n",
        ),
        "dropped 1 1 1 1\n",
    );
}

/// Runs `read` after an external routine's public routines were merged into
/// the main program's package by a call from a procedure, and a loop that
/// allocates, and asserts it printed `expected`.
fn assert_external_merge_prints(label: &str, read: &str, expected: &str) {
    let ext5 = "return 1\n::routine pubr public\n  return .context~package~name~right(8)\n";
    let program = format!(
        "call sub\ndo zi = 1 to 3\n  zz = copies('x', 100000)\nend\n{read}exit\n\
         sub: procedure\n  x = ext5()\n  return\n"
    );
    assert_stress_prints_beside(label, &[("ext5.rex", ext5)], &program, expected);
}

/// An external routine's public routines, merged into its caller's package,
/// keep the routine's package alive after the call returns. Measured, oracle
/// rc 0, `e5 ext5.rex` with a 3000-pass loop of `copies('x', 100000)`.
#[test]
fn an_external_routines_merged_routine_runs_in_its_package() {
    assert_external_merge_prints(
        "<external-merged-call>",
        "say 'e5' pubr()\n",
        "e5 ext5.rex\n",
    );
}

/// The same after the caller's own package was built. Measured, oracle rc
/// 0, `e6 main.rex ext5.rex` with the oracle's filler.
#[test]
fn an_external_routines_merged_routine_runs_beside_its_callers_package() {
    assert_external_merge_prints(
        "<external-merged-after-own-package>",
        "say 'e6' .context~package~name~right(8) pubr()\n",
        "e6 main.rex ext5.rex\n",
    );
}

/// The same routine found through `~findRoutine`. Measured, oracle rc 0,
/// `e7 0 ext5.rex` with the oracle's filler.
#[test]
fn an_external_routines_merged_routine_is_found() {
    assert_external_merge_prints(
        "<external-merged-found>",
        "r = .context~package~findRoutine('PUBR')\nsay 'e7' (r == .nil) r~call\n",
        "e7 0 ext5.rex\n",
    );
}

/// A held `StackFrame` keeps the program of the `Method` it ran, after every
/// other reference to the method is dropped. Measured, oracle rc 0, with a
/// 300-pass loop of `copies('x', 100000)` in place of the filler.
#[test]
fn a_held_stack_frame_keeps_its_program() {
    assert_stress_prints(
        "<held-frame-program>",
        concat!(
            "do i = 1 to 3\n",
            "  m = .Method~new('m' i, ('x = ' i, 'return .context~stackFrames'))\n",
            "  frames = .runner~new~go(m)\n",
            "  drop m\n",
            "end\n",
            "do i = 1 to 3\n",
            "  zj = copies('x', 1000)\n",
            "end\n",
            "f = frames[1]\n",
            "e = f~executable\n",
            "say 'frame' f~name f~line f~traceline\n",
            "say 'executable' e~class~id e~source~items e~source[1] e~package~sourceline(1)\n",
            "say 'runs' .runner~new~go(e)[1]~traceline\n",
            "::class runner\n",
            "::method go\n",
            "  use arg m\n",
            "  return self~run(m)\n",
        ),
        concat!(
            "frame *UNNAMED* 2      2 *-* return .context~stackFrames\n",
            "executable Method 2 x =  3 x =  3\n",
            "runs      2 *-* return .context~stackFrames\n",
        ),
    );
}

/// A running `Method` or `Routine` keeps its program, the method below once
/// it has unset its own entry and so is reached by its frame alone. Measured,
/// oracle rc 0, with 300 passes of `copies('y', 100000)`.
#[test]
fn a_running_frame_keeps_its_program() {
    assert_stress_prints(
        "<running-frame-program>",
        concat!(
            "say .runner~new~go(.Method~new('m', ('do i = 1 to 3', ",
            "'  z = copies(\"y\", 1000)', 'end', 'call sub', ",
            "'return \"ran\" .context~executable~source~items .context~package~sourceline(1)', ",
            "'sub: return')))\n",
            ".local['R'] = .Routine~new('r', ('x = .local~remove(\"R\")', 'do i = 1 to 3', ",
            "'  z = copies(\"y\", 1000)', 'end', ",
            "'return \"routine\" .context~executable~source~items .context~package~sourceline(1)'))\n",
            "say .local['R']~call\n",
            "o = .runner~new\n",
            "o~put('X', ('self~unsetMethod(\"X\")', 'do i = 1 to 3', ",
            "'  z = copies(\"y\", 1000)', 'end', 'call sub', ",
            "'return \"unset\" .context~executable~source~items .context~package~sourceline(2)', ",
            "'sub: return'))\n",
            "say o~x o~hasMethod('X')\n",
            "::class runner\n",
            "::method go\n",
            "  use arg m\n",
            "  return self~run(m)\n",
            "::method put\n",
            "  use arg name, m\n",
            "  self~setMethod(name, m)\n",
        ),
        "ran 6 do i = 1 to 3\nroutine 5 x = .local~remove(\"R\")\nunset 7 do i = 1 to 3 0\n",
    );
}

/// A method that unset its own entry and is parked in `GUARD WHEN` on a
/// started activity is reached by that idle activity's frame alone, which
/// keeps its program. Measured, oracle rc 0 in 3 of 3 runs, with 300 passes
/// of `copies('x', 100000)` in `filler`.
#[test]
fn a_parked_activitys_frame_keeps_its_program() {
    assert_stress_prints(
        "<parked-frame-program>",
        concat!(
            "o = .k~new\n",
            "o~put('REL', ('expose flag', 'flag = 1'))\n",
            "o~put('W', ('expose flag', 'self~unsetMethod(\"W\")', 'guard on when flag = 1', ",
            "'call filler2', ",
            "'return \"parked\" .context~executable~source~items .context~package~sourceline(3)', ",
            "'filler2: do j = 1 to 3; z = copies(\"q\", 1000); end; return'))\n",
            "msg = o~start('W')\n",
            "call syssleep 0.3\n",
            "call filler\n",
            "say 'hasW' o~hasMethod('W')\n",
            "o~rel\n",
            "say msg~result\n",
            "exit\n",
            "filler: do j = 1 to 3; z = copies('x', 1000); end; return\n",
            "::class k\n",
            "::method put\n",
            "  use arg name, m\n",
            "  self~setMethod(name, m)\n",
        ),
        "hasW 0\nparked 6 guard on when flag = 1\n",
    );
}

/// A `loadExternalMethod` answer installed by `define` or `setMethod`, a copy
/// of one, and one flagged private keep running after the program drops
/// them, while fresh answers come and go. Measured, oracle rc 0, with a
/// 300-pass loop of `copies('x', 100000)` as the filler.
#[test]
fn an_installed_loaded_method_survives_its_dropped_object() {
    assert_stress_prints(
        "<installed-loaded-method>",
        concat!(
            ".k~define('SEP', .Method~loadExternalMethod('m', 'LIBRARY REXX file_separator'))\n",
            "call filler\n",
            "say 'define' .k~new~sep\n",
            "m = .Method~loadExternalMethod('m', 'LIBRARY REXX file_separator')\n",
            "m~setPrivate\n",
            ".k~define('PSEP', m)\n",
            "drop m\n",
            "call filler\n",
            "o = .k~new\n",
            "say 'private' o~callp o~hasMethod('PSEP') .k~method('PSEP')~isPrivate\n",
            "o~put('OSEP', .Method~loadExternalMethod('m', 'LIBRARY REXX file_separator')~copy)\n",
            "call filler\n",
            "say 'own' o~osep\n",
            "do i = 1 to 3\n",
            "  q = .Method~loadExternalMethod('q', 'LIBRARY REXX file_separator')\n",
            "  q~setProtected\n",
            "end\n",
            "drop q\n",
            "call filler\n",
            "say 'again' .k~new~sep o~osep o~callp\n",
            "exit\n",
            "filler: procedure\n",
            "  do i = 1 to 3\n",
            "    z = copies('x', 1000)\n",
            "  end\n",
            "  return\n",
            "::class k\n",
            "::method callp\n",
            "  return self~psep\n",
            "::method put\n",
            "  use arg name, m\n",
            "  self~setMethod(name, m)\n",
        ),
        "define /\nprivate / 1 1\nown /\nagain / / /\n",
    );
}
