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

//! The recorded-output harness: this crate's curated programs, each run once
//! and required to answer the bytes committed for it, plus a sweep that runs
//! every population to completion with no body refused.

use std::fs;
use std::path::{Path, PathBuf};

mod watchdog;

use rayon::prelude::*;
use rexx_exec::{Invocation, Outcome};
use rexx_extract::bif::extract_bif;
use rexx_extract::keyword::extract_keyword;
use rexx_extract::{AssertionRow, Form, extract_assertions, find_test_groups};

/// The brief's own first test: a two-clause program, byte for byte.
#[test]
fn the_smallest_program_answers() {
    let outcome = run(b"n1 = 2\nsay n1 + 3\n".to_vec());
    assert_eq!(outcome.stdout, b"5\n");
    assert_eq!(outcome.stderr, b"");
    assert_eq!(outcome.exit_code, 0);
}

fn run(text: Vec<u8>) -> Outcome {
    watchdog::run_bounded(INLINE_PATH, text, Invocation::none())
}

/// One inline program with the answer recorded for it.
struct InlineCase {
    name: &'static str,
    program: &'static str,
    stdout: &'static str,
    /// The trace sink, empty for a case that sets no `TRACE`. Written out in
    /// full for the traced cases rather than summarised: a construct's trace
    /// is where a re-implementation diverges first, because a `DO`/`END` pair
    /// re-echoes per pass and an `IF`'s `THEN`/`ELSE` markers echo at lines
    /// and indents no `SAY` can observe.
    stderr: &'static str,
    /// The exit status, so a case whose whole point is a raised condition
    /// pins the status as well as the message.
    exit_code: i32,
}

/// The shapes the loop promotion has to keep: one per `LoopKind` this crate
/// runs, both `LoopConditional` spellings, both `LEAVE` and `ITERATE`, and two
/// traced loops.
const LOOP_CASES: &[InlineCase] = &[
    InlineCase {
        name: "controlled",
        program: "do i = 1 to 3\n  say i\nend\n",
        stdout: "1\n2\n3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The per-pass control-variable re-read, which is the behaviour a
        // reviewer once caught being called unreachable: the body writes the
        // control variable and the next pass reads `10` back, adds the `BY`,
        // and stops because `11 > 3`.
        name: "controlled, body writes the control variable",
        program: "do i = 1 to 3\n  i = 10\nend\nsay i\n",
        stdout: "11\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "controlled with BY and FOR",
        program: "do i = 1 to 10 by 3 for 2\n  say i\nend\nsay i\n",
        stdout: "1\n4\n7\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "bare repeat count",
        program: "do 3\n  say 'zz'\nend\n",
        stdout: "zz\nzz\nzz\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "while",
        program: "n1 = 0\ndo while n1 < 3\n  n1 = n1 + 1\n  say n1\nend\n",
        stdout: "1\n2\n3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "until",
        program: "n1 = 0\ndo until n1 >= 3\n  n1 = n1 + 1\n  say n1\nend\n",
        stdout: "1\n2\n3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "forever with leave",
        program: "n1 = 0\ndo forever\n  n1 = n1 + 1\n  if n1 = 3 then leave\nend\nsay n1\n",
        stdout: "3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The label search, across two frames: the `ITERATE` names the outer
        // loop from inside the inner one, so the inner loop is left and the
        // outer one re-tested.
        name: "nested, labelled iterate",
        program: "zz = 0\ndo label lbl outer = 1 to 3\n  do inner = 1 to 3\n    \
                  if inner = 2 then iterate lbl\n    zz = zz + 1\n  end\nend\nsay zz\n",
        stdout: "3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // `DO OVER` in the single-iteration form this crate implements for a
        // non-stem target: the target yields itself, once.
        name: "do over a non-stem target",
        program: "do qq over 4.5\n  say qq\nend\n",
        stdout: "4.5\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A block, not a loop: exactly one pass, and its `END` echoes once.
        name: "simple block",
        program: "do\n  say 'a'\n  say 'zz'\nend\nsay 'after'\n",
        stdout: "a\nzz\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The same block reached through an `IF`'s true branch, which is a
        // different route into it: `If` steps its branch itself, so this
        // block's own clauses arrive from `If`'s driver rather than from the
        // enclosing body's.
        name: "simple block inside an if",
        program: "if 1 = 1 then do\n  say 'a'\n  say 'zz'\nend\n",
        stdout: "a\nzz\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A labelled `Simple` block is leavable by name where an unlabelled
        // one is not, and it owns a search frame either way.
        name: "labelled simple block, left by name",
        program: "do label blk\n  say 'a'\n  leave blk\n  say 'never'\nend\nsay 'after'\n",
        stdout: "a\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The `DO` clause re-echoes once per pass and `END` once per pass that
        // falls through to it, and the control step's four intermediate lines
        // straddle the `BY` addition.
        name: "controlled under trace i",
        program: "trace i\ndo i = 1 to 2\n  nop\nend\n",
        stdout: "",
        stderr: "     2 *-* do i = 1 to 2\n       >L>   \"1\"\n       >L>   \"2\"\n       \
                 >K>   \"TO\" => \"2\"\n       >=>   I <= \"1\"\n     3 *-*   nop\n     \
                 4 *-* end\n     2 *-* do i = 1 to 2\n       >V>     I => \"1\"\n       \
                 >>>     \"1\"\n       >>>     \"2\"\n       >=>     I <= \"2\"\n     \
                 3 *-*   nop\n     4 *-* end\n     2 *-* do i = 1 to 2\n       \
                 >V>     I => \"2\"\n       >>>     \"2\"\n       >>>     \"3\"\n       \
                 >=>     I <= \"3\"\n",
        exit_code: 0,
    },
    InlineCase {
        // `UNTIL` gets a second, unconditional `DO` re-echo of its own,
        // between `END` and the test, and no top-of-loop one.
        name: "until under trace r",
        program: "trace r\nn1 = 0\ndo until n1 >= 2\n  n1 = n1 + 1\nend\n",
        stdout: "",
        stderr: "     2 *-* n1 = 0\n       >>>   \"0\"\n     3 *-* do until n1 >= 2\n     \
                 4 *-*   n1 = n1 + 1\n       >>>     \"1\"\n     5 *-* end\n     \
                 3 *-* do until n1 >= 2\n       >K>     \"UNTIL\" => \"0\"\n     \
                 4 *-*   n1 = n1 + 1\n       >>>     \"2\"\n     5 *-* end\n     \
                 3 *-* do until n1 >= 2\n       >K>     \"UNTIL\" => \"1\"\n",
        exit_code: 0,
    },
];

