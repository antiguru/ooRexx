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

//! `Raised`: the payload of a real Rexx condition.
//!
//! **Only the payload exists here.** Task 12 ("Errors, the message
//! catalogue, and the exit code") owns the message catalogue, the
//! oracle's exact two-line stderr format, the clause echo, and the
//! `256 - number` exit-code mapping -- none of that is built in this
//! task. What exists is enough to assert *which* condition was raised
//! (the condition name, the number and sub-number, and the substitution
//! values), not to reproduce what the oracle prints for it. A test
//! against this type checks "did `1/0` raise 42.3", never "does stderr
//! read `Error 42.3: ...` and does the process exit 214".
//!
//! `Failure` is the other half: `step` and everything above it can fail
//! either because 4a does not implement a construct (`Loud`, `lib.rs`) or
//! because a real condition was raised (`Raised`), and a clause containing
//! an expression can do either, so the propagation type has to carry both.

use crate::Loud;
use rexx_num::ArithError;

/// A real Rexx condition raised during evaluation.
#[derive(Clone, Debug)]
pub(crate) struct Raised {
    /// The condition name a trapped Rexx program would see from
    /// `condition('c')`. Every raiser this task produces is `SYNTAX`, and
    /// it is carried as a field rather than hardcoded at each call site
    /// because the spec's own shape includes it and later tasks (4b's
    /// `NOVALUE`, `NOMETHOD`, ...) need to set it to something else.
    ///
    /// **Nothing reads it yet**, and until `execute` was wired to
    /// `Raised::report` that was hidden: the placeholder stderr line printed
    /// this field, so it had a reader that existed only because the real
    /// rendering did not. The oracle's report names the *number*, not the
    /// condition, so the first genuine reader is 4b's `SIGNAL ON` and
    /// `condition('c')`.
    ///
    /// `expect` rather than `allow` on purpose: it is itself a warning once
    /// the lint stops firing, so the day 4b reads this field the annotation
    /// asks to be deleted instead of quietly outliving its reason.
    #[expect(
        dead_code,
        reason = "no reader until 4b's SIGNAL ON and condition('c'); expect self-expires"
    )]
    pub(crate) condition: &'static str,
    pub(crate) number: u16,
    pub(crate) sub: u16,
    pub(crate) additional: Vec<String>,
}

impl Raised {
    fn syntax(number: u16, sub: u16, additional: Vec<String>) -> Raised {
        Raised {
            condition: "SYNTAX",
            number,
            sub,
            additional,
        }
    }

    /// 41.1: a nonnumeric value used in arithmetic. `value` is the
    /// operand's own text, verbatim -- measured, `say 'abc' + 1` reports
    /// `Nonnumeric value ("abc")`, the operand as it renders, not upcased
    /// or otherwise transformed.
    pub(crate) fn nonnumeric(value: &[u8]) -> Raised {
        Raised::syntax(41, 1, vec![String::from_utf8_lossy(value).into_owned()])
    }

