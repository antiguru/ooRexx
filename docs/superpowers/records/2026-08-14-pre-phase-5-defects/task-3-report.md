# Task 3 report: the `>K>` line a `DO OVER ... FOR` does not print

**Status:** fixed. `HeaderRole::OverFor::keyword()` now answers `Some("FOR")`,
matching the oracle byte for byte on both engines, under both `TRACE I` and
`TRACE R`. Step 1 found no second withheld keyword: of the seven `HeaderRole`
variants, `Initial` is the only one the oracle itself echoes nothing for, and
that was independently confirmed rather than trusted from its own comment.

---

## Step 1: every `HeaderRole` variant against the live oracle

One program per variant, each isolating that role as far as the grammar
allows (`Initial` is mandatory on every `Controlled` loop, so `To`/`By`/`For`
necessarily carry it alongside their own role; `OverFor` cannot occur without
`Over`). Every probe used a `leave` as the body so only the header's own echo
is at issue, run from a fresh `mkdir`ed directory, wrapped exactly as
`rust/CLAUDE.md` specifies, stdout/stderr/rc read as three separate files.
All 14 oracle runs exited 0.

| role | program | `trace i` echoes | `trace r` echoes |
| --- | --- | --- | --- |
| `Initial` | `do ii = 1` | *(none)* | *(none)* |
| `To` | `do ii = 1 to 2` | `>K>   "TO" => "2"` | `>K>   "TO" => "2"` |
| `By` | `do ii = 1 by 1` | `>K>   "BY" => "1"` | `>K>   "BY" => "1"` |
| `For` | `do ii = 1 for 3` | `>K>   "FOR" => "3"` | `>K>   "FOR" => "3"` |
| `Count` | `do 3` | `>K>   "FOR" => "3"` | `>K>   "FOR" => "3"` |
| `Over` | `zs='abc'` / `do qq over zs` | `>K>   "OVER" => "abc"` | `>K>   "OVER" => "abc"` |
| `OverFor` | `zs='abc'` / `do qq over zs for 1` | `>K>   "FOR" => "1"` | `>K>   "FOR" => "1"` |

Full transcripts (`initial` under `trace i`, as the sharpest example of the
"confirm, don't trust the comment" instruction):

```
     2 *-* do ii = 1
       >L>   "1"
       >=>   II <= "1"
     3 *-*   leave
```

No `>K>` anywhere in it -- `Initial`'s own doc comment (`run.rs:722`-`724`,
citing `do ii = 1 to 2`) is confirmed, not merely trusted, and by a program
that carries no other role at all rather than one where a neighbouring `TO`
might have been doing the work.

`OverFor`'s own transcript (`trace i`), the one the brief names:

```
     2 *-* zs = 'abc'
       >L>   "abc"
       >>>   "abc"
       >=>   ZS <= "abc"
     3 *-* do qq over zs
       >V>   ZS => "abc"
       >K>   "OVER" => "abc"
       >=>     QQ <= "abc"
     4 *-*   leave
```

and with `for 1` added, the extra line lands between the count's own `>L>`
and the control variable's `>=>`:

```
     3 *-* do qq over zs for 1
       >V>   ZS => "abc"
       >K>   "OVER" => "abc"
       >L>   "1"
       >K>   "FOR" => "1"
       >=>     QQ <= "abc"
     4 *-*   leave
```

**Result: `OverFor` is alone.** Every other role's echo matches what
`run.rs`'s pre-fix `keyword()` already answered (`To`->`TO`, `By`->`BY`,
`For`/`Count`->`FOR`, `Over`->`OVER`); only `OverFor` disagreed, answering
`None` where the oracle answers `FOR`. The fix is the one-line table change
the brief anticipated, not a table rewrite.

---

## The fix

`rust/crates/rexx-exec/src/run.rs`, `HeaderRole::keyword()`:

```rust
match self {
    HeaderRole::Initial => None,
    HeaderRole::To => Some("TO"),
    HeaderRole::By => Some("BY"),
    HeaderRole::For | HeaderRole::Count | HeaderRole::OverFor => Some("FOR"),
    HeaderRole::Over => Some("OVER"),
}
```

