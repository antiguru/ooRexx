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
//!
//! # The mode did not exist before this task
//!
//! `Heap::alloc_with` (as it was named before this task) never collected,
//! and `Heap::collect` had no caller outside `rexx-core`'s own tests --
//! this criterion was written as though the mode already existed, but it
//! had never once run against `rexx-exec`. Task 16 built it:
//! `rexx-core::Heap::alloc_with_uncollected` is the renamed, never-collects
//! primitive (renamed so a *new* allocation site written the natural way
//! announces at the call site that it bypasses the stress hook, rather
//! than silently doing so); `rexx_exec::Interp::alloc_with` (`lib.rs`) is
//! the one production entry point every value/stem constructor now calls,
//! and it collects when [`run_program_collect_every_alloc`] enabled it and
//! does nothing extra otherwise. **This means criterion 4's pass here is
//! the first time this mode has ever run against this crate's rooting
//! discipline, not a re-confirmation of something exercised throughout the
//! phase.** See the gate document for why that matters and what it does
//! and does not prove.
//!
//! # Collect BEFORE the allocation, not after -- and this was not the first
//! design tried
//!
//! `Interp::alloc_with`'s own doc comment has the full account: an earlier
//! version collected immediately *after* allocating, which cannot work --
//! the caller has not had a chance to root the value the allocation is
//! about to return, so every single allocation swept its own result,
//! unconditionally, on every program including ones with no rooting
//! question at stake. Measured at the time: 29 of 29 subset programs
//! panicked, even `say 1`. Collecting *before* the allocation asks the
//! right question instead -- is everything already rooted by an *earlier*
//! call's `push_temp` still reachable now that a *new* allocation is about
//! to happen -- and that is what this file actually exercises.
//!
//! # The negative control
//!
//! **Verified by hand, not by an assertion in this file**, because it
//! means deleting a line of production code, which nothing here should do
//! on every run. With `eval.rs`'s `eval_arithmetic`'s
//! `self.roots.push_temp(left_value);` removed (the call that roots the
//! left operand while the right operand's own evaluation runs and can
//! allocate arbitrarily), **7 of the 29 subset programs panic** under
//! [`run_program_collect_every_alloc`] with `to_text`'s "a live value"
//! (`arith_digits.rex`, `trace_output.rex`, `notation_thresholds.rex`,
//! `number_identity.rex`, `deep_nested_expr.rex`, `trace_results.rex`,
//! `mutation_digits_at_render.rex`) -- rebuilt and re-run against a clean
//! tree afterward, all 29 pass again. A different site, the analogous
//! `push_temp(right_value)` two lines below, turned out **not** to be a
//! useful control at this particular call shape: `right_value` is read
//! exactly once, immediately, by `arith_operand`, with no allocation
//! between its creation and that read, so nothing here ever asks whether
//! it survived -- deleting its root is inert for this reason alone, not
//! because rooting does not matter for it in general. That is why
//! `left_value`'s is the site named as this criterion's control, not
//! `right_value`'s: the criterion asks for a site whose deletion a subset
//! program actually catches, and this is the one that does.
//!
//! # Why comparison is against `run_program`, not the oracle, directly
//!
//! `tests/corpus.rs` (criterion 1) already establishes, byte for byte,
//! that `run_program`'s output matches the oracle for every program named
//! in `phase-4a.txt`. So `stress_output == plain_output` combined with
//! that already-established fact gives `stress_output == oracle_output`
//! transitively, without this file needing its own oracle invocation (and
//! its own `ulimit` wrapper, `LD_LIBRARY_PATH`, missing-binary handling --
//! all of which `corpus.rs` already owns). It is also the more direct
//! question this criterion is actually asking: does turning the mode on
//! change what the interpreter produces, not does the interpreter still
//! agree with a second program.

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
///
/// Takes a slice rather than a single path so that a later phase's own subset
/// file runs *alongside* the earlier ones rather than replacing them -- see
/// `coverage.rs`'s own copy of this function for the fuller argument. The
/// caller below reads every phase subset file the corpus has, which is what
/// puts a phase's own constructs under the collector at all: a program calling
/// a builtin lives only in `phase-4c.txt`, and until it was read here every
/// allocation a builtin makes was outside this harness's reach.
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
///
/// A named constant rather than a literal at the call site so that
/// [`the_stress_subset_reads_every_phase_subset_file`] can assert it against
/// the corpus directory itself. Which files a harness reads is not something
/// any other check here can see: `coverage.rs`'s
/// `phase_*_subset_matches_the_committed_list` tests pin each file's
/// **contents**, and every assertion in
/// [`the_l0_subset_passes_again_under_collect_on_every_allocation`] holds just
/// as well over a smaller union. Measured by deleting the subject: with
/// `phase-4c.txt` removed from this list, the whole workspace stays green and
/// byte-identical, and the builtins leave the collector's reach silently.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
];

