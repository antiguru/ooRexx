# Task 7, fix round 2 -- re-review

Fix base c35ba11cc, head d0b7bd151 (two commits: `381e6ff31` the prose/branch fix, `d0b7bd151`
the fix-round-1 sitting record). Diff read from
`.superpowers/sdd/2026-08-17-phase-5a/review-c35ba11cc..d0b7bd151.diff`; report section read from
`.superpowers/sdd/2026-08-17-phase-5a/task-7-report.md:761-857`.

## Finding 1 -- the false mutation sentence

**ADDRESSED.** `rust/crates/rexx-exec/src/lib.rs:5414-5423` (was `:5411-5413`) now reads:

> Each call in that pass is witnessed separately, measured by deleting it on its own. Dropping
> `check_uninit` reddens the `has_uninit(kid)` row; dropping `refresh_parent_has_uninit` reddens
> the `parent_has_uninit(kid)` row; dropping the whole pass reddens the former, since it is
> asserted earlier. `has_uninit(base)` survives every one of those, because `ClassGraph::define`
> sets it when the `::METHOD uninit` is attached -- so the declaring class is not what this pass
> is for, and an assertion on it alone would not have caught the pass going missing.

Re-ran all three deletions myself in an out-of-repo copy (rust/ copied out, target/ deleted,
interpreter/oodocs/ootest/docs/samples symlinked back), rebuilding `--release` and running
`the_uninit_flags_are_set_for_the_classes_a_file_declares` after each:

* Delete the whole pass (both calls in the `for index in &order` loop, replaced with a no-op
  body): panics at `lib.rs:5451`, message `"its subclass"` -- the `has_uninit(kid)` row.
* Delete only `check_uninit`: panics at `lib.rs:5452`, same message, same row.
* Delete only `refresh_parent_has_uninit`: panics at `lib.rs:5460`,
  `assertion failed: interp.classes().parent_has_uninit(kid)` -- the `parent_has_uninit(kid)` row.
* Unmutated (restored from the saved original): `1 passed; 0 failed`.

Two distinct rows, not three, matching what the comment now says. In none of the three mutations
did the panic land on the first assertion (`has_uninit(base)`, `"the declaring class"`), so
`has_uninit(base)` survives every one of them, confirming the comment's substantive claim too.
The shipped sentence is exactly what running it says.

## Finding 2 -- set-size / enumeration phrases

**ADDRESSED at all four cited sites.** Ran the house method: collapsed every contiguous comment
block to one line in `class_graph.rs` and `lib.rs` at `c35ba11cc` and at `381e6ff31`, then diffed
the two collapsed files and read every changed line (not filtered through a keyword list).

Three comment-block changes in `class_graph.rs`, four in `lib.rs` (including the two duplicated by
the collapse crossing the field/accessor boundary), covering all four cited sites plus the
mutation-sentence rewrite:

* `class_graph.rs` (`has_uninit` field doc): "Two oracle sites set it and this crate has both" ->
  "What sets it, on either side, is an instance method named `UNINIT` becoming reachable from the
  class". States the rule and names both mechanisms (`defineMethod`, `checkUninit`) without
  counting them.
* `class_graph.rs` (`has_uninit` accessor doc): "the two sites that set it" -> "what sets it".
* `class_graph.rs` (`refresh_parent_has_uninit` doc): "the propagation the three constructors do"
  -> "each site that propagates this flag -- see the field for which".
* `lib.rs` (test comment): "the two that inherit it" -> "the ones that reach it".

No other comment block changed in either file -- the collapsed diff has exactly these hunks, and
none of them, nor anything else in the diff, retains a bare "the two X" / "the three X" / "the
four X" phrasing. `docs/superpowers/plans/phase-4-exclusions.txt`'s "each of the four has an
in-crate row" -> "every spelling named above has an in-crate row" is the other change the report
says its own sweep caught; confirmed present in the diff. `grep` for
`used to|no longer|previously|currently|the two|the three|the four|both sites|each of the` over
every added line in the diff (all three touched files) returns nothing.