The doc comment on `keyword()` and on the `OverFor` variant are corrected in
the same edit. The old `keyword()` doc read "`None` for the roles this table
withholds one from" and asserted `OverFor` was "measured not to" match the
oracle -- both false now, and the second was always a description of a bug
rather than a design fact. It now names the one role (`Initial`) that
actually withholds a keyword. `OverFor`'s own variant doc no longer narrates
the divergence (that belongs in this report and the commit message, not the
enum); it states the current, measured fact the same way `Count`'s
neighbouring doc already does: which tag, and the oracle citation.

## Did the compiled engine need a change of its own? Verified, not assumed.

No. `ir/compile.rs:398` already reads `if role.keyword().is_some() {
ops.push(Op::TraceKeyword { role, src: dst }); }` -- it was written to follow
the table, not to special-case `OverFor`. Verified by running the existing
golden IR test (`ir::golden_tests::a_block_has_an_empty_header_region_and_a_
do_over_echoes_only_its_target`, since renamed) right after the `run.rs` fix
alone, with `compile.rs` itself untouched: the compiled stream for
`do qq over 4.5 for 2` picked up `7: TraceKeyword role=OverFor src=1` on its
own, shifting every following op index by one. `compile.rs` was read to
locate the gate and never edited.

That test's own committed expectation was pinned to the pre-fix (wrong)
shape, so it had to be updated to the new one -- this is the plan's
"mechanically forced second site" exception (Global Constraints), not a
second divergence found and fixed: the reader (a golden IR dump) has to
answer for the corrected table, and it is a shared function's *own* test file
by construction. Renamed
`a_block_has_an_empty_header_region_and_a_do_over_echoes_only_its_target` ->
`..._do_over_for_echoes_both_its_target_and_count`, since the old name
asserted the bug.

---

## Which transcripts were re-captured, and from where

**`ir_dual_cases/loop-header-values`**, the `DO OVER ... FOR` row (the file's
own comment already named this row as the shape closing the gap would
change). Re-captured live, never by hand: `zs = 'abc'` / `do qq over zs for
1` / `say qq` / `end`, run through the oracle wrapper from a fresh directory,
`trace i`. The only change from the old committed block is the inserted
`>K>   "FOR" => "1"` line; every surrounding byte, including the loop's
retest re-entry at the end, matched what was already committed. The row's own
comment and the file's module-level "except the one row..." caveat (which
described this exact gap) are both rewritten -- the file no longer has an
exception to its own "every expected byte was measured against the oracle"
claim.

`grep -rniE "do +[a-z_]+ +over .* for " rust/` (excluding `target/`) turned up
no other `trace_oracle` transcript, corpus program, or crate test holding a
`DO OVER ... FOR` under trace. Besides this row and the golden IR test above,
it found: three `corpus-l1/DoOver_test_for_*.rex` files (untracked, extracted
from `ootest`, testing `FOR`'s own numeric/negative/whole-count error
validation, no `TRACE` in any of them); two rows in
`corpus/errors/parse-errors.tsv` (`do i over x for 1 to 2` and `do i over x
for 2 while 1`, both about the parser's own 49.2/legality classification, not
run at all); and `rexx-parse/src/instruction/tests.rs`'s own parser unit
tests for the same two shapes, neither of which executes anything. None of
these run under `TRACE`, so none needed re-capturing.

