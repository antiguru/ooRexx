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

//! This file stays `mod.rs`, not `support.rs`, because `tests/` is Cargo's
//! integration-test root: a bare `tests/support.rs` would be auto-discovered
//! as its own test binary in addition to being pulled in as a module by the
//! files that declare `mod support;`, compiling these helpers standalone
//! for no reason. Renaming it to `support.rs` would add that extra binary
//! back.

/// The arity probe both `collection_arity.rs` and `introspection_arity.rs`
/// drive: one machine, two tables, parameterised by the four files each
/// reads and writes. Its own module doc carries the verdicts and the rule
/// that a list is only real if the oracle completes the send.
pub mod arity;

/// `Setup.cpp`'s per-scope method tables, joined onto the oracle's own scope
/// answer by both `collection_scopes.rs` and `introspection_scopes.rs`.
pub mod setup_cpp;

/// Running a program through the C++ oracle and comparing the three
/// observable channels. Its own module doc carries the memory limit, the
/// missing-binary rule and why an invocation count is kept there. Shared for
/// the same reason [`normalize_stderr`] is: every differential harness has
/// to invoke the oracle identically or their results are not comparable.
pub mod oracle;

/// `RexxActivation.cpp:3567`-`3587`'s `trace_prefix_table`, all nineteen --
/// not only the subset this crate has an emitter for. See the module doc
/// for why the markers nothing here has ever produced are carried anyway:
/// the alternative is a list that has to be remembered and extended each
/// time a later phase adds an emitter, which is exactly the kind of
/// silent-drift risk this project keeps finding in its own harnesses.
/// `tests/trace_oracle.rs`'s
/// `the_trace_surfaces_coverage_is_thirteen_of_nineteen_with_owners_for_the_rest`
/// asserts its own `PREFIX_COVERAGE` equals this list, so the two cannot
/// drift and neither can be shortened quietly.
pub const TRACE_PREFIXES: &[[u8; 3]] = &[
    *b"*-*", // TRACE_PREFIX_CLAUSE
    *b"+++", // TRACE_PREFIX_ERROR
    *b">>>", // TRACE_PREFIX_RESULT
    *b">.>", // TRACE_PREFIX_DUMMY
    *b">V>", // TRACE_PREFIX_VARIABLE
    *b">E>", // TRACE_PREFIX_DOTVARIABLE
    *b">L>", // TRACE_PREFIX_LITERAL
    *b">F>", // TRACE_PREFIX_FUNCTION
    *b">P>", // TRACE_PREFIX_PREFIX
    *b">O>", // TRACE_PREFIX_OPERATOR
    *b">C>", // TRACE_PREFIX_COMPOUND
    *b">M>", // TRACE_PREFIX_MESSAGE
    *b">A>", // TRACE_PREFIX_ARGUMENT
    *b">=>", // TRACE_PREFIX_ASSIGNMENT
    *b">I>", // TRACE_PREFIX_INVOCATION
    *b">N>", // TRACE_PREFIX_NAMESPACE
    *b">K>", // TRACE_PREFIX_KEYWORD
    *b">R>", // TRACE_PREFIX_ALIAS
    *b"<I<", // TRACE_PREFIX_INVOCATION_EXIT
];

/// Byte offset of the 3-byte prefix marker, identical on every trace line
/// regardless of shape: `rexx-exec/src/trace.rs`'s `push_clause` puts
/// `*-*` at 7 (`{line:>6} ` is 7 bytes), and its `push_prefixed_blanks`
/// puts a value line's own prefix at 7 too (7 blanks standing in for the
/// unused line-number field). Re-derived here rather than imported: this
/// crate's constant is `pub(crate)`, invisible to an integration test,
/// which compiles as a separate crate linking against the library only
/// through its public surface.
const PREFIX_OFFSET: usize = 7;
const PREFIX_LENGTH: usize = 3;

/// Collapses the run of ASCII space bytes between a trace line's prefix
/// marker and its content down to one canonical space, in every line of
/// `stderr` that has a known marker at [`PREFIX_OFFSET`] and is not
/// itself the raw continuation of a value opened on an earlier physical
/// line. See the module doc for the full scope statement, including the
/// "a traced value can itself contain a raw newline" paragraph this
/// continuation tracking exists for.
pub fn normalize_stderr(bytes: &[u8]) -> Vec<u8> {
    let had_trailing_newline = bytes.last() == Some(&b'\n');
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if had_trailing_newline {
        // A trailing `\n` makes `split` yield one final empty slice; drop
        // it so the rejoin below does not invent a second trailing `\n`.
        lines.pop();
    }

    let mut out = Vec::with_capacity(bytes.len());
    // True while a genuine trace line opened a `"` (`push_quoted`/
    // `push_quoted_tag`) that its own physical line did not close --
    // every line read in that state is a raw continuation of that
    // value's own bytes, not a fresh record, so it is copied through
    // untouched and cannot itself open or close anything. See the module
    // doc for why only a *recognised* trace line, never an arbitrary
    // line, is allowed to arm this.
    let mut inside_open_quote = false;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push(b'\n');
        }
        if inside_open_quote {
            out.extend_from_slice(line);
            if has_odd_quote_count(line) {
                inside_open_quote = false;
            }
        } else {
            out.extend_from_slice(&normalize_line(line));
            if is_trace_line(line) && has_odd_quote_count(line) {
                inside_open_quote = true;
            }
        }
    }
    if had_trailing_newline {
        out.push(b'\n');
    }
    out
}

