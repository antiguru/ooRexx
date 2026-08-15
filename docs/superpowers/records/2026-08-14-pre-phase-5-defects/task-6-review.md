# Task 6 review — `1f4176b47..01f8010b7`

**Verdict: request changes.** The behavioural fix is right and well witnessed for the three
boundary changes; I re-measured every shape the brief named and the two defects it set out to
close are closed on both engines. But the commit turns one previously-agreeing program into a
divergence (a plain `DO` with a body inside an `INTERPRET`), and it leaves three passages of
surviving prose false — one of which asserts a re-measurement that no longer holds. That is this
plan's own named failure mode, on the same page as the fix.

All commands below were run from `rust/` unless stated. Oracle invocations were wrapped as the
brief specifies, from a fresh empty directory, with stdout/stderr/rc read as three descriptors.
Where a pre/post comparison is claimed I built `1f4176b47` in a throwaway worktree with its own
target directory and ran the two binaries against each other and against the oracle.

---

## Critical

### C1. A plain `DO` with a body inside `INTERPRET` matched before this commit and diverges now — both engines, stdout, `rc 0`

`rust/crates/rexx-exec/src/run.rs:6117-6141` (the new header clause in the `Simple` arm).

Program:

```rexx
call on user c1 name h1
call on user c2 name h2
zr = raiser()
interpret 'do; say ''body''; end'
say 'after' zr
exit 0
raiser:
raise user c1 return 5
h1:
say 'h1' sigl
raise user c2 return 1
h2:
say 'h2' sigl
return
```

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 4` / `after 5` |
| `1f4176b47`, both engines | `h1 3` / `body` / `h2 4` / `after 5` — **matched** |
| `01f8010b7`, both engines | `h1 3` / `h2 4` / `body` / `after 5` — **diverges** |

The oracle runs an `INTERPRET` fragment in an activation whose condition queue is separate, so a
condition pending when the `INTERPRET` clause runs is not offered any boundary *inside* the
fragment — it waits for the `INTERPRET` clause's own. `clause.rs`'s module doc already states
exactly that rule and cites the measurement behind it. The new header clause is a boundary inside
the fragment, and it takes the delivery.

The *mechanism* is pre-existing — I measured two siblings that already diverged at `1f4176b47`
and are unchanged by this commit: `interpret 'if 1 = 1 then say ''body'''` and
`interpret 'do zi = 1 to 1; say ''body''; end'` both produce the same early delivery before and
after. So this commit does not invent the family; it adds the plain `DO` to it, and that is the
part the record has to answer for, because the OUTCOME's sweep claim (plan line 473) covers only
`corpus/` and `bench-programs/` and cannot see it.

Two ways out, and the choice is the author's:

* **Close it.** Give the new header and `END` clauses no boundary when a fragment's
  `clause_line_override` is in force. That needs its own measurement rather than a guess: the
  empty-fragment shape `interpret 'do; end'` under a double requeue diverges *identically* before
  and after this commit, so a suppression has to be measured against that program too, not only
  against this one.
* **Record it**, in `ir_dual_cases/loop-header-boundaries` alongside the other non-row
  divergences, and correct the sweep sentence — but write it as caused, not as found. The plan's
  "do not fix any other divergence" rule is about divergences the task *encounters*; this one it
  creates.

---

## Important

### I1. The zero-pass bullet in the case file is now false, and the sentence under it claims a re-measurement that no longer holds

`rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries:44-53` and `:62-67`.

The bullet records `do i = 1 to raiser()` with `raiser` returning 0 and a requeueing handler:
"the oracle prints `after` and then `G ran 6` ... and this crate prints `G ran 3` and then
`after`". Reconstructed from the bullet's own numbers (`do` on line 3, `g:` reachable, `say
'after'` on line 6) that program reproduces the recorded transcripts *exactly* at `1f4176b47`:

```
oracle                     after / G ran 6
1f4176b47, both engines    G ran 3 / after
01f8010b7, both engines    after / G ran 6      <- matches the oracle
```

So the divergence this bullet records was closed by this commit — it was the step boundary, and
the third change removed it. Line 62-67 then says of that same bullet: "Re-measured, and both
engines still print what that bullet records." That is false as of `e74780054`.

The whole framing at `:39-42` ("LOOP-HEADER BOUNDARIES THAT DIVERGE FROM THE ORACLE ARE NOT ROWS
HERE ... Each was measured on both engines, and the engines agree on each") no longer holds for
its first bullet either. Fix by **deleting** the bullet and its paragraph and promoting the
program to a row, rather than by rewriting the claim in place — this plan's own record is that
rewrites are where the next false statement lands.

### I2. The "runs the other way" paragraph in the same file is false in both halves

`rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries:89-101`.

```
#   oracle       after unset / G ran 7
#   this crate   G ran 6 / after set
```

Both lines are wrong now. The plan itself corrected the oracle half in this very diff — "CORRECTED:
this line read `after unset` when it was written, and the oracle prints `after set`" — and the
case file, which is the other place the same transcript lives, kept the uncorrected wording. The
crate half is closed: the new `DO UNTIL` stanza fifty lines below in the same file records
`H ran 6` / `after set` / `G ran 7`, and I confirmed the corpus program's F block prints the same
against the live oracle. The surrounding sentence "Every other shape recorded in this file has
this crate delivering *later* than the oracle. This one delivers *earlier*, on both engines" is
false about a shape the file now pins as agreeing, two hundred lines apart.

### I3. "Repeating loops with a real header were never wrong on this route" is false

`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md:464`.

Measured at `1f4176b47` against the oracle, both engines, the same `raise ... return` route, one
requeue in the loop body:

| program | oracle | `1f4176b47` |
|---|---|---|
| `do zi = 1 to 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | `h1 5` `h2 6` `h3 6` `after 5` |
| `do 1 / zr = raiser() / end / ...` | same | same divergence |
| `do while zn < 1 / ... / zr = raiser() / end / ...` | `h1 7` `h2 8` `after 5` `h3 9` | `h1 7` `h2 8` `h3 8` `after 5` |
| `do forever / zr = raiser() / leave / end / ...` | `h1 5` `h2 6` `after 5` `h3 8` | `h1 5` `h2 6` `h3 6` `after 5` |

