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

use super::routines::routine_program;

/// Runs `source` and hands back `(exit code, stdout, stderr)`.
fn annotate_outcome(source: &str) -> (i32, String, String) {
    let outcome = crate::run_program(
        "/abs/annotate.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    (
        outcome.exit_code,
        String::from_utf8_lossy(&outcome.stdout).into_owned(),
        String::from_utf8_lossy(&outcome.stderr).into_owned(),
    )
}

/// Every `::ANNOTATE` target the accumulated package does not hold is the
/// oracle's 99.945, naming the keyword and the upcased target.
#[test]
fn an_annotate_target_the_package_does_not_hold_is_the_oracles_own_refusal() {
    let rows: &[(&str, &str)] = &[
        (
            "say 'main'\n::annotate class nosuch a 1\n",
            "class \"NOSUCH\"",
        ),
        (
            "say 'main'\n::annotate routine nosuch a 1\n",
            "routine \"NOSUCH\"",
        ),
        (
            "say 'main'\n::annotate method nosuch a 1\n",
            "method \"NOSUCH\"",
        ),
        (
            "say 'main'\n::annotate attribute nosuch a 1\n",
            "attribute \"NOSUCH\"",
        ),
        (
            "say 'main'\n::annotate constant nosuch a 1\n",
            "constant \"NOSUCH\"",
        ),
        (
            "say 'main'\n::class K\n::method a\n  return 1\n::annotate attribute a x 1\n",
            "attribute \"A\"",
        ),
        (
            "say 'main'\n::class K\n::method c\n  return 1\n::annotate constant c x 1\n",
            "constant \"C\"",
        ),
    ];
    for (source, target) in rows {
        let (code, stdout, stderr) = annotate_outcome(source);
        assert_eq!((code, stdout.as_str()), (157, ""), "{source:?}: {stderr:?}");
        assert!(
            stderr.contains(&format!(
                "Error 99.945:  ::ANNOTATE target {target} not found."
            )),
            "{source:?}: stderr was {stderr:?}"
        );
    }
}

/// What an `::ANNOTATE` resolves to, in the cases a single readback program
/// cannot separate.
#[test]
fn an_annotate_target_resolves_the_way_the_directive_walk_accumulates() {
    let rows: &[(&str, &str)] = &[
        (
            "say .K~annotation('A') .K~annotation('B')\n::class K\n\
             ::annotate class K a 1 b 2\n::annotate class K a 3\n",
            "3 2\n",
        ),
        (
            "say .K~method('M')~annotation('X')\n::class K\n::method m class\n  return 1\n\
             ::method m\n  return 2\n::annotate method m x 'instance'\n",
            "instance\n",
        ),
        (
            "say .K~method('A')~annotation('X') .K~method('A=')~annotation('X')\n\
             ::class K\n::attribute a\n::annotate attribute a x 'pair'\n",
            "pair pair\n",
        ),
        (
            "say .K~method('A')~annotation('X') .K~method('A=')~annotation('X')\n\
             ::class K\n::attribute a\n::annotate method A x 'getter'\n",
            "getter The NIL object\n",
        ),
        (
            "say 'installed'\n::class K\n::attribute a class\n::annotate attribute a x 'c'\n",
            "installed\n",
        ),
        (
            "say .methods~m~annotation('X')\n::method m\n  return 1\n\
             ::annotate method m x 'unattached'\n",
            "unattached\n",
        ),
    ];
    for (source, stdout) in rows {
        let (code, seen, stderr) = annotate_outcome(source);
        assert_eq!(
            (code, seen.as_str()),
            (0, *stdout),
            "{source:?}: stderr was {stderr:?}"
        );
    }
}

