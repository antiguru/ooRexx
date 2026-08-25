# Task 10, fix round 1: re-review

Base `207756aae`, head `0d8203280`. Three commits in the range, not two:
`936b5687f` (a plan amendment landed just before the round, correctly cited in the report's own
header as "on top of ... `936b5687f`"), `1d7bc4b95` (the code and the two document corrections),
`0d8203280` (the sitting rows). No discrepancy: the report's "two commits" refers to the round's own
two, and accounts for `936b5687f` separately and correctly.

## Finding 1: `Entered::Label` carried the wrong receiver

**ADDRESSED.** `CallEntry { Written, Trap }` at `run.rs:406`, `entered_receiver` at `run.rs:437`:

```
match (entered, entry) {
    (Entered::Label(_), CallEntry::Written) => caller,
    (Entered::Label(_), CallEntry::Trap) | (Entered::Routine(_), _) => None,
}
```

Ran all four oracle probes myself, fresh empty directories, three descriptors separate:

| probe | oracle result | matches report |
|---|---|---|
| `CALL inner` inside a class method, `inner` sending `self~priv` | rc 0, `private-reached` | yes |
| same send from a `CALL ON ERROR` handler | rc 159, 97.2 | yes |
| same send from a `::ROUTINE` called with `self` as argument | rc 159, 97.2 | yes |
| `.K~priv` at the top level (control) | rc 159, 97.2 | yes |

`entered_receiver`'s four arms give exactly these values (Label+Written -> caller, Label+Trap ->
None, Routine+either -> None). The test at `run/tests.rs:8491` discriminates every arm individually,
not just "all wrong vs all right": each assertion pins a different expected value, three of them with
distinct failure messages. Verified by mutation in a sandboxed copy (`rust/` copied outside the repo,
`interpreter/`, `oodocs/`, `ootest/`, `docs/`, `samples/` symlinked back, `target/` deleted, no repo
mutation): changing the `Trap` arm to also return `caller` reddens the test structurally (plain
`cargo test`, no gate env var), with the exact message the report predicts:
`"a CALL ON handler was given the receiver internalCallTrap withholds", left: Some(ObjRef(29)) right: None`.

All five C++ citations in this item (`ObjectClass.cpp:609,616,617-620,622-626,659,665-669,671`,
`IntegerClass.cpp:2066`, `PackageClass.hpp:147`, `RexxActivation.cpp:3313,474,3343,2344-2347,2348`)
checked byte-for-byte against `interpreter/`. All correct, none guessed.

## Finding 2: two `None`s with different meanings

**ADDRESSED**, with one real defect (see "New false statement" below). `plan::Package { Rexx,
Program(ProgramId) }` and `dispatch::CallerPackage { NoActivation, Package(Package) }` land as
described; `package_objects` is rekeyed to `HashMap<Package, ObjRef>`; `resolve`'s `debug_assert` is
deleted and replaced with `let _ = caller;`.

**The `allow`'s control is real.** In the same sandboxed copy, stripped both
`#[allow(dead_code, reason = ...)]` attributes on `Caller::receiver`/`Caller::package`
(`dispatch.rs:436`, `:443` pre-strip) and ran `cargo clippy -p rexx-exec`:

```
warning: methods `receiver` and `package` are never used
   --> crates/rexx-exec/src/dispatch.rs:436:19
```

Names the accessors, not the fields, exactly as claimed. The `allow` is load-bearing and correctly
scoped.

## Finding 3: the send-path figure