/// The subset programs that allocate nothing, so collect-on-every-allocation
/// has nothing to fire on.
///
/// **Two inline representations put programs here, not one.**
/// `Interp::literal` inlines a literal spelling a canonical small integer,
/// and `Interp::text_bytes` inlines any byte string short enough to travel
/// in the handle. `deep_nested_expr.rex` is the clearest case of the first:
/// three thousand terms, all of them the literal `1`, and not one allocation
/// between them. The second is why the list is long -- short strings are
/// most strings, so a program has to produce a wide one, a stem, or a
/// non-integral number before the heap hears about it at all.
///
/// A program belongs here because of what it contains, not because it was
/// inconvenient -- see the both-directions assertion at the use site.
const NO_ALLOCATION_PROGRAMS: &[&str] = &[
    // The class-directive refusals: each is refused before the main body's
    // first clause, and nothing has been asked of the arena by then.
    "lang/class_abstract_metaclass.rex",
    "lang/class_abstract_metaclass_after_inherit.rex",
    "lang/class_abstract_metaclass_subclass.rex",
    "lang/class_inherit_base_class.rex",
    "lang/class_inherit_cycle.rex",
    "lang/class_inherit_not_a_mixin.rex",
    "lang/class_inherit_not_found.rex",
    "lang/class_inherit_recursive.rex",
    "lang/class_inherit_trailing_keyword.rex",
    "lang/class_metaclass_cycle.rex",
    "lang/class_metaclass_not_a_metaclass.rex",
    "lang/class_metaclass_not_found.rex",
    "lang/class_subclass_cycle.rex",
    "lang/class_subclass_not_found.rex",
    // These run rather than refusing, and still allocate nothing: every
    // value they say is either a class method's short literal result or a
    // class object's own `~defaultName`, which is rendered out of the
    // registry rather than built as a value.
    "lang/class_inherit_order.rex",
    "lang/class_mixinclass.rex",
    // These reach their first clause and raise 97.1 from the send in it. The
    // report substitutes the receiver's `~defaultName` and the message name,
    // both rendered out of the registry and the plan rather than built as
    // values, so nothing is asked of the arena on that path either.
    "lang/class_metaclass_class_method_does_not_donate.rex",
    "lang/class_metaclass_superclass_wins.rex",
    // `~method`'s own 97.1, which is the same shape: the name it looks up is
    // upcased into a local buffer and the target is the class object's
    // `~defaultName`, so the refusal never reaches the arena. Its sibling
    // `class_method_own_dictionary.rex` does, because the rows before its own
    // refusal each build a `Method` object.
    "lang/class_method_class_side_raises.rex",
    // A class-side `ACTIVATE` raising 42.3 before the main body's first
    // clause, which is the same shape as the class-directive refusals above
    // once the divide has run: the operands are canonical small integers and
    // the report's own substitutions come from the catalogue.
    "lang/class_activate_failure_blames_the_last_installed_class.rex",
    // The same 42.3, reached through a class method a `::CONSTANT`
    // expression calls, so the run enters and unwinds a method activation
    // before the main body's first clause and still asks the arena for
    // nothing: the operands are canonical small integers and the report's
    // substitutions come from the catalogue. Its siblings
    // `class_constant_instance_method.rex` and
    // `class_constant_expression_self.rex` are deliberately absent -- each
    // builds values the arena holds.
    "lang/class_constant_expression_method_failure.rex",
    // The duplicate-member refusals, which are translation errors: the walk
    // that finds them runs before any class is created and before the main
    // body's first clause, so nothing has been asked of the arena. The
    // negative control beside them, `class_member_names_per_side.rex`, runs
    // and allocates and is deliberately absent.
    "lang/class_duplicate_attribute.rex",
    "lang/class_duplicate_class.rex",
    "lang/class_duplicate_constant.rex",
    "lang/class_duplicate_constant_and_method.rex",
    "lang/class_duplicate_method.rex",
    "lang/class_member_class_keyword_needs_class.rex",
    "lang/comparison_families.rex",
    "lang/comparison_operators_remaining.rex",
    "lang/deep_nested_expr.rex",
    "lang/directive_annotate_missing_target.rex",
    "lang/directive_constant_blames_the_last_installed_class.rex",
    "lang/directive_constant_expression_blames_the_last_class.rex",
    "lang/directive_constant_expression_fails.rex",
    "lang/directive_constant_expression_installs.rex",
    "lang/directive_constant_expression_needs_class.rex",
    "lang/do_loop_forms.rex",
    // Both resolve every name they read out of the running package's own
    // class table, which is consulted ahead of `.environment` and so never
    // builds it -- the two directory objects are the only allocation a
    // `.NAME` makes.
    "lang/environment_package_class.rex",
    "lang/environment_special_dot_variables_are_not_resolved.rex",
    "lang/exit_no_value.rex",
    "lang/exit_with_value.rex",
    // Every value it names is short enough to live in the handle, so the run
    // reaches its 98.992 without asking the arena for anything.
    "lang/expose_outside_a_method.rex",
    "lang/if_else_chain.rex",
    "lang/iterate_from_select.rex",
    "lang/message_assignment_form.rex",
    "lang/message_instruction.rex",
    "lang/message_send_argument_not_a_string.rex",
    "lang/message_send_missing_argument.rex",
    "lang/message_send_scope_override.rex",
    "lang/message_send_too_many_arguments.rex",
    "lang/message_send_unknown_method.rex",
    "lang/message_send_unknown_method_on_a_number.rex",
    "lang/message_send_unknown_method_on_nil.rex",
    // A `::METHOD` activation allocates nothing of its own: the frame is
    // slots and the `SELF`/`SUPER` bindings are handles the caller already
    // held. So what puts a method program here is whatever its body and its
    // main line do, exactly as for any other program, and the ones absent
    // from this list are absent for a concatenation or a wide string rather
    // than for the activation.
    "lang/method_access_private_refused.rex",
    "lang/method_attribute_body.rex",
    // A refusal that a generated accessor's argument bounds produce is
    // reached before the accessor touches a pool, so nothing is allocated on
    // the way to it. The rest of Task 15's programs are absent: the value
    // program builds strings and stores one of them, and the `no_result` and
    // `abstract_send` programs each print before they fail.
    "lang/method_attribute_generated_getter_arguments.rex",
    "lang/method_attribute_generated_setter_arguments.rex",
    "lang/method_attribute_generated_setter_omitted.rex",
    "lang/method_body_raises.rex",
    "lang/method_class_side_lookup.rex",
    // Task 16. Both instructions' legality refusals reach their raise before
    // anything is built, and the two `REPLY` programs beside them hand out
    // values short enough to ride in the handle. Task 16's other three
    // programs are absent because they do allocate, which is what puts a
    // collection between a `REPLY`'s park and its resume;
    // `a_parked_reply_keeps_its_variables_across_a_collection` below is the
    // narrow test of that window.
    "lang/method_guard_outside_method.rex",
    "lang/method_no_result_is_an_error.rex",
    "lang/method_reply_exit_status.rex",
    "lang/method_reply_no_result.rex",
    "lang/method_reply_outside_method.rex",
    "lang/method_reply_twice.rex",
    "lang/method_trace_invocation.rex",
    "lang/method_trace_nested.rex",
    "lang/mutation_controlled_order.rex",
    "lang/no_trailing_newline.rex",
    "lang/prefix_dotvar_logical_over_label.rex",
    "lang/raise_array_substitution.rex",
    // Phase 5a (2026-08-17 plan) Task 14: the programs whose whole purpose is a
    // condition raised inside a `makeString` reached through the
    // required-string protocol. Each ends on its second clause, before
    // anything wide enough to allocate, so none collects at all -- the same
    // reason the refusal programs above are here.
    "lang/required_string_argument_make_string_raises.rex",
    "lang/required_string_make_string_raises.rex",
    "lang/select_when.rex",
    "lang/select_when_absorption.rex",
    "lang/select_when_bodies.rex",
    "lang/trace_numeric_request.rex",
    "lang/trace_output.rex",
    "lang/trace_results.rex",
    "num/comparison.rex",
];

