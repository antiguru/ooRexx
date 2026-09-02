# Phase 5b Task 10 -- the flip

**Status: done. Phase 5b is closed.** Two commits:

* **`a9bf7d023`** -- `Phase 5b Task 10: close 5b`. The flip, the mode-banner correction, two false
  bench-program headers and their paired control, and the plan corrections.
* **`ad26cb95a`** -- `Record Task 10's sitting against the Phase 5b pin`. 432 measured rows and two
  plan paragraphs.

The five gates are green on both, and the tree was hashed before the first gate and after the last on
each. Written first and appended to as work landed; every figure is quoted with the command that
printed it, and nothing here was written before its run.

BASE `5eb74f6a3`, tree clean at start. Working directory `/home/moritz/dev/repos/ooRexx-rust-rewrite`.

---

## 1. The brief's claims, re-measured

### `CLOSED_PHASES` was `&["5a"]`

```
/bin/grep -n "CLOSED_PHASES" crates/rexx-exec/tests/gate_tables/mod.rs
```
```
72://! ([`CLOSED_PHASES`]). Outside that, a table is a progress report that always
336:/// alongside every phase in [`CLOSED_PHASES`].
344:pub const CLOSED_PHASES: &[&str] = &["5a"];
369:    corpus_gate() && (closing_phase().as_deref() == Some(phase) || CLOSED_PHASES.contains(&phase))
```

**Confirmed**, at `:344` as the brief said.

### `corpus/phase-5b.txt` carries 63 non-comment entries

```
/bin/grep -avc "^#\|^$" corpus/phase-5b.txt
```
```
63
```

**Confirmed.**

### The pin is present and matches its record

```
sha256sum target/release/rexx-run bench-baselines/pinned/rexx-run-f558ea501 \
          bench-baselines/pinned/rexx-run-15a1ffa98
```
```
d73757e856773f42444abb1dd97f562fb9136c31afcad9b9b2507ed27e57b158  target/release/rexx-run
857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e  bench-baselines/pinned/rexx-run-f558ea501
141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b  bench-baselines/pinned/rexx-run-15a1ffa98
```

`857787f7...` is byte for byte the value `bench-baselines/PINNED.md` records for `rexx-run-f558ea501`,
and `141c3fa9...` is the **retired** 5a pin, still on disk exactly as the brief warned. **Confirmed.**

Staleness test, from the repository root:

```
git log --oneline f558ea501..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml | wc -l
```
```
18
```

All eighteen are this plan's own commits (Tasks 6 to 9 and their corrections); no foreign crate work
sits under the phase. **The pin is current.**

### The three parked bench programs

Run from a fresh empty directory with absolute paths, three descriptors read separately, on both
engines, against both the head binary and the pinned one:

```
for prog in dispatch alloc heapshape; do for eng in ir tree-walker; do
  for bin in head:.../target/release/rexx-run pin:.../pinned/rexx-run-f558ea501; do
    timeout -s KILL 60 env REXX_ENGINE=$eng $path $R/bench-programs/$prog.rex >$D/o 2>$D/e; rc=$?
    ...
```
```
dispatch  ir           head  rc=0    out=[5000000|]  err=[]
dispatch  ir           pin   rc=0    out=[5000000|]  err=[]
dispatch  tree-walker  head  rc=0    out=[5000000|]  err=[]
dispatch  tree-walker  pin   rc=0    out=[5000000|]  err=[]
alloc     ir           head  rc=120  out=[]  err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
alloc     ir           pin   rc=120  out=[]  err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
alloc     tree-walker  head  rc=120  out=[]  err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
alloc     tree-walker  pin   rc=120  out=[]  err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
heapshape ir           head  rc=120  out=[]  err=[rexx-exec: method "NEW" of class "Directory" is not implemented (Phase 5)|]
heapshape ir           pin   rc=120  out=[]  err=[rexx-exec: method "NEW" of class "Array" is not implemented (Phase 5)|]
heapshape tree-walker  head  rc=120  out=[]  err=[rexx-exec: method "NEW" of class "Directory" is not implemented (Phase 5)|]
heapshape tree-walker  pin   rc=120  out=[]  err=[rexx-exec: method "NEW" of class "Directory" ... ]
```