/// A `Method` object is one object per dictionary entry, which two of the
/// oracle's own methods can see.
#[test]
fn a_class_answers_one_method_object_per_dictionary_entry() {
    let class = "::class K\n::method m\n  return 1\n::method p\n  return 2\n";
    let (code, stdout, stderr) = annotate_outcome(&format!(
        "say (.K~method('M')~identityHash = .K~method('M')~identityHash)\n{class}"
    ));
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert_eq!(stdout, "1\n");

    let (code, stdout, stderr) = annotate_outcome(&format!(
        ".K~method('M')~objectName = 'renamed'\nsay .K~method('M')\nsay .K~method('P')\n{class}"
    ));
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert_eq!(stdout, "renamed\na Method\n");
}

/// Two runs of one program in one process allocate the annotation tables in
/// the same order.
#[test]
fn two_runs_in_one_process_allocate_the_annotation_tables_alike() {
    let source = "say .K~annotations~identityHash\n\
                  say .K~method('M')~annotations~identityHash\n\
                  say .K~method('A')~annotations~identityHash\n\
                  say .K~method('A=')~annotations~identityHash\n\
                  say .routines~r~annotations~identityHash\n\
                  say .methods~u~annotations~identityHash\n\
                  ::method u\n  return 0\n::annotate method u a 0\n\
                  ::class K\n::annotate class K a 1\n\
                  ::method m\n  return 1\n::annotate method m a 2\n\
                  ::attribute a\n::annotate attribute a a 3\n\
                  ::routine r\n  return 2\n::annotate routine r a 4\n";
    let first = annotate_outcome(source);
    assert_eq!(first.0, 0, "stderr {:?}", first.2);
    assert_eq!(first.1.lines().count(), 6, "stdout {:?}", first.1);
    for _ in 0..8 {
        assert_eq!(
            annotate_outcome(source),
            first,
            "a later run in this process allocated the annotation tables in a different order"
        );
    }
}

/// Every directive form whose installation this crate **can** perform
/// leaves the program running byte for byte as the oracle runs it, one
/// program per form.
#[test]
fn every_directive_this_crate_can_install_leaves_the_program_alone() {
    let sources: &[(&str, &[u8])] = &[
        ("a bare ::CLASS", b"say 'main ran'\n::class foo\n"),
        (
            "::CLASS with a ::METHOD",
            b"say 'main ran'\n::class foo\n::method bar\nreturn 1\n",
        ),
        (
            "::CLASS with a ::ATTRIBUTE",
            b"say 'main ran'\n::class foo\n::attribute baz\n",
        ),
        (
            "::CLASS MIXINCLASS",
            b"say 'main ran'\n::class mx mixinclass object\n",
        ),
        (
            "::CLASS INHERIT",
            b"say 'main ran'\n::class mx mixinclass object\n::class foo inherit mx\n",
        ),
        (
            "a loose ::METHOD with no ::CLASS",
            b"say 'main ran'\n::method loose\nreturn 1\n",
        ),
        ("::CONSTANT", b"say 'main ran'\n::constant kk 5\n"),
        (
            "::RESOURCE",
            b"say 'main ran'\n::resource foo\nsome text\n::END\n",
        ),
        (
            "::ANNOTATE PACKAGE",
            b"say 'main ran'\n::annotate package author 'me'\n",
        ),
        (
            "::ANNOTATE CLASS",
            b"say 'main ran'\n::class foo\n::annotate class foo author 'me'\n",
        ),
        (
            "::ANNOTATE ROUTINE",
            b"say 'main ran'\n::routine r\nreturn 1\n::annotate routine r author 'me'\n",
        ),
        (
            "::ANNOTATE METHOD",
            b"say 'main ran'\n::class foo\n::method m\nreturn 1\n::annotate method m author 'me'\n",
        ),
        (
            "::ANNOTATE ATTRIBUTE",
            b"say 'main ran'\n::class foo\n::attribute a\n::annotate attribute a author 'me'\n",
        ),
        (
            "::ANNOTATE CONSTANT",
            b"say 'main ran'\n::class foo\n::constant c 5\n::annotate constant c author 'me'\n",
        ),
    ];
    for (what, source) in sources {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            0,
            "{what}: stderr {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"main ran\n".to_vec(), "{what}: stdout");
        assert!(outcome.stderr.is_empty(), "{what}: stderr must be empty");
    }
}