/// What a branch promotion has to keep, in two kinds.
const BRANCH_CASES: &[InlineCase] = &[
    InlineCase {
        name: "if with an else, true path",
        program: "if 1 = 1 then say 'then'\nelse say 'else'\nsay 'after'\n",
        stdout: "then\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The path the compiled form changes most: a jump has to land on the
        // same `ELSE` the enclosing loop's fallthrough reaches.
        name: "if with an else, false path",
        program: "if 1 = 0 then say 'then'\nelse say 'else'\nsay 'after'\n",
        stdout: "else\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "if with no else, false path",
        program: "if 1 = 0 then say 'then'\nsay 'after'\n",
        stdout: "after\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // Its own case rather than the mirror of the one above, because the
        // compiled form of a branch with no `ELSE` emits no branch-end jump:
        // the true path leaves by falling off its end, which is a different
        // mechanism from every other true path here.
        name: "if with no else, true path",
        program: "if 1 = 1 then say 'then'\nsay 'after'\n",
        stdout: "then\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The outer `ELSE` has to be skipped by the inner branch's own
        // resume, which is the one target neither `false_target` gives.
        name: "nested if, both true",
        program: "if 1 = 1 then\n  if 2 = 2 then say 'inner'\n  else say 'inner else'\n\
                  else say 'outer else'\nsay 'after'\n",
        stdout: "inner\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A null clause as the whole consequence: the true branch is the
        // `THEN` marker and nothing else.
        name: "empty then",
        program: "if 1 = 1 then;\nsay 'after'\n",
        stdout: "after\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "select with otherwise, second when matches",
        program: "select\n  when 1 = 0 then say 'a'\n  when 2 = 2 then say 'b'\n  \
                  otherwise say 'o'\nend\nsay 'after'\n",
        stdout: "b\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "select with no matching when",
        program: "select\n  when 1 = 0 then say 'a'\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     3 *-* end\nError 7 running <PATH> line 3:  WHEN or OTHERWISE expected.\n\
                 Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n",
        exit_code: 249,
    },
    InlineCase {
        name: "select case with no matching when",
        program: "select case 2\n  when 1 then say 'a'\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     3 *-* end\nError 7 running <PATH> line 3:  WHEN or OTHERWISE expected.\n\
                 Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n",
        exit_code: 249,
    },
    InlineCase {
        // The `>K>   "CASE"` line, the two `>>>` lines each `WHEN CASE`
        // comparison produces, and the `THEN` marker: a `SELECT CASE`'s whole
        // trace, which is where a re-implementation of the scan diverges first.
        name: "select case under trace r, second when matches",
        program: "trace r\nselect case 1 + 1\n  when 1 then say 'a'\n  when 2 then say 'b'\n  \
                  otherwise say 'o'\nend\nsay 'after'\n",
        stdout: "b\nafter\n",
        stderr: "     2 *-* select case 1 + 1\n       >K>   \"CASE\" => \"2\"\n     3 *-*   \
                 when 1 \n       >>>     \"1\"\n       >>>     \"0\"\n     4 *-*   when 2 \n       \
                 >>>     \"2\"\n       >>>     \"1\"\n     4 *-*     then\n     4 *-*       \
                 say 'b'\n       >>>         \"b\"\n     7 *-* say 'after'\n       \
                 >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The `OTHERWISE` marker echoes on its own line at the scan level, and
        // the `END` that closes an `OTHERWISE` echoes and does nothing --
        // neither is reached by any path a matched `WHEN` takes.
        name: "select with otherwise under trace r",
        program: "trace r\nselect\n  when 1 = 0 then say 'a'\n  otherwise say 'o'\nend\n\
                  say 'after'\n",
        stdout: "o\nafter\n",
        stderr: "     2 *-* select\n     3 *-*   when 1 = 0 \n       >>>     \"0\"\n     \
                 4 *-*   otherwise\n     4 *-*     say 'o'\n       >>>       \"o\"\n     \
                 5 *-* end\n     6 *-* say 'after'\n       >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // An **absorbed** `WHEN CASE` -- one this `SELECT`'s `whens` never
        // collected, because it is the listed `WHEN`'s own `THEN` consequence
        // -- whose false path escapes the matched body and lands on the `END`.
        // Its residual indent rides along, so the 7.3 clause echoes four
        // columns further in than the `END`'s own position.
        name: "select case whose absorbed when case falls through to 7.3",
        program: "trace r\nselect case 2\n  when 2 then\n  when 3 then nop\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     2 *-* select case 2\n       >K>   \"CASE\" => \"2\"\n     3 *-*   \
                 when 2 \n       >>>     \"2\"\n       >>>     \"1\"\n     3 *-*     then\n     \
                 4 *-*       when 3 \n       >>>         \"3\"\n       >>>         \"0\"\n     \
                 5 *-*     end\n     5 *-*     end\nError 7 running <PATH> line 5:  WHEN or \
                 OTHERWISE expected.\nError 7.3:  All WHEN expressions of SELECT are false; \
                 OTHERWISE expected.\n",
        exit_code: 249,
    },
    InlineCase {
        // F-EX1: the same absorbed escape landing exactly on this `SELECT`'s
        // own `OTHERWISE` marker, which has to run with this `SELECT`'s search
        // frame still standing -- the `leave s` is what says so, since a
        // redirect that lost the frame reports 28.3 instead.
        name: "select label whose absorbed when case escapes into otherwise",
        program: "select label s case 2\n  when 2 then\n  when 3 then nop\n  otherwise say 'O'\n  \
                  leave s\nend\nsay 'after'\n",
        stdout: "O\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `SELECT` is never a repetitive block, so an `ITERATE` naming one is
        // 28.5 even though the name matches -- the one flow a matching label
        // does not consume.
        name: "iterate naming a select is 28.5",
        program: "select label s\n  when 1 = 1 then iterate s\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     2 *-*       iterate s\nError 28 running <PATH> line 2:  Invalid LEAVE or \
                 ITERATE.\nError 28.5:  Symbol following ITERATE (\"S\") does not match a \
                 repetitive block instruction.\n",
        exit_code: 228,
    },
    InlineCase {
        // A matching `LEAVE` is consumed and resumes past the whole `SELECT`,
        // from inside a `DO` block that is itself the matched branch -- so the
        // search walks the block's frame before it reaches the `SELECT`'s.
        name: "leave naming a select from inside a do block in its branch",
        program: "select label s\n  when 1 = 1 then do\n    say 'in'\n    leave s\n    \
                  say 'never'\n  end\nend\nsay 'after'\n",
        stdout: "in\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A bare `LEAVE` from inside a matched `WHEN` is not the `SELECT`'s: it
        // is forwarded outward and consumed by the enclosing loop.
        name: "select inside a loop body, leaving the loop from a when",
        program: "zn = 0\ndo i = 1 to 5\n  select\n    when i = 3 then leave\n    \
                  otherwise zn = zn + 1\n  end\nend\nsay zn\n",
        stdout: "2\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A `SELECT` resumes at two different places, and this is the one
        // that is not the other.** A matching `LEAVE` from `OTHERWISE` resumes
        // *past* the `END`, where the same branch falling off its own end runs
        // the `END` (the pair below). Measured against the oracle under
        // `trace r`: no `5 *-* end` line here and one there. A single resume
        // for both echoes an `END` the oracle does not.
        name: "leave naming a select from inside its otherwise",
        program: "trace r\nselect label s\n  when 1 = 0 then nop\n  otherwise leave s\nend\n\
                  say 'after'\n",
        stdout: "after\n",
        stderr: "     2 *-* select label s\n     3 *-*   when 1 = 0 \n       >>>     \"0\"\n     \
                 4 *-*   otherwise\n     4 *-*     leave s\n     6 *-* say 'after'\n       \
                 >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The adjacent success for the case above: the same `OTHERWISE`
        // finishing normally *does* run the `END`, which is the arrival that
        // makes the two resumes different rather than one being wrong.
        name: "an otherwise that finishes normally runs its own end",
        program: "trace r\nselect label s\n  when 1 = 0 then nop\n  otherwise nop\nend\n\
                  say 'after'\n",
        stdout: "after\n",
        stderr: "     2 *-* select label s\n     3 *-*   when 1 = 0 \n       >>>     \"0\"\n     \
                 4 *-*   otherwise\n     4 *-*     nop\n     5 *-* end\n     6 *-* say 'after'\n  \
                 \x20    >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The escape elevation an absorbed `WHEN CASE` sets is restored once
        // `OTHERWISE`'s whole dispatch is over, and this is the program that
        // says so: the `END` and the clause after the `SELECT` both trace at
        // their own positions. Left in force they would each print four
        // columns further in, which no case whose `OTHERWISE` is the last
        // thing that traces can see.
        name: "the escape elevation is restored after an escaped otherwise",
        program: "trace r\nselect case 2\n  when 2 then\n  when 3 then nop\n  otherwise nop\nend\n\
                  say 'after'\n",
        stdout: "after\n",
        stderr: "     2 *-* select case 2\n       >K>   \"CASE\" => \"2\"\n     3 *-*   when 2 \n  \
                 \x20    >>>     \"2\"\n       >>>     \"1\"\n     3 *-*     then\n     \
                 4 *-*       when 3 \n       >>>         \"3\"\n       >>>         \"0\"\n     \
                 5 *-*       otherwise\n     5 *-*         nop\n     6 *-* end\n     \
                 7 *-* say 'after'\n       >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // A `SELECT CASE` inside a matched `WHEN`'s branch of another: the
        // inner one's own value must not reclaim the register the outer one is
        // still comparing later `WHEN`s against, and the outer's must survive
        // the inner construct entirely.
        name: "select case nested in a matched when of another",
        program: "select case 2\n  when 1 then say 'no'\n  when 2 then do\n    \
                  select case 'zz'\n      when 'zz' then say 'inner'\n      \
                  otherwise say 'inner o'\n    end\n  end\n  when 3 then say 'no3'\n  \
                  otherwise say 'outer o'\nend\nsay 'after'\n",
        stdout: "inner\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `SELECT` inside an `INTERPRET` fragment, which compiles to no
        // chunk at all: `run_fragment` walks its instructions rather than
        // driving a stream, so this says the construct still works from the
        // one path a compiled program does not take.
        name: "select inside an interpret fragment",
        program: "interpret \"select; when 1 = 1 then say 'frag'; end\"\nsay 'after'\n",
        stdout: "frag\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `LEAVE` naming nothing at all, forwarded past the `SELECT` and then
        // past the loop until the search runs out: the indent it is reported at
        // is the residual each forwarded-past frame resets, which is what
        // `pop_search_frame` decides.
        name: "leave naming nothing, forwarded out of a when",
        program: "do i = 1 to 1\n  select\n    when 1 = 1 then leave nope\n  end\nend\n",
        stdout: "",
        stderr: "     3 *-* leave nope\nError 28 running <PATH> line 3:  Invalid LEAVE or \
                 ITERATE.\nError 28.3:  Symbol following LEAVE (\"NOPE\") must either match the \
                 label of a current loop or block instruction.\n",
        exit_code: 228,
    },
    InlineCase {
        // The same from inside `OTHERWISE`, which reaches the forwarding
        // through a different dispatch than a matched `WHEN` does.
        name: "leave naming nothing, forwarded out of an otherwise",
        program: "do i = 1 to 1\n  select\n    when 1 = 0 then nop\n    otherwise leave nope\n  \
                  end\nend\n",
        stdout: "",
        stderr: "     4 *-* leave nope\nError 28 running <PATH> line 4:  Invalid LEAVE or \
                 ITERATE.\nError 28.3:  Symbol following LEAVE (\"NOPE\") must either match the \
                 label of a current loop or block instruction.\n",
        exit_code: 228,
    },
    InlineCase {
        // **A boundary case, not a shape.** The handler is queued by the
        // `SELECT CASE` expression and has to run at the *header's* own clause
        // boundary, before the first `WHEN` is tested -- the matched branch
        // reads the variable the handler set. A header that never ends its own
        // clause delivers late and prints `v unset`.
        name: "a call on handler queued by a select case expression",
        program: "call on user zx name h\nzv = 'unset'\nselect case raiser()\n  \
                  when 'V' then say 'v' zv\n  otherwise say 'o'\nend\nexit\nraiser:\n\
                  raise user zx return 'V'\nh:\nzv = 'set'\nreturn\n",
        stdout: "v set\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A boundary case, and the one a review found that this table's own
        // author had argued was unreachable.** The premise it was missing:
        // a delivered handler can *leave a new trap queued behind it*, and a
        // boundary drains only what was queued when it began, so that new trap
        // is not one it owes. So the last member clause's boundary is not the
        // last boundary with work, which is the assumption a flattened
        // construct's single boundary rests on.
        name: "a handler that queues again at a matched when's own branch end",
        program: "call on user zx name h\ncall on user zy name g\nselect\n\
                  when 1 = 1 then zq = raiser()\nend\nsay 'after'\nexit\nraiser:\n\
                  raise user zx return 'V'\nh:\nraise user zy return 1\ng:\nsay 'G ran' sigl\n\
                  return\n",
        stdout: "G ran 4\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **One boundary per construct, not one per branch entered.** The
        // absorbed `WHEN CASE`'s own clause queues `zy` and its false path
        // escapes onto the `OTHERWISE` marker, which is F-EX1's redirect --
        // and a redirect is this `SELECT` still running, so the branch it left
        // owes no end-of-branch boundary. The delivery therefore lands at the
        // `OTHERWISE` marker's own clause, `SIGL` 6, where a `SELECT` that
        // closed the branch it was redirected out of would report 5.
        name: "a handler that queues again where a when is redirected to otherwise",
        program: "call on user zx name h\ncall on user zy name g\nselect case 2\nwhen 2 then\n\
                  when raiser() then nop\notherwise say 'O'\nend\nsay 'after'\nexit\nraiser:\n\
                  raise user zx return 'V'\nh:\nraise user zy return 1\ng:\nsay 'G ran' sigl\n\
                  return\n",
        stdout: "G ran 6\nO\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The `IF` spelling of the case above, which is the same defect one
        // construct over and was already there before `SELECT` was promoted.
        // Measured: `G ran 3` then `after`.
        name: "a handler that queues again at an if's own branch end",
        program: "call on user zx name h\ncall on user zy name g\nif 1 = 1 then zq = raiser()\n\
                  say 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\n\
                  raise user zy return 1\ng:\nsay 'G ran' sigl\nreturn\n",
        stdout: "G ran 3\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A boundary case, not a shape, and the one that found a live
        // defect.** The handler is queued by a listed `WHEN`'s own condition
        // and delivered at *that* `WHEN`'s clause boundary, where it fails --
        // so the clause blamed is the `WHEN`, and the handler's own clauses
        // print at the `WHEN`'s indent plus two. Measured against the oracle:
        // `3 *-*   when raiser() = 'V'` at indent 2 and `11 *-*     say 1/0`
        // at indent 4. Before a `WHEN` was opened by the clause unit this
        // echoed `2 *-* select` at indent 0 and the handler two columns short,
        // with every other test in the workspace green.
        name: "a call on handler failing at a listed when's own boundary",
        program: "call on user zx name h\nselect\n  when raiser() = 'V' then say 'then'\n  \
                  otherwise say 'o'\nend\nsay 'after'\nexit\nraiser:\nraise user zx return 'V'\n\
                  h:\nsay 1/0\nreturn\n",
        stdout: "",
        stderr: "    11 *-*     say 1/0\n     3 *-*   when raiser() = 'V' \nError 42 running \
                 <PATH> line 11:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic \
                 overflow; divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // **A boundary case, not a shape.** The handler is queued inside a
        // matched `WHEN`'s body clause and fails at *that* clause's boundary,
        // so the clause blamed is the body's own and not the `WHEN`'s or the
        // `SELECT`'s.
        name: "a call on handler failing at a when body clause's boundary",
        program: "call on user zx name h\nselect\n  when 1 = 1 then say raiser()\n  \
                  otherwise nop\nend\nsay 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\n\
                  say 1/0\nreturn\n",
        stdout: "V\n",
        stderr: "    11 *-*         say 1/0\n     3 *-*       say raiser()\nError 42 running \
                 <PATH> line 11:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic \
                 overflow; divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // **A boundary case, not a shape.** A matched `WHEN`'s body does not
        // inherit the first-instruction permission any more than a branch
        // does, so a `PROCEDURE` there is 17.1.
        name: "procedure reached through a matched when",
        program: "call sub 1\nexit\nsub:\n  select\n    when arg(1) = 1 then procedure\n    \
                  otherwise nop\n  end\n  return\n",
        stdout: "",
        stderr: "     5 *-*         procedure\n     1 *-* call sub 1\nError 17 running <PATH> \
                 line 5:  Unexpected PROCEDURE.\nError 17.1:  PROCEDURE is valid only when it is \
                 the first instruction executed after an internal CALL or function \
                 invocation.\n",
        exit_code: 239,
    },
    InlineCase {
        // The `IF`'s own clause is what the failure is attributed to, and its
        // echo carries the clause text up to the `THEN`.
        name: "if condition is not a logical value",
        program: "if 'x' then say 'y'\n",
        stdout: "",
        stderr: "     1 *-* if 'x' \nError 34 running <PATH> line 1:  Logical value not 0 or 1.\n\
                 Error 34.1:  Value of expression following IF keyword must be exactly \"0\" \
                 or \"1\"; found \"x\".\n",
        exit_code: 222,
    },
    InlineCase {
        // The trap offer is made once, by the activation the condition
        // unwound, whether or not the `IF` resolves its branch inside its own
        // clause step.
        name: "if condition raises into a signal on trap",
        program: "signal on syntax\nif 1/0 = 1 then say 'y'\nsay 'unreached'\nexit\n\
                  syntax:\nsay 'trapped' rc\n",
        stdout: "trapped 42\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The `THEN` marker echoes at the `IF`'s own line and the consequence
        // two columns further in, both of them clauses in their own right.
        name: "if under trace i, true path",
        program: "trace i\nif 1 = 1 then say 'y'\nelse say 'n'\n",
        stdout: "y\n",
        stderr: "     2 *-* if 1 = 1 \n       >L>   \"1\"\n       >L>   \"1\"\n       \
                 >O>   \"=\" => \"1\"\n       >>>   \"1\"\n     2 *-*   then\n     \
                 2 *-*     say 'y'\n       >L>       \"y\"\n       >>>       \"y\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The `ELSE` marker echoes at its own line, which is the false path's
        // whole observable difference from the true one.
        name: "if under trace r, false path",
        program: "trace r\nif 1 = 0 then say 'y'\nelse say 'n'\n",
        stdout: "n\n",
        stderr: "     2 *-* if 1 = 0 \n       >>>   \"0\"\n     3 *-*   else\n     \
                 3 *-*     say 'n'\n       >>>       \"n\"\n",
        exit_code: 0,
    },
    InlineCase {
        // A `LEAVE` from inside a true branch inside a loop body: the flow
        // has to escape the branch and be consumed by the loop.
        name: "if inside a loop body, leaving it",
        program: "n1 = 0\ndo i = 1 to 5\n  if i = 3 then leave\n  n1 = n1 + 1\nend\nsay n1\n",
        stdout: "2\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `DO` block as the whole true branch, with an `ELSE` after it: the
        // block's own resume lands on the `ELSE`, which must not run.
        name: "if whose true branch is a do block, with an else",
        program: "if 1 = 1 then do\n  say 'a'\n  say 'b'\nend\nelse say 'c'\nsay 'after'\n",
        stdout: "a\nb\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "if in a called label",
        program: "call sub\nsay result\nexit\nsub:\n  if 1 = 1 then say 'in sub'\n  return 7\n",
        stdout: "in sub\n7\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A boundary case, not a shape.** The `CALL ON` handler is queued by
        // the `IF`'s own condition and delivered at the `IF`'s clause
        // boundary, where it raises -- so the clause the failure is attributed
        // to is the one whose boundary ran the handler. Measured against the
        // oracle: `2 *-* if raiser() = 'V'`. A promoted clause that records
        // its own failure but not its boundary's prints only the handler's
        // line here, and no shape case can see that, because every shape case
        // asks what a branch does rather than what its clause unit owes.
        name: "a call on handler failing at an if's own boundary",
        program: "call on user zx name h\nif raiser() = 'V' then say 'then'\nelse say 'else'\n\
                  say 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\nsay 1/0\nreturn\n",
        stdout: "",
        stderr: "     9 *-*   say 1/0\n     2 *-* if raiser() = 'V' \nError 42 running <PATH> \
                 line 9:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic overflow; \
                 divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // The same boundary one construct deeper, where the failure to record
        // is not merely silent: the enclosing `DO`'s own site wins the
        // first-wins race instead, so the wrong clause is echoed rather than
        // none. Measured against the oracle: `3 *-* if raiser() = 'V'`, not
        // `2 *-* do i = 1 to 1`.
        name: "a call on handler failing at an if's boundary inside a loop",
        program: "call on user zx name h\ndo i = 1 to 1\n  if raiser() = 'V' then say 'then'\n\
                  end\nsay 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\nzq = 1/0\nreturn\n",
        stdout: "",
        stderr: "    10 *-*     zq = 1/0\n     3 *-*   if raiser() = 'V' \nError 42 running \
                 <PATH> line 10:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic \
                 overflow; divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // A branch does not inherit the first-instruction permission, so a
        // `PROCEDURE` reached through one is 17.1 -- the observable that says
        // `grant_procedure_permission` is spent identically under both
        // engines.
        name: "procedure reached through a true branch",
        program: "call sub 1\nexit\nsub:\n  if arg(1) = 1 then procedure\n  return\n",
        stdout: "",
        stderr: "     4 *-*       procedure\n     1 *-* call sub 1\nError 17 running <PATH> \
                 line 4:  Unexpected PROCEDURE.\nError 17.1:  PROCEDURE is valid only when it \
                 is the first instruction executed after an internal CALL or function \
                 invocation.\n",
        exit_code: 239,
    },
];