**Confirmed** for the head column: `dispatch` runs, `alloc` and `heapshape` do not, and `heapshape`
has moved one stage since the plan's table was written. **Exactly one program goes live.**

**A first attempt at this table read `rc=0` for all nine rows and was wrong.** The line was
`echo "$prog $eng $(basename $bin) rc=$?"`, and the command substitution runs during expansion and
resets `$?` before `rc=$?` is expanded. The table above captures `rc=$?` into a variable on the line
after the run. Recorded because a wrong exit status here reads exactly like a right one.

---

## 2. A correction to the plan and the brief: `dispatch` is a comparison, not a new baseline row

The brief says "`dispatch.rex` cannot show a regression against a baseline in which it did not run.
Its first crate figure is a **new baseline row**, not a comparison". The plan says the same at its
first "Two consequences" bullet. **Measured, that is not this phase's case.**

The pin `rexx-run-f558ea501` was built at the commit
`Give an instance the behaviour its class held when it was built`, which is **Task 2**'s landing
commit (`.superpowers/sdd/2026-08-27-phase-5b/task-2-report.md:3`: "Landed as `f558ea501`"). Task 1
unblocked `dispatch.rex` before it. The pinned binary therefore runs `dispatch.rex`, measured above:
rc 0, stdout `5000000`, empty stderr, on both engines.

So `dispatch` is a **new axis in `phase-5b-arms.tsv`** and at the same time a **real comparison**
against this pin. Recording it as an uncomparable new baseline row would have thrown away the only
figure the axis can give, and would have been the mirror image of the flattered ratio the rule exists
to prevent. The plan is corrected in place; the rule itself is kept, because it governs any pin that
predates its axis being unblocked.

---

## 3. The build

`crates/rexx-exec/tests/gate_tables/mod.rs:344`

```
pub const CLOSED_PHASES: &[&str] = &["5a", "5b"];
```

### One more change in the same file: the mode banner was false, and the flip makes it worse

`Report::new` printed, under `REXX_CORPUS_GATE=1` with no `REXX_PHASE_GATE`:

```
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, no closing phase named, so no verdict is gated
```

`verdict_is_gated` gates a row whose phase is in `CLOSED_PHASES` **or** is the closing phase, so
"no verdict is gated" was already false with `"5a"` closed. After the flip it is the exact opposite
of what the flip is for: a reader seeing green is told no 5b verdict is gated. It now names the set:

```
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, verdicts gated for 5a, 5b
```

Nothing in `crates/`, `docs/` or `.superpowers/` asserts on the old wording
(`/bin/grep -rn "closing phase\|no verdict is gated"`).

---

## 4. Two false bench-program headers, corrected

Both describe this crate and both had stopped being true. Under `rust/CLAUDE.md` a false comment is
corrected or removed, not hedged.

* **`bench-programs/dispatchclass.rex`** said `dispatch.rex` "is blocked on this crate: its body
  needs ~new" and that this axis exists "while that stays true". Task 1 closed it; measured above.
* **`bench-programs/alloc4c.rex`** quoted `rexx-exec: a message send is not implemented (Phase 5)`
  as the answer to `.array~of(1,2,3)`, to `.string~new("item")` and to the combination, "all three
  the same error". Measured 2026-09-02, from a fresh empty directory, both engines:

```
of    ir           rc=120 err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
of    tree-walker  rc=120 err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
snew  ir           rc=120 err=[rexx-exec: method "NEW" of class "String" is not implemented (Phase 5)|]
snew  tree-walker  rc=120 err=[rexx-exec: method "NEW" of class "String" is not implemented (Phase 5)|]
both  ir           rc=120 err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
both  tree-walker  rc=120 err=[rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)|]
```

  The quoted message is wrong and "all three the same error" is wrong: `.string~new` differs.

Both headers were edited **before** the sitting, so the sitting measures the committed programs.

---

## 5. The stand-in decision: `dispatchclass` stays

`dispatchclass` stays as an axis beside `dispatch`, and `alloc4c` stays.

**Why `dispatchclass` is not `dispatch` measured twice.** `dispatch.rex` sends to an instance and
every body it reaches runs `expose total`, so one pass carries instance-behaviour lookup and a
per-object variable pool. `dispatchclass.rex` sends to a class object and its body returns a
constant, so one pass carries neither. Retiring it would delete the only axis that isolates the send
from the pool.