/// The refusals Task 21 leaves where the oracle answers, asserted here
/// because nothing else can assert them.
#[test]
fn the_refusals_this_task_leaves_where_the_oracle_answers_still_fire() {
    let cases: &[(&[u8], &str)] = &[(
        b".K~defineMethods(.local)\n::class K\n",
        "a directory whose entries this crate does not fill is not implemented (Phase 10)",
    )];
    for (source, message) in cases {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            crate::NOT_IMPLEMENTED_EXIT,
            "{message}: exit code"
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            format!("rexx-exec: {message}\n"),
            "{message}: stderr"
        );
    }
}

/// The source shapes a method compiled from source text refuses, each of
/// which the shipped oracle takes.
#[test]
fn the_method_source_shapes_this_task_leaves_refuse_loudly() {
    let cases: &[(&[u8], &str)] = &[
        (
            b".k~define(\"m\", .environment)\n::class k\n",
            "a method source that is neither a string nor an array is not implemented (Phase 5)",
        ),
        (
            b".k~define(\"bad\", 'this is not rexx +++')\n::class k\n",
            "reporting a method source that does not parse (bad, 35.901: Invalid expression.) \
             is not implemented (Phase 5)",
        ),
        (
            b".k~define(\"m\", 'say 1' || '0a'x || 'say 2')\n::class k\n",
            "reporting a method source that does not parse (m, 13.1: Invalid character in \
             program.) is not implemented (Phase 5)",
        ),
        (
            b".k~define(\"m\", ('return 1', '::class zz'))\n::class k\n",
            "a method source that carries a directive is not implemented (Phase 5)",
        ),
        (
            b".methods~put('return 1', 'M')\n\
              zk = .object~subclass(\"k\", .Class, .methods)\n\
              ::method z\n  return 1\n",
            "a class method built from source text is not implemented (Phase 5)",
        ),
    ];
    for (source, message) in cases {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            crate::NOT_IMPLEMENTED_EXIT,
            "{message}: exit code"
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            format!("rexx-exec: {message}\n"),
            "{message}: stderr"
        );
    }
}

/// Every directive form whose installation this crate **cannot** perform
/// refuses the program before its first clause, naming the owning phase.
#[test]
fn every_directive_this_crate_cannot_install_refuses_before_the_first_clause() {
    // **The row that says which `EXTERNAL` form is still refused**:
    // `REGISTERED`, naming a library and a procedure that resolve, so the only
    // thing left to refuse it is the directive. The `LIBRARY <other>` forms
    // are `a_directive_naming_a_library_that_is_not_there_is_98_903`'s, where
    // the oracle's own condition is what answers.
    let cases: &[(&[u8], &str)] = &[(
        b"say 'main ran'\n::routine r external \"REGISTERED rxmath RxCalcSqrt\"\n",
        "::ROUTINE EXTERNAL naming REGISTERED is not implemented (Phase 10)",
    )];
    for (source, message) in cases {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            crate::NOT_IMPLEMENTED_EXIT,
            "{message}: exit code"
        );
        assert_eq!(
            outcome.stdout, b"",
            "{message}: stdout must be empty -- main must not have run"
        );
        assert_eq!(
            outcome.stderr,
            format!("rexx-exec: {message}\n").into_bytes(),
            "{message}: stderr"
        );
    }
}