/// Every [`LOOP_CASES`] program, byte for byte, against the bytes recorded
/// for it.
#[test]
fn every_loop_shape_answers_as_recorded() {
    compare_inline_cases(LOOP_CASES);
}

/// Every [`BRANCH_CASES`] program, the same way.
#[test]
fn every_branch_shape_answers_as_recorded() {
    compare_inline_cases(BRANCH_CASES);
}

/// Where the case files this harness reads live.
const CASE_DIR: &str = "tests/ir_recorded_cases";

/// The cases in [`CASE_DIR`], which are the ones written as data rather than
/// as a `const` table in this file (the plan's Tech Stack section: a data file
/// from Task 6 onward, and the three existing tables not retrofitted because
/// they hold witnesses nothing else catches).
#[test]
fn every_case_file_answers_as_recorded() {
    assert!(
        std::env::var_os("REWRITE").is_none(),
        "REWRITE would replace this crate's oracle-measured expectations with whatever it \
         currently prints. Regenerate a row from the oracle instead -- {CASE_DIR}'s own header \
         has the command"
    );
    let mut cases = 0;
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            render_engine(&case.input)
        });
    });
    // A directory that produced no *stanzas* -- because it was emptied, renamed,
    // or because every stanza's directive stopped being recognised -- otherwise
    // passes this test with nothing run at all.
    assert!(
        cases > 0,
        "{CASE_DIR} produced no cases, so this test asserts nothing"
    );
}

