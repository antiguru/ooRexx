# Task 10a report: `rexx-bench/src/bin/rexx-bench-suite.rs`

BASE `a9bf19aeb`. Commit `4d65ccf5a` ("Split rexx-bench-suite.rs's rendering
into report.rs").

## Status

Done. All required commands run and read: `cargo fmt --all --check`,
`cargo clippy -j 4 --workspace --all-targets -- -D warnings` (from a
`cargo clean -p rexx-bench`'d target directory), `cargo test -j 4 --release
-p rexx-bench --no-fail-fast`, `cargo build --release -p rexx-bench --bin
rexx-bench-suite`, and `cargo doc --no-deps -p rexx-bench` are all exit 0.
Tree clean at `4d65ccf5a`.

**Fix round 2 replaced instrument 3's own decoder and its output**, after
fix round 1's version turned out to be blind to most of its subject (a real
bug in the instrument, not in the move). "## Fix round 2" below has the
corrected methodology, control, and numbers; "## The four instruments"'s
own instrument-3 bullet still describes fix round 1's evidence, left as
written per this report's own established practice of appending
corrections rather than rewriting reviewed sections in place.

This report supersedes the working-copy report this task originally wrote
under `.superpowers/sdd/2026-09-15-file-split/task-10a-report.md` (untracked,
per this project's standing convention for that directory), which is left in
place but is not the record: this file, committed under
`docs/superpowers/records/`, is. The move itself was not touched by this
round -- `4d65ccf5a` is unamended -- only the evidence and its write-up.

## Commits

1. `4d65ccf5a` -- the split. Unchanged from the original report.
2. `353d5c601` ("Task 10a fix round 1: run instrument 3 for real, cite
   artifacts") -- adds the instruments' scripts and outputs under
   `task-10a-files/`, this report, no code changes.
3. This fix round's own commit (see "Fix round 2" below) -- replaces
   instrument 3's decoder (fix round 1's version was blind to literals
   inside macro arguments) and its output, adds a control and a whole-file
   cross-check, no code changes.

## Fix round 1: what was wrong and what changed

The review found the move itself sound: an independent re-run of instrument 3
(see below) reproduced 236/236 base literal-bearing tokens and found exactly
one new literal, the `#[path = "..."]` string this task's own report already
named and explained. What needed fixing was the evidence for instruments 1
and 3, and two sentences of prose:

1. **Instrument 3 was never actually run.** The original report's script
   (`extract_and_compare.py`, whitespace-stripped token-stream comparison)
   strips whitespace *inside* string and byte-string literals too -- so two
   literals that decode to different values but differ only in which
   whitespace character they contain (a space vs. a tab, or a real newline
   vs. an escaped `\n`) would compare equal under that script, and a
   backslash-newline continuation (the exact case the plan's own instrument
   3 wording names) is whitespace by construction and would be invisible to
   it either way. The original report's line "PASS, implied by (2)" is
   false: whitespace-stripped-token-stream-equal does not imply
   literal-decode-equal, because the stripping operation is not blind to
   literal *interiors*, it is blind to *everything*, interiors included.
   Fixed: `instrument3_lit_decoder/` (a two-file Rust crate using `syn` --
   Rust's own parser, already vendored in this workspace's offline registry
   cache, `syn = "2"` + `proc-macro2 = "1"`) parses each moved item's exact
   source text as a `syn::File` and walks every `syn::Lit` node with
   `syn::visit::Visit`, printing `KIND {decoded-value:?}` one per literal in
   source order -- `Lit::Str::value()`, `Lit::ByteStr::value()`,
   `Lit::Char::value()`, `Lit::Byte::value()` do the actual escape/
   backslash-newline-continuation decoding, not a hand-written decoder. This
   is Rust's own literal grammar, not a reimplementation of it. Verified
   against a throwaway three-literal probe first (a backslash-newline
   continuation, a `\u{41}` escape, and a `\xHH` byte escape) to confirm it
   decodes each correctly before trusting it on the real files.
2. **Instrument 1 was reviewed by eye (`git diff`, hunk by hunk), not run
   "as specified."** The plan's own words are "`cmp` of extracted ranges."
   Fixed: `instrument1_unmoved_ranges.py` uses `difflib.SequenceMatcher` to
   find every maximal matching block between BASE's and HEAD's
   `rexx-bench-suite.rs` (so the boundaries are discovered, not hand-picked
   from reading the diff), then both a direct Python-string equality check
   *and* an independent shell-out to the real `cmp` binary confirm each
   block byte-identical.
3. **No artifact was committed.** Every instrument's script and output now
   lives under `task-10a-files/` (paths below), matching the plan's bullet
   added at `a9bf19aeb`: "Each instrument names the artifact the task leaves
   behind, and the task commits or cites it."
4. **The commit message overcounts by one.** `4d65ccf5a`'s message says
   "Five tests stay in the root file rather than following their subjects."
   The correct count is **four**:
   `the_joint_coverage_bound_is_clamped`, `the_caveat_matches_the_committed_baseline`,
   `the_performance_line_parses`, `a_quoted_line_that_is_absent_says_so` --
   the four tests that call a rendering-side helper (`interval_for`,
   `joint_coverage_bound`, `ratio_interval_caveat`, `quoted_line`,
   `parse_cps`) and were kept in the root's single `mod tests` for exactly
   that reason. (All *ten* of the root's tests stay in the root file, since
   none of them moved -- "five" was neither the four with a rendering-side
   dependency nor the ten total, and does not correspond to any set this
   task actually drew.) Per this project's standing rule, commits are never
   amended; this paragraph is the correction, and it lives here rather than
   in `4d65ccf5a`'s own text.

## What moved, and where the boundary was re-derived

Unchanged from the original report. The proposal's two groups --
measurement (`main`, `measure_interleaved`, `Stats`, the median interval,
`fingerprint`, `loop_count`) and rendering (`write_provenance` through
`write_blocked`) -- come from the survey's line range at `5d84dd8cb`, not
from reading what each function actually touches. Reading it settles two
things the range gets wrong:

* **`Stats` and its whole interval family are rendering, not measurement.**
  `Stats::of`, `interval_for`, `interval_coverage`, `joint_coverage_bound`,
  `ratio_interval_caveat`, `seconds` and `counter_median` are called
  exclusively from `write_offset`, `write_counters`, `write_axes` and
  `write_rexxcps` -- never from `main` or `measure_interleaved`. Moved to
  `report.rs` along with `fingerprint` (called only from `write_provenance`).
* **`TARGET_COVERAGE`** is used only inside `interval_for` and
  `write_provenance`, both rendering; moved to `report.rs` rather than
  staying beside `PAIRS` and its siblings in the root.

`capture` stays in the root: `oracle_objects` (measurement, called directly
by `main`) needs it too, and it has no dependency on either side's own
types, so it stays where it already sat. `report.rs` reaches it as
`super::capture`.

## Row types: where they live, and why

`Paired` and `AxisRow` stay in the root file, with their producer:
`measure_interleaved` constructs `Paired`, `main` constructs `AxisRow`.
`report.rs`'s writers only ever *read* them. Rust's privacy rule makes this
free: a private item in a parent module is visible to every descendant
module, so `report.rs` sees `Paired`, `AxisRow`, `Role`, `AXES` and
`capture` with no visibility change to any of them -- `report.rs` just
`use super::{AXES, AxisRow, Paired, Role, capture};`. `Role`, `Axis` and
`AXES` stay in the root for the same reason: `main`'s own axis loops need
them directly, and `write_blocked` (the one rendering function that reads
them) is a descendant too and needs no wider visibility than that.

The reverse direction needs `pub(super)` on exactly what the root's own code
reaches: the seven `write_*` entry points `main` calls (qualified
`report::write_provenance(...)` etc., matching this crate's own
`dispatch.rs`/`hash.rs` convention), plus five more -- `interval_for`,
`joint_coverage_bound`, `ratio_interval_caveat`, `quoted_line`, `parse_cps`
-- that the four tests named above call directly. Everything else in
`report.rs` is private to it.

## Test placement: a departure from "tests follow their subject"

Tried first, and reverted: nesting a `#[cfg(test)] mod tests` inside
`report.rs` changes each moved test's fully-qualified name from
`tests::the_joint_coverage_bound_is_clamped` to
`report::tests::the_joint_coverage_bound_is_clamped` -- a literal violation
of the plan's fourth instrument, "the same test names." All ten tests stay
in the root's single `mod tests`, unmoved, unchanged; the four naming
rendering-side helpers reach them through one added import (`use
super::report::{interval_for, joint_coverage_bound, parse_cps, quoted_line,
ratio_interval_caveat};`), leaving every test body byte-identical to BASE.

## The `[[bin]]` submodule-directory discovery

A `[[bin]]` crate root's implicit submodule directory is the one it sits in
(`src/bin/`), not one named after its own file stem -- the same rule
`main.rs`/`lib.rs` follow, extended to every binary crate root. A bare
`mod report;` fails with `E0583` ("create file `src/bin/report.rs`"),
verified with a standalone two-file `cargo build` probe before touching the
real file. Fixed with an explicit `#[path = "rexx-bench-suite/report.rs"]
mod report;`, with a short comment recording why.

## Pinned-path check

`/bin/grep -rn 'rexx-bench-suite' rust/ --exclude-dir=target` found three
prose mentions in `bench-rexxcps/README.md` and `bench-control/README.md`
naming the tool generically (no correction needed -- the binary's own
top-level file path is unchanged), `rexx-bench/src/lib.rs:14`'s doc comment
naming `src/bin/rexx-bench-suite.rs` (still accurate), and the file's own
internal `"rexx-bench-suite: ..."` strings (unchanged, in the root file). No
hits in `unsafe_sites.rs`, `dispatch_seam.rs`, `environment_seam.rs`, or
`corpus/refusal-sites.tsv`.

## The four instruments, run and cited

1. **Unmoved lines byte-identical, `cmp` of extracted ranges.** PASS.
   Script: `task-10a-files/instrument1_unmoved_ranges.py`. Output:
   `task-10a-files/instrument1_output.txt`. Method: `difflib.SequenceMatcher`
   over BASE's and HEAD's whole `rexx-bench-suite.rs`, every matching block
   (13 of them, 795 lines total) checked twice -- Python string equality,
   then an independent shell-out to the actual `cmp` binary on each block
   written to its own temp file pair. **0 of 13 blocks failed either check.**
   The 13 blocks are exactly the regions git's own diff shows as untouched
   context; this instrument discovers them independently rather than reading
   them off a diff.
2. **Whitespace-stripped token streams of every moved item identical, per
   item.** PASS. Script: `task-10a-files/instrument2_token_streams.py`.
   Output: `task-10a-files/instrument2_output.txt`. Extracts each of the 20
   moved items (`TARGET_COVERAGE`, `Stats`+its `impl`, `interval_for`,
   `interval_coverage`, `joint_coverage_bound`, `ratio_interval_caveat`,
   `seconds`, `fingerprint`, `counter_median`, `self_timed_figures`,
   `median_of`, `quoted_line`, `parse_cps`, and the seven `write_*`
   functions) from BASE (`git show a9bf19aeb:...`) and from the committed
   `report.rs`, tolerant of an added `pub(super)`, and compares with all
   whitespace removed. **0 mismatches of 20.**
3. **Every string/char literal decodes to the same value.** PASS, and now
   actually checked, not inferred. Scripts:
   `task-10a-files/instrument3_extract_items.py` (writes each of the 20
   moved items' *raw*, unnormalised source text to `base/<name>.rs` and
   `head/<name>.rs`) and `task-10a-files/instrument3_lit_decoder/` (the
   `syn`-based decoder described above). Output:
   `task-10a-files/instrument3_output.txt`. Run per item: decode BASE's
   literals in source order, decode HEAD's, diff the two decoded lists.
   **0 mismatches of 20 items, 43 literals compared** (the items with no
   string/char/byte literal report 0 and trivially match; `TARGET_COVERAGE`'s
   one "literal" is its float value, `0.95`, included for completeness
   though numeric literals carry no escape syntax for this instrument to
   exercise). No parse errors on either side, for any item.
4. **Test results per test binary and per result block identical.** PASS.
   Outputs: `task-10a-files/instrument4_test_base.txt` (BASE, built and run
   in an isolated `git worktree add --detach <path> a9bf19aeb`, `rexx-run`
   built inside that same worktree's own `target/` first) and
   `task-10a-files/instrument4_test_head.txt` (HEAD, `4d65ccf5a`, this
   checkout). Both `cargo test --release -p rexx-bench --no-fail-fast`.
   BASE: `rexx-bench-suite` binary reports 10 tests, all `tests::*`, **10
   passed, 0 failed**; the other five binaries/blocks (lib, `rexx-arms`,
   `rexx-bench-band`, `rexx-time`, doc-tests) report 29/0/10/0/0. HEAD:
   identical breakdown, **10 passed, 0 failed** in `rexx-bench-suite` with
   the same ten names, and 29/0/10/0/0 unchanged elsewhere. Same names, same
   pass/fail per name, same count per binary, on both sides.

## `cargo doc --no-deps -p rexx-bench`

Outputs: `task-10a-files/cargo_doc_before.txt` (BASE, isolated worktree, 0
warnings, exit 0) and `task-10a-files/cargo_doc_after.txt` (HEAD, cold
target directory -- `cargo clean -p rexx-bench` then `rm -rf target/doc`
before running -- 0 warnings, exit 0). The one warning this task hit and
fixed mid-round (`interval_coverage`'s `` [`PAIRS`] `` intra-doc link,
unresolvable once `PAIRS` left `report.rs`'s scope, corrected to
`` [`crate::PAIRS`] ``) is already folded into `4d65ccf5a`; both captures
here are of the tree as committed, so neither shows it.

## Corpus-gated runs

Not run, per the brief and the plan's own Task 10 framing: this task moves
code in a benchmark tool, not interpreter code.

## Performance

Skipped by the brief's own ruling: no measured binary (`rexx-run`) is built
from this task's own files.

## Concerns

None found. The `#[path]` requirement is worth the controller flagging for
any later task in this plan that creates a new child module under a
`[[bin]]` crate root.

## Fix round 2: instrument 3's decoder was blind to most of its subject

The move itself is still sound -- nothing in this round changes the verdict
above. What was wrong is fix round 1's own instrument-3 tooling, caught by
a check that compared its output against the raw source rather than
trusting a clean `0 mismatches` on faith.

**The bug.** `` write_provenance ``, the largest single function this task
moved, reported **0 literals** under fix round 1's decoder, while its body
in `report.rs` contains 19 double-quote characters on inspection (46
counting both the extracted-item copy's own duplication of the `capture`
call sites -- the point is that it is obviously not zero). The decoder used
`syn::visit::Visit` over a parsed `syn::File`/`syn::Item`. Syn's typed AST
does not descend into a macro invocation's arguments: `Macro::tokens` is an
opaque `proc_macro2::TokenStream` that syn does not know how to parse
further (it does not know `writeln!`'s or `format!`'s grammar), so
`visit_lit` is never called for anything inside one. Every one of
`report.rs`'s writer functions is built almost entirely out of
`writeln!(report, "...")` calls, so this blind spot ate nearly the whole
instrument: fix round 1's reported total was **43 literals across the 20
moved items**; the true total, below, is **119**.

**The fix.** `task-10a-files/instrument3_lit_decoder/main.rs` no longer
parses a `syn::File` at all. It tokenizes the raw source with
`proc_macro2::TokenStream::from_str` and walks every `TokenTree`
recursively, decoding each `TokenTree::Literal` via `syn::Lit::new(...)`
and recursing into every `TokenTree::Group`. A macro's parenthesized
argument list is, at the raw-token level, an ordinary `Group` -- nothing
marks it as macro-internal until a macro-aware parser (which syn's default
AST walk is, and which this no longer needs) tries to interpret it -- so
this reaches every literal regardless of what syntactic position it sits
in, macro argument included.

**The control this fix adds, and its own result.** A second, independent
implementation -- `task-10a-files/instrument3_independent_lexer_count.py`,
a hand-written character-by-character scanner with no dependency on `syn`
or `proc_macro2` -- counts the same class of tokens (string, byte-string,
char, byte, including the one genuine subtlety worth naming: a `///`/`//!`
doc-comment line is lexically desugared into a `#[doc = "..."]` string
literal by rustc's own tokenizer, which `proc_macro2` mirrors and which the
Rust decoder therefore correctly counts, so the independent lexer counts
doc-comment lines too, deliberately, rather than trying to make the
numbers agree by narrowing scope). `task-10a-files/instrument3_compare.py`
runs both counters over every item, both sides, and fails loudly on any
disagreement: **the two agree exactly on every one of the 20 items, both
base and head (0 control mismatches)**. This control would have caught the
original bug immediately -- on `write_provenance` alone it would have
reported the decoder's 0 against the lexer's 23 -- and was itself validated
against exactly that case before being trusted (re-run by hand against
`write_provenance` first, matching 23/23, and the `Stats` item's one
`///` doc-comment token, missed by the lexer's first draft and fixed
before this round's numbers were taken, recorded in the script's own
comment rather than quietly dropped).

**A second, whole-file cross-check, run for extra assurance rather than
because the per-item one needed it.**
`task-10a-files/instrument3_whole_file_check.py` decodes BASE's entire
`rexx-bench-suite.rs` and HEAD's `rexx-bench-suite.rs` +
`rexx-bench-suite/report.rs` combined (not just the 20 moved items -- the
whole file on each side) and multiset-diffs the two literal lists,
order-independent since the split legitimately reorders content across two
files. Output: `task-10a-files/instrument3_whole_file_output.txt`. BASE:
383 literals. HEAD: 386. The diff is exactly four literals, and every one
of the four is a change this report already names as deliberate: the one
doc-comment correction (`` [`PAIRS`] `` -> `` [`crate::PAIRS`] ``, one
string removed and its corrected replacement added) and the two new
literals `4d65ccf5a` itself introduced (`"rexx-bench-suite/report.rs"`,
the `#[path]` string, and `report.rs`'s own two-line module doc comment,
each `//!` line its own literal, so two more). Nothing else in either
whole file differs. This is not a required instrument by the plan's own
wording (which asks for moved items, not a whole-file diff), but is
strictly stronger evidence than the per-item comparison alone and is kept
as a corroborating artifact.

**Corrected instrument 3 result, replacing fix round 1's `43`.** PASS.
`task-10a-files/instrument3_output.txt` (script:
`task-10a-files/instrument3_compare.py`, consuming
`task-10a-files/instrument3_extract_items.py`'s raw per-item extraction,
unchanged from fix round 1): **0 decode mismatches of 20 items, 119
decodable literals (STR/BYTESTR/CHAR/BYTE) compared, 0 control
mismatches.** Corroborated by the whole-file check above.
