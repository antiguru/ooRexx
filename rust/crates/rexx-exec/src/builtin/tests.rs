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

use super::*;

/// The three partial rows are builtin names, and the whole exclusions are
/// not.
#[test]
fn the_partial_exclusions_are_builtin_names_and_the_whole_ones_are_not() {
    for name in rexx_inventory::builtins::PARTIALLY_EXCLUDED {
        assert!(
            is_builtin(name.as_bytes()),
            "{name} is excluded only in part, so its in-scope form must dispatch"
        );
    }
    for name in rexx_inventory::builtins::wholly_excluded() {
        assert!(
            !is_builtin(name.as_bytes()),
            "{name} is excluded outright, so nothing here may claim it"
        );
    }
    assert!(is_builtin(b"LENGTH"));
    assert!(!is_builtin(b"length"), "the table is not case-folded");
    assert!(!is_builtin(b"ZORKOLO"));
    assert!(!is_builtin(&[0xff, 0xfe]), "and does not need valid UTF-8");
}

/// [`resolve`] partitions the in-scope set exactly, and answers `None`
/// for everything outside it.
#[test]
fn resolve_partitions_the_in_scope_set_into_rows_and_gaps() {
    let mut rows_seen = 0;
    let mut gaps_seen = 0;
    for name in in_scope() {
        let bytes = name.as_bytes();
        match (rows().get(bytes), resolve(bytes)) {
            (Some(row), Some(BuiltinTarget::Row(answered))) => {
                assert_eq!(*row, answered, "{name} resolved to the wrong row");
                rows_seen += 1;
            }
            (None, Some(BuiltinTarget::Gap)) => gaps_seen += 1,
            (row, answered) => {
                panic!("{name}: row map says {row:?}, resolve says {answered:?}")
            }
        }
    }
    assert_eq!(
        rows_seen,
        IMPLEMENTED.len(),
        "every row is reachable by name"
    );

    // **`BuiltinTarget::Gap` is unreachable today, and this is where that
    // is written down.** Phase 4's in-scope set and this crate's table are
    // currently the same 66 names, so no call can produce it -- found by
    // this test failing an earlier assertion that demanded a gap exist.
    assert_eq!(
        gaps_seen,
        0,
        "an in-scope builtin has no row: {} in scope against {} rows",
        in_scope().len(),
        IMPLEMENTED.len()
    );

    assert_eq!(resolve(b"ZORKOLO"), None, "not a builtin at all");
    assert_eq!(resolve(b"length"), None, "the table is not case-folded");
    assert_eq!(
        resolve(&[0xff, 0xfe]),
        None,
        "and does not need valid UTF-8"
    );
}

/// Every implemented row names a real in-scope builtin.
#[test]
fn every_implemented_row_names_an_in_scope_builtin() {
    for builtin in IMPLEMENTED {
        let name = std::str::from_utf8(builtin.name).expect("a builtin name is ASCII");
        assert!(
            is_builtin(builtin.name),
            "{name} has an implementation but is not an in-scope builtin name"
        );
        assert!(
            builtin.max.is_none_or(|max| max >= builtin.min),
            "{name} has a maximum below its minimum"
        );
    }
}

/// `dispatch` answers `None` for a name that is not a builtin, which is
/// the answer resolution needs to carry on past this step.
#[test]
fn dispatch_declines_a_name_that_is_not_a_builtin() {
    let mut interp = Interp::new();
    assert!(dispatch(&mut interp, b"ZORKOLO", &[]).is_none());
    assert!(
        dispatch(&mut interp, b"RXQUEUE", &[]).is_none(),
        "a whole exclusion is not a builtin name here either"
    );
}

fn never_run(_: &mut Interp, _: &'static [u8], _: Args<'_>) -> Result<ObjRef, Failure> {
    unreachable!("check_arity never runs the builtin")
}

/// A builtin with a required argument in a middle position, which is the
/// only shape that can reach 40.5. `LENGTH` takes one argument and a lone
/// omitted argument is a trailing omission that never arrives, so its own
/// row cannot produce that sub-code.
fn substr_arity() -> Builtin {
    let row = IMPLEMENTED
        .iter()
        .find(|builtin| builtin.name == b"SUBSTR")
        .expect("SUBSTR has a row");
    Builtin {
        run: never_run,
        ..*row
    }
}