/// Runs one case's program and renders its answer in the tagged form the case
/// files record.
fn render_engine(program: &str) -> String {
    let outcome = run(program.as_bytes().to_vec());
    assert_eq!(outcome.chunks_refused, 0, "the compiler refused this body");

    let mut out = format!("rc> {}\n", outcome.exit_code);
    for line in String::from_utf8_lossy(&outcome.stdout).lines() {
        out.push_str(&format!("out> {line}\n"));
    }
    for line in String::from_utf8_lossy(&outcome.stderr).lines() {
        out.push_str(&format!("err> {line}\n"));
    }
    out
}

/// Runs each case and asserts its recorded bytes.
fn compare_inline_cases(cases: &[InlineCase]) {
    assert!(!cases.is_empty(), "an empty case table asserts nothing");
    for case in cases {
        let outcome = run(case.program.as_bytes().to_vec());
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            case.stdout,
            "[{}] stdout moved",
            case.name
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            case.stderr.replace("<PATH>", INLINE_PATH),
            "[{}] trace moved",
            case.name
        );
        assert_eq!(
            outcome.exit_code, case.exit_code,
            "[{}] exit status moved",
            case.name
        );
        assert_eq!(
            outcome.chunks_refused, 0,
            "[{}] the compiler refused this body",
            case.name
        );
    }
}