**The new unit test**, `run::tests::a_do_over_for_echoes_the_for_keyword_the_
oracle_prints`, was written *before* the fix (Step 2), confirmed red against
the unmodified tree (`left` missing the `>K>   "FOR"` line, `right` the
oracle's bytes), then confirmed green after. It exists because nothing else
in the tree compares this crate's trace output to literal oracle bytes for
this shape -- `trace_oracle.rs` has no witness for it and
`ir_dual_cases/loop-header-values` compares the two engines to each other.
Same reasoning and the same precedent
(`task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_
them`) as Task 2's own unit-test pins.

---

## Corpus sweep

**Through the differential harness:** full workspace suite green (below).
`both_engines_agree_on_every_case_file`, `both_engines_agree_across_every_
population`, and `the_sweep_runs_every_ootest_suite_a_sibling_harness_runs`
all pass, so nothing in the `ootest`-derived populations `ir_dual.rs` draws
from moved in a way that harness can see.

**Raw A/B**, matching Task 2's own method: every git-tracked `.rex` under
`rust/` (`git ls-files 'rust/*.rex' 'rust/**/*.rex'`), run through
`rexx-run` under both `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`,
comparing stdout, stderr and exit status raw between a pre-fix binary (built
in an isolated `git worktree` at this branch's tip before any edit, so as not
to touch or be touched by other agents' concurrent uncommitted changes in the
shared working tree) and the post-fix binary.

* **105 programs x 2 engines = 210 streams compared; 0 changed.**

Zero is the expected answer here and not a vacuous one: none of the 105
tracked programs contains a `DO OVER ... FOR` (confirmed by the same `grep`
above), so this population could not have moved for this fix regardless of
correctness -- the number that means something is Step 1's oracle table and
the new unit test, not this sweep. The sweep is reported because the brief
asks how many corpus programs changed, and the honest answer is "none, and
here is why that is expected for a construct this narrow."

---

## Gates

From `rust/`, every exit status read unpiped:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, run
  after `cargo clean` (10039 files, 3.6 GiB removed first), so the green
  reflects a linter that actually re-examined the code.
* `memcap 8G cargo test --release --workspace --no-fail-fast` -- exit 0,
  **1509 passed, 0 failed** (release binaries pre-built with `cargo build
  --release --workspace --tests` first, outside the cap -- Task 2's report
  has the caution: the same command run straight after `cargo clean` was
  itself OOM-killed at 8G while still compiling).
* The same under `REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 REXX_BIF_GATE=1
  REXX_KEYWORD_GATE=1` -- exit 0, all four STRICT banners printed, corpus
  **52 of 52 matching**, **1509 passed, 0 failed**.

(1509 is Task 2's 1508 plus this task's one new unit test.)

---

## The mutation witness

Backed up `run.rs` by `cp`, checksummed. Mutated `keyword()` back to its
pre-fix arms (`Initial | OverFor => None`, `For | Count => Some("FOR")`).
`memcap 8G cargo test --workspace --no-fail-fast` (dev profile): **exit 101,
627 passed, 2 failed** in the `rexx-exec` lib target plus **8 passed, 1
failed** in `ir_dual` --

| test | red? |
| --- | --- |
| `run::tests::a_do_over_for_echoes_the_for_keyword_the_oracle_prints` | red |
| `ir::golden_tests::a_block_has_an_empty_header_region_and_a_do_over_for_echoes_both_its_target_and_count` | red |
| `both_engines_agree_on_every_case_file` (`ir_dual`, the `loop-header-values` row) | red |

All three catchers are files this task itself touched; each is its own
witness, which is the Global Constraints exception's own attribution test
("give each half its own test and its own mutation witness, and say plainly
in the report that you did"). Restored `run.rs` from the `cp` backup,
`touch`ed, `sha256sum -c`-verified, rebuilt: **exit 0, 1509 passed, 0 failed**
again.

**Where this task's edits sit against the plan's scope rule.** Three files
changed: `run.rs` is the assigned fix. `ir_dual_cases/loop-header-values` and
`golden_tests.rs` are the "mechanically forced second site" the exception
describes -- both are readers pinned to the exact bytes `HeaderRole::keyword()`
produces, by construction rather than by choice, and both went from encoding
the bug to encoding the fix as a direct, unavoidable consequence of the one
enum-arm change. Nothing else was touched, so there is no divergence on the
found-and-not-fixed list to attribute.

---

## Found and deliberately not fixed

Nothing. The `grep` sweep for `DO ... OVER ... FOR` under trace (above) found
no other site, Step 1 found no second withheld keyword, and no other
divergence surfaced while doing this work.
