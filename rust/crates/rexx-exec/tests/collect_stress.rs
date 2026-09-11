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

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{run_program, run_program_collect_every_alloc};

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
];

/// **Thirty-six programs left this set in Phase 5j, and none joined it.**
/// A class became an ordinary arena object, so declaring one is an
/// allocation and the stress mode has something to fire on. Every one of the
/// thirty-six contains a `::CLASS`, `~subclass` or `~mixinClass`, derived by
/// reading them rather than assumed from the names. The direction is the
/// reassuring one: a program *joining* this set would have had an allocation
/// silently removed.
const NO_ALLOCATION_PROGRAMS: &[&str] = &[
    "gate-tables/directives/attribute__external__subkeyword.rex",
    "gate-tables/directives/method__external__subkeyword.rex",
    "lang/class_duplicate_attribute.rex",
    "lang/class_duplicate_class.rex",
    "lang/class_duplicate_constant.rex",
    "lang/class_duplicate_constant_and_method.rex",
    "lang/class_duplicate_method.rex",
    "lang/class_inherit_cycle.rex",
    "lang/class_member_class_keyword_needs_class.rex",
    "lang/class_metaclass_cycle.rex",
    "lang/class_metaclass_not_a_metaclass.rex",
    "lang/class_metaclass_not_found.rex",
    "lang/class_rexx_defined_delete.rex",
    "lang/class_rexx_defined_inherit.rex",
    "lang/class_rexx_defined_library_define.rex",
    "lang/class_rexx_defined_library_inherit.rex",
    "lang/class_rexx_defined_library_no_mutation.rex",
    "lang/class_rexx_defined_uninherit.rex",
    "lang/class_subclass_cycle.rex",
    "lang/class_subclass_not_found.rex",
    "lang/comparison_families.rex",
    "lang/comparison_operators_remaining.rex",
    "lang/deep_nested_expr.rex",
    "lang/directive_annotate_missing_target.rex",
    "lang/directive_attribute_external_get_third_word.rex",
    "lang/directive_attribute_external_missing.rex",
    "lang/directive_attribute_external_set_default.rex",
    "lang/directive_constant_expression_needs_class.rex",
    "lang/directive_method_attribute_external_missing.rex",
    "lang/directive_method_external_before_duplicate.rex",
    "lang/directive_method_external_duplicate_wins.rex",
    "lang/directive_method_external_missing.rex",
    "lang/directive_method_external_source_order.rex",
    "lang/directive_options_digits_below_fuzz.rex",
    "lang/directive_options_trace.rex",
    "lang/do_loop_forms.rex",
    "lang/exit_no_value.rex",
    "lang/exit_with_value.rex",
    "lang/expose_outside_a_method.rex",
    "lang/forward_outside_method.rex",
    "lang/if_else_chain.rex",
    "lang/iterate_from_select.rex",
    "lang/library_method_traceback.rex",
    "lang/library_method_traceback_nested.rex",
    "lang/message_assignment_form.rex",
    "lang/message_instruction.rex",
    "lang/message_send_argument_not_a_string.rex",
    "lang/message_send_missing_argument.rex",
    "lang/message_send_scope_override.rex",
    "lang/message_send_too_many_arguments.rex",
    "lang/message_send_unknown_method.rex",
    "lang/message_send_unknown_method_on_a_number.rex",
    "lang/message_send_unknown_method_on_nil.rex",
    "lang/method_guard_outside_method.rex",
    "lang/method_reply_outside_method.rex",
    "lang/mutation_controlled_order.rex",
    "lang/no_trailing_newline.rex",
    "lang/raise_array_substitution.rex",
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

    let mut total_collections: u64 = 0;
    let mut mismatches = Vec::new();
    let mut zero_collection_programs = Vec::new();

    for rel_path in &subset {
        let abs = corpus_dir.join(rel_path);
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let path_str = abs
            .to_str()
            .unwrap_or_else(|| panic!("corpus path {} is not valid UTF-8", abs.display()));

        let plain = run_program(path_str, text.clone(), rexx_exec::Invocation::none());
        let stress = run_program_collect_every_alloc(path_str, text, rexx_exec::Invocation::none());

        if stress.collections == 0 {
            zero_collection_programs.push(rel_path.clone());
        }
        total_collections += stress.collections;

        if plain.exit_code != stress.exit_code
            || plain.stdout != stress.stdout
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
            rexx_exec::Invocation::none(),
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
            rexx_exec::Invocation::none(),
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
            rexx_exec::Invocation::none(),
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

/// A method body a `REPLY` left owed still reads its own variables when it
/// resumes, with a collection at every allocation in between.
#[test]
fn a_parked_reply_keeps_its_variables_across_a_collection() {
    let program = concat!(
        "say .K~m('aaaaaaaaaaaaaaaa')\n",
        "sender = 'bbbbbbbbbbbbbbbb' || 'cccccccccccccccc'\n",
        "say sender\n",
        "::class K\n",
        "::method m class\n",
        "  expose held\n",
        "  local = 'dddddddddddddddd' || 'eeeeeeeeeeeeeeee'\n",
        "  held = 'ffffffffffffffff' || 'gggggggggggggggg'\n",
        "  reply 'replied'\n",
        "  say local\n",
        "  say held\n",
        "  say arg(1)\n",
        "  return\n",
    );
    let expected = concat!(
        "replied\n",
        "bbbbbbbbbbbbbbbbcccccccccccccccc\n",
        "ddddddddddddddddeeeeeeeeeeeeeeee\n",
        "ffffffffffffffffgggggggggggggggg\n",
        "aaaaaaaaaaaaaaaa\n",
    );
    let stress = run_program_collect_every_alloc(
        "<parked-reply-rooting>",
        program.as_bytes().to_vec(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(stress.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        expected,
        "a parked REPLY lost a value under collect-on-every-allocation"
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
        rexx_exec::Invocation::none(),
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
        rexx_exec::Invocation::none(),
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
        rexx_exec::Invocation::none(),
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
