# Task 15 re-review: fix rounds 1 and 2

Scope `dd01df2b4..6cd464849`, five commits. Round 1 is `e9f1e830b` and `df48d53f0`; round 2 is
`eb85dfdd6` through `6cd464849`.

## Verdict

**CHANGES REQUESTED, on prose only.** Every one of the original review's eight findings is closed,
and I could not fault the behaviour change, the instrument that covers it, or either committed
sitting. Three statements must change before the task closes: a disclosure the plan's own guard rule
requires and the report does not make, a provenance sentence in a doc comment that the committed
artifact contradicts, and a residual instance of the shape finding 8 existed to remove -- together
with the report sentence that claims the sweep found no such instance. Three further items are
parked below.

Nothing here is a behaviour defect and nothing here is a test-instrument defect.

## What I ran

Two mutations, both applied to `crates/rexx-exec/src/lib.rs`, both restored from a copy taken before
the first edit (`md5sum` matched on restore, `git status --porcelain` empty), and the workspace
rebuilt afterwards from the restored source -- `cp -p` preserves mtime, so `cargo build` reported
`Finished` in 0.02 s without recompiling until the file was `touch`ed, which is precisely the stale
binary hazard. Rebuilt: `cargo build --release --workspace`, 23.8 s, and the tree is clean at
`6cd464849`.

* **Round 2's liveness.** The `DELEGATE` arm reduced to `vec![(upper, None)]`, i.e. the setter's key
  not installed. `cargo test --release -p rexx-exec --lib --no-fail-fast`:
  `a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run` FAILED at
  `dispatch.rs:3634`, `720 passed; 1 failed`, on the setter row
  (`".K~a = 5\nsay 'stored'\n..."`), left `(159, "", "... Error 97.1:  Object \"The K class\" does
  not understand message \"A=\".\n")` against right `(120, "", "rexx-exec: a ::METHOD with no body
  of its own is not implemented (Phase 5)\n")`. Byte-identical to the transcript the report quotes.
* **The same mutation against the whole workspace.**
  `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`: the same single catcher,
  `177 of 177 matching`. So "a row per half, and nothing else" is verified in the strong direction --
  no other test and no corpus program moves -- and the negative claim about the corpus is now
  measured rather than only grepped.
* **The width assertion.** `kind: Option<GeneratedKind>` added to `InstalledMethodBody`;
  `cargo check -p rexx-exec` gives `error[E0080]: evaluation panicked: assertion failed:
  size_of::<InstalledMethodBody>() == 16` pointing at the assertion (line 3218 with the field added,
  3217 without, which is why the report's quoted `3218:15` and the assertion's committed line differ
  by one).
* **`cargo doc --no-deps -p rexx-exec --document-private-items`**: 15 `unresolved link` warnings.
* Read-only: `interpreter/parser/DirectiveParser.cpp`, the TSV, the corpus, `gate_table_d`'s row
  table.

**What these checks could not see.** The mutation run cannot see a row that asserts the wrong
subject -- both new rows would still pass if their expected text were wrong in the same way the
installer is -- and the report says exactly that about its own instruments. The `cargo doc` count
cannot see a link that resolves to the wrong item. Neither run re-measures a sitting.

## The three items the coordinator raised

### 1. The accumulated drift over 1% is not disclosed, and the attribution holds

**The report does not contain the sentence.** `grep -in 'accumulat\|1%\|owns the answer'` over the
report matches twice, both in the sitting section: "`pinned>head` is the accumulated phase drift and
is not this task's delta" and the `cycles:u` paragraph. There is no statement that the accumulated
ratio is over 1% on any axis and no statement of who raised it. The plan's rule
(`docs/superpowers/plans/2026-08-17-phase-5a.md:470`-`:472`) is explicit: "a task whose contribution
is nil against an accumulated ratio over 1% says that, and the earlier task that raised it owns the
answer." Round 2's sitting is that case exactly -- contribution within 9 ppm of 1.000000 on every
axis and arm, against `pinned>head` of 1.012257 to 1.012597 on `dispatchclass` and 1.015430 and
1.018638 on `rexxcps`. The numbers are printed in the report's own tables; the sentence the rule asks
for is not written. **This is the finding, and it is a must-change.**