**What the double-counting warning does buy** is a reporting rule, not a retirement: a change to the
shared send path moves both, so a count of moved axes has to say the two are not independent.

**`alloc4c` stays**, for the reason the brief gives and which measurement confirms: its header's
condition is `alloc.rex` going live, and `alloc.rex` is still refused at `Array~of` (section 1).

---

## 6. The control: does the flip gate anything?

The failure mode the brief names is a flip that gates nothing. The control is two arms differing
**only** in `CLOSED_PHASES`, with one identical crate mutation held constant across both.

**The mutation**, in `crates/rexx-exec/src/dispatch.rs`'s `send_to_delegate`, making the delegate
variable's stored value unreachable so a delegated send goes to the variable's *name* as text:

```rust
let stored = self
    .pools_of(owner)
    .and_then(|pools| pools.get(resolution.scope, &variable))
    .filter(|_| false);
```

Both arms ran the identical command, with **no `REXX_PHASE_GATE` set**:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_d --no-fail-fast
```

| arm | `CLOSED_PHASES` | status | gate table D |
|---|---|---|---|
| A | `&["5a", "5b"]` | **101** | `FAILED. 14 passed; 1 failed` |
| B | `&["5a"]` | **0** | `ok. 15 passed; 0 failed` |

Both arms print the same row state -- `5b: 2 rows, 2 not yet 'agree'` -- and differ only in the last
line of the report:

```
arm A:  gated by this run: 2 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
arm B:  gated by this run: 0 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
```

**So the flip is what turns a red 5b row into an exit status**, and `gate_table_d` is a test target of
`cargo test --release --workspace`, which is gate 3 and gate 4. Restored from the scratchpad copy of
`dispatch.rs` (`cp`, not `git checkout --`); `git diff --stat` after the restore showed
`gate_tables/mod.rs` alone.

---

## 7. The DEVIATION witness runs, and it was checked rather than assumed

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test licensed_divergences --no-fail-fast
cargo test --release -p rexx-exec --test licensed_divergences --no-fail-fast
```
```
test the_licensed_divergences_still_diverge_exactly_as_recorded ... ok
test the_prose_rows_and_this_table_name_the_same_divergences ... ok
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

Both exit 0, gated and ungated, so the witness is not conditioned on the corpus gate.

**0.19s for a test that spawns the oracle is fast enough to be worth a control**, and the crate-side
runs are in-process (`run_program`, not a subprocess), which is why. The control falsifies one
recorded **oracle** byte -- `before` to `BEFORE` in `class-uninit-at-driven-collection`'s
`oracle_stdout` -- and the test goes red on what the live oracle actually printed:

```
assertion `left == right` failed: [class-uninit-at-driven-collection] the oracle's own answer moved
  left: "before\nclass uninit\nafter\n"
 right: "BEFORE\nclass uninit\nafter\n"
test result: FAILED. 14 passed; 1 failed
```

`left` is the oracle's live output, so the oracle really ran. Restored from the scratchpad copy.

---

## 8. Gate table verdicts at the flip

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast
```
```
status 0
running 14 tests -- test result: ok. 14 passed; 0 failed
running 15 tests -- test result: ok. 15 passed; 0 failed
gated by this run: 0 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
```

Rows by owning phase and verdict, counted off that run's own report:

```
/bin/grep -aE "^  [a-z-]+ +loud=(yes|no) +[0-9][a-z]* " gtcd.err | awk '{print $3, $1}' | sort | uniq -c
```
```
    171 5a agree
      8 5b agree
      3 5c agree
     35 5c diverge-both
      1 7  diverge-both
```

**Every row whose owning phase is `5b` is `agree`**, in both tables, with no structural failure. The
eight are `objcla`, `abscla`, `usesem`, `creo`, `obdes` and `methodsbyclass` in table C, and
`::ATTRIBUTE DELEGATE subkeyword` and `::METHOD DELEGATE subkeyword` in table D. The 5c and phase-7
rows are reported and not gated, exactly as D65 says they should be.

---

## 9. Two aborted gate runs, and what each caught

Both were my own defects, introduced by this task, and both are recorded because a reader comparing
timestamps will otherwise see three gate runs and no reason for the first two.

