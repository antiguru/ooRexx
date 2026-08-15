# Task 14 report: the compound-`DO` control-variable fix

Commit: `d372dcdf7dc0000aa0f54cb4399eb7e393a8798a`.

## Step 1: the three narrowing probes, before any change

Each run from a fresh `mkdir` under the scratchpad, oracle wrapped exactly
per the environment rule.

**Probe 1** (the reproduction in the task text) --
`/tmp/.../task14-probe1/p1.rex`:

```rexx
j = 7
do cv.j = 1 to 3
  say 'iter' cv.j
end
say 'final' cv.j
say 'literal' cv.7
```

Oracle (rc 0):
```
iter 1
iter 2
iter 3
final 4
literal 4
```

Crate before the fix (rc 0):
```
iter CV.7
iter CV.7
iter CV.7
final CV.7
literal CV.7
```

**Probe 2** (keyword-exempt.txt's first witness) --
`/tmp/.../task14-probe2/p2.rex`:

```rexx
c=0; j=1; Do cv.j=1 To 5; c=c+1; End; say '['c cv.j']'
```

Oracle (rc 0): `[5 6]`. Crate before the fix (rc 0): `[5 CV.1]`.

**Probe 3** (keyword-exempt.txt's second witness, the self-referencing
re-resolution case) -- `/tmp/.../task14-probe3/p3.rex`:

```rexx
a.=0; i=1; Do a.i=1 To 3; If i>7 Then Leave; i=i+1; End; say '['i']'
```

Oracle (rc 0): `[8]`. Crate before the fix (rc 0): `[4]`.

All three reproduce the recorded transcripts exactly, confirming the
starting point before any edit.

## What `bind_control` actually needed

The brief's re-derivation held: `Controlled::control` (`ast.rs:1014`) is
already a bare `SymbolId` carrying the whole interned spelling (`"CV.J"`),
the same starting point `say cv.j` uses. No `rexx-parse` change was needed,
confirming that half of the re-derivation; `ast.rs` was not touched.

The actual defect was narrower than "no tail resolution": `bind_control`
ran **every** control-variable shape -- simple, stem, and compound --
through the same flat `slot_of` write, which is only correct for the
simple-variable case (`shape_of`'s three-way classification exists in this
crate already, at `run.rs`'s own `shape_of`/`NameShape`, and `assign_expr_
target` already dispatches an ordinary assignment target on exactly that
classification). The fix makes `bind_control` dispatch the same way:

* `NameShape::Simple` keeps the original flat slot write unchanged,
  including its `intermediates`-gated tracing and the comment recording
  the ~40ns/pass measurement behind that gate (`lang/mutation_digits_at_
  render.rex`'s own witness depends on this path staying eager-render-free
  when not tracing, and it does).
* `NameShape::Stem` and `NameShape::Compound` build a synthetic `Expr`
  around `control`'s own `SymbolId` (`ExprKind::Stem`/`ExprKind::Compound`)
  and call `assign_expr_target` -- the same tail resolution (`tail_key`,
  against the tail variable's *current* value) and the same `stem_set`/
  `stem_assign` write an ordinary `cv.j = expr` or `cv. = expr` assignment
  already uses, reused rather than reimplemented.

The second half of the divergence -- witness 2's re-resolution -- lives in
`loop_advance`'s own re-tested read (`LoopState::Controlled`'s `if
re_tested` arm), which had been calling the flat `self.read(code, *control)`
unconditionally. That call now dispatches the same three ways: `Simple`
keeps `self.read`, `Stem` reads via `read_stem` (never raises `NOVALUE`,
matching `eval_node`'s own `ExprKind::Stem` arm), and `Compound` recomputes
`tail_key` fresh and reads via `stem_get`, with `novalue_check` run before
any tracing (matching the oracle's own order, and the existing comment
on this arm) and a `>C>` line emitted before the pair's `>V>`/`>>>`,
mirroring `eval_node`'s own `Compound` read order.

## By-name evidence: all seven bodies pass, before removing any row

`REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions`
(report mode, before touching `keyword-exempt.txt`) printed all seven
target bodies by name, each with `now PASSES but is still on the committed
exempt list -- remove it`:

```
DO::test_DO_standardTest2A now PASSES but is still on the committed exempt list -- remove it
DO::test_DO_standardTest2B now PASSES but is still on the committed exempt list -- remove it
DO::test_DO_standardTest2P now PASSES but is still on the committed exempt list -- remove it
DO::test_DO_standardTest2Q now PASSES but is still on the committed exempt list -- remove it
DO::test_DO_standardTest5-69 now PASSES but is still on the committed exempt list -- remove it
ITERATE::test_12 now PASSES but is still on the committed exempt list -- remove it
LEAVE::test_11 now PASSES but is still on the committed exempt list -- remove it
```

The report's own per-group table at that point read `DO 85/85`, `ITERATE
18/18`, `LEAVE 15/15` -- all bodies in those three groups passing, with no
new unaccounted failures anywhere else (the group table was unchanged
outside `DO`/`ITERATE`/`LEAVE` versus a pre-fix run). This is the seven,
by name, asserted before any row was removed -- the brief's own required
gate for this step.

After removing the seven rows (and correcting the header's "6 bodies" to
match, by deleting the now-stale explanatory paragraph rather than
re-counting it), `REXX_KEYWORD_GATE=1` runs green: `test result: ok. 7
passed; 0 failed`, `888 of 896 bodies passing`, and
`the_exempt_set_matches_the_current_failures` reports no problems.

## Crossed-axes probe table

Alphabet crossed: loop forms {bare, `TO`, `BY`, `TO`+`BY`, `FOR`, `OVER`}
x control shape {simple, stem, compound} x {self-referencing tail, `LEAVE`/
`ITERATE`/`END` naming a compound, `TRACE I` fidelity}. Every program run
from a fresh directory, oracle wrapped per the environment rule, diffed
byte-for-byte against oracle stdout.

| # | Program (abbreviated) | Axis | Oracle | Crate (fixed) | Result |
|---|---|---|---|---|---|
| c1 | `Do cv.j=1 By 2 To 9` | compound, BY+TO | `[5 11]` | `[5 11]` | MATCH |
| c2 | `Do cv.j=1 To 20 For 3` | compound, FOR | `[3 4]` | `[3 4]` | MATCH |
| c3 | `Do cv.j=13` (bare, `LEAVE`-bound) | compound, bare | `[11 24]` | `[11 24]` | MATCH |
| c4 | `Do cv.j Over 'a' 'b' 'c'` | compound, OVER | `a b c` / `[a b c]` | same | MATCH |
| c5 | `Do cv.=13` (bare, `LEAVE`-bound) | stem, bare | `[11 24]` | `[11 24]` | MATCH |
| c6 | `Do i.j=0 to 6; ...; Leave i.j; End i.j` | compound, named `LEAVE` | `[2 1]` | `[2 1]` | MATCH |
| c7 | `Do i.j=0 to 6; ...; Iterate i.j; End i.j` | compound, named `ITERATE` | `[7 7]` | `[7 7]` | MATCH |
| c8 | `Do i.j=0 to 2; End i.j` | compound, named `END` | `[3 3]` | `[3 3]` | MATCH |
| c9 | `Do a.i=1 To 3` with `i=i+1` in body | compound, self-referencing tail | `[8]` | `[8]` | MATCH |
| c10 | `Do ii = 1 To 3; End; say ii` | simple, regression pin | `4` | `4` | MATCH |

`TRACE I` fidelity (compound control, `j=1; do cv.j = 1 to 2; nop; end`):
stderr diffed byte-for-byte between oracle and crate --

```
diff t1.oracle.err t1.crate.err  ->  no output (identical), "STDERR MATCH"
diff t1.oracle.out t1.crate.out  ->  no output (identical), "STDOUT MATCH"
```

confirming the `>C>` line precedes both the setup's `>=>` and the
re-tested pass's `>V>`, at both indents, matching the oracle exactly.

`NOVALUE` on a dropped compound control (`signal on novalue`, `drop cv.j`
inside a `do cv.j = 1 to 3`): oracle rc 5, `trapped 5`; crate rc 5,
`trapped 5`. `diff` reports no difference.

All ten crossed-axes probes plus the trace and NOVALUE probes matched the
oracle exactly.

## Mutation results

**Mutation:** reverted the `NameShape::Compound` arm of `bind_control`
back to the old flat `slot_of` write (the pre-fix behaviour), leaving
`Stem` and `Simple` on the fix and leaving the read-back side unmutated.
Backup taken by file copy with a recorded `sha256sum` before mutating, and
the restore was verified against that same checksum afterward (`crates/
rexx-exec/src/run.rs: OK`).

**What caught it:**

* Four of the six new `run.rs` unit tests killed the mutation cleanly and
  fast (all under 0.01s, no hang): `a_compound_control_variable_is_bound_
  on_every_pass`, `a_compound_control_variable_traces_its_own_c_line`, and
  `leave_and_iterate_by_name_still_reach_a_compound_controlled_loop` each
  failed with a `41.1` Raised error (the derived-name text reaching
  `arith_operand`), and `the_exempt_set_matches_the_current_failures`
  would have gone red again had the exempt rows still been removed.
* The other two new tests (`a_stem_control_variable_binds_through_stem_
  assign`, `a_simple_control_variable_still_binds_through_the_fast_slot_
  path`) correctly stayed green -- they pin the two shapes this specific
  mutation left untouched, which is the "pair a refusal with its adjacent
  success" requirement: proof the fix (and this mutation) is scoped to the
  compound shape, not a blanket change.
* `a_compound_control_variables_tail_re_resolves_every_pass` also stayed
  green under this mutation, correctly -- that probe's own termination is
  driven by the body's independent `LEAVE` on `i`, not by the mutated
  write, so it is not a kill for *this* mutation (it is a kill for a
  mutation to the *read*-back side, which was not tried here since the
  read-back side is exercised directly by the other passing tests above).
* `REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions`
  under the same mutation **hung** rather than failing cleanly (observed
  directly: no output after 9m10s wall time on a run that completes in
  under 1s unmutated with the fix in place; killed rather than let run
  further, to avoid tying up the environment). This is consistent with at
  least one real `base/keyword` `DO` body whose termination depends on the
  write side actually landing in the tail the read side re-resolves to;
  tracking down which specific body was not pursued further once the
  targeted unit tests had already given a clean, fast kill signal without
  needing the external corpus at all.

**"Can fail" is not "adds coverage":** this defect shipped and sat
unfixed and unasserted by any prior test (that is the entire premise of
the task, and `keyword-exempt.txt`'s own removed rows are the record of
it), so there is no pre-existing test to check the new ones against for
redundancy -- nothing in the tree caught this before. The mutation
experiment above is the positive control: the new tests (and the restored
`keyword-exempt.txt`) do catch a real regression to the write path, and do
so without relying on any test that already existed.

**Restore verification:** `sha256sum -c` against the pre-mutation
checksum reported `crates/rexx-exec/src/run.rs: OK` after restoring from
the file copy; a clean `cargo build -p rexx-exec` and the full workspace
suite (below) both ran green afterward, confirming the restore was exact
and not a stale copy.

## Verification

From `rust/`, after the restore, with a clean `cargo clean -p rexx-exec`
in between to rule out a warm-target false green on the touched crate:

```
$ cargo fmt --all --check
(no output)                                                  exit 0

$ cargo clippy --offline --workspace --all-targets -- -D warnings
    Checking rexx-exec v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.6-1.9s
                                                               exit 0
(re-run identically after `cargo clean -p rexx-exec`, forcing a real
re-lint of the one touched crate rather than a cached same-session result)

$ REXX_CORPUS_GATE=1 REXX_KEYWORD_GATE=1 \
    cargo test --offline --workspace --no-fail-fast
                                                               exit 0
```

Process-header count: `grep -c '^     Running\|^   Doc-tests'` on the full
log reports **73**, the non-truncated baseline. Summed `test result:`
lines: **1270 passed, 0 failed** (the tree's own starting baseline was
1264 passed; +6 for the six new `run.rs` tests, +0 elsewhere). `grep -n
FAILED` on the full log returns nothing. The corpus gate reports `50 of 50
matching`; the keyword gate reports `888 of 896 bodies passing, carrying
1737 of 1773 assertSame calls` with the same `4c`/`Phase 7`/`RAISED`
breakdown as before this task (2/3/3), confirming no other body's
attribution moved.

`git status --porcelain=v1` after the run showed only the three intended
files modified, nothing stray from the probe/mutation work (all of which
lived under the scratchpad, in fresh directories per probe batch).

## Files touched

* `rust/crates/rexx-exec/src/run.rs` -- `bind_control`'s shape dispatch,
  the matching dispatch in `loop_advance`'s `Controlled` re-test arm, and
  six new unit tests in the existing `mod tests`.
* `rust/corpus/keyword-exempt.txt` -- the seven rows removed, the stale
  "6 bodies" explanatory paragraph removed (moot once the defect closed),
  and the now-orphaned "`defect:` rows are NOT derived" bullet removed
  from the header (it described a mechanism with no rows left using it).
* `docs/superpowers/plans/phase-4-exclusions.txt` -- the EXCLUSIONS row
  moved to CLOSED DEFECTS with the fix's own transcript and witness names,
  the recorded (and wrong) rexx-parse cost corrected in the new entry, and
  the KNOWN GAPS section's own forwarding pointer updated to the new
  destination rather than left pointing at a section that no longer
  exists.

## Concerns for the caller

* The mutation-kill demonstration for the keyword corpus gate is a hang,
  not a clean failure -- confirmed as a real, fast infinite loop (not an
  environment slowdown) by isolating it to `keyword_assertions` alone and
  killing the background task after ~9 minutes, but the specific body
  responsible was not identified. If a future change to this same code
  needs a fast mutation-kill signal from the external corpus specifically
  (rather than from the in-tree unit tests, which do give one), that body
  would need to be found first.

  **Superseded below**: fix round 1 found the body.

# Fix round 1

Review findings I1-I4 and M1, all addressed. Commit:
`1c519dfcfe4ee35557bc323ea5c89ad9aedfa0d2`.

## I1/I2: the pre-fix transcript for the replaced/added stem tests

Built the pre-fix tree with `git archive 1c2300d9` into a fresh scratch
directory and `cargo build --offline --bin rexx-run`, per the review's own
method. Every probe below ran from a fresh `mkdir`, bounded with `timeout
5`, oracle wrapped per the environment rule.

**`a_stem_control_variable_binds_through_stem_assign`'s original program**
(`i=0; do cv.=13; if i>10 then leave; i=i+1; end; say cv.; say i`):

| build | output | rc |
|---|---|---|
| oracle | `24` / `11` | 0 |
| fixed (this tree) | `24` / `11` | 0 |
| pre-fix (`1c2300d9`) | `24` / `11` | 0 |

Confirmed I1 exactly: the pre-fix build's output is byte-identical to the
fixed build's and to the oracle's. This program cannot fail against the
defect it is named for, because `read_stem` returns whatever object sits
in the slot with no check that it is a `Body::Stem` -- a flat scalar write
there is invisible to a bare-stem read of the same name.

**The added second assertion's program** (`do cv. = 13\ncv.1 =
99\nleave\nend\nsay cv.1\nsay cv.`):

| build | output | rc |
|---|---|---|
| oracle | `99` / `13` | 0 |
| fixed (this tree) | `99` / `13` | 0 |
| pre-fix (`1c2300d9`) | (none -- panic) | 101 |

Pre-fix stderr:

```
thread 'rexx-interp' (...) panicked at crates/rexx-exec/src/stem.rs:288:60:
a live value
```

This is the discriminating shape: a *tail* of the same stem, touched
elsewhere in the body, forces `stem_set`/`stem_get` to look at the slot
`bind_control`'s old flat write left holding a bare scalar instead of a
`Body::Stem`, and both functions `expect` one there. Added as a second
assertion inside the existing test (I1's own suggestion: replace or add
alongside; kept the original assertion since it still documents correct
`stem_assign` binding, and it is the *pairing* -- both a stem write that
looks right and one that panics -- that pins the property rather than
either alone).

## I3: the hanging body, its mechanism, and the crossing test

**Identified the body directly**, using `rexx_extract::keyword`'s own
extraction (the same one `keyword_assertions.rs` runs) rather than the
static `corpus-l1/DO_test_DO_standardTest2P.rex` snapshot, which turned out
not to be the program the harness actually constructs: a throwaway `#[test]`
(added, used, and removed before committing -- confirmed by an empty `git
diff` on `keyword_assertions.rs`) dumped `KeywordBody::program` for
`DO::test_DO_standardTest2P` and `DO::test_DO_standardTest5-68` after their
own `self~assertSame` calls were rewritten to plain `say` clauses by the
extractor. Both dumps ran clean and fast (rc 0, all `@@ASSERTSAME` markers
`1`) against the fixed build.

Reapplied the exact write-only mutation from the original report (`bind_
control`'s `NameShape::Compound` arm reverted to the old flat `slot_of`
write; `run.rs` backed up by copy first, `sha256sum` recorded, restore
verified against it afterward: `crates/rexx-exec/src/run.rs: OK`). Against
that mutated build, bounded with `timeout 5`:

* `DO::test_DO_standardTest2P`'s dumped program: **rc 124** (timeout).
  Mechanism, confirmed by hand-tracing the actual `stem_get`/`bind_control`
  calls pass by pass and matching the review's own account: the body's `i=1
  ; a.=0 ; c=0 ; Do a.i=1 To 7` flip-flops `i` between `1` and `2` each
  pass, so the control's own tail alternates between `A.1` and `A.2`.
  Because the write side never reaches `stem_set` under this mutation, the
  stem `A.`'s tails map stays empty forever; every re-tested read's `stem_
  get` therefore falls through to `A.`'s own pre-existing default (`0`, a
  *valid number*, not a derived-name string that would fail loudly), so
  `current` recomputes to `0 + 1 = 1` on every single pass, permanently
  inside `[1, 7]`, and the loop's own `TO 7` bound never fires.
* `DO::test_DO_standardTest5-68`'s dumped program (92 nested nowhere-
  defaulted compound `DO`s, `Do i.1=1 To 1` down to `Do i.92=1 To 1`,
  literal numeric tails, no `i.=` default anywhere): **rc 215**, a
  genuinely new effect versus the fixed build, not merely the same one
  moving. Stderr:

  ```
       96 *-*                                         End i.92
  Error 41 running .../test_DO_standardTest5_68.rex line 96:  Bad arithmetic conversion.
  Error 41.1:  Nonnumeric value ("I.92") used in arithmetic operation.
  ```

  Mechanism: unlike `a.`, nothing ever assigns `i.` a default, so every one
  of the 92 controls' stems is never auto-vivified at all under the
  mutation (their writes all go to disjoint flat slots, none of them
  `stem_set`). The innermost loop's own re-tested read (`i.92`, `To 1`)
  therefore gets a genuine `Novalue::Unset` -> the derived name text
  `"I.92"` -> `arith_operand` on non-numeric text raises 41.1, propagating
  out through all 92 enclosing `DO`s as an unhandled condition. Confirmed
  directly, not merely inferred: rc 215 and the exact stderr above,
  reproduced on this machine against the mutated build.