/// The three incorrect-call sub-codes and their substitutions, against
/// the oracle transcripts in [`check_arity`]'s own doc.
#[test]
fn the_arity_checks_answer_the_oracles_own_sub_codes() {
    let value = ObjRef::small_int(1).expect("1 is a small int");

    let failure =
        check_arity(&substr_arity(), &[Some(value)]).expect_err("one argument is too few");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (40, 3));
    assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"2".to_vec()]);

    let five = [Some(value); 5];
    let failure = check_arity(&substr_arity(), &five).expect_err("five arguments are too many");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (40, 4));
    assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"4".to_vec()]);

    let failure = check_arity(&substr_arity(), &[Some(value), None, Some(value)])
        .expect_err("argument 2 is required");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (40, 5));
    assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"2".to_vec()]);

    // The adjacent success: an omission *past* the required positions is
    // not an error at all -- measured, `say substr('abc',2,)` prints
    // `bc`.
    check_arity(&substr_arity(), &[Some(value), Some(value), None]).expect("that call is legal");
}

/// The maximum is checked before the required positions, which is the one
/// ordering a program can tell apart.
#[test]
fn too_many_arguments_wins_over_a_missing_required_one() {
    let value = ObjRef::small_int(1).expect("1 is a small int");
    let failure = check_arity(
        &substr_arity(),
        &[None, Some(value), Some(value), Some(value), Some(value)],
    )
    .expect_err("five arguments are too many");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (40, 4));
}
/// The oracle's own source is what says which argument positions are
/// fetched raw, and this re-derives [`RAW_ARGUMENT_POSITIONS`] from it.
#[test]
fn the_raw_argument_table_re_derives_from_the_oracles_own_source() {
    /// The blocks that pass positions on without naming a constant for
    /// them, which the derivation below cannot classify. See this test's
    /// own doc for what ruled them.
    const EXTRA_ARGUMENT_BLOCKS: &[&str] = &["MAX", "MIN"];
    /// The accessors that run the protocol on the position they fetch.
    const CONVERTING: &[&str] = &[
        "required_string",
        "optional_string",
        "required_integer",
        "optional_integer",
        "required_big_integer",
        "optional_big_integer",
        "optional_pad",
    ];
    /// The accessors that are `stack->peek` and convert nothing.
    const RAW: &[&str] = &["get_arg", "optional_argument", "arg_exists", "arg_omitted"];

    let path = std::path::PathBuf::from(
        "/home/moritz/dev/repos/ooRexx/interpreter/expression/BuiltinFunctions.cpp",
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {} -- the builtin bodies this table derives from are part of \
             the read-only C++ tree, so a missing one means the tree moved rather than \
             that the fact is gone: {e}",
            path.display()
        )
    });

    let lines: Vec<&str> = text.lines().collect();
    let mut derived: Vec<(String, Vec<usize>)> = Vec::new();
    let mut blocks = 0usize;
    let mut extra_argument_blocks: Vec<String> = Vec::new();
    let mut at = 0usize;
    while at < lines.len() {
        let Some(name) = lines[at]
            .strip_prefix("BUILTIN(")
            .and_then(|rest| rest.trim_end().strip_suffix(')'))
        else {
            at += 1;
            continue;
        };
        // Brace-matched from the line after the signature, which is the
        // opening `{`, so a nested block cannot end the body early.
        let mut depth = 0isize;
        let mut end = at + 1;
        while end < lines.len() {
            depth += isize::try_from(lines[end].matches('{').count()).expect("a short line");
            depth -= isize::try_from(lines[end].matches('}').count()).expect("a short line");
            if depth == 0 && lines[end].contains('}') {
                break;
            }
            end += 1;
        }
        let body = lines[at + 1..end.min(lines.len())].join("\n");
        blocks += 1;
        if body.contains("stack->arguments(") {
            extra_argument_blocks.push(name.to_string());
        }

        // `const size_t <NAME>_<arg> = N;`, the block's own position
        // constants. `Min` and `Max` are the arity pair and not positions.
        let mut positions: Vec<(String, usize)> = Vec::new();
        for line in body.lines() {
            let Some(rest) = line.trim_start().strip_prefix("const size_t ") else {
                continue;
            };
            let Some((declared, value)) = rest.split_once('=') else {
                continue;
            };
            let Some(argument) = declared.trim().strip_prefix(&format!("{name}_")) else {
                continue;
            };
            if argument == "Min" || argument == "Max" {
                continue;
            }
            let Ok(value) = value.trim().trim_end_matches(';').trim().parse::<usize>() else {
                continue;
            };
            positions.push((argument.to_string(), value));
        }

        let mentions = |family: &[&str], argument: &str| {
            family.iter().any(|macro_name| {
                body.contains(&format!("{macro_name}({name}, {argument})"))
                    || body.contains(&format!("{macro_name}({name},{argument})"))
            })
        };
        let mut raw_only: Vec<usize> = positions
            .iter()
            .filter(|(argument, _)| mentions(RAW, argument) && !mentions(CONVERTING, argument))
            .map(|(_, value)| *value)
            .collect();
        raw_only.sort_unstable();
        if !raw_only.is_empty() {
            derived.push((name.to_string(), raw_only));
        }
        at = end + 1;
    }

    assert!(
        blocks > 0,
        "no BUILTIN(x) block parsed out of {} -- the file's shape moved",
        path.display()
    );
    extra_argument_blocks.sort();
    assert_eq!(
        extra_argument_blocks,
        EXTRA_ARGUMENT_BLOCKS
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>(),
        "the set of blocks that pass positions on without a constant has moved, and this \
         derivation cannot classify those positions -- run the probes this test's doc names \
         against the new member before widening the list"
    );

    // Restricted to what this crate can reach, for the reason the doc
    // gives.
    let mut derived: Vec<(String, Vec<usize>)> = derived
        .into_iter()
        .filter(|(name, _)| IMPLEMENTED.iter().any(|row| row.name == name.as_bytes()))
        .collect();
    derived.sort();
    let mut committed: Vec<(String, Vec<usize>)> = RAW_ARGUMENT_POSITIONS
        .iter()
        .map(|(name, positions)| {
            (
                String::from_utf8_lossy(name).into_owned(),
                positions.to_vec(),
            )
        })
        .collect();
    committed.sort();
    assert_eq!(
        derived,
        committed,
        "RAW_ARGUMENT_POSITIONS disagrees with {} -- a position the oracle fetches raw and \
         this table does not exempt is a silent wrong answer, and one it exempts and the \
         oracle converts is another",
        path.display()
    );
}