**Both readings confirmed, from the TSV with `instrument` retained in every projection.**

`task`, `scope=across_builds`, `build=pinned>head`, `instrument=instructions:u`, axis
`dispatchclass`, in commit order:

```
13-fixround-1  d3758d520   ir large 1.009290  ir small 1.009267  tw large 1.009953  tw small 1.009940
14             6f7899514   ir large 1.011803  ir small 1.011780  tw large 1.012133  tw small 1.012110
14-fixround-1  13219ab92   ir large 1.011804  ir small 1.011790  tw large 1.012128  tw small 1.012118
14-fixround-2  04cc77f3d   ir large 1.011802  ir small 1.011786  tw large 1.012132  tw small 1.012119
15             aed89a3e7   ir large 1.012276  ir small 1.012258  tw large 1.012597  tw small 1.012583
15-fixround-2  eb85dfdd6   ir large 1.012279  ir small 1.012257  tw large 1.012597  tw small 1.012572
```

**The axis crossed 1% during Task 14, not Task 15.** Task 13's fixround, the first sitting the axis
has, sat at 1.009267 to 1.009953 -- under the line on all four cells. Task 14 took it over, and its
own contribution is the crossing: `pinned>head` over `pinned>base` in Task 14's own sitting is
1.002492 (ir, both sizes), 1.002153 (tw large) and 1.002149 (tw small), against a base of 1.009265
to 1.009958. Task 15's two sittings add 464 to 467 ppm and then nil. So the sentence Task 15 owes on
`dispatchclass` names Task 14.

**`rexxcps` has been measured by Task 15 alone.** `axis` takes eight distinct values in the file and
`rexxcps` appears under `task` values `15` and `15-fixround-2` only; `grep -c bench-rexxcps` on the
TSV is 0, so it is not hiding under the path spelling either. The plan already licenses the
consequence at `:320`-`:322`: the axis "is added from Task 15", is not retroactive, and "a first
reading is a position rather than a delta", so no task can be named for the 1.0154/1.0186. That is
what Task 15 should say about it -- an unattributable position, not an owner.

### 2. Round 2's instrument claim, verified in all four parts

* **A row per half, and nothing else.** `git show --stat eb85dfdd6` touches `dispatch.rs` (+22, the
  two rows and their comment) and `lib.rs` (the arm) and nothing else; `6cd464849` touches only that
  comment; `e243c8755` only the TSV. The two rows are in
  `a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run`, which drives
  `both_engines(source)` (`dispatch.rs:3348`) -- it runs `Engine::TreeWalker` and `Engine::Ir` and
  asserts they agree, so both engines are covered by the rows themselves, not only by the report's
  hand-run probes. The mutation run above confirms nothing else in the workspace sees the change.
* **No corpus program covers the combination in either direction.** `grep -ril delegate corpus/`
  matches five files: the two `DELEGATE` table D probes (`::method m delegate d`,
  `::attribute at delegate d` -- neither carries `ATTRIBUTE`), `lang/directive_attribute_installs.rex`
  (a comment saying it has no `DELEGATE` option), and `corpus/docs/directive-options.txt`, which is
  the table itself. Confirmed empirically by the whole-workspace mutation run: `177 of 177`.
* **Table D's row identity is one keyword.** `corpus/docs/directive-options.txt`'s header states one
  `directive<TAB>keyword<TAB>position<TAB>side<TAB>evidence` per line, and the rows read that way
  (`::METHOD  DELEGATE  subkeyword  both  DirectiveParser.cpp:796 ...`). A two-keyword combination
  has no row to be.
* **Liveness.** Run above; the failure text is the one the commit message and the report predict.

The refusal itself satisfies the three-descriptor rule: both rows refuse on the program's first
clause, and the assertion pins `stdout` to `""`, the code to `NOT_IMPLEMENTED_EXIT` and `stderr` to
the refusal line. No table D verdict cell is involved, so there is no `diverge-status` question here.

### 3. Round 2's sitting, and the bimodal pair