/// Whether `line` has a recognised prefix marker at
/// `PREFIX_OFFSET..PREFIX_OFFSET + PREFIX_LENGTH` -- shared between
/// [`normalize_line`] (what to do with such a line) and
/// [`normalize_stderr`] (whether such a line's own quote parity may arm
/// the continuation tracking there).
fn is_trace_line(line: &[u8]) -> bool {
    let marker_end = PREFIX_OFFSET + PREFIX_LENGTH;
    line.len() >= marker_end
        && TRACE_PREFIXES
            .iter()
            .any(|known| known.as_slice() == &line[PREFIX_OFFSET..marker_end])
}

/// Whether `line` contains an odd number of `"` bytes. `push_quoted`/
/// `push_quoted_tag` (`trace.rs`) always emit `"` in a pair -- open, then
/// close -- so a physical line carrying only one of the pair is exactly
/// the signal that the value it belongs to continues onto the next
/// physical line.
fn has_odd_quote_count(line: &[u8]) -> bool {
    line.iter().filter(|&&b| b == b'"').count() % 2 == 1
}

/// One line of [`normalize_stderr`], for a line already known not to be a
/// continuation. Returns `line` untouched unless [`is_trace_line`], in
/// which case the run of spaces immediately after the marker -- and
/// *only* that first run, stopping at the first non-space byte -- is
/// collapsed to exactly one space.
fn normalize_line(line: &[u8]) -> Vec<u8> {
    if !is_trace_line(line) {
        return line.to_vec();
    }
    let marker_end = PREFIX_OFFSET + PREFIX_LENGTH;

    let after_marker = &line[marker_end..];
    let space_run = after_marker.iter().take_while(|&&b| b == b' ').count();
    let content = &after_marker[space_run..];

    let mut out = Vec::with_capacity(line.len());
    out.extend_from_slice(&line[..marker_end]);
    // Every formatter in `trace.rs` guarantees at least one space here
    // (the fixed template byte on a clause line, or the fixed `3 +
    // indent` run on a value line), so `space_run` is never really 0 in
    // practice; guarded anyway so a malformed or truncated line never
    // grows a space that was not there.
    if space_run > 0 {
        out.push(b' ');
    }
    out.extend_from_slice(content);
    out
}

#[cfg(test)]
mod tests {
    use super::normalize_stderr;

    /// The shape DEVIATION 0 exists for: the same clause, at the same
    /// line number, with two different indent widths (0 and 2 -- exactly
    /// the oracle-counter-defect delta the exclusions file measures). This
    /// is the positive case: normalisation must make these equal, or the
    /// whole deviation buys nothing.
    #[test]
    fn two_clause_lines_differing_only_in_indent_width_normalise_equal() {
        let indent_0 = b"     4 *-* say 1/0\n";
        let indent_2 = b"     4 *-*   say 1/0\n";
        assert_ne!(
            indent_0.as_slice(),
            indent_2.as_slice(),
            "sanity: byte-exact still differs"
        );
        assert_eq!(normalize_stderr(indent_0), normalize_stderr(indent_2));
    }

    /// The same shape for a value line's own indent-driven gap
    /// (`push_prefixed_blanks`'s `3 + indent`), independent of the clause
    /// case above: indent 0 (3 spaces after the marker) versus indent 2
    /// (5 spaces), same prefix, same quoted content.
    #[test]
    fn two_value_lines_differing_only_in_indent_width_normalise_equal() {
        let indent_0 = b"       >>>   \"2\"\n";
        let indent_2 = b"       >>>     \"2\"\n";
        assert_ne!(
            indent_0.as_slice(),
            indent_2.as_slice(),
            "sanity: byte-exact still differs"
        );
        assert_eq!(normalize_stderr(indent_0), normalize_stderr(indent_2));
    }