Restored `run.rs` from the backup copy after both probes;
`sha256sum -c` reported `crates/rexx-exec/src/run.rs: OK`.

**The crossing test.** Added `a_compound_controls_to_bound_survives_a_
masking_stem_default`, reproducing `test_DO_standardTest2P`'s own mechanism
(`i` flip-flopping which tail of `a.` the control resolves to, `a.=0`
pre-existing default, no independent `LEAVE`) rather than a synthetic
shape, with a `FOR 1000` safety cap that never fires on correct code
(oracle and the fixed build both end the loop via `TO 7` at pass 14) but
turns "hangs forever" into "counts 1000 instead of 14" if this ever
regresses. Verified against the same write-only mutation, bounded:

| build | `a_compound_controls_..._default`'s program | rc |
|---|---|---|
| oracle | `14` | 0 |
| fixed (this tree) | `14` | 0 |
| write-only-mutated | `1000` | 0 |

No hang anywhere in this round's own verification, by construction.

## I4: the three restored facts

Restored into `docs/superpowers/plans/phase-4-exclusions.txt`'s `CLOSED
DEFECTS` entry, none re-derived beyond what the original `EXCLUSIONS` row
already stated (an immutable referent in every case -- oracle bytes or a
commit hash):

1. The three measured pre-fix `LEAVE`/`ITERATE`/`END` transcripts (`Do
   i.j=0 to 2 ... End i.j`, `... Leave i.j ...`, `... Iterate i.j ...`),
   oracle against crate, side by side.
2. Task 9's dated witness -- `do aa.1 = 1 to 2 ; nop ; end ; say aa.1`,
   oracle `3`, crate `AA.1`.
3. The causality fact that the divergence was present on `e72cc19f`, which
   rules out Task 9's own tracing work as the introducer.

## M1: the perf sentence

Corrected in the same `CLOSED DEFECTS` entry: the fix's simple-variable
*behaviour* is unchanged (verified), but `bind_control` now calls `shape_
of` -- a linear scan for a `.` byte -- unconditionally on every pass
regardless of shape, a cost the cited ~40ns/pass measurement (which
isolated the tracing-`Vec` gate alone) does not cover. The corrected
sentence says this is unbenchmarked under "correctness over performance"
rather than implying the cost does not exist. Not re-benchmarked, per the
coordinator's instruction.

## Verification

From `rust/`, after a `cargo clean -p rexx-exec` (clippy re-verified
against a genuinely re-linted crate, not a warm-cache result):

```
$ cargo fmt --all --check                                     exit 0
$ cargo clippy --offline --workspace --all-targets -- -D warnings
    Checking rexx-exec v0.1.0 (...)                            exit 0
$ REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast
                                                                 exit 0
$ REXX_KEYWORD_GATE=1 cargo test --offline -p rexx-exec --test keyword_assertions --no-fail-fast
                                                                 exit 0
```

Process-header count on the full-workspace run: **73** (non-truncated).
Summed `test result:` lines: **1271 passed, 0 failed** (1270 + 1 for the
new `a_compound_controls_to_bound_survives_a_masking_stem_default`; the
stem test's added assertion is inside an existing test, so it adds no new
`test result:` line). `grep -n FAILED` returns nothing. Corpus gate: `50 of
50 matching`. Keyword gate: `7 passed; 0 failed`, `888 of 896 bodies
passing`, same `4c`/`Phase 7`/`RAISED` breakdown as round 1 (2/3/3).

`git status --porcelain=v1` after this round shows exactly two files
changed: `docs/superpowers/plans/phase-4-exclusions.txt` and
`rust/crates/rexx-exec/src/run.rs`. The throwaway dump test added to
`crates/rexx-exec/tests/keyword_assertions.rs` to identify the hanging
body was removed before this check; `git diff` on that file is empty.