**The rows.** `task=15-fixround-2`: 600 rows, one `commit` value `eb85dfdd6`, `build` in
`{pinned, base, head, pinned>base, pinned>head}`, `value_rounds` 5 on every row, `arm` covering
`ir`/`tw` (plus the derived `ir/tw` and `ir-tw` scopes), `size` both, eight axes, both instruments,
scopes `absolute`/`across_builds`/`arm_ratio`/`fixed`/`per_pass`/`per_pass_gap`. Re-deriving
`pinned>head / pinned>base` on `instructions:u` for every axis, arm and size reproduces the report's
30-row table digit for digit, widest `dispatchclass`/tw/small at 0.999991 and `rexxcps`/ir at
0.999995, both negative. `instrument` was in every projection.

**The `perf stat` pair supports the conclusion drawn from it, with one caveat the report itself
states.** Four runs, alternating, each build producing one ~25.7359e9 and one ~25.788e9 reading, is
the right shape of evidence: it shows the 0.2% is not a property of the binary. Two runs per build
cannot resolve 0.2% on their own, and the report says so ("a single pair of runs on this axis cannot
settle anything"), so the conclusion rests on the five-round sitting -- which is the correct
ordering. It is corroborated independently by the same sitting's own spread: base-arm `per_pass`
`value_min`/`value_max` on `dispatchclass` are 6496.358670/6556.351206 (tw) and 6387.393660/
6443.388558 (ir) in Task 15's main sitting, a 0.87 to 0.92% span, four times the 0.2% in question.
No objection.

### 4. Round 1's F5 remedy: provenance stated, nothing relabelled

`git diff dd01df2b4..6cd464849 --numstat` on the TSV is `832 0` -- 232 rows from `df48d53f0` and 600
from `e243c8755`, and **zero deleted lines**, so no surviving row could have been renamed to stand in
for a lost one. `task=15-breach` is 232 rows, all at `commit=dd01df2b4`, with `build` values
`a-base-979522f74` through `h-split-dd01df2b4` and the seven `a-base>...` ratio builds. The
declaration is in the report twice: "**These rows are re-measured, not the original sitting's**" in
"The breach sitting, and where it lives now", and "**These rows are re-measured** ... nothing was
relabelled" in fix round 1's item 5.

The re-measured table matches the report's quoted figures exactly (`6417.359`, `6538.416`,
`6535.436`, `6520.379`, `6547.375`, `6534.405`, `6417.416`, `6420.446` per pass on ir). The prose
`+1.86%` for the original sitting is consistent with the re-measurement: `b-presplit`'s tw cells read
1.018552 and 1.018589, and its ir cells 1.018783 and 1.018823.

**But see finding N2 below**: the doc comment that consumes this artifact describes it as the source
of figures it did not produce.

## The original eight findings

* **F1 CLOSED.** `install_method`'s comment now cites `parser/DirectiveParser.cpp:826`-`:915` and
  says `DELEGATE` under `ATTRIBUTE` installs the pair. Checked against the source: the quoted
  sentence is at `:826`, the `isAttribute` branch calling `createDelegateMethod(setterName, ...)` at
  `:848`-`:852`, and the precedence `DELEGATE` (`:826`), `ATTRIBUTE` (`:850`), `ABSTRACT` (`:901`)
  runs to about `:915`. The installer installs the setter's key first and the plain name second,
  which is also the C++ order. Round 1's transcripts are kept and pointed at round 2.