/// **A routine one of the oracle's internal packages exports refuses loudly**,
/// naming the phase that owes it, where a routine some phase has written
/// answers. Either way the name is the oracle's own, and answering 43.1 for
/// it -- "no such routine", which a program cannot tell from its own typo --
/// is what this boundary exists to prevent. The excluded builtins take their
/// owner from the same table, and the last case is the adjacent one: a name
/// no package exports still raises 43.1.
#[test]
fn an_internal_routine_refuses_loudly_where_an_unknown_name_still_raises() {
    // The delivered side of the same boundary: a row that has a body answers,
    // and answers through the very resolver step that refuses the rest. Without
    // these the test could go green over a resolver that refuses everything.
    for (source, expected) in [
        (&b"say filespec('N','/a/b.c')\n"[..], "b.c\n"),
        (b"say length(directory()) > 0\n", "1\n"),
        (b"say SysFileExists('.')\n", "1\n"),
        (b"say words(SysVersion()) > 0\n", "1\n"),
    ] {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), expected);
    }

    let cases: &[(&[u8], &str)] = &[
        (
            b"say SysStemSort('a.')\n",
            "routine \"SYSSTEMSORT\" is not implemented (Phase 10)",
        ),
        (
            b"say SysWaitEventSem(1)\n",
            "routine \"SYSWAITEVENTSEM\" is not implemented (Phase 6)",
        ),
        (
            b"say rxqueue('G')\n",
            "routine \"RXQUEUE\" is not implemented (Phase 10)",
        ),
    ];
    for (source, message) in cases {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            crate::NOT_IMPLEMENTED_EXIT,
            "{message}: exit code"
        );
        assert_eq!(outcome.stdout, b"", "{message}: stdout");
        assert_eq!(
            outcome.stderr,
            format!("rexx-exec: {message}\n").into_bytes(),
            "{message}: stderr"
        );
    }

    let outcome = routine_program(b"say zorkolo()\n");
    assert_eq!(
        outcome.exit_code, 213,
        "a name no package exports is the oracle's own 43.1, not a refusal"
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
    assert!(
        stderr.contains("Error 43.1:") && stderr.contains("ZORKOLO"),
        "43.1 must still name the routine: {stderr}"
    );
}

/// **A gap the oracle diagnoses before it creates any class refuses before
/// this crate creates one either**, so a `::CLASS` that cannot install does
/// not answer in its place.
/// ```text
/// ::requires 'zzznosuchfile.rex'                        43.901 rc 213, the ::REQUIRES line
/// ::routine zz external "LIBRARY nosuchlib nosuchfn"    98.903 rc 158, the ::ROUTINE line
/// ::method mm external "LIBRARY nosuchlib nosuchfn"     98.903 rc 158, the ::METHOD line
/// ::attribute aa external "LIBRARY nosuchlib nosuchfn"  98.903 rc 158, the ::ATTRIBUTE line
/// ::options digits 12                                   98.909 rc 158, the ::CLASS line
/// ```
#[test]
fn a_gap_the_oracle_diagnoses_before_a_class_refuses_ahead_of_the_class_error() {
    let failing_class = "::class a subclass zzznotaclass\n";
    let refusing: &[&str] = &[
        "::routine zz external \"LIBRARY nosuchlib nosuchfn\"\n",
        "::class kk\n::method mm external \"LIBRARY nosuchlib nosuchfn\"\n",
        "::class kk\n::attribute aa external \"LIBRARY nosuchlib nosuchfn\"\n",
    ];
    for gap in refusing {
        for source in [
            format!("say 'main ran'\n{gap}{failing_class}"),
            format!("say 'main ran'\n{failing_class}{gap}"),
        ] {
            let outcome = routine_program(source.as_bytes());
            let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
            assert_eq!(
                outcome.exit_code, 158,
                "{source}: exit code, stderr {stderr}"
            );
            assert_eq!(outcome.stdout, b"", "{source}: stdout");
            assert!(
                stderr.contains("Error 98.903:  Unable to load library \"nosuchlib\"."),
                "{source}: stderr {stderr}"
            );
        }
    }
    for source in [
        format!("say 'main ran'\n::requires 'zzznosuchfile.rex'\n{failing_class}"),
        format!("say 'main ran'\n{failing_class}::requires 'zzznosuchfile.rex'\n"),
    ] {
        let outcome = routine_program(source.as_bytes());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert_eq!(
            outcome.exit_code, 213,
            "{source}: exit code, stderr {stderr}"
        );
        assert_eq!(outcome.stdout, b"", "{source}: stdout");
        assert!(
            stderr.contains("*-* ::requires 'zzznosuchfile.rex'\n")
                && stderr.contains(
                    "Error 43.901:  Could not find file \"zzznosuchfile.rex\" for ::REQUIRES."
                ),
            "{source}: stderr {stderr}"
        );
    }
    for source in [
        format!("say 'main ran'\n::options digits 12\n{failing_class}"),
        format!("say 'main ran'\n{failing_class}::options digits 12\n"),
    ] {
        let outcome = routine_program(source.as_bytes());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert_eq!(
            outcome.exit_code, 158,
            "{source}: exit code, stderr {stderr}"
        );
        assert_eq!(outcome.stdout, b"", "{source}: stdout");
        assert!(
            stderr.contains("Error 98.909:  Class \"ZZZNOTACLASS\" not found."),
            "{source}: stderr {stderr}"
        );
    }
}