All four now agree at `01f8010b7`. So three of the four shapes the bullet names by name
(`do zi = 1 to 2`, `do while`, `do 2`) *were* wrong on this route, and the bullet's stated reason
— that `run_repeating` "drains the queue there" — is the wrong mechanism: `run_repeating` drains
the queue the body left at the *next* header clause, and the requeue that handler leaves behind
was still being taken by the step boundary. The third change is what closed them.

The fourth name is wrong in a second way. `do label zl` with no header is `LoopKind::Simple`
(`crates/rexx-parse/src/instruction.rs:960-975`) and never reaches `run_repeating` at all; it
diverged before this commit and agrees after, exactly as the plan's own earlier text says
("`do label zl` behaves the same") and as the new corpus program's block K comment says. Measured:

```
oracle                     h1 3 / h2 4 / body / after 5
1f4176b47, both engines    h1 3 / body / h2 5 / after 5
01f8010b7, both engines    h1 3 / h2 4 / body / after 5
```

This bullet understates what the fix closed and misattributes the cause; it is the kind of
sentence that will be cited later as "repeating loops were always fine". Delete it or replace it
with the measured statement: every `LoopKind` was wrong on the trailing delivery, and the step
boundary was the single cause.

### I4. Both new `settle_block_indent` calls are unwitnessed — flipping either leaves the whole `rexx-exec` suite green

`rust/crates/rexx-exec/src/run.rs:6137` and `rust/crates/rexx-exec/src/run.rs:6206`.