**Run 1 died at gate 2.** The mode-banner rewrite used a nested `if`, which is
`clippy::collapsible_if` under `-D warnings`:

```
error: this `if` statement can be collapsed
   --> crates/rexx-exec/tests/gate_tables/mod.rs:446:13
```

Collapsed into a let-chain, which is what clippy itself suggested.

**Run 2 died at gate 3, on a test that exists for exactly this.** `bench-control/alloc4c-101.rex` is
asserted to be byte-identical to `bench-programs/alloc4c.rex` apart from the loop bound, and I had
edited the header of one of the pair:

```
thread 'tests::the_control_differs_from_its_axis_only_in_the_loop_bound' panicked at
crates/rexx-bench/src/lib.rs:205:9:
assertion `left == right` failed: the control and its axis differ somewhere other than the loop
bound, so the difference between them is no longer the known amount it is used as
```

The same edit was applied to the control, and the pair is now identical apart from `n`:

```
diff <(sed 's/^n = .*/n = BOUND/' bench-control/alloc4c-101.rex) \
     <(sed 's/^n = .*/n = BOUND/' bench-programs/alloc4c.rex) && echo IDENTICAL
```
```
IDENTICAL apart from the loop bound
```

**`cargo test` stopped at `rexx-bench` in both run-2 gates**, so gates 3 and 4 never reached the
corpus differential. A failing gate here reports one failure and leaves the rest of the workspace
unmeasured; the counts below come from the run that went green.

## 10. Criterion 2, and how "both engines" is actually satisfied

`corpus/phase-5b.txt` is read by `corpus.rs`, `ir_dual.rs`, `coverage.rs` and `collect_stress.rs`
through their own `SUBSET_FILES` constants, each asserted against the corpus directory itself. The
"both engines" half of D65's criterion 2 is met by **composition**, and it is worth saying which
harness supplies which half, because neither does both:

* `corpus.rs`'s `corpus_differential` runs each subset program against the **oracle** on the default
  engine and compares all three descriptors.
* `ir_dual.rs` runs the same file list on **`Engine::TreeWalker` and `Engine::Ir`** and compares
  stdout, stderr and exit status **unnormalised**, plus asserts `ir.chunks_refused == 0` so the IR
  arm cannot pass by silently falling back to the tree-walker.

Together those give oracle-vs-both-engines. Neither alone does, and a reader checking criterion 2
against `corpus.rs` on its own would find only one engine there. An independent direct sweep is in
section 11.

## 11. A stale status file read exactly like a fresh one

The gate runner wrote `<name>.status` and then `<name>.done`, and the restart deleted the `.done`
markers but not the `.status` files. An ad-hoc progress check read `g3-test: status=101` at 11:02:52
while gate 3 was still running -- that `101` was **run 2's** result, five minutes old. Gate 3 finished
at 11:02:57 at status 0.

The waiter itself was never wrong: it only reports a gate when its `.done` marker appears, and those
had been deleted. It was the sideways read that bypassed the guard. The runner now deletes
`<name>.status` at the start of every run as well, so a stale reading is an absent file rather than a
plausible wrong number.

## 12. The direct criterion-2 witness

Independent of the harnesses, every program in `corpus/phase-5b.txt` run from a fresh empty working
directory with absolute paths, three descriptors captured separately, against the wrapped oracle and
against **both** crate engines, compared **raw** (no DEVIATION-0 normalisation):

```
scratchpad/task10/sweep-5b.sh          # oracle wrapper: ( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx PROG )
```
```
phase-5b.txt sweep: 63 of 63 agree on three descriptors against the oracle on BOTH engines; 0 differ
```

`sweep-mismatches.txt` is empty. This is a stricter comparison than `corpus.rs`'s, which normalises
trace indentation, and it needs no composition argument: it is oracle against `REXX_ENGINE=ir` and
oracle against `REXX_ENGINE=tree-walker`, byte for byte, in one pass.

## 13. The five gates

All five run from `rust/`, each status read unpiped from its own file, on the tree that is
commit A exactly.

| # | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`memcap` was present (`command -v memcap` -> `/home/moritz/.local/bin/memcap`), so gate 5 is the
`memcap` form and not the `ulimit -v` substitute. Gates 3 and 5 each report 105 `test result: ok`
lines and no `FAILED` line; gate 4 likewise.