    /// 26.8: `**`'s right operand is not a whole number, **including not
    /// being a number at all**. Measured: `2 ** 'x'` and `2 ** 2.5` both
    /// give 26.8 ("found \"x\""/"found \"2.5\""), while the identical
    /// failure on the *left* operand is the ordinary 41.1 (`'y' ** 2` is
    /// 41.1, `'y' ** 'x'` is still 41.1 -- the base is checked first).
    /// This is deliberately not routed through `nonnumeric`: the oracle's
    /// own asymmetry between the two operands is the fact being
    /// reproduced, not an implementation shortcut. `found` is the
    /// exponent's own text; used when the exponent does not even parse as
    /// a number, so there is no `Number` for `rexx-num`'s own
    /// `ArithError::PowerExponentNotWhole` to carry.
    pub(crate) fn power_exponent_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(26, 8, vec![String::from_utf8_lossy(found).into_owned()])
    }

    /// 34.901: the prefix `\` operator's operand is not a logical value.
    /// A logical value is *exactly* the one-byte string `0` or `1`, no
    /// coercion -- this is a text check, never a numeric one, which is
    /// why the caller passes the operand's own rendered text rather than
    /// anything from `to_number`. Measured: `say \'abc'` gives 34.901,
    /// `Logical value must be exactly "0" or "1"; found "abc"`.
    pub(crate) fn not_logical(found: &[u8]) -> Raised {
        Raised::syntax(34, 901, vec![String::from_utf8_lossy(found).into_owned()])
    }

    /// 11.1: "Insufficient control stack space" -- D19's evaluation-depth
    /// limit (`eval.rs`'s own `MAX_EVAL_DEPTH`). No substitution: measured
    /// against the oracle's own parse-side 11.1 (nested parens/calls,
    /// `phase-4-exclusions.txt`'s Deviation 2), the catalogue's `(11, 1)`
    /// entry carries none either.
    pub(crate) fn insufficient_stack() -> Raised {
        Raised::syntax(11, 1, Vec::new())
    }

    /// 34.6: one element of a comma-separated logical list
    /// (`ExprKind::Logical`, `if a, b then` and friends) is not a logical
    /// value. A distinct sub-number from `not_logical`'s 34.901, and
    /// deliberately not shared with it even though the underlying check is
    /// identical (exactly `0` or `1`, text not numeric) -- measured, `if 1,
    /// 'x' then` gives 34.6 ("Value of logical list expression element
    /// must be exactly \"0\" or \"1\"; found \"x\""), a different message
    /// from `&`'s 34.901 for the identical bad value. `IF`/`WHEN`/`WHILE`/
    /// `UNTIL`'s own 34.1/34.2/34.3/34.4 are for when the *whole* condition
    /// is a single expression, not a list, and are Tasks 9-11's to raise
    /// when they exist; this crate has no instruction context yet to
    /// prefer one of those over 34.6, so 34.6 is `ExprKind::Logical`'s own
    /// answer regardless of which keyword built the list.
    pub(crate) fn logical_list_element(found: &[u8]) -> Raised {
        Raised::syntax(34, 6, vec![String::from_utf8_lossy(found).into_owned()])
    }
}

/// Converts a `rexx-num` arithmetic failure into a `Raised`.
///
/// The `(major, sub)` pair comes from `ArithError::sub_code`, made `pub` in
/// `rexx-num` for exactly this caller (`4a320f1c`) rather than hand-copied
/// here: this task originally flagged that `sub_code` was private and
/// shipped a two-variant stopgap covering only what its own tests had
/// independently verified against the oracle (`DivideByZero`,
/// `PowerExponentNotWhole`), sub `0` elsewhere. The accessor landing
/// retires that stopgap -- every `ArithError` variant now gets its real
/// sub-number, not only the two this task happened to exercise.
impl From<ArithError> for Raised {
    fn from(error: ArithError) -> Raised {
        // `additional()` and `sub_code()` both borrow, so either can run
        // first; ordered to match the doc comment's own telling.
        let additional = error.additional();
        let (number, sub) = error.sub_code();
        Raised::syntax(number, sub, additional)
    }
}

/// Either kind of failure a clause can produce: a construct 4a does not
/// implement (`Loud`) or a real Rexx condition (`Raised`). `step` and
/// everything above it propagate this rather than either alone, since a
/// clause containing an expression can fail either way -- `eval`'s own
/// `ExprKind::Call` arm is `Loud` (not implemented), its `1 / 0` arm is
/// `Raised` (implemented, and this is what it does).
#[derive(Debug)]
pub(crate) enum Failure {
    Loud(Loud),
    Raised(Raised),
}

impl From<Loud> for Failure {
    fn from(loud: Loud) -> Failure {
        Failure::Loud(loud)
    }
}

impl From<Raised> for Failure {
    fn from(raised: Raised) -> Failure {
        Failure::Raised(raised)
    }
}