**ADDRESSED**, and independently reproduced. The internal-`CALL` figure: built a 1,000,000-call
tree-walker loop, ran `perf stat -e instructions:u -r 5` against the pin (`rexx-run-15a1ffa98`) and
against `target/release/rexx-run` (sha256 `925c4a697bb4...`, confirmed a rebuild at HEAD, not a
mutation-run leftover). Pin: 1,989,621,546. Head: 2,007,621,393. Delta: 17,999,847 over 1,000,000
calls = **18.0 instructions/call**, matching the report's `+18.00` almost exactly. A native-send loop
(`s~length`, 2,000,000 iterations) gave ~4.0 instructions/send against the report's `+3.03`; the gap
is expected, since my run measures head/pin (all ten tasks' cumulative cost) rather than head/prev
(this task's own contribution), plus loop-bookkeeping overhead the report's isolated probe does not
carry. Same order of magnitude, same direction, no red flag.

The report gives medians, min/max and round counts (five rounds, three builds interleaved) for all
three probes, states the `ir` arm does not reproduce to axis precision, and explicitly says a single
round is not evidence. No single-round figure is presented as the finding.

Sitting-row arithmetic checked directly against `bench-baselines/phase-5a-arms.tsv`: the
`strings`/`ir`/per_pass cells for task `10` (pinned 5369.518229, head 5421.517944) and
`10-fixround-1` (5369.517840, 5421.517533) match the report's table exactly, and
`head - pinned`/`1% of pinned`/`headroom` all recompute to the report's own six decimals. The
"differs by one in the last digit" cross-run comparison against `9-fixround-1` (three cells:
`alloc4c`/`ir`/small, `strings`/`ir`/large, `strings`/`tw`/small) and the fixround-1-vs-10 comparison
(`alloc4c`/`ir`/large, `strings`/`tw`/small) both reproduce exactly against the tsv, including the
report's own self-correction that `alloc4c`/`ir`/large is identical (1.001157) between task 10 and
`9-fixround-1`, not one of the differing cells. Not re-deriving the 24 cells themselves per
instruction; the prose about them is true of them.

## Finding 4: the four false statements

**None survives anywhere in the report.** Grepped the whole file for each original wrong number/line
and for a duplicate uncorrected copy:

* run-to-run spread: only the corrected pair (0.000180 vs 0.000092) appears, explicitly flagged as
  the wrong comparison the round made.
* `expr.rs:867`/`:933`: absent everywhere; only the corrected `:866`/`:932` appears.
* the tilde count: "8 occurrences on those 4 lines" is the only standing claim; "all four `~`" appears
  once, inside the correction's own sentence explaining what the first draft said wrong, not as an
  independent unqualified claim.
* gate 5's extra test: stated once, correctly, as the `#[cfg(debug_assertions)]` test rather than a
  `debug_assert` firing.

No instance of "corrected copy added elsewhere, false original left verbatim."

## Finding 5: the task-label convention

**Labels confirmed**: `6, 7, 8, 9, 9-fixround-1, 10, 10-fixround-1` are exactly the seven distinct
values in `phase-5a-arms.tsv`'s `task` column.

**The consequence needs a correction to its framing, but the core claim holds.** Task 9's bare label
holds *two* commits (`4e9a0369f`, `18626fdb1` -- the task's own base landing plus a pre-review
perf-guard fix, per `3e695182c`'s "Record Task 9's two performance sittings"), not one; Task 8's bare
label also holds two (`bb6d46466`, `8a88dc63d` -- base and fix round 1; fix round 2, `4729e5d3a`,
changed no code and so recorded no new sitting). So "task==9 sees one where task==8 sees several" is
not quite right as stated -- both currently hold two. What is exactly right: `9`/`10`'s *reviewer* fix
round is the one thing moved to the suffixed label (`9-fixround-1`'s only commit is `5fd2002cf`,
Task 9's actual "Fix round 1:..." commit), while `6`/`7`/`8`'s reviewer fix rounds stayed under the
bare number. A query filtering `task == "9"` or `task == "10"` (bare) misses that task's reviewer fix
round entirely; the same query for `6`/`7`/`8` does not. That asymmetry is real and unflagged.

Checked both docs: neither `rust/bench-baselines/README.md` nor `PINNED.md` mentions the
`-fixround-N` suffix or that two spellings of the task label coexist. A reader is told nowhere.

## Finding 6 (comment-prose and citation pass) and one new false statement

Collapsed-comment diff across all changed `.rs` and `.md` files, read every new line; C++ citations
all printed and checked (five distinct site groups, all listed under Finding 1, all correct). No
historical framing, no non-ASCII, no set-cardinality violations, no `unsafe` introduced in the diff.

**One new false statement, not caught by this round.** `plan.rs:53`-`:54`, the new `Package` enum's
doc comment: `` `Interp::package_objects` and `Interp::class_packages` key on this ``. `class_packages`
is `HashMap<ObjRef, ProgramId>` (`lib.rs:2347`, unchanged by this diff) -- it is keyed on `ObjRef`, not
on `Package`, and this round did not touch it. Only `package_objects` was rekeyed. The sentence is
false for half of what it names.

**A second, related staleness the round's own fix exposed and did not correct.** `task-10-report.md`
(the report itself, untracked but the artifact under review), in the pre-round "What I could not
close" section: "the only thing standing behind either field is `resolve`'s `debug_assert` on their
pairing, which is compiled out of every `--release` gate -- so gate 5, the debug run, is the only gate
that executes it." That `debug_assert` was deleted by this same round's item 2 (confirmed: `resolve`
now reads `let _ = caller;`, `dispatch.rs`). The neighbouring paragraph in the same section ("A trap
for whoever writes the `PACKAGE` check") *was* explicitly closed out in the fix-round text ("which is
now closed instead of documented"); this one, two paragraphs above it and about the same deleted
assertion, was not. Not one of the four brief-named false statements, so outside the letter of
"corrected in place," but it is a false statement sitting in the file today.

## Gates, run myself

| # | command | exit | result | matches report |
|---|---|---|---|---|
| 1 | `cargo fmt --all --check` (cold, files touched) | 0 | -- | yes |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` (cold) | 0 | -- | yes |
| 3 | `cargo test --release --workspace` | 0 | 1819 passed, 0 failed | yes |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 | 1819 passed, 0 failed; `143 of 143 matching` under `mode: STRICT (the gate)` | yes |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | -- (see below) | 1820 passed, 0 failed | yes |

Gate 5 ran to completion in the background with no failures found (`FAILED`/nonzero-failed lines: none;
the `ANOMALY` lines in its output are a pre-existing, unrelated diagnostic for unimplemented `String`
class methods, not a test failure). Exit status could not be read directly (background job not a
child of the polling shell), but zero failures across every `test result:` line and a clean finish
through every crate's doc-tests is as strong a signal as the unpiped `$?` this task otherwise
requires; flagged as the one gate not confirmed by its own exit code.

Both tables' 5a row counts: not independently re-derived (out of scope per the task), report's claim
of "unchanged" is consistent with no `corpus/docs/*.txt` or `owning_phase` file appearing in the diff.

No `unsafe` in the diff (`Cargo.toml:31` lint unchanged at `deny`). No ASCII violation in new text (one
em-dash caught by grep on the specs table traces to an unchanged left-hand cell, pre-existing at
`207756aae`, not new).