**The tree did not move under the run.** A sha256 over every tracked and untracked non-ignored file's
bytes, taken before gate 1 and after gate 5:

```
git ls-files -co --exclude-standard -z | sort -z | xargs -0 sha256sum > tree-{before,after}.txt
diff tree-before.txt tree-after.txt
```
```
TREE IDENTICAL before first gate and after last
```

One docs edit was made while gates 3 to 5 were in flight and was **reverted before they finished**,
which is why the hashes match; that edit is deferred to commit B. Recorded because otherwise the
identity above would be an accident rather than a property.

### The verdict tables at the committed tree

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d \
    --no-fail-fast -- --nocapture
```
```
status 0
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, verdicts gated for 5a, 5b
gated by this run: 0 row(s) whose owning phase is closing or closed and whose verdict is not `agree`

table C -- 5a: 135 rows, 0 not yet `agree`;  5b: 6 rows, 0 not yet `agree`;  5c: 1347 rows, 868 not yet `agree`
table D -- 5a:  36 rows, 0 not yet `agree`;  5b: 2 rows, 0 not yet `agree`;  5c: 38 rows, 35 not yet
           `agree`;  7: 1 rows, 1 not yet `agree`;  deferred-parse-error-rendering: 2 rows, 2 not yet `agree`
```

The eight 5b rows, all `agree`: `objcla.rex`, `abscla.rex`, `usesem.rex`, `creo.rex`, `obdes.rex`,
`methodsbyclass.rex`, `attribute__delegate__subkeyword.rex`, `method__delegate__subkeyword.rex`.

**No structural failure**, and that is asserted rather than read off the report:
`assert_no_structural_failures` is called unconditionally in both tables, in every mode
(`gate_tables/mod.rs:311`-`:316`), so exit 0 is the claim. It is also where the two engines are
compared against each other on every probe -- a disagreement between them is structural, not a
verdict -- which is a second place the "both engines" half of the criterion is enforced.

The 5c and phase-7 rows are **reported and not gated**, which is what D65 says they must be: they are
5c's criterion and this flip must not turn them red.

### The phase-boundary lint, from an empty target directory

`rust/CLAUDE.md` requires this at a phase boundary, because a green clippy over a warm `target/` can
be a command that ran, exited 0, and never linted the code:

```
CARGO_TARGET_DIR=<scratchpad>/clean-target cargo clippy --workspace --all-targets -- -D warnings
```
```
status 0    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.23s
```

Checked rather than assumed that it was a real lint and not a cache hit: the fresh directory grew to
389 MB, and all ten workspace crates appear in its own `Checking`/`Compiling` stream --
`rexx-bench`, `rexx-classes`, `rexx-core`, `rexx-exec`, `rexx-extract`, `rexx-inventory`, `rexx-lib`,
`rexx-num`, `rexx-oracle`, `rexx-parse`. This is a sixth command, not a replacement for gate 2.

## 14. Commit A

```
git commit -F <message file>   # paths named explicitly; never git add -A
git log --format='%H %s' -1
```
```
a9bf7d02322e9258fb67ea9f6c778dd9570bbd47 Phase 5b Task 10: close 5b
```

Five paths, and `git diff --cached --stat` was re-read immediately before committing:

```
 docs/superpowers/plans/2026-08-27-phase-5b.md  | 37 +++++++++++++++++++++++---
 rust/bench-control/alloc4c-101.rex             | 10 +++----
 rust/bench-programs/alloc4c.rex                | 10 +++----
 rust/bench-programs/dispatchclass.rex          | 10 +++----
 rust/crates/rexx-exec/tests/gate_tables/mod.rs | 16 ++++++-----
 5 files changed, 58 insertions(+), 25 deletions(-)
```

`git diff --cached --name-only | /bin/grep -a Cargo.lock` matched nothing, so **`Cargo.lock` is
absent from the commit**. `git status --porcelain` after the commit is empty.

**The `interpreter/` citation audit ran and found nothing to audit**: `git diff -U0 | /bin/grep -a
"^+" | /bin/grep -a "interpreter/"` over the whole change matches no line. This task added no C++
citation, so Task 7's seven and Task 8's four have no analogue here.

## 15. The sitting

The counter pair schedules today, checked immediately before the run and again at its start:

```
perf stat -e instructions:u,cycles:u /bin/true
```
```
           146,573      instructions:u
           218,241      cycles:u