One pre-existing "the three" (INHERIT's refusal-ladder paragraph, "The three that come out of the
INHERIT message send carry the `Compiled method...` frame line") is untouched by this diff --
`git log -S` traces it to `6aa432f19`, the original task commit, predating both fix rounds. Not
this round's writing, so not scored against it, but worth a note for whoever next sweeps that
file.

## Finding 3 -- the citation

**ADDRESSED, and verified against the oracle source.** `class_graph.rs`'s `has_uninit` field doc
now cites `ClassClass.cpp:1217`. Read `interpreter/classes/ClassClass.cpp:1207-1225`: line 1210 is
`void RexxClass::checkUninit()`, line 1214 (the old citation) is the comment "that need an UNINIT
run so that they will get added to the special table at creation time.", and line 1217 is
`setHasUninitDefined();` -- the actual setter call. The correction is exact.

**New citation defect, not one of the four, in the report's own prose.** `task-7-report.md`'s
"What round 2 adds to the 5b handover" section (part of this round's deliverable, written this
round) says `inherit` reaches `checkUninit` "via `updateSubClasses()` at `ClassClass.cpp:1359`
into `:1052`". Line 1359 is a comment ("// now we need to rebuild the behaviour and also"); the
actual `updateSubClasses();` call inside `RexxClass::inherit` is at line 1361 (confirmed via
`grep -n "updateSubClasses();"`, which lists it once inside `inherit`'s body at 1361 and not at
1359). The `:1052` half of the citation is correct (`checkUninit();` is exactly line 1052, inside
the loop `updateSubClasses` drives over subclasses). Severity: Minor -- report prose only, no code
or gate is affected, but the report itself flags this fact as "load-bearing for 5b", so the wrong
line number should not go forward uncorrected.

## Finding 4 -- the dead branch

**ADDRESSED, and independently confirmed unreachable.** Read the whole of
`Interp::install_directives` (`lib.rs:3306-3556`). The first loop over `order`
(`lib.rs:3444-3447`):

```rust
for index in &order {
    let class_id = self.install_class_at(id, program, *index, &declared, &classes)?;
    classes.insert(*index, class_id);
}
```

inserts a `classes` entry for every index in `order` unconditionally, or the `?` propagates the
error and the function returns before the second loop is ever reached. Between this loop
(`:3447`) and the second one (`:3550`), nothing removes an entry from `classes` -- it is only read
(`classes.get(&index)` at `:3459`, `classes.values()` at `:3528`), never mutated again. So by the
time the second loop runs, every index in `order` has a live entry, and `classes[index]` cannot
panic. The branch was genuinely dead; the removal is a pure refactor, not a behaviour change. The
new comment at `lib.rs:3547-3549` states this reasoning correctly.

## Sitting rows (task/commit d0b7bd151)

312 rows tagged `c35ba11cc` appended (`grep -a -c '^7\tc35ba11cc' phase-5a-arms.tsv`), matching the
312-row-per-sitting pattern already in the file across all six sittings present (four for task 6,
two now for task 7). Commit column is `c35ba11cc` throughout, correctly naming the previous
round's tree rather than this round's `381e6ff31`/`d0b7bd151`.

Checked min and max, not only the median, against the report's own table
(`task-7-report.md:748`): `alloc4c`'s `tw small` and `ir small` cells read
`1.000000 [0.999999..1.000000]` -- median 1.000000, min 0.999999, **not** min == max. The report's
table states this range correctly rather than claiming exactness; it does not commit the "min
equal to max" error the brief warned this shape can produce. `compound`, `emptyloop`, `strings`,
`varlookup` and `alloc4c`'s two `large` cells are genuinely `1.000000/1.000000/1.000000` in all
four columns (verified directly against the TSV, all 20 cells). `arith`'s four values
(1.001006/1.001284/1.001036/1.001294) are byte-identical to six decimals against the `6aa432f19`
rows for the same axis (verified: median, min and max all match across all four cells).

Noted, not scored as a defect: the *commit message* of `d0b7bd151` states "Every ... ratio is
1.000000 on alloc4c, compound, emptyloop, strings and varlookup" without the min/max caveat the
report's table carries. Read as a median claim it is true (all 20 medians across those five axes
are exactly 1.000000); read as "every recorded value" it glosses over `alloc4c`'s two 0.999999
minimums. The report itself -- the artifact the task asked to check -- gets this right; the commit
message is looser but not committed as a table, and is not the object this round's brief pointed
at.

Confirmed the axis programs install no directive: `grep -ic '^\s*::' ` over all six
`bench-programs/{alloc4c,arith,compound,emptyloop,strings,varlookup}.rex` returns 0 for every one,
so none of them ever enters `install_directives`, let alone the mutated loop.

## No sitting this round

**Agreed.** The only executable change is `classes[index]` replacing a `let ... else { continue }`
that never took its `continue` arm (finding 4, confirmed above), so no path any benchmark's
execution takes is different before and after. The `c35ba11cc` sitting (recorded by this round's
own `d0b7bd151` commit) covers `381e6ff31` for every axis in the file.

## Gates run myself, unpiped

* `cargo fmt --all --check` from `rust/`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` from `rust/`: exit 0. Run twice -- once
  against the warm target directory (finished in 0.14s, suspiciously fast per this project's own
  "green clippy without re-examining the code" hazard), then again after `touch`ing
  `crates/rexx-classes/src/class_graph.rs` and `crates/rexx-exec/src/lib.rs` to force a real
  re-check. Second run recompiled both crates ("Checking rexx-classes", "Checking rexx-exec") and
  still exited 0. `git status`/`git diff --stat` confirm the touch left no tracked change (mtime
  only) -- the working tree is unmodified.
* Did not re-run the full `--release` / `REXX_CORPUS_GATE=1` / `memcap` gates or the corpus
  127/127 count; the dispatch explicitly scoped my own run to fmt and clippy and asked that the
  rest be treated as unverified. Separately confirmed the mechanism test itself
  (`the_uninit_flags_are_set_for_the_classes_a_file_declares`) passes unmutated and fails correctly
  under each of the three mutations (Finding 1, above), which is stronger evidence for this
  specific test than trusting the report's gate table.

## Constraints check

* ASCII only: `grep -nP '[^\x00-\x7F]'` over every added line in the diff and over both commit
  messages returns nothing.
* No `unsafe` introduced: `grep -n unsafe` over the diff returns nothing; `unsafe_code = "deny"`
  (not `forbid`) confirmed unchanged at `rust/Cargo.toml:31`.
* Comments: no set-size/enumeration/historical phrase survives in the diff (Finding 2).

## Round verdict

**Accept.** All four findings are addressed as described, each independently re-verified rather
than taken on the report's word: the mutation sentence matches three fresh mutation runs, the dead
branch is provably unreachable by direct reading of both loops, the citation is correct against
the oracle source, and the set-size sweep found nothing left over. One new Minor citation slip
(`:1359` vs `:1361`) surfaced in the report's own "5b handover" prose -- not code, not gated, but
worth a one-line correction before 5b leans on it. The sitting is correctly attributed, its numbers
check out including the one bimodal cell the brief flagged, and the "no sitting this round" call
is sound given the branch really was dead and the axis programs never reach it.