/// Every row of [`RAW_ARGUMENT_POSITIONS`] names a builtin this crate
/// implements, at a position that builtin can be given, and the table is
/// not empty.
#[test]
fn every_raw_argument_row_names_a_builtin_and_a_position_it_can_take() {
    assert!(
        !RAW_ARGUMENT_POSITIONS.is_empty(),
        "RAW_ARGUMENT_POSITIONS is empty, which passes every arm below \
         vacuously -- the oracle has at least one raw argument position and \
         `the_raw_argument_table_re_derives_from_the_oracles_own_source` \
         is what says which"
    );
    for (name, positions) in RAW_ARGUMENT_POSITIONS {
        let builtin = IMPLEMENTED
            .iter()
            .find(|row| row.name == *name)
            .unwrap_or_else(|| {
                panic!(
                    "RAW_ARGUMENT_POSITIONS names {}, which is not an implemented builtin",
                    String::from_utf8_lossy(name)
                )
            });
        assert!(
            !positions.is_empty(),
            "{} has an empty raw-position list, which is what omitting the row means",
            String::from_utf8_lossy(name)
        );
        for position in *positions {
            assert!(
                builtin.max.is_none_or(|max| *position <= max),
                "{} declares a raw position {position} past its own maximum",
                String::from_utf8_lossy(name)
            );
        }
    }
}