/// Where a failing clause was found -- `Interp::failure_site`'s own type
/// (`lib.rs`), and what `run.rs`'s `record_failure_site` fills in.
///
/// A named struct rather than a `(usize, Vec<u8>, usize)` tuple **on
/// purpose**: `line` and `indent` are both bare `usize`s, and a position-only
/// tuple lets the two transpose with nothing to catch it -- the failure mode
/// would be plausible-looking, wrong stderr, not a compile error or a panic.
/// Naming the fields removes that whole class rather than trusting call-site
/// order.
pub(crate) struct FailureSite {
    pub(crate) line: usize,
    pub(crate) text: Vec<u8>,
    /// Spaces to prefix `text` with on the echo line, Task 11's own
    /// nesting-depth quantity. **Computed statically from the AST** (`run.rs`'s
    /// `static_indent`), never carried on a running counter: Task 10's own
    /// report concluded the depth is derivable from the instruction list
    /// alone with no runtime block stack, and this task's own oracle
    /// measurements confirm it for the ordinary case and for one
    /// LEAVE/ITERATE error family (28.5) besides -- see `static_indent`'s
    /// own doc comment and the report for the transcripts. A mutable
    /// per-`Interp` counter was the first design tried here and was
    /// abandoned once it became clear it would need perfect symmetric
    /// bookkeeping on every exit path out of every construct, including the
    /// error paths and the `run_bounded` `Goto`-absorption case `Flow`'s own
    /// doc comment warns about -- exactly the class of defect this crate's
    /// skipped-`pop_frame` discussion elsewhere already flags. A pure
    /// function of `(instructions, index)` cannot desync, because there is
    /// nothing stateful to desync.
    pub(crate) indent: usize,
}

/// Where the failing clause is, which is everything the report needs from
/// outside this module.
///
/// Passed in rather than reached for: `error.rs` owns the *format*, and the
/// instruction loop owns knowing which clause failed. That split is why this
/// module needs no access to `Interp`, the program or the source. Built from
/// a `FailureSite` plus the one thing it does not carry, the program's own
/// path -- `execute` (`lib.rs`) is the one place both are in hand together.
pub(crate) struct ClauseSite<'a> {
    /// The program's path **as the oracle prints it**, absolute. Measured:
    /// the major line carries the full path, and `rexx-oracle`'s `normalize`
    /// masks the cwd, so an absolute path is comparable across machines.
    pub(crate) path: &'a str,
    /// The clause's 1-based source line.
    pub(crate) line: usize,
    /// The clause's own bytes, exactly as `Instruction::clause_span` covers
    /// them. Not trimmed: measured, `if 'x' then nop` echoes `if 'x' ` with
    /// the trailing space, because an `IF`'s span stops at the start of the
    /// token that ended its condition.
    pub(crate) text: &'a [u8],
    /// Spaces to prefix `text` with on the echo line -- `FailureSite`'s own
    /// `indent`, forwarded unchanged. Zero for every clause this crate
    /// reported before Task 11, so every pre-existing call site keeps its
    /// old behaviour by passing `0`.
    pub(crate) indent: usize,
}

impl Raised {
    /// `256 - major`, the whole rule.
    ///
    /// Verified across nine majors rather than the four the plan recorded:
    /// 7 -> 249, 24 -> 232, 25 -> 231, 26 -> 230, 33 -> 223, 34 -> 222,
    /// 41 -> 215, 42 -> 214, 98 -> 158.
    ///
    /// This is also why `NOT_IMPLEMENTED_EXIT` must stay outside 157..=253:
    /// majors 3 to 99 fill that band, so a loud failure inside it would be
    /// indistinguishable from a raised condition and a program *expecting*
    /// that condition would pass against a gap.
    pub(crate) fn exit_code(&self) -> i32 {
        256 - i32::from(self.number)
    }

