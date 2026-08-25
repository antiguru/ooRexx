# Task 5, re-review of fix round 3

Base `989fb26e1`, head `fe0364c08`. Diff read once from
`.superpowers/sdd/2026-08-17-phase-5a/review-989fb26e1..fe0364c08.diff` (19 KB, 3 files).
Findings under verification: A (impossibility claim), B ("like the rest"), C (history sentence),
D (three stale enumerations), E (third zero-count arm, `Concept::oracle_lines`).

## Verdicts

**A -- ADDRESSED.** `gate_table_d.rs:218-248` (module doc) and `:284-289` (`refusal_answered`'s
doc) no longer claim a line printed before the refusal is impossible. Both now carry the
construction (row's own directive preceded by a `::REQUIRES` of a helper whose prologue prints)
and its seven per-row results, ending "the bound is on `stderr` because a report there is the
answer **every** one of these rows gives, not because no other answer could exist for any of
them." `README.md:55-63` carries the same correction.

**B -- ADDRESSED.** The false "every probe here already opens with `say 'main'`"
(`gate_table_d.rs`) and "like the rest" (`README.md:61`) are gone. Replaced by a real, load-bearing
guard: `check_refusing_probe_says` (`gate_table_d.rs:303-329`), called unconditionally for every
`ORACLE_REFUSES` row at `:564-566`, before any verdict logic. Confirmed structural and able to
fail (see Guards, below).

**C -- ADDRESSED.** The history sentence ("Measured, replacing such a row's probe with a program
reading `nop` did exactly that and took the 5a open count down by one") is struck from
`refusal_answered`'s doc (`gate_table_d.rs:294-296` in base). Deciding test applied to the two
preceding sentences: both say the same thing about the code as it is with no historical framing,
so they stand unchanged in substance (one dropped "reintroduced" for "through", not a
content change).

**D -- ADDRESSED.** All three enumerations updated to the round's own change: `gate_table_c.rs`
module doc's wiring-row bullet (`:92-96`), its `OracleShape` paragraph (`:119-123`), and
`README.md`'s `classes/` bullet (`:30-33`) all now name the `.environment`-entry/class questions
before the class-surface questions. Verified against code: `class_probe_shape(name, entry)`
(`gate_table_c.rs:627-632`) reads the `entry` column as claimed, and `check_entry_present` is
paired with the `OracleShape` bound for every class row at `gate_table_c.rs:1578-1585` ("Two
checks, and neither subsumes the other") -- a claim about pre-existing code, not part of this
diff, and true.

**E -- ADDRESSED.** `check_concept_line_counts` (`gate_table_c.rs:645-670`) makes a committed
`oracle_lines: 0` structural, called unconditionally at `:1513`, before `check_entry_kinds` and
before any verdict is computed. Confirmed structural and able to fail (see Guards, below).

## New prose: what I checked, and how

Ran the house method (collapse each contiguous `///`/`//!`/`//` block, and each markdown
paragraph, to one line per file for base and head; diffed the collapsed files; read every new
line -- no keyword filter) over all three changed files. Every new sentence is one of: the A/B/C/D/E
correction described above, a doc comment for the two new functions, or the one-line call-site
insertion. No new sentence introduces a claim outside what the findings called for.

Checkable claims verified directly, beyond the diff:
- `check_entry_present` (referenced by the new C-file module-doc sentence) exists and is not part
  of this diff (`gate_table_c.rs:701`, base and head identical there).
- `class_probe_shape` reads `entry` as the new doc claims (`gate_table_c.rs:627-632`).
- The seven `ORACLE_REFUSES` probes each open with a `say` before their first `::` line (read all
  seven files under `rust/corpus/gate-tables/directives/`); `check_refusing_probe_says`'s
  `take_while` + `starts_with("say ")` logic matches their actual text.
- ASCII-only: no non-ASCII bytes anywhere in the diff (`grep -P '[^\x00-\x7F]'`, no match).
- No set-cardinality violations in the added comments: the only bare number words added are "two"
  referring to "two interpreters"/"two sides" (a fixed structural constant of the harness, not a
  set size); rc values and the "244 bytes" figure are measurements, which the constraint exempts.
- No historical-framing words ("used to", "no longer", "previously") in any added line; the two
  "what was measured" occurrences describe an evidentiary construction, which is what finding A's
  ruling explicitly asked for ("name the construction and its result, the way a measurement is
  written"), not narration about the code's own history.
- No `unsafe` added.

No new false statement found in the rewritten prose.

## Guards: real, unconditional, and able to fail

Built the workspace in an isolated copy outside the repository (`rust/` copied fresh, `target/`
never carried over, `interpreter/` and `oodocs/` symlinked to the real ones so `CARGO_MANIFEST_DIR`
resolves against the copy) to test both new checks by mutation without touching this checkout.
`cargo build --release -p rexx-exec --tests` succeeded from clean in ~67s.

- **`check_refusing_probe_says`**: stripped the `say 'main'` line from
  `attribute__external__subkeyword.rex` in the copy. `cargo test --release -p rexx-exec --test
  gate_table_d` (no `REXX_CORPUS_GATE`, plain report mode) -- **exit 101**, panic at
  `gate_tables/mod.rs:324` (`assert_no_structural_failures`), message: "`ORACLE_REFUSES` names
  ::ATTRIBUTE EXTERNAL ... This one has no `say` clause there" -- matching the report's control
  verbatim. Fires with no gate env set at all, so it is structural, not routed through a verdict
  channel a mode could relax.
- **`check_concept_line_counts`**: set the `typcla` concept's `oracle_lines: 4` to `0` in the copy.
  `cargo test --release -p rexx-exec --test gate_table_c` -- **exit 101**, same
  `assert_no_structural_failures` panic site, co-firing with `check_oracle_shape`'s own answer
  ("the oracle answered 4 line(s) where this row's probe asks for exactly 0"), matching the
  report's control verbatim.
- Both mutations reverted in the copy immediately after; `git status --short` on the real
  checkout confirmed clean before and after every step in this section.
- Also re-ran, unmutated, at HEAD in the same copy: `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1`
  against gate table D -- exit 101, exactly the 10 named rows (2 of which,
  `attribute__external__subkeyword.rex` and `method__external__subkeyword.rex`, are
  `ORACLE_REFUSES` rows); against gate table C -- exit 101, 135 rows. Both match the report's
  106/106, 10, and 135 counts.
- Combined control: turned `attribute__external__subkeyword.rex` (an `ORACLE_REFUSES` row that is
  also one of table D's 10 non-`agree` 5a rows) into `nop` and re-ran the 5a gate -- exit 101, both
  instruments fire together (`check_refusing_probe_says`'s message and the `stderr`-bound
  `refusal_missing` message), matching the report's fifth control.

## Per-row table spot checks (priority 3)

Ran the exact oracle invocation from a fresh empty directory, absolute paths, three descriptors
read separately (`ulimit -v 1048576; LD_LIBRARY_PATH=.../lib timeout -s KILL 10 .../rexx FILE`),
against the report's construction (`say 'main'` / `::requires 'helper.rex'` (prints
`helper-ran`) / the row's own directive):

| row | reported | measured |
|---|---|---|
| `::REQUIRES NAMESPACE` | rc 213, stdout `helper-ran` | rc 213, stdout `helper-ran` -- match |
| `::CLASS CLASS` | rc 231, stdout empty | rc 231, stdout empty -- match |
| `::REQUIRES LIBRARY` | rc 158, stdout empty | rc 158, stdout empty -- match |
| `::ATTRIBUTE EXTERNAL` | rc 166, stdout empty | rc 166, stdout empty -- match |

Four rows spot-checked (more than the two the task required), including the `::REQUIRES
NAMESPACE` positive case and three negatives spanning both the translate-time (`CLASS CLASS`) and
install-time (`REQUIRES LIBRARY`, `ATTRIBUTE EXTERNAL`) reasons the module doc distinguishes. All
four match exactly.

## Does the new `stderr`-bound reason overclaim? (priority 4)

No. The new text is careful to scope the falsified claim to what was actually falsified: it says
installing a `::REQUIRES` runs the required program (true, and the only mechanism among the seven
that runs another program's clause during install), not that any install-time failure runs code.
My own measurements of `::REQUIRES LIBRARY` and `::ATTRIBUTE EXTERNAL` -- both install-time
failures, neither a `::REQUIRES` of a Rexx program -- confirm `stdout` stayed empty for both, so
the narrower claim holds where the old, struck claim did not. The text explicitly declines to
build a per-row bound and defers the open question (`::REQUIRES NAMESPACE`'s own `stdout` bound)
to a future task, which is exactly what the ruling asked for.

## `cargo fmt` / `cargo clippy`

Ran both myself, unpiped, on the real checkout:
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0 (cache hit, "Finished" with no
  warnings).

## Out-of-scope observations (do not block)

- `gate_table_d.rs`'s `ORACLE_REFUSES` doc comment (the bullet list just above the `const`, e.g.
  "...neither is present on this build, so the failure is at install time, before the program's
  own first clause") is untouched by this diff. Read literally it remains true for the actual
  committed single-line probes (their own first clause, `say 'main'`, never runs -- oracle
  `stdout` is empty for all seven, unchanged by this round), so it is not a residual instance of
  finding A's falsified claim; it is a narrower statement about "this program's own clause" that
  round 3 did not need to touch. Flagging only because it sits beside code this diff does touch.

## Round verdict

**ACCEPT.** All five findings addressed. The two new guards are real: unconditional (fire under
plain `cargo test`, no gate env needed, panicking through
`assert_no_structural_failures`/`check_oracle_shape` rather than any verdict channel a mode could
relax) and demonstrably able to fail, both confirmed by mutation in an isolated copy with the real
checkout left untouched throughout. The per-row table checks out on every row I re-measured. No
new false statement, no set-cardinality or historical-framing violation, no `unsafe`, ASCII holds.
`cargo fmt` and `cargo clippy` both exit 0. Nothing behavioral is open.