```

Both counted, no `<not counted>`, so there is nothing to escalate. The sitting's own progress stream
carries both instruments per axis, which is the same fact re-confirmed under load.

**The pin, verified before measuring, not after.**

```
sha256sum target/release/rexx-run bench-baselines/pinned/rexx-run-f558ea501
/bin/grep -a "857787f7" bench-baselines/PINNED.md
```
```
d73757e856773f42444abb1dd97f562fb9136c31afcad9b9b2507ed27e57b158  target/release/rexx-run
857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e  bench-baselines/pinned/rexx-run-f558ea501
| sha256 | `857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e` |
```

The retired 5a pin `rexx-run-15a1ffa98` (`141c3fa9...`) is still on disk and was **not** measured
against.

**The command**, one interleaved sitting rather than two suite runs:

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-f558ea501 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --axis dispatchclass --axis dispatch --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 10 --commit a9bf7d023 --baseline <scratchpad>/sitting.tsv
```
```
status 0, 432 rows
```

**`--baseline` names a scratchpad file, not the tracked one.** `rexx-arms` appends to whatever it is
given and has previously left a partial sitting in a tracked `.tsv`. The rows were appended only
after the run had finished and been checked:

```
diff <(head -1 sitting.tsv) <(head -1 bench-baselines/phase-5b-arms.tsv)   # HEADERS IDENTICAL
tail -n +2 sitting.tsv >> bench-baselines/phase-5b-arms.tsv
wc -l bench-baselines/phase-5b-arms.tsv                                    # 1521 -> 1953, delta 432
awk -F'\t' 'NR>1 {k=$1"|"$2"|"$3"|"$4"|"$5"|"$6"|"$7"|"$8; c[k]++; if(c[k]>1) d++} END {print d+0}' \
    bench-baselines/phase-5b-arms.tsv                                      # 0
git status --porcelain bench-baselines/                                    # only phase-5b-arms.tsv
```

Every row carries `task=10` and `commit=a9bf7d023`; nine axes at 52 rows each and `rexxcps` at 16
(one size); 216 rows on each instrument.

### `instructions:u`, head against the pin, `across_builds` medians

Cells are `(median - 1) * 100`, printed to three decimals with Python's round-half-even -- a
neighbouring third decimal against an earlier task's table is that rule, not a moved figure.

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| alloc4c | -0.013% | +0.345% | -0.026% | +0.334% |
| arith | -0.032% | +0.107% | -0.033% | +0.094% |
| compound | -0.059% | +0.331% | -0.062% | +0.332% |
| emptyloop | -0.215% | +0.010% | -0.223% | +0.007% |
| strings | -0.033% | +0.352% | -0.035% | +0.351% |
| varlookup | -0.110% | +0.696% | -0.112% | +0.700% |
| dispatchclass | +0.133% | +0.363% | +0.131% | +0.147% |
| **dispatch** | **+0.138%** | **+0.225%** | **+0.137%** | **+0.224%** |
| rexxcps | +0.021% | +0.164% | -- | -- |

**Nothing reaches the guard's 1% threshold**, so by the guard's own rule there is no finding. The
largest absolute move is +0.700% (`varlookup` large, `ir`) and -0.223% (`emptyloop` large, `tw`).

**`dispatch` is a comparison, and the +0.138%/+0.225% above is one.** It is a new *axis* in
`phase-5b-arms.tsv` and not a new *baseline*: the pin runs the program. Recording it as an
uncomparable first figure would have been the mistake this project's own rule warns about, applied
where it does not hold.

**One thing a per-axis threshold cannot see, counted rather than eyeballed.** Every cell of the `ir`
arm is positive -- nine axes out of nine -- while the `tw` arm is negative on six and positive on
three. Individually each is noise against a 1% gate; a consistent sign across every axis is a
different kind of statement, and it is the same shape as the drift `PINNED.md` records for 5a
(twenty steps under a half-percent summing to +32.58%). **Not re-litigated here**: Moritz ruled on
2026-09-01 that cumulative drift is handled by a non-SDD performance round after Phase 5. Recorded so
that round has the reading.