    /// The exact bytes the oracle writes to stderr for this condition.
    ///
    /// Three lines, and every part of the shape is measured rather than
    /// inferred (`say 1` then a `SELECT` with no true `WHEN`, `cat -A`):
    ///
    /// ```text
    ///      4 *-* end
    /// Error 7 running /abs/path/f.rex line 4:  WHEN or OTHERWISE expected.
    /// Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
    /// ```
    ///
    /// * The **clause echo appears with trace off**, which is the part that
    ///   surprises: this is not trace output and is not suppressed by
    ///   `TRACE OFF`.
    /// * The line number is **right-aligned in a six-character field**,
    ///   measured at one, two and three digits: `     4`, `    12`, `   105`.
    /// * **Two spaces after each colon**, on both error lines.
    /// * The major line's text is the catalogue's `(major, 0)` entry and the
    ///   sub line's is `(major, sub)`.
    ///
    /// `SAY` output goes to stdout and all of this to stderr, so their
    /// relative order is not observable (D17).
    pub(crate) fn report(&self, site: &ClauseSite<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(format!("{:>6} *-* ", site.line).as_bytes());
        // `Task 11`'s own addition: `site.indent` spaces of nesting depth
        // before the clause's own text, never before it -- measured, an
        // unindented top-level clause (`indent == 0`) is byte-identical to
        // every pre-Task-11 report this module already had a test for.
        out.extend(std::iter::repeat_n(b' ', site.indent));
        out.extend_from_slice(site.text);
        out.push(b'\n');
        out.extend_from_slice(
            format!(
                "Error {} running {} line {}:  {}\n",
                self.number,
                site.path,
                site.line,
                self.message(self.number, 0)
            )
            .as_bytes(),
        );
        out.extend_from_slice(
            format!(
                "Error {}.{}:  {}\n",
                self.number,
                self.sub,
                self.message(self.number, self.sub)
            )
            .as_bytes(),
        );
        out
    }

    /// One catalogue entry with this error's substitutions applied.
    ///
    /// The text comes from `rexx-inventory`'s generated table, derived from
    /// `interpreter/messages/rexxmsg.xml`, never hand-transcribed here: 704
    /// messages the tree already generates, and criterion 1 compares these
    /// bytes exactly.
    ///
    /// A miss renders visibly rather than panicking or rendering empty. The
    /// catalogue and the oracle come from one source, so a miss is a bug in
    /// this crate's numbering, and the error path is the worst possible place
    /// to abort: it would turn a reportable condition into a crash, which is
    /// the outcome the whole failing-loudly rule exists to prevent.
    fn message(&self, major: u16, sub: u16) -> String {
        match rexx_inventory::errors::lookup(major, sub) {
            Some(entry) => substitute(entry.text, &self.additional),
            None => format!("<no message {major}.{sub} in the catalogue>"),
        }
    }
}