/// **An unresolvable `::CLASS` keyword is diagnosed inside the class pass,
/// and that is asserted here rather than left incidental.**
#[test]
fn a_class_keyword_gap_is_raised_inside_the_class_pass() {
    let failing_class = "::class a subclass zzznotaclass\n";
    let cycle = "::class a subclass b\n::class b subclass a\n";
    let cases: &[&str] = &[
        "::class q subclass ns:other\n",
        "::class q mixinclass ns:other\n",
        "::class q inherit ns:other\n",
        "::class q metaclass ns:other\n",
    ];
    for gap in cases {
        let source = format!("say 'main ran'\n{gap}{failing_class}");
        let outcome = routine_program(source.as_bytes());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert_eq!(
            outcome.exit_code, 158,
            "{source}: exit code, stderr {stderr}"
        );
        assert_eq!(outcome.stdout, b"", "{source}: stdout");
        assert!(
            stderr.contains("Error 98.987:  Namespace \"NS\" not found in package"),
            "{source}: stderr {stderr}"
        );
        assert!(
            stderr.contains(&format!("*-* {}", gap.trim_end())),
            "{source}: the report does not echo the qualifier's own clause: {stderr}"
        );

        let source = format!("say 'main ran'\n{gap}{cycle}");
        let outcome = routine_program(source.as_bytes());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert_eq!(
            outcome.exit_code, 158,
            "{source}: exit code, stderr {stderr}"
        );
        assert_eq!(outcome.stdout, b"", "{source}: stdout");
        assert!(
            stderr.contains("Error 98.911:  Cyclic inheritance in program"),
            "{source}: stderr {stderr}"
        );
    }
}

/// **When one directive owes both a translation error and a gap, the
/// translation error is the answer**, which is where the gap check sits
/// relative to `Interp::install_directives`' first loop.
/// ```text
/// say 'main ran'                                     rc 157
/// ::routine dup                                        4 *-* ::routine dup external ...
///   return 1                                         Error 99.903: Duplicate ::ROUTINE
/// ::routine dup external "LIBRARY nosuchlib ..."     directive instruction.
/// ```
#[test]
fn a_directive_owing_both_a_translation_error_and_a_gap_answers_the_translation_error() {
    let outcome = routine_program(
        b"say 'main ran'\n::routine dup\n  return 1\n          ::routine dup external \"LIBRARY nosuchlib nosuchfn\"\n",
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
    assert_eq!(outcome.exit_code, 157, "exit code, stderr {stderr}");
    assert_eq!(outcome.stdout, b"", "stdout");
    assert!(
        stderr.contains("Error 99.903:  Duplicate ::ROUTINE directive instruction."),
        "stderr {stderr}"
    );
    assert!(
        stderr.contains("     4 *-* ::routine dup external"),
        "the blamed clause must be the second directive, not the first: stderr {stderr}"
    );

    let outcome = routine_program(
        b"say 'main ran'\n::routine dup external \"LIBRARY nosuchlib nosuchfn\"\n          ::routine dup\n  return 1\n",
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
    assert_eq!(outcome.exit_code, 158, "exit code, stderr {stderr}");
    assert_eq!(outcome.stdout, b"", "stdout");
    assert!(
        stderr.contains("Error 98.903:  Unable to load library \"nosuchlib\"."),
        "stderr {stderr}"
    );
}