// **The two known engine divergences are gone with the engine.** Both were a
// `CALL ON` handler that itself raised a second trapped condition, where the
// tree-walker dropped a queued handler the compiled stream delivered. The
// table recorded what each engine printed and what the oracle printed, and in
// both rows the compiled stream's answer was the oracle's -- so retiring the
// tree-walker settles them by removing the side that was wrong, and there is
// no divergence left to record. The programs themselves are not lost: the
// oracle harnesses run that shape.
const INLINE_PATH: &str = "/nonexistent/ir-dual-case.rex";

/// One program, run once per engine.
struct Case {
    /// What a divergence report names this case by.
    name: String,
    /// The path the program is reported under, which reaches output through
    /// a raised condition's middle line and so has to be identical on both
    /// arms.
    path: String,
    text: Vec<u8>,
}

/// A named group of cases, drawn from one source.
struct Population {
    name: &'static str,
    cases: Vec<Case>,
}

/// The `ootest/ooRexx/base` suites this sweep draws programs from, sorted.
const OOTEST_SUITES: &[&str] = &["bif", "expressions", "keyword"];

/// The name of the population that is not an `ootest` suite.
const CORPUS_POPULATION: &str = "corpus";

/// The subset files the corpus population reads, in union order.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
    "phase-5b.txt",
    "phase-5c.txt",
    "phase-5d.txt",
    "phase-5j.txt",
    "phase-7.txt",
];

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn ootest_dir(suite: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../ootest/ooRexx/base")
        .join(suite)
}

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