The task's constraint is "a test that cannot fail is a defect; every fix needs a witness", and the
OUTCOME is careful to say only that the *boundaries* were mutated. But the two `settle_block_indent`
arguments are part of the fix and are observable:

* `run.rs:6137`, `settle_block_indent(true, do_indent)` → `false`: builds clean; `corpus`,
  `trace_oracle`, `trace_indent` and all of `ir_dual` stay green. Against the oracle it is a live
  divergence — on `trace r` with a handler delivered at a plain `DO`'s header clause, the
  handler's activation loses two columns (`15 *-*     h2:` becomes `15 *-*   h2:`).
* `run.rs:6206`, `settle_block_indent(false, do_indent)` → `true`: same, green everywhere,
  two columns gained on the handler delivered at `END`'s clause.

Both arguments are *correct* — I verified three `trace r` programs (plain `DO` with a body, empty
`DO` with a double requeue, nested plain `DO`s) match the oracle byte for byte on all three
descriptors and on both engines, and that the pre-fix binary got the indent wrong on two of them.
Nothing in the suite pins that. The corpus program deliberately carries no `TRACE`, which is the
right call for its own subject, so the witness has to be a `trace r` stanza — an
`ir_dual_cases` row or a `trace_oracle` case — not a change to
`corpus/lang/do_clause_boundaries.rex`.

---

## Minor

### N1. Three comments name `run_loop` as the function that opens the header and `END` clauses; it opens neither

`rust/crates/rexx-exec/src/run.rs:5146`, `rust/crates/rexx-exec/src/clause.rs:523`, and
`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md:457`.

`run_loop` (`run.rs:5923`) evaluates the header plan and delegates. The `Simple` arm that opens
both clauses is in `run_loop_with_header` (`run.rs:6111`); a repeating loop's are in
`run_repeating`. And the compiled engine never calls `run_loop` at all — `Op::LoopRun` enters
`run_loop_with_header` directly (`ir/drive.rs:1337`), which is the whole reason the suppression at
`run.rs:5149` had to go in `leave_stepped_clause` to reach both engines. A reader who follows the
pointer lands in a function with no `in_clause` in it.

While there: `clause.rs:519-524` says the header's and `END`'s boundaries are "both of which
`run_loop` opens as clauses in their own right". For a repeating loop `END` has no clause of its
own — the next header re-test carries `END`'s line (`HeaderClause::End`, `run.rs:6443`). True
of `Simple`, which is what the paragraph is about, but the sentence generalises to `DO`/`LOOP`.

### N2. A divergence found while probing, to write down and leave: two conditions pending at one boundary lose one

Pre-existing, unchanged by this commit, and not in the OUTCOME's found-and-not-fixed list.

```rexx
call on user c1 name h1
call on user c2 name h2
call on user c3 name h3
do zi = 1 to 2
  zr = raiser()
end
say 'after' zr
```
(with `h1`/`h2` each `raise ... return`, `h3` plain)

```
oracle                     h1 5 / h2 6 / h3 5 / h1 5 / h2 6 / after 5 / h3 7
01f8010b7, both engines    h1 5 / h2 6 /         h1 5 / h2 6 / after 5 / h3 7
1f4176b47, both engines    h1 5 / h2 6 /         h1 5 / h2 6 / h3 6 / after 5
```

One delivery is lost outright. The oracle's boundary **drains**: on the second pass the clause at
line 5 delivers `h3` (left over from the previous pass's `END`) and then `h1` (queued by that same
clause), both reporting `SIGL 5`, before `END`'s boundary takes the next. Only what a handler
queues *during* delivery is deferred. This crate's `pending_trap` is a single `Option`
(`lib.rs:1683`), so the second condition overwrites the first.

`clause.rs:455-459` describes the one-at-a-time rule as matching the oracle — it does for a
condition queued during a delivery, which is what that comment measured, but not for two queued
before one boundary.

