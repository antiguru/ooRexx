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

//! DEVIATION 0 (`docs/superpowers/plans/phase-4-exclusions.txt`): the one
//! normalisation the differential harnesses are allowed to apply to
//! `stderr`, shared by `tests/corpus.rs` and `tests/trace_oracle.rs` rather
//! than kept as two copies of the same function.
//!
//! **Why one copy, not two.** `rexx-exec/src/trace.rs`'s own module doc
//! records what "one quantity, two formatters" cost this project already:
//! `error.rs`'s `report` used to hold a second copy of `push_clause`'s four
//! lines, and the two drifted until a clamp had to be added to both by
//! hand. This function is compared against on two call sites and must give
//! the identical answer on both, so it is written once here. `tests/` files
//! that are not direct children of this directory are not auto-discovered
//! by Cargo as their own integration-test binaries, so `mod support;` in
//! each of the two consuming files pulls this in without adding a third
//! test target.
//!
//! **Scope, matching DEVIATION 0's own wording exactly.** Collapse the run
//! of ASCII space bytes between a trace line's 3-byte prefix marker and its
//! content down to one canonical space -- nothing else. A line only
//! qualifies at all if its own bytes 7..10 (`PREFIX_OFFSET`/`PREFIX_LENGTH`,
//! `rexx-exec/src/trace.rs`'s own constants, re-derived from
//! `RexxActivation.cpp:3567`-`3611`) are one of the nineteen markers
//! `trace_prefix_table` lists. That is deliberately generous -- covering the
//! ten prefixes this crate can emit today (D17's own reachable-from-4a
//! list) plus the nine it cannot reach yet -- so a later phase that adds an
//! emitter for one of the other nine gets the same normalisation from the
//! day it lands, rather than silently comparing byte-exact (and looking
//! "done" for the wrong reason) until someone remembers to extend this
//! list.
//!
//! Everything else is untouched: the line-number field, the marker itself,
//! the fixed `" => "`/`" <= "` tag markers, a quoted value's own bytes
//! (including any embedded spaces), and -- because normalisation runs per
//! line and never merges, drops or reorders one -- the presence, absence
//! and order of every line in `stderr`. A line with no known marker at
//! that offset (a `SAY`, or an `error.rs::report` banner line such as
//! `Error 42 running ... line 8:  ...`) is returned exactly as given.

/// `RexxActivation.cpp:3567`-`3587`'s `trace_prefix_table`, all nineteen --
/// not only the ten this crate can emit yet. See the module doc for why
/// the extra nine are included even though nothing here has ever produced
/// one: the alternative is a list that has to be remembered and extended
/// each time a later phase adds an emitter, which is exactly the kind of
/// silent-drift risk this project keeps finding in its own harnesses.
const TRACE_PREFIXES: &[[u8; 3]] = &[
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
/// `stderr` that has a known marker at [`PREFIX_OFFSET`]. See the module
/// doc for the full scope statement.
///
/// Lines are split on `\n` and rejoined the same way, one at a time, so
/// this can only reshape the inside of a line that already qualifies; it
/// cannot merge two lines, drop one, or change their order.
pub fn normalize_stderr(bytes: &[u8]) -> Vec<u8> {
    let had_trailing_newline = bytes.last() == Some(&b'\n');
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if had_trailing_newline {
        // A trailing `\n` makes `split` yield one final empty slice; drop
        // it so the rejoin below does not invent a second trailing `\n`.
        lines.pop();
    }

    let mut out = Vec::with_capacity(bytes.len());
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push(b'\n');
        }
        out.extend_from_slice(&normalize_line(line));
    }
    if had_trailing_newline {
        out.push(b'\n');
    }
    out
}

/// One line of [`normalize_stderr`]. Returns `line` untouched unless bytes
/// `PREFIX_OFFSET..PREFIX_OFFSET + PREFIX_LENGTH` are one of
/// [`TRACE_PREFIXES`], in which case the run of spaces immediately after
/// the marker -- and *only* that first run, stopping at the first
/// non-space byte -- is collapsed to exactly one space.
fn normalize_line(line: &[u8]) -> Vec<u8> {
    let marker_end = PREFIX_OFFSET + PREFIX_LENGTH;
    if line.len() < marker_end {
        return line.to_vec();
    }
    let marker = &line[PREFIX_OFFSET..marker_end];
    if !TRACE_PREFIXES
        .iter()
        .any(|known| known.as_slice() == marker)
    {
        return line.to_vec();
    }

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
}