/// Every corpus program named by a phase subset file, each once.
fn corpus_cases() -> Vec<Case> {
    let dir = corpus_dir();
    let mut seen = std::collections::HashSet::new();
    let mut cases = Vec::new();
    for name in SUBSET_FILES {
        let list_path = dir.join(name);
        let text = fs::read_to_string(&list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || !seen.insert(line.to_string()) {
                continue;
            }
            let path = dir.join(line);
            // Canonicalised for the same reason `corpus.rs`'s runner passes
            // an absolute path: a raised condition's report names the
            // program, so a relative path would reach stderr. Both arms get
            // the identical string either way, so this is about the
            // comparison being run on realistic bytes rather than about the
            // two agreeing.
            let path = fs::canonicalize(&path)
                .unwrap_or_else(|e| panic!("cannot canonicalise {}: {e}", path.display()));
            let text =
                fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            cases.push(Case {
                name: format!("corpus {line}"),
                path: path.to_string_lossy().into_owned(),
                text,
            });
        }
    }
    cases
}

/// The `NUMERIC FORM` keyword for a row's [`Form`].
fn form_keyword(form: Form) -> &'static str {
    match form {
        Form::Scientific => "SCIENTIFIC",
        Form::Engineering => "ENGINEERING",
    }
}

/// A program that establishes one row's state and then `SAY`s each clause.
fn row_program(digits: u32, form: Form, prelude: &[String], clauses: &[&str]) -> Vec<u8> {
    let mut text = String::new();
    text.push_str(&format!("numeric digits {digits}\n"));
    text.push_str(&format!("numeric form {}\n", form_keyword(form)));
    for line in prelude {
        text.push_str(line);
        text.push('\n');
    }
    for clause in clauses {
        text.push_str("say ");
        text.push_str(clause);
        text.push('\n');
    }
    text.into_bytes()
}