A second shape, same status (pre-existing, byte-identical before and after this commit, both
engines): a requeue delivered across an `ITERATE`. With `do while zn < 2` on line 5,
`zn = zn + 1` on 6, `zr = raiser()` on 7, `iterate` on 8, `end` on 9 and `say 'after' zr` on 10,
the oracle defers the second requeue past the re-test and delivers it at the *next pass's* first
body clause (`h3 6`), then leaves the last one for the `say` (`h3 10`); this crate delivers both
at the `ITERATE`'s own line (`h3 8`, twice) and prints the last one before `after`.

**This also contradicts a sentence that landed one commit later, outside this diff.**
`docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md:9-10` records as a known
ANSI-vs-ooRexx disagreement that "8.2.4 drains a boundary where **ooRexx and this crate** deliver
one condition and do not re-check". The transcript above says ooRexx drains too, and only this
crate does not — so the divergence is against ooRexx, not a licensed deviation from ANSI. Phase 7
would design against that premise. Worth correcting there as well as recording here.

---

## What I re-measured and found sound

* Both defects are closed on both engines. `corpus/lang/do_clause_boundaries.rex` matches the live
  oracle byte for byte on stdout, stderr and `rc`, on the tree-walker and the IR; every block
  comment in it (A–K) reads back correctly against the transcript, including the E block's
  three-way split and the F block's inverted order.
* Escape paths are clean. `LEAVE` out of a labelled block, `SIGNAL` out of a block, `RETURN` from a
  block inside an internal routine, a routine falling off its end inside a block, an error
  unwinding out of a block, a handler that `EXIT`s at the new header clause and at the new `END`
  clause (`rc 7` and `rc 9` matched), a block as an activation's last construct, nested blocks and
  a block inside a `SELECT`'s `WHEN` all match the oracle on all three descriptors, both engines.
  `ITERATE` is byte-identical before and after this commit on both the agreeing and the diverging
  shape I found (N2).
* Each of the three boundary changes has a distinct live witness. Mutating the header clause away,
  the `END` clause away, `END`'s clause line to the `DO`'s, and the step-boundary suppression away
  each reddens `both_engines_agree_on_every_case_file`, and each hits a *different* one of the
  three new stanzas (`loop-header-boundaries:268`, `:295`, `:323`). Under `REXX_CORPUS_GATE=1` the
  suppression mutation also reddens `corpus_differential`; note that in the plan's stated gate
  command (`cargo test --release --workspace`, no gate env) `corpus_differential` runs in REPORT
  mode and exits 0 regardless, so the `ir_dual` stanzas are what actually gate this.
* `do_indent` holds the `DO`'s own indent on both engines — verified with a nested plain `DO` at a
  non-zero indent, where the tree-walker and the IR both match the oracle's handler indent.
* The before/after sweep claim holds: every `.rex` under `corpus/` and `bench-programs/`, both
  engines, all three descriptors, moves only `lang/do_clause_boundaries.rex`.
* All three "found and NOT fixed" items are accurate as written. Plain `SELECT` is still two
  columns short with stdout and `rc` agreeing; `if 1 = 1 then zr = raiser()` puts the handler four
  columns deeper than the oracle and the `select`/`when` shape six, both unchanged by this commit;
  `do zi over zz.` exits `rexx-exec: DO is not implemented` at `rc 120` against the oracle's
  `rc 0`.
* The C++ citation is right: `RexxInstructionSimpleDo::execute`
  (`interpreter/instructions/SimpleDoInstruction.cpp:71-95`) traces, opens the block and returns
  without running the body.
* `sourceline_oracle/do_clause_boundaries.txt` is a byte-identical copy of the corpus program and
  its `count 167` matches.
* Gates, run unpiped from `rust/`: `cargo fmt --all --check` 0, `cargo clippy --workspace
  --all-targets -- -D warnings` 0, `cargo test --release --workspace` 0. I also ran the debug
  suite (`cargo test --workspace`, exit 0) and every probe in this review under a debug build, to
  put the `enter_clause` tripwire in play — it never fires, including on the `INTERPRET` shape in
  C1.