* **F2 CLOSED.** `const _: () = assert!(size_of::<InstalledMethodBody>() == 16);` at `lib.rs:3217`,
  proved live above. Its doc carries the caveat ("**This catches the change and does not guard the
  mechanism**") and the counter-measurement (26,243,886,462 against 26,207,891,553). The `+1.88%` and
  `121` in that doc both trace: `15-breach` `b-presplit` ir large is 1.018823, and
  (26,203,012,030 - 25,718,896,752) / 4,000,000 = 121.03 from the committed rows.
* **F3 CLOSED.** The concern is gone and the measured result is in the instrument section: M1 fails
  `an_abstract_send_is_refused_at_the_send_naming_the_message` and takes the corpus to `176 of 177`
  naming `lang/method_abstract_send.rex`. Not re-run here; the previous review measured it
  independently and round 2 touches neither the raise nor the program.
* **F4 CLOSED, and every figure re-derived.** Four axes at exactly 1.000000 (`alloc4c`, `arith`,
  `compound`, `varlookup`); `emptyloop`/ir/small and `strings`/ir/small at 0.999999; `rexxcps` at
  1.000010 and 1.000026; `dispatchclass` at 1.000458 to 1.000467. Per-pass movement 6496.432133 to
  6499.347899 (tw, 2.92) and 6417.444253 to 6420.423053 (ir, 2.98). The in-sitting `value_min`/
  `value_max` bounds are quoted correctly, head's tw minimum (6469.417368) is below base's
  (6496.358670), and head's ir median sits inside base's range. The `~13` figure is now labelled
  cross-sitting. See N5 for the one loose edge.
* **F5 CLOSED**, with N2 attached.
* **F6 CLOSED.** `grep -rn MethodRole --include='*.rs'` over the workspace matches nothing.
  `cargo doc` reports 15 unresolved links; none is in a file this task changed except
  `lib.rs:2332` (`TraceMode::OFF`). Spot-checked five of the 15 with `git blame -w`: `23730478e`
  (2026-08-19), `abd44f8fb` (2026-08-16), `f906aabc8` (2026-08-03), `567dcb122` (2026-08-18),
  `5678d5f1b` (2026-08-12) -- all older than Task 15's base `979522f74` (2026-08-23). The
  `[NativeMethod]` link is gone and "a primitive method" is the crate's own word for it
  (`dispatch.rs:2152`). `install_attribute`'s second paragraph now describes the two tables.
* **F7 CLOSED.** The count is replaced by the set: every `attribute__*.rex` and `method__*.rex` table
  D probe, matching on both engines except `attribute__external__subkeyword.rex` and
  `method__external__subkeyword.rex`, both named, both `ORACLE_REFUSES` rows owned by Phase 7 and
  already gated at `979522f74`. The retained counts trace: the rewritten set is enumerated as 9 plus
  10 and the gate listing shows 19 rows, and the 7 gated rows are named as 5 plus 2.
* **F8 CLOSED at the named sites, with a residual.** `collect_stress.rs`, `coverage.rs`,
  `corpus/phase-5a.txt`, the `::ABSTRACT` doc and the `method_attribute_generated_*` doc are all
  rewritten to name their sets, and the rewrites are true: `collect_stress.rs`'s generic singular
  covers each of the three programs beneath it, and `phase-5a.txt`'s "separate programs because a
  program can only fail once" is the right reason. The `install_attribute` site the sweep found is
  rewritten and the order it now names -- external, then abstract, then delegate -- matches
  `attributeDirective` (`:1673`, `:1697`, `:1703`, and the same order in the `GET` and `SET` arms).
  **The sweep did not relocate what it removed at those sites**; it did leave the shape elsewhere,
  and added one instance of it -- see N3.

## New findings

### N1. The accumulated-drift disclosure the plan requires is missing -- **prose, must-change**

Evidence and attribution in section 1 above. The fix is one sentence in the round-2 sitting section:
this task's contribution is nil on every axis while `pinned>head` is over 1% on `dispatchclass` and
`rexxcps`; `dispatchclass` crossed during Task 14, whose own contribution was 1.002149 to 1.002492
against a base of 1.009265 to 1.009958; `rexxcps` has rows from Task 15 only, so its 1.0154/1.0186 is
an unattributable position, which the plan's own axis paragraph already says of a first reading.

### N2. `GeneratedMethod`'s doc names the committed sitting as the source of figures it did not produce -- **prose, must-change**

The doc closes with "The sitting behind those figures is `bench-baselines/phase-5a-arms.tsv`,
`task=15-breach`." The four figures above it are direct `perf stat` readings, and every one differs
from its committed counterpart:

```
doc                 15-breach absolute (ir large)   delta
25,723,898,929      25,718,896,752                  +5,002,177
26,207,891,553      26,203,012,030                  +4,879,523
26,243,886,462      26,239,005,782                  +4,880,680
26,191,934,770      26,186,940,948                  +4,993,822
```

The report gets this right -- the readings "were taken directly on `bench-programs/dispatchclass.rex`
while the attributions were being chosen, and the sitting above is the committed form of the same
comparison". The comment compresses that into a provenance the artifact does not support, and this is
the one doc a future task reads before deciding whether the separation still matters, which is the
whole reason F5 was raised. The conclusion is unharmed: the committed rows reproduce the same 121 per
send and the same ordering of the attributions. Restate the pointer as a re-measurement of the same
comparison rather than as the source.

### N3. The set-size shape survives in `lib.rs`, in the commit that swept for it, and the report says otherwise -- **prose, must-change**

Two sites say "beside the two fields", counting the fields of `InstalledMethodBody`, which the struct
definition enumerates two and six lines above respectively:

* the width assertion's doc, `lib.rs:3223` -- **added by round 1**, the same commit as the F8 sweep;
* `GeneratedMethod`'s doc, `lib.rs:3225`, pre-existing from round 0 and inside the diff the sweep ran
  over.

The constraint admits true counts, so both are violations, and "beside its fields" costs nothing.
Worse, the report's F8 item states "The remaining `two`s in the diff are back-references to a pair
named in the same sentence ... not cardinalities over an enumerated set" -- a universal claim over
the diff that is false for these two. Neither sentence names the pair it is counting. Running the
report's own sweep pattern (a cardinality word followed by a plural noun) over
`git diff 979522f74..6cd464849 -- crates/ corpus/` on added comment lines returns these two among the
back-references, so the pattern was right and the reading of its output was not. Fix the two comments
and the report sentence.

### N4. "Files changed" does not name round 2's sitting rows -- **prose, park**

The bullet reads "`rust/bench-baselines/phase-5a-arms.tsv` -- the sitting, and fix round 1's
`15-breach` rows". Round 2 added 600 rows under `task=15-fixround-2`. True after round 1, incomplete
after round 2; the rows themselves are committed and the round-2 section describes them, so nothing
is unverifiable, which is why this is a park.

### N5. The `cycles:u` disclosure compares a size-restricted interval against an unrestricted one -- **prose, park**

"on axes that provably cannot reach the new code, `cycles:u` `pinned>head` runs from 0.931735
(`arith`/ir) to 1.036705 (`strings`/ir)" is exactly the `small` rows of Task 15's sitting. Across both
sizes the interval is 0.931535 (`arith`/ir/large) to 1.059877 (`strings`/tw/large), while the
`dispatchclass` figures quoted beside it, 1.051351 to 1.058532, span all four cells. The conclusion
gets stronger under the wider reading, so this is a labelling nit, not an error in the argument.

### N6. `install_attribute`'s rewritten comment generalises the body clause across all three styles -- **prose, park**

"`attributeDirective` asks the same questions in the same order for each style ... and only where none
of them answers does the presence of a body decide: `hasBody()` there, `attribute.body` here." The
question order does hold for all three styles. The body clause holds for `ATTRIBUTE_GET` and
`ATTRIBUTE_SET` (`hasBody()` at `DirectiveParser.cpp:1773` and `:1836`); `ATTRIBUTE_BOTH` forbids a
body outright with `checkDirective(Error_Translation_body_error)` at `:1670`, so there a body decides
nothing and is a syntax error. The crate-side half of the sentence is accurate about the crate. This
wording predates the fix rounds and was reworded rather than introduced by them, and whether the
crate should refuse a body on the `BOTH` style is a parse-level question this task never opened.

## Also worth correcting in the record

The coordinator's brief says the report states "a copy of F1's false claim was also found in the
report body". No such sentence is in the report; `grep -in 'copy of\|the report body\|same false'`
matches nothing. What the report has is round 2's "Prose around the edit" paragraph, which lists four
sentences it rewrote because they were true before the change and false after it -- including "What I
implemented"'s claim about where the two engines part on the install. Those four all read correctly
now, and `grep -n 'The K class'` finds the old receiver only inside round 1's transcript, which is
explicitly marked as superseded.

## Tree state

Two mutations applied and restored (`md5sum 109e8c23b8446d858f20f21601a99f8c` before and after each),
`crates/rexx-exec/src/lib.rs` `touch`ed and the workspace rebuilt so no mutated binary survives in
`target/release/`, and `git status --porcelain` empty at `6cd464849`.