/// Every `.testGroup` under `ootest/ooRexx/base/<suite>`, read once.
fn suite_sources(suite: &str) -> Vec<(String, String)> {
    let dir = ootest_dir(suite);
    let mut groups = find_test_groups(&dir);
    groups.sort();
    assert!(
        !groups.is_empty(),
        "no .testGroup files under {} -- the ootest checkout is missing base/{suite}",
        dir.display()
    );
    groups
        .iter()
        .map(|path| {
            let bytes =
                fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            let group = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("group")
                .to_string();
            (group, String::from_utf8_lossy(&bytes).into_owned())
        })
        .collect()
}

fn case_of(row: &AssertionRow, index: usize) -> Case {
    // Both texts in one program, where `assertions.rs` says the expected
    // value only for a value row. The two arms are compared against each
    // other rather than against the row's own claim, so the extra clause
    // costs nothing and exercises a second expression per case.
    let clauses: Vec<&str> = if row.expect_raise.is_some() {
        vec![row.expr.as_str()]
    } else {
        vec![row.expr.as_str(), row.expected.as_str()]
    };
    Case {
        name: format!("{}::{}#{index}", row.group, row.method),
        path: INLINE_PATH.to_string(),
        text: row_program(row.digits, row.form, &row.prelude, &clauses),
    }
}

/// Every case, one population per entry of [`OOTEST_SUITES`] plus the
/// corpus.
fn populations() -> Vec<Population> {
    let mut out = vec![Population {
        name: CORPUS_POPULATION,
        cases: corpus_cases(),
    }];
    for suite in OOTEST_SUITES {
        let cases = match *suite {
            "expressions" => expression_cases(suite),
            "bif" => bif_cases(suite),
            "keyword" => keyword_cases(suite),
            other => panic!(
                "ootest suite base/{other} is declared in OOTEST_SUITES and nothing here turns \
                 its .testGroup files into programs"
            ),
        };
        out.push(Population { name: suite, cases });
    }
    out
}

fn expression_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        for (index, row) in extract_assertions(&group, &source).rows.iter().enumerate() {
            cases.push(case_of(row, index));
        }
    }
    cases
}