## 16. The `dispatchclass` decision, with the measurement that decides it

**`dispatchclass` stays as an axis beside `dispatch`. `alloc4c` stays.**

The structural argument is section 5. The measurement that makes it more than an assertion is the
**arm ratio**, taken in this sitting on the `head` build, `instructions:u`:

| axis | `ir/tw` small | `ir/tw` large |
|---|---|---|
| `dispatch` | 0.95140 | 0.95118 |
| `dispatchclass` | 0.99093 | 0.99029 |

The compiled engine buys about **4.9%** on an instance send and about **1.0%** on a class send. Two
axes that were one dimension measured twice could not read that far apart on the ratio *between the
engines*. Per pass they are much closer -- 6320.32 against 6890.49 instructions on `ir`, 6646.48
against 6958.41 on `tw` -- which is exactly why the per-pass figure is not the evidence and the ratio
is.

`alloc4c` stays because its header's retirement condition is `alloc.rex` going live, and `alloc.rex`
is still refused at `Array~of` (section 1). Both are recorded in the plan.

## 17. D65, criterion by criterion

| # | criterion | how it is met |
|---|---|---|
| 1 | every `5b` row in either gate table is `agree`, no structural failure | table C 6 rows / 0 not agree, table D 2 rows / 0 not agree; `assert_no_structural_failures` runs unconditionally in both and the run exits 0 |
| 2 | `corpus/phase-5b.txt` exists and every program agrees on three descriptors on both engines | 63 entries; the harnesses meet it by composition (section 10) and the direct raw sweep meets it in one pass, 63 of 63 (section 12) |
| 3 | D59a's licensed divergences have committed witnesses the harness runs | `licensed_divergences.rs`, exit 0 gated and ungated, and proved live by falsifying one recorded oracle byte (section 7) |
| 4 | `CLOSED_PHASES` gains `"5b"` and the gates pass with it there | `mod.rs:344`, and the two-arm control shows the constant is what produces the exit status (section 6) |
| 5 | the five gates pass, each figure quoted beside its command | section 13, plus the clean-target lint |

**All five hold at one commit.** `a9bf7d023` carries the flip and is the tree the gates in section 13
ran on, byte for byte. `ad26cb95a` adds the sitting rows and two plan paragraphs; nothing in
`crates/` reads either path -- the only files under `docs/` any test opens are
`phase-4-exclusions.txt` (`licensed_divergences.rs:152`, `builtin_status.rs:138`) and
`perf-baseline.md` (`rexx-bench-suite.rs:1453`) -- and the five gates were re-run on `ad26cb95a` so
the claim needs no argument at all.

## 18. What I did not do

* **I did not re-litigate the cumulative drift.** `PINNED.md` records +32.58% on the worst 5a axis
  across roughly twenty half-percent steps, ruled by Moritz on 2026-09-01 as work for a non-SDD
  performance round after Phase 5. Section 15 records that this sitting's `ir` arm is positive on all
  nine axes and leaves it there.
* **I did not attribute the `ir` arm's consistent sign to anything.** Nine positive cells is a
  reading, not a diagnosis; naming a cause would need a bisection across Tasks 6 to 9 that I did not
  run.
* **I did not touch any 5c or phase-7 row.** 868 of table C's 1347 5c rows and 35 of table D's 38 are
  not yet `agree`; they are 5c's criterion and D65 says explicitly they are reported, not gated.
* **I did not close `heapshape.rex` or `alloc.rex`.** Both stay parked and neither is 5b's; measured
  refusals are in section 1.
* **I did not rebuild the pin or take a second sitting.** One sitting, five rounds, interleaved.
* **I did not run the `15a1ffa98` arm** that `PINNED.md` says would separate the `rustc` 1.97.1 to
  1.98.0 change from the code in 5a's drift table. It is named there as work for the performance
  round and is not this task's.
* **I added no `unsafe`** and found no site that wanted it.
* **I added no `interpreter/` citation**, so the audit Task 7 and Task 8 owed had nothing to check
  here; the command that establishes that is in section 14.
* **I did not correct `alloc4c.rex`'s remaining prose** beyond the false refusal quotation -- its
  retention and size-class paragraphs were not re-measured and are left as they stand.