    /// A line with no known marker at [`PREFIX_OFFSET`] -- ordinary
    /// program output, or an `error.rs::report` banner line -- is
    /// returned byte-for-byte, runs of spaces and all: this is what
    /// proves the normalisation cannot reach outside trace lines.
    #[test]
    fn a_non_trace_line_is_untouched_including_its_own_spaces() {
        let banner = b"Error 42 running /tmp/x.rex line 8:  Interpretation error\n";
        assert_eq!(normalize_stderr(banner), banner.to_vec());

        let padded_say = b"a    b\n"; // arbitrary program output, not a trace line
        assert_eq!(normalize_stderr(padded_say), padded_say.to_vec());
    }

    /// Base two-line transcript every negative control below mutates:
    /// a clause echo (indent 1, simulating one flavour of the counter
    /// defect) followed by its own `>>>` value line (indent 0) -- two
    /// different indent widths in the same transcript, so a control that
    /// still differs after normalisation is not merely showing that
    /// indent normalisation is a no-op here.
    fn base_transcript() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"     2 *-*  x = 1 + 1\n");
        out.extend_from_slice(b"       >>>   \"2\"\n");
        out
    }

    /// Negative control 1: a missing line. Deleting the value line
    /// entirely is a **content** divergence (a line's presence), not an
    /// indent one, and normalisation must not paper over it.
    #[test]
    fn a_missing_line_still_differs_after_normalisation() {
        let base = base_transcript();
        let mut missing_line = Vec::new();
        missing_line.extend_from_slice(b"     2 *-*    x = 1 + 1\n"); // wider indent
        // (no second line at all)
        assert_ne!(
            normalize_stderr(&base),
            normalize_stderr(&missing_line),
            "a dropped line must not normalise away"
        );
    }

    /// Negative control 2: the same two lines, reordered. Line **order**
    /// is required to stay byte-exact, and normalisation operates per
    /// line, so swapping them must still be caught.
    #[test]
    fn a_reordered_line_still_differs_after_normalisation() {
        let base = base_transcript();
        let mut reordered = Vec::new();
        reordered.extend_from_slice(b"       >>>     \"2\"\n"); // wider indent
        reordered.extend_from_slice(b"     2 *-* x = 1 + 1\n");
        assert_ne!(
            normalize_stderr(&base),
            normalize_stderr(&reordered),
            "a reordered pair of lines must not normalise away"
        );
    }

    /// Negative control 3: the value itself changed (`"2"` to `"3"`).
    /// Value-line **content** is exactly what DEVIATION 0's own "why"
    /// paragraph carves out as never touched, since nothing else in the
    /// output exposes an intermediate result or evaluation order.
    #[test]
    fn a_changed_value_still_differs_after_normalisation() {
        let base = base_transcript();
        let mut changed_value = Vec::new();
        changed_value.extend_from_slice(b"     2 *-*    x = 1 + 1\n"); // wider indent
        changed_value.extend_from_slice(b"       >>>     \"3\"\n"); // wider indent, changed value
        assert_ne!(
            normalize_stderr(&base),
            normalize_stderr(&changed_value),
            "a changed value must not normalise away"
        );
    }

    /// Negative control 4 -- the review-round-1 finding (I1). Real oracle
    /// stderr, captured (not hand-written) from `trace i` /
    /// `x = '0a'x || "       >>>   z"` (`'0a'x` is one raw byte, 0x0A):
    /// the value's own embedded newline splits its `>O>`/`>>>`/`>=>`
    /// records' quoted content across two physical lines each, and the
    /// tail of each -- `       >>>   z"` -- lines up with `PREFIX_OFFSET`
    /// exactly like a fresh `>>>` record purely by chance.
    #[test]
    fn a_traced_values_own_embedded_newline_does_not_let_its_continuation_absorb_a_content_change()
    {
        let correct: &[u8] = b"     2 *-* x = '0a'x || \"       >>>   z\"\n       >L>   \"\n\"\n       >L>   \"       >>>   z\"\n       >O>   \"||\" => \"\n       >>>   z\"\n       >>>   \"\n       >>>   z\"\n       >=>   X <= \"\n       >>>   z\"\n";
        let mutated: &[u8] = b"     2 *-* x = '0a'x || \"       >>>   z\"\n       >L>   \"\n\"\n       >L>   \"       >>>   z\"\n       >O>   \"||\" => \"\n       >>>  z\"\n       >>>   \"\n       >>>   z\"\n       >=>   X <= \"\n       >>>   z\"\n";
        assert_ne!(correct, mutated, "sanity: byte-exact still differs");
        assert_ne!(
            normalize_stderr(correct),
            normalize_stderr(mutated),
            "a content change inside a value's own embedded-newline \
             continuation must not normalise away"
        );
    }
}