fn bif_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        let extraction = extract_bif(&group, &source);
        for (index, row) in extraction.rows.iter().enumerate() {
            cases.push(case_of(row, index));
        }
        for (index, row) in extraction.raises.iter().enumerate() {
            let operands: Vec<&str> = row.operands.iter().map(String::as_str).collect();
            cases.push(Case {
                name: format!("{}::{} raise #{index}", row.group, row.method),
                path: INLINE_PATH.to_string(),
                text: row_program(row.digits, row.form, &row.prelude, &operands),
            });
        }
    }
    cases
}

fn keyword_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        for body in extract_keyword(&group, &source).bodies {
            cases.push(Case {
                name: format!("{}::{}", body.group, body.method),
                path: INLINE_PATH.to_string(),
                text: body.program.clone().into_bytes(),
            });
        }
    }
    cases
}

/// Runs one case and describes what went wrong with it, if anything.
fn compare(case: &Case) -> Option<String> {
    // A directory of the case's own: a corpus program that writes files would
    // otherwise write them beside this crate's sources.
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ir-recorded-run")
        .join(case.name.replace(['/', ':', ' '], "__"));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap_or_else(|e| panic!("cannot empty {}: {e}", dir.display()));
    }
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    let outcome = watchdog::run_bounded(
        &case.path,
        case.text.clone(),
        Invocation::none().with_directory(dir),
    );
    if watchdog::did_not_finish(&outcome) {
        return Some(
            String::from_utf8_lossy(&outcome.stderr)
                .trim_end()
                .to_string(),
        );
    }
    // **The assertion this sweep exists for now.** A body the compiler refuses
    // used to run on the tree-walker, which made a refusal invisible: both
    // arms agreed because both tree-walked. There is no fallback now, so a
    // refusal is a failure -- and this population is the widest evidence in
    // the tree that none happens.
    if outcome.chunks_refused != 0 {
        return Some(format!(
            "the compiler refused a body {} times",
            outcome.chunks_refused
        ));
    }
    None
}

/// This harness reads **every** phase subset file the corpus has.
#[test]
fn the_harness_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the sweep does not read every phase subset file in \
         rust/corpus/"
    );
}

/// Every `ootest` suite a sibling harness in `tests/` runs programs from,
/// read out of those files rather than listed here a second time.
fn ootest_suites_sibling_harnesses_read() -> Vec<String> {
    const MARKER: &str = "ootest/ooRexx/base/";
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut suites = std::collections::BTreeSet::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs")
            || path.file_name().is_some_and(|n| n == "ir_recorded.rs")
        {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        for (offset, _) in text.match_indices(MARKER) {
            let tail = &text[offset + MARKER.len()..];
            let end = tail
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(tail.len());
            if end > 0 {
                suites.insert(tail[..end].to_string());
            }
        }
    }
    suites.into_iter().collect()
}

/// The sweep runs every `ootest` suite some other harness in this crate runs.
#[test]
fn the_sweep_runs_every_ootest_suite_a_sibling_harness_runs() {
    let on_disk = ootest_suites_sibling_harnesses_read();
    assert!(
        !on_disk.is_empty(),
        "no sibling harness in tests/ names an ootest suite root, so this pin \
         found nothing to compare against and would accept any sweep at all"
    );
    assert_eq!(
        OOTEST_SUITES, on_disk,
        "this sweep and this crate's other harnesses do not run the \
         same ootest suites. A suite only they run is one the compiled engine is \
         never compared on"
    );
}

/// Every population the tree calls for is built, and every one of them found
/// programs.
#[test]
fn every_population_the_tree_calls_for_is_present_and_non_empty() {
    let mut required = vec![CORPUS_POPULATION.to_string()];
    assert!(
        !phase_subset_files_on_disk().is_empty(),
        "rust/corpus/ has no phase subset file, so nothing here requires the \
         corpus population to exist"
    );
    required.extend(ootest_suites_sibling_harnesses_read());
    required.sort();

    let populations = populations();
    let mut names: Vec<String> = populations.iter().map(|p| p.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names, required,
        "a population the tree calls for is missing"
    );

    for population in &populations {
        assert!(
            !population.cases.is_empty(),
            "the {} population is empty, so the sweep asserts nothing about it",
            population.name
        );
    }
}

/// The sweep: every case of every population, run once, to completion, with
/// no body refused.
#[test]
fn every_population_runs_without_a_refusal() {
    let populations = populations();
    // Flattened first so work stealing spans populations rather than stalling
    // on the longest one's tail, then folded in the original order.
    let pairs: Vec<_> = populations
        .iter()
        .flat_map(|population| population.cases.iter().map(move |case| (population, case)))
        .collect();
    let reasons: Vec<Option<String>> = pairs.par_iter().map(|(_, case)| compare(case)).collect();

    let mut divergences = Vec::new();
    let mut compared = 0usize;
    for ((population, case), reason) in pairs.iter().zip(reasons) {
        compared += 1;
        if let Some(reason) = reason {
            divergences.push(format!("[{}] {}: {reason}", population.name, case.name));
        }
    }
    assert!(
        compared > 0,
        "the sweep compared nothing at all, which is a harness defect and not \
         an empty pass"
    );
    assert!(
        divergences.is_empty(),
        "{} of {compared} programs did not run cleanly:\n{}",
        divergences.len(),
        divergences.join("\n")
    );
}