/// The phase subset files that exist in the corpus directory, sorted.
///
/// Read from the directory rather than listed a second time, so the assertion
/// below cannot be satisfied by a copy of [`SUBSET_FILES`] that was edited in
/// the same change -- and so a subset file added later and forgotten here is
/// red rather than silently unread.
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
///
/// This is the pin on *which files* the harness reads, which is a different
/// question from what any of them contains. Without it the third entry of
/// [`SUBSET_FILES`] can be dropped -- by an edit, or by a merge -- and nothing
/// in the workspace moves: the run still passes, still reports a non-zero
/// collection count for every program it did read, and still exercises no
/// builtin at all, because no program in `phase-4a.txt` or `phase-4b.txt`
/// calls one.
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
    //
    // The set is committed data rather than an emptiness check, because a
    // program that allocates nothing is a legitimate state: a program whose
    // only values are literals spelling canonical small integers allocates
    // nothing at all, since `Interp::literal` inlines those into the handle
    // instead of the heap.
    //
    // **Both directions.** A program joining this set has had an allocation
    // silently removed; a program leaving it has gained one. Either is a
    // change a human should look at, which a one-directional exemption
    // would not force -- and an exemption nobody can fail is how a
    // shrinking subset goes unnoticed.
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
///
/// The shape is a clause whose value is created *and consumed* across an
/// activation boundary: `return bb() || 'TAIL'`, where `bb` queues the
/// condition. By the time the boundary runs, the concatenation's own
/// one-clause temps frame is already popped, and the handler is a nested
/// activation that allocates -- so the only thing keeping the returned
/// `ObjRef` alive is the boundary's own `push_temp`.
///
/// Three rows because the crate has two `Flow` variants that carry a value
/// and both must be rooted: a `RETURN` at top level, the same inside a `DO`
/// body (a different `run_bounded` drives it), and an `EXIT`.
///
/// **Checked by deleting its subject**, which is the only thing that makes
/// this a test rather than a re-run of the plain interpreter: with
/// `ClauseValue for Flow` returning `None`, all three rows panic on
/// `a live value` at `value.rs`, and the plain (non-stress) run of the same
/// three programs still passes, so nothing but the collector sees it.
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
///
/// It is an `ObjRef` created before the first clause runs and read by a clause
/// that may be the program's last, and `Interp::call_context` is **not** walked
/// by the collector -- so the only thing keeping it reachable is the
/// `push_temp` `execute` takes before `Interp::run`. Nothing else in the tree
/// can see that: the plain interpreter never collects, and every differential
/// harness runs a program short enough that a swept value would still be
/// sitting in freed-but-untouched memory.
///
/// **Checked by deleting its subject.** With `execute`'s `push_temp(value)`
/// removed, this test panics at `value.rs`'s `a live value` while
/// `tests/input_oracle.rs` and the whole plain suite stay green.
///
/// The program allocates repeatedly before reading the argument, and reads it
/// last, so a collect between the two has somewhere to happen.
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
///
/// `Interp::stem_get`'s "no object at all" branch (`stem.rs`) derives a
/// fresh, slot-less `Body::Text` for the read-before-write answer when the
/// stem has never been touched, and `Interp::stem_set` then allocates that
/// stem's *first* object on the identical branch -- so without rooting the
/// old value first, that second allocation's own pre-sweep can collect the
/// first with nothing left pointing at it.
///
/// **Checked by deleting its subject**, the `push_temp` `value`'s
/// `SymbolKind::CompoundName` arm now takes: with it removed, the fourth
/// row panics at `value.rs`'s `a live value`, and the plain (non-stress)
/// run of the same program still passes, so nothing but the collector
/// sees it. The first three rows are the adjacent successes that pin the
/// defect to *this* write path rather than to compounds, `VALUE`, or
/// reads in general -- a direct compound assignment, a stem already
/// carrying a default, and `VALUE`'s own read-only form all stay green
/// under the mutant.
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
///
/// `Interp::loop_advance`'s `Controlled` arm opens a temps frame of its own
/// and pops it once `bind_control` has written the new control value into the
/// variable's storage. Both halves are load-bearing and the frame's boundary
/// is not obvious from the site: the value is created by `Interp::number`,
/// then *rendered* for a trace line and then written through a path that can
/// resolve a compound tail -- both of which allocate, and either of which can
/// collect.
///
/// **The rows that catch it are the ones where the loop's own values are heap
/// objects.** A counted loop over small integers puts a tagged immediate in
/// the control variable and allocates nothing per pass, so it is green under
/// any placement of the frame; a `BY 0.5` control is a `Number` on the arena
/// and a `cv.j` control resolves a tail key on every write. Rows 1 to 3 are
/// the adjacent successes that pin the failure to the second shape rather
/// than to loops in general.
///
/// **Checked by deleting its subject, twice, and by the harder check
/// beside it.** Moving the `pop_frame` to before `bind_control`, and
/// separately removing the `push_temp` of the bound value, each panic row 5
/// on `a live value`; the plain run of the same programs stays green under
/// both, so nothing but the collector sees it. And the whole workspace suite
/// **without this test** stays green under both mutations as well -- the
/// 10,461-case dual-engine sweep included, run with the stress mode on for
/// every case -- so these rows add coverage rather than merely being able to
/// fail.
///
/// Every expected string is the oracle's own, measured on `build/bin/rexx`.
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
        for engine in [rexx_exec::Engine::TreeWalker, rexx_exec::Engine::Ir] {
            let path = format!("<loop-pass-rooting: {}>", row.name);
            let stress = run_program_collect_every_alloc(
                &path,
                row.program.as_bytes().to_vec(),
                rexx_exec::Invocation::none().with_engine(engine),
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
}

/// A method body a `REPLY` left owed still reads its own variables when it
/// resumes, with a collection at every allocation in between.
///
/// **The window this is about exists nowhere else.** Every other activation's
/// variables are in an open slot frame for the whole of its life, and
/// `RootSet::iter` walks that frame. A parked one has no frame: its values are
/// copied out and handed to `RootSet::park`, its arguments and receiver with
/// them, and the sender then runs on and allocates. So a value the park
/// forgot is swept between the two halves rather than merely retained.
///
/// The program is built so that every value the resumed half prints is wide
/// enough to need a heap slot -- a short one rides in the handle and would
/// survive a missing root by not being an object -- and so that the sender
/// allocates after the reply, which is what makes the collection happen inside
/// the window rather than before it.
///
/// **Checked by taking its subject away**, one mutation at a time: removing
/// `RootSet::iter`'s `self.parked` chain, and removing `park_reply`'s
/// `context.object_roots(&mut anchor)`. Each panics here on `a live value` on
/// both engines, and under each the plain (non-stress) run of this same
/// program still prints all five lines correctly on both engines -- so nothing
/// but the collector sees either one.
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
    for engine in [rexx_exec::Engine::TreeWalker, rexx_exec::Engine::Ir] {
        let stress = run_program_collect_every_alloc(
            "<parked-reply-rooting>",
            program.as_bytes().to_vec(),
            rexx_exec::Invocation::none().with_engine(engine),
        );
        assert_eq!(stress.exit_code, 0, "{:?}", engine);
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            expected,
            "a parked REPLY lost a value under collect-on-every-allocation, {engine:?}"
        );
        assert!(
            stress.collections > 0,
            "the stress mode did not collect, so this proves nothing, {engine:?}"
        );
    }
}