/// Replaces `&1`, `&2`, ... with the raiser's substitution values.
///
/// The catalogue spells substitutions the way `rexxmsg.xml` does, so this is
/// the one piece of message rendering that is ours rather than generated.
/// Scans rather than chaining `replace`, so a substitution value that itself
/// contains `&2` cannot be re-substituted -- a real risk here, since these
/// values are arbitrary Rexx data (`say '&1' + 1` puts `&1` in the message).
///
/// An `&` not followed by a digit, and a digit with no matching value, are
/// both passed through unchanged rather than swallowed.
fn substitute(text: &str, values: &[String]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        match chars.peek().and_then(|d| d.to_digit(10)) {
            Some(index) if index >= 1 => {
                chars.next();
                match values.get(index as usize - 1) {
                    Some(value) => out.push_str(value),
                    None => {
                        out.push('&');
                        out.push_str(&index.to_string());
                    }
                }
            }
            _ => out.push('&'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 7.3 transcript, captured from `build/bin/rexx` with `cat -A` so the
    /// trailing bytes are the oracle's and not a guess.
    ///
    /// Program: `say 1` / `select` / `when 1=0 then nop` / `end`. Stdout gets
    /// `1`; all three lines below go to stderr; rc is 249.
    #[test]
    fn the_7_3_report_matches_the_oracle_byte_for_byte() {
        let raised = Raised::syntax(7, 3, vec![]);
        let site = ClauseSite {
            path: "/abs/path/f.rex",
            line: 4,
            text: b"end",
            indent: 0,
        };
        assert_eq!(
            String::from_utf8(raised.report(&site)).unwrap(),
            "     4 *-* end\n\
             Error 7 running /abs/path/f.rex line 4:  WHEN or OTHERWISE expected.\n\
             Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n"
        );
        assert_eq!(raised.exit_code(), 249);
    }

    /// A substituted message, and a clause echo that keeps its trailing space.
    ///
    /// Captured: `if 'x' then nop` on line 12 of a twelve-line program echoes
    /// `    12 *-* if 'x' ` -- the span stops at the start of `then`, so the
    /// space before it belongs to the clause. Trimming it would diverge on
    /// every `IF`.
    #[test]
    fn a_substituted_message_and_a_clause_echo_that_keeps_its_trailing_space() {
        let raised = Raised::not_logical(b"x");
        let site = ClauseSite {
            path: "/abs/w.rex",
            line: 12,
            text: b"if 'x' ",
            indent: 0,
        };
        let report = String::from_utf8(raised.report(&site)).unwrap();
        assert_eq!(
            report,
            "    12 *-* if 'x' \n\
             Error 34 running /abs/w.rex line 12:  Logical value not 0 or 1.\n\
             Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"x\".\n"
        );
        assert_eq!(raised.exit_code(), 222);
    }

    /// The line number is right-aligned in a six-character field, measured at
    /// one, two and three digits against the oracle: `     4`, `    12`,
    /// `   105`.
    #[test]
    fn the_line_number_field_is_six_wide() {
        for (line, expected) in [(4usize, "     4"), (12, "    12"), (105, "   105")] {
            let site = ClauseSite {
                path: "/p",
                line,
                text: b"nop",
                indent: 0,
            };
            let report = Raised::syntax(7, 3, vec![]).report(&site);
            let first = String::from_utf8(report).unwrap();
            let first = first.lines().next().unwrap().to_string();
            assert_eq!(&first[..6], expected, "line {line}");
        }
    }

    /// `256 - major`, over every major 4a is measured to raise.
    ///
    /// Nine, not the four the plan recorded, each confirmed by running the
    /// construct under the oracle and reading `$?`.
    #[test]
    fn the_exit_code_is_256_minus_the_major() {
        for (major, sub, rc) in [
            (7u16, 3u16, 249i32),
            (11, 1, 245),
            (24, 901, 232),
            (25, 11, 231),
            (26, 5, 230),
            (28, 3, 228),
            (33, 1, 223),
            (34, 1, 222),
            (41, 1, 215),
            (42, 3, 214),
            (98, 913, 158),
        ] {
            assert_eq!(
                Raised::syntax(major, sub, vec![]).exit_code(),
                rc,
                "{major}"
            );
        }
    }

    /// Every raiser family 4a is measured to produce has catalogue text for
    /// both its lines.
    ///
    /// This is the test that would have caught a hand-transcribed catalogue
    /// going stale, and it is why the text is looked up rather than written
    /// here: it asserts the entries *exist* and are non-empty, never what they
    /// say, so it cannot drift from `rexxmsg.xml` the way a copy would.
    #[test]
    fn every_measured_family_has_catalogue_text() {
        for (major, sub) in [
            (7u16, 3u16),
            (11, 1),
            (24, 1),
            (24, 901),
            (25, 11),
            (26, 2),
            (26, 3),
            (26, 5),
            (26, 6),
            (26, 8),
            (33, 1),
            (34, 1),
            (34, 2),
            (34, 3),
            (34, 4),
            (28, 1),
            (28, 2),
            (28, 3),
            (28, 4),
            (28, 5),
            (34, 6),
            (34, 901),
            (41, 1),
            (42, 3),
            (42, 901),
            (98, 913),
        ] {
            for (m, s) in [(major, 0), (major, sub)] {
                let entry = rexx_inventory::errors::lookup(m, s)
                    .unwrap_or_else(|| panic!("no catalogue entry for {m}.{s}"));
                assert!(!entry.text.is_empty(), "{m}.{s} has empty text");
            }
        }
    }

    /// A substitution value containing `&1` is not re-substituted.
    ///
    /// Reachable from a Rexx program: `say '&1' + 1` raises 41.1 with the
    /// operand text `&1`, so a `replace`-chaining implementation would expand
    /// the value into itself. Scanning once is what makes that impossible.
    #[test]
    fn a_substitution_value_containing_an_ampersand_digit_is_left_alone() {
        let raised = Raised::nonnumeric(b"&1");
        assert_eq!(
            raised.message(41, 1),
            "Nonnumeric value (\"&1\") used in arithmetic operation."
        );
    }

    /// An `&` that is not a substitution, and a missing value, both pass
    /// through rather than being swallowed.
    #[test]
    fn a_bare_ampersand_and_a_missing_value_pass_through() {
        assert_eq!(substitute("a & b", &[]), "a & b");
        assert_eq!(substitute("x &1 y", &[]), "x &1 y");
        assert_eq!(substitute("&1 and &2", &["one".into()]), "one and &2");
    }

    /// A catalogue miss renders visibly instead of panicking or rendering
    /// empty: the error path is the worst place to abort, since it would turn
    /// a reportable condition into a crash.
    #[test]
    fn a_catalogue_miss_is_visible_rather_than_silent() {
        let raised = Raised::syntax(999, 999, vec![]);
        assert_eq!(
            raised.message(999, 999),
            "<no message 999.999 in the catalogue>"
        );
    }

    /// Task 11's own addition: `site.indent` prefixes the clause echo with
    /// that many spaces, and nothing else on the report moves.
    ///
    /// Captured against the oracle: `do i = 1 to 3 / say 1/0 / end`
    /// reports `     2 *-*   say 1/0` -- two spaces for the one enclosing
    /// `DO`. Kills a mutation that applies the indent to the wrong line (the
    /// `Error 42 running ...` line, say), one that appends it after `text`
    /// instead of before, and one that never applies it at all (which the
    /// pre-existing `indent: 0` tests above would not catch, since they are
    /// silent about anything `indent` does when it is nonzero).
    #[test]
    fn the_indent_field_prefixes_the_clause_echo_with_that_many_spaces() {
        let raised = Raised::syntax(42, 3, vec![]);
        let site = ClauseSite {
            path: "/abs/do1.rex",
            line: 2,
            text: b"say 1/0",
            indent: 2,
        };
        let report = String::from_utf8(raised.report(&site)).unwrap();
        assert_eq!(
            report.lines().next().unwrap(),
            "     2 *-*   say 1/0",
            "two spaces before the clause text, none anywhere else on the line"
        );
    }

    // The new Task 11 raisers themselves -- 26.2/26.3/28.1-28.5/34.3/34.4 --
    // live in `run.rs` as local `fn raised_*` free functions, matching that
    // file's own established convention for every other instruction-
    // specific raiser (`raised_if_not_logical`, `raised_select_no_when`,
    // `raised_symbol_expected`, ...), not as `Raised::` methods here: this
    // module holds only the raisers `eval.rs` also needs (cross-module), and
    // `insufficient_stack` is the one member of this task's own set that
    // qualifies. Their wording is exercised end to end by `run.rs`'s own
    // tests (`run_source` against a real program, checking `raised.number`/
    // `.sub`/`.additional`), not spot-checked again here -- `Raised::message`
    // is private to this module and `every_measured_family_has_catalogue_text`
    // above already proves every one of their catalogue entries exists.
}
