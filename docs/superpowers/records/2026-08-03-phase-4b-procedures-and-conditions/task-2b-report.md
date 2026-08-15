# Task 2b report: normalise leading indentation in the differential harnesses

## Baseline, confirmed before touching anything

Working tree clean at `4ea98cc0`. `cargo test --workspace` (from `rust/`):
`879 passed; 0 failed; 4 ignored`. Corpus report: `33 of 33 matching`.
Assertion table: `4224 of 4259 rows passing`. All three match the figures
the team lead's brief stated. Proceeded.

## What was built

* `rust/crates/rexx-exec/tests/support/mod.rs` (new file, not listed in the
  brief's own `Files:` section but added anyway): `pub fn normalize_stderr`,
  the one normalising function, plus its own `#[cfg(test)] mod tests`
  (positive cases + the three required negative controls, see below).
  Reasoning for a shared file rather than two copies: `rexx-exec/src/
  trace.rs`'s own module doc records that this project already paid once
  for "one quantity, two formatters" drifting apart (`error.rs::report`
  held a second copy of `push_clause`'s four lines until Task 2 needed to
  clamp one of them and found the other hadn't been). `corpus.rs` and
  `trace_oracle.rs` are two independent integration-test crates that
  cannot `mod` each other's file directly, so `tests/support/mod.rs` — a
  subdirectory, which Cargo does not auto-discover as its own test target —
  is the standard way to share code between them without either duplicating
  the function or adding a third test binary.
* `rust/crates/rexx-exec/tests/corpus.rs`: `mod support;`, `check_case`'s
  stderr comparison now runs both sides through `support::normalize_stderr`
  before comparing (previously `rust.stderr != cpp.stderr`). Module doc
  gained a "DEVIATION 0" section stating the scope and naming the pinned
  witnesses (below). The one now-inaccurate summary line ("compared byte
  for byte on stdout, stderr and exit code") was corrected rather than left
  to go stale next to the new section that contradicts it.
* `rust/crates/rexx-exec/tests/trace_oracle.rs`: `mod support;`,
  `check_witness`'s stderr `assert_eq!` now compares
  `support::normalize_stderr` of both sides. Module doc gained a paragraph
  stating the comparison is normalised but regeneration (the documented
  `ulimit`/`rexx`/redirect recipe) is not, so a committed `.expected` file
  stays real, un-normalised oracle output. `check_witness`'s own doc
  comment, which asserted "byte for byte on all three" unconditionally, was
  corrected to say stdout/exit code are byte-exact and stderr is byte-exact
  up to the deviation — it had gone false the moment the assertion below it
  changed, and a false comment gets fixed, not hedged.
* `docs/superpowers/plans/phase-4-exclusions.txt`, DEVIATION 0's row: the
  `NOT YET IMPLEMENTED` banner and its warning paragraph are gone, replaced
  with an `IMPLEMENTED by 4b Task 2b` paragraph naming where the code lives
  and stating that no figure moved. The "WHAT SURVIVES" paragraph now names
  the three pinned witnesses and explicitly excludes the weak one, instead
  of describing them only in the abstract.

## The normalisation itself

Every trace line — regardless of shape — has its own 3-byte prefix marker
(`*-*`, `>>>`, `>V>`, ...) starting at byte offset 7
(`rexx-exec/src/trace.rs`'s own `PREFIX_OFFSET`/`PREFIX_LENGTH` constants,
re-derived from `RexxActivation.cpp:3567`-`3611`; verified this holds for
*both* `push_clause`'s `{line:>6} *-* ` shape and `push_prefixed_blanks`'s
7-blanks-then-prefix shape by reading `trace.rs` itself, not inferred).
`normalize_line` checks bytes `7..10` against all nineteen markers in
`trace_prefix_table` (not only the ten this crate can emit today, so a
later phase's emitter is covered from day one). If it matches, the run of
space bytes immediately following the marker — and only that first run,
stopping at the first non-space byte — is collapsed to exactly one space;
everything before byte 7 (the line number field) and everything from the
first non-space byte onward (clause text, a quoted value and its own
embedded spaces, the `" => "`/`" <= "` tag markers) is untouched. A line
with no known marker at that offset — ordinary output, or an
`error.rs::report` banner line such as `Error 42 running ... line 8:  ...`
— is returned exactly as given; verified this against the actual banner
format in `error.rs::report` (`"Error {} running {} line {}:  {}\n"`),
which never puts a 3-byte marker at offset 7. `normalize_stderr` splits on
`\n`, normalises each line independently, and rejoins — it cannot merge,
drop, or reorder a line, only reshape what is inside one that already
qualifies.

## The three negative controls (Step 1)

All in `tests/support/mod.rs`'s own test module, run from a shared
`base_transcript()` (a two-line clause+value transcript with *already*
differing, oracle-defect-shaped indent widths on the two lines, so a
control that still fails after normalisation is not merely showing
normalisation is a no-op):

```
$ cargo test -p rexx-exec --test corpus support::tests
running 6 tests
test support::tests::a_changed_value_still_differs_after_normalisation ... ok
test support::tests::a_missing_line_still_differs_after_normalisation ... ok
test support::tests::a_non_trace_line_is_untouched_including_its_own_spaces ... ok
test support::tests::two_clause_lines_differing_only_in_indent_width_normalise_equal ... ok
test support::tests::a_reordered_line_still_differs_after_normalisation ... ok
test support::tests::two_value_lines_differing_only_in_indent_width_normalise_equal ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

$ cargo test -p rexx-exec --test trace_oracle support::tests
running 6 tests
test support::tests::a_missing_line_still_differs_after_normalisation ... ok
test support::tests::a_reordered_line_still_differs_after_normalisation ... ok
test support::tests::two_value_lines_differing_only_in_indent_width_normalise_equal ... ok
test support::tests::a_non_trace_line_is_untouched_including_its_own_spaces ... ok
test support::tests::two_clause_lines_differing_only_in_indent_width_normalise_equal ... ok
test support::tests::a_changed_value_still_differs_after_normalisation ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

(All six run under both `corpus.rs` and `trace_oracle.rs`, since both do
`mod support;` on the same shared file — the 879 → 891 rise in the
workspace-wide passed count below is exactly these six, counted twice.)

* **Missing line** (`a_missing_line_still_differs_after_normalisation`):
  the base transcript's own value line dropped entirely, everything else
  (including the surviving line's indent width) different too. Asserts
  `normalize_stderr(base) != normalize_stderr(missing_line)`. This is a
  content difference (a line's presence), and normalisation — which never
  merges or invents a line — cannot make the two equal.
* **Reordered line** (`a_reordered_line_still_differs_after_normalisation`):
  the same two lines, swapped. Line order is required to stay byte-exact;
  normalisation runs per line at a fixed position and has no reordering
  step, so the two remain unequal.
* **Changed value** (`a_changed_value_still_differs_after_normalisation`):
  the base transcript's `"2"` changed to `"3"`, both lines also given a
  different (but still normalisable) indent width. The value bytes sit
  after the first non-space byte the collapsing stops at, so they are
  never touched, and the two stay unequal.

Two positive controls sit alongside these to show the deviation buys
something at all: `two_clause_lines_differing_only_in_indent_width_
normalise_equal` and `two_value_lines_differing_only_in_indent_width_
normalise_equal`, each pinning `assert_ne!` on the *raw* bytes first (so
the test would fail loudly if the two literals were accidentally made
identical) before asserting `normalize_stderr` closes the gap. A third
control, `a_non_trace_line_is_untouched_including_its_own_spaces`, checks
both an `error.rs`-shaped banner line and an arbitrary padded output line
(`"a    b\n"`) pass through completely unchanged — this is what shows the
normalisation cannot reach outside a recognised trace line at all.

## Pinned no-normalisation witnesses (Step 2)

Per the brief: these already existed as `rexx-exec/src/run.rs` unit tests
before this task, and normalisation cannot reach a unit test's own
`assert_eq!` regardless of what `corpus.rs`/`trace_oracle.rs` do, so no new
test was written — they are named and pinned by comment instead, in both
`corpus.rs`'s module doc and DEVIATION 0's own row:

* `one_two_and_three_enclosing_dos_indent_by_two_four_and_six`
* `the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes`
* `an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_
  residual_indent`

(Corrected in fix round 1, I2: an earlier version of this bullet called
the first of these three "precisely DEVIATION 0's own oracle-counter
shape". It is not -- see the fix-round section at the end of this report
for why, and for where else that claim was corrected.)

**Not counted**: `the_indent_after_a_loop_has_already_exited_is_not_left_
over_from_it`. It runs at top level, where the oracle's own counter is
already clamped at 0, so the correct and an incorrectly-unwound model agree
there regardless — it looks like a witness for the "loop leaves the
counter one notch low" gap and is not one. Both `corpus.rs`'s module doc
and the exclusions row now say this explicitly, not only that the test
exists.

## Did any previously-failing shape start passing?

**No.** The corpus was already `33 of 33 matching` at baseline (byte-exact,
before this task), so there was nothing left for a stderr normalisation to
newly fix — every corpus program already agreed on stderr byte-for-byte,
indentation included. After this change the corpus report is still
`33 of 33 matching`, run identically. This is a fact worth stating plainly
rather than a gap: the deviation is now in force and demonstrably safe
(the negative controls), but it has not yet bought a passing program,
because nothing was failing on indentation alone in this corpus to begin
with. The known indent-shaped divergence this project has measured (the
re-tested `Controlled`-loop pass's missing `>>>` lines) is explicitly a
*content* gap, not an indent one, and DEVIATION 0's own row already says
normalisation does not touch it.

## Verification

```
$ cargo fmt --all --check   # from rust/
(exit 0, no diff)

$ cargo clippy --workspace --all-targets -- -D warnings   # from rust/
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
(exit 0, no warnings)

$ cargo test --workspace   # from rust/
... 891 passed; 0 failed; 4 ignored ...
rexx-exec assertion-table report: 4224 of 4259 rows passing -- REPORT MODE, NOT THE GATE
rexx-exec differential corpus report: 33 of 33 matching -- REPORT MODE, NOT THE GATE
```

879 → 891 is exactly the six new `support::tests` running once in the
`corpus` binary and once in the `trace_oracle` binary (both `mod support;`
the same file); 4 ignored is unchanged; the corpus and assertion-table
figures are byte-identical to baseline.

## Files touched

* `rust/crates/rexx-exec/tests/support/mod.rs` (new)
* `rust/crates/rexx-exec/tests/corpus.rs`
* `rust/crates/rexx-exec/tests/trace_oracle.rs`
* `docs/superpowers/plans/phase-4-exclusions.txt`

## Fix round 1 (review findings I1, I2)

**I1 -- the normalisation reached past trace lines into the tail of a
quoted value that itself contains a raw newline.** `trace.rs`'s
`push_quoted`/`push_quoted_tag` wrap a traced value in `"..."` with no
escaping at all. If the value itself contains a `0x0A` byte, splitting
`stderr` on `\n` (the whole mechanism `normalize_stderr` is built on) cuts
one logical trace record into two physical lines, and the second one
starts wherever the value's own bytes happened to leave off -- which can,
by coincidence, place a known 3-byte marker at `PREFIX_OFFSET` and get
treated as a fresh trace line. The old, unconditional `normalize_line`
call would then collapse that "line"'s own leading run of spaces, but
those spaces were never indentation -- they were the traced value's own
literal content.

Reproduced against the real oracle (fresh, empty scratch directory,
`ulimit -v 1048576`, stdout/stderr captured separately), exactly the
reviewer's own repro:

```
$ cat repro.rex
trace i
x = '0a'x || "       >>>   z"

$ ( ulimit -v 1048576; LD_LIBRARY_PATH=.../ooRexx/build/lib \
    .../ooRexx/build/bin/rexx repro.rex ) 1>out.txt 2>err.txt
$ xxd err.txt   # (full bytes in the fix commit's own test literal)
```

Ten physical lines came back; three of them (the tails of the `>O>`,
`>>>` and `>=>` records) are exactly the shape described: `       >>>
z"`, `PREFIX_OFFSET..+3` reading `>>>` purely by chance, the rest being
the concatenated value's own trailing text and closing quote.

**Fix**: one bit of state carried across lines in `normalize_stderr`.
Only a *recognised* trace line (`is_trace_line`, factored out of
`normalize_line` so both share one check) whose own line ends with an odd
count of `"` bytes has opened a value quote its own physical line did not
close; every line read while that bit is set is a raw continuation --
copied through untouched, and not itself eligible to open or close
anything -- until a continuation line's own odd `"` count closes it. This
mirrors `push_quoted`/`push_quoted_tag`'s own open-then-close pairing
exactly for the failure case that matters (embedded newline, no embedded
quote character); a value that embeds a literal `"` instead can leave the
parity looking odd for an unrelated reason, but the only consequence is a
following genuine trace line being conservatively left un-normalised --
stricter, never looser, so it cannot make two different transcripts
compare equal, which is the property this module has to hold.

Added as negative control 4,
`a_traced_values_own_embedded_newline_does_not_let_its_continuation_absorb_a_content_change`,
using the real measured bytes above (not hand-written) against a mutant
with one space dropped from the `>O>` record's own continuation text --
the shape a real concatenation bug would produce. **Verified this control
would have caught the bug it names**: reverted to the pre-fix
`normalize_line`-only function (old commit's `tests/support/mod.rs` plus
this one new test appended), ran just that test, and it failed --
`normalize_stderr(correct)` and `normalize_stderr(mutated)` came back
byte-identical, both losing the same three-space run down to one space
regardless of which was fed in. Restored the fix; the same test passes.

**I2 -- `one_two_and_three_enclosing_dos_indent_by_two_four_and_six` does
not witness the counter defect.** Removed the "(precisely DEVIATION 0's
own oracle-counter shape)" parenthetical from `corpus.rs`'s module doc,
the exclusions row, and this report's own Step 2 section above. All three
of that test's cases raise on a *first* loop iteration -- no pass
completes, the loop never ends -- so `BaseDoInstruction.cpp`'s two
divergent exit paths are never reached by it at all; it is a correct
pinned witness for lexical nesting depth and nothing more. Both `corpus.rs`
and the exclusions row now say plainly that no pinned witness demonstrates
the counter defect itself, rather than implying one does.

**Verification after both fixes**, from `rust/`:

```
$ cargo fmt --all --check
(exit 0, no diff)

$ cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s
(exit 0, no warnings)

$ cargo test --workspace
... 893 passed; 0 failed; 4 ignored ...
rexx-exec assertion-table report: 4224 of 4259 rows passing -- REPORT MODE, NOT THE GATE
rexx-exec differential corpus report: 33 of 33 matching -- REPORT MODE, NOT THE GATE
```

891 → 893 is the one new negative control, run once in each of the
`corpus` and `trace_oracle` binaries. Corpus (33 of 33) and the assertion
table (4224 of 4259) are unchanged from both the original baseline and
round 1's own numbers -- no figure moved.

Commit: `344677e6`.
