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
