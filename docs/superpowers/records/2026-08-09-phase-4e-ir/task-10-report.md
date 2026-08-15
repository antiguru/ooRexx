# Task 10: promote the call forms

BASE `16077ea1`. Four commits, read back from `git log`:

| commit | what |
| --- | --- |
| `ab562714` | the dual case rows, written while both engines still answered them identically |
| `0742b77b` | the resolve/invoke split, no behaviour change |
| `6b5fac3b` | `Op::Call`, the compiler arm, the driver arm and the per-site resolution table |
| `32d83e9b` | what the case file was measured to catch, and the builtin path's dead `Rc::clone` |

Gates at head: `cargo test --workspace --no-fail-fast` **1423 passed, 0 failed** (BASE 1417; +4 golden,
+2 driver). `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets -- -D warnings`
exit 0. `REXX_CORPUS_GATE=1` corpus 51 of 51. `REXX_ASSERTIONS_GATE=1` green. `Interp::chunks_refused`
asserted zero on both arms. `size_of::<Op>() == 12` still compiles, so the stream did not widen.

---

## The prediction, recorded before anything is measured

**No registered benchmark axis executes a `CALL` instruction at all.** Counted, not assumed: zero
clause-initial `call` across all ten programs in `rust/bench-programs/`, the three this crate cannot
run included. So `Op::Call`, the region it sits in and the resolution table it reads **run zero times
on every axis**, and no figure Task 11 takes can be evidence about them in either direction. If a
figure for the resolution table is wanted, it needs an axis that does not exist yet: a `CALL` inside a
measured loop.

What the axes *do* reach is the split, on the expression route. Builtin function calls per run:
`strings.rex` 4 per pass x 3,000,000 = 12,000,000; `alloc4c.rex` 1 per pass x 1,000,000. `emptyloop`,
`varlookup`, `arith`, `compound` and `startup` execute no call of either kind. Every one of those
12,000,000 goes through `eval_call`, which now enters `resolve_call` and then `invoke_call` where it
used to enter one fused function -- same statements, same order, one more non-inlined boundary -- and
through `invoke_call`, whose `Rc::clone` of the caller's program now sits *below* the builtin return
instead of above it, so a builtin call does one refcount pair fewer than at BASE.

So:

* `emptyloop`, `varlookup`, `arith`, `compound`, `startup`: **no movement**, either arm, either
  instrument. Nothing this task touched executes on them.
* `strings`, `alloc4c`: **flat, or very slightly faster**, on both arms, with the `ir/tw` ratio
  unchanged -- everything that moved is shared between the engines, so it cannot move a ratio.
* **The axis I would expect to go the wrong way first, if one does, is `strings`**, and the mechanism
  is the extra call boundary: at 12,000,000 calls a per-call cost is visible there and nowhere else,
  and if the boundary costs more than the saved refcount pair the sign flips. I have no measurement
  either way and did not take one.

**This prediction is not derived from any recorded figure**, which is deliberate: the three wrong
predictions in this plan each cited a correct figure whose referent was a different change. There is
no earlier figure for a call, because no axis makes one.

---

## What was built

**`Interp::resolve_call(&mut self, name: &[u8], search_labels: bool) -> Result<Resolved, Failure>`**
is the resolution half, exactly the brief's signature. **It takes no `Code`, and that is the fix for
the brief's own trap made structural rather than remembered**: the search goes against the running
activation's body, and after this split there is no walked body in scope for it to search by mistake.
The spike's first version got this wrong and passed every test with no `INTERPRET` in it; that class
of error is now a compile error rather than a test that has to exist.

**`Interp::invoke_call`** is the invocation half: the argument loop with its `>A>` lines, the builtin
dispatch, the depth guard, the activation push and the five pieces of level state around it.
`resolve_and_run_call` is the two composed and is what the tree-walker's routes still use.

**`Interp::invoke_named_call`** is `exec_call` past its resolution -- `invoke_call` plus the `RESULT`
settle and its `>>>`. This is a correction to the brief's Interfaces line, below.

**`Op::Call { index, site }`** is one op inside a `Clause` region. It reads `name`/`literal`/`args`
off the region's own instruction, answers from `Chunk::calls[site]` or resolves and records, and hands
over to `invoke_named_call`. Every failure path leaves through `break 'region Err(..)`; there is no
`?` anywhere in the region block, so the enter/leave pair runs exactly once on every path.

**The resolution table** is `Chunk::calls`, dense over the call ops and indexed off the op the way
`Chunk::hints` is. A hit needs no guard because nothing can invalidate one, and each step has its own
reason: a `Resolved::Label` indexes the running activation's body and `run_activation` looks a chunk
up under that activation's own `body_key`, so a chunk is only ever entered for the body it was
compiled from; the builtin table is static; and `Interp::routines` never rebinds a name it has already
bound, because a second `::ROUTINE` for one refuses the program with 99.903 (fix round 1 replaced a
stronger "written once, before the first clause" here -- see concern 2). A raise is deliberately not
recorded, so a site that answered 43.1 asks again. `CALL_SITE_CACHE` is the one-line switch that
removes the table, for the reason `QUICKENING` is one.

**`CALL ON`/`OFF` and `CALL (expr)` stay `Generic`**, each for its own reason -- the first resolves no
name, the second learns its name at run time and could only keep an answer behind a guard, which is
the shape D24's amendment says this task has no evidence about. `a_trap_call_and_a_dynamic_call_stay_generic`
pins both.

---

## What a promoted call owes in trace lines, and the instrument that follows

**It owes nothing, and that is a conclusion I reached before emitting anything rather than an
omission.** The lines a call produces are `>A>` per argument, the argument expressions' own
`>L>`/`>V>`/`>O>`, the callee's clause echoes, and the `>>>` of the `RESULT` settle. Every one is
emitted inside `resolve_call`/`invoke_call`/`invoke_named_call`, which `Op::Call` *enters* rather than
replaces. That is the difference from `Op::Const`: a literal's value was taken away from `eval.rs`, so
its `>L>` had to be given back by an op; a call's arguments were not taken away from anything.

The reason they were not is in the golden test's own assertion that a promoted `CALL` allocates **no
register**: an argument is not an `ObjRef`. A `>name` reference carries the caller's slot with it,
which no register holds, so the argument list cannot become registers without a second channel for
that slot -- and `Op::EvalExpr` names an expression *slot of an instruction*, not a subexpression, so
it cannot serve as the per-argument fallback either.

**The instrument that follows from it, measured rather than argued.** Both arms of `tests/ir_dual.rs`
are this crate, so a line both emit from one shared function is one the sweep structurally cannot
police. Removing the `>A>` emission from the shared argument loop: `both_engines_agree_across_every_population`
**stays green over 10,391 programs**, and `both_engines_agree_on_every_case_file` goes red. The
oracle-pinned rows are what hold those lines; the sweep is what holds the clause echo the promotion
newly decides at compile time -- removing the `Op::TraceClause` for a `CALL` reddens the sweep and it
names a program (`corpus lang/call_return.rex`, missing `46 *-* call inner`).

---

## What the case file was measured to be worth

Five mutations, each run over the whole workspace with `--no-fail-fast`, with the file in and held
out:

| mutation | catchers with the file | catchers without it |
| --- | --- | --- |
| no clause echo op for a `CALL` | sweep, case file, golden | sweep, golden |
| `>A>` emission a no-op | case file + 3 existing | 3 existing |
| kept resolution answers the wrong thing | case file, new driver test | new driver test |
| compiled `CALL "name"` searches labels | sweep, case file | sweep |
| `::ROUTINE` searched in front of the builtin | case file | one existing unit test |

**No row is the only catcher for any of them**, so the file is thin by mutation and its header now
says so, with the shared-line argument for keeping it anyway. This is Task 9's finding repeated, and
it is the answer to "do not assume a row earns its place because it is new".

Two things the table does say. **A wrong resolution on the hit path is caught by two things and the
sweep is not one of them** -- the case file and the new driver test both see it, and the sweep over
10,391 programs does not, because almost nothing in those populations reaches one call site twice in a
way a wrong answer would show. (An earlier draft of concern 3 below said "exactly one guard"; the
table above was right and the concern was not, and fix round 1's review reproduced the pair.) And
**21 rows was 20
until the mutation ran**: every original row set its own `TRACE` inside the body that then called,
which makes the chunk stale and hands the echo decision back to the run-time gate, so not one of them
ever reached the compiled echo op. The added row enters the callee's body with the setting already in
force.

---

## Where the brief is wrong, with the sentences

**1. "Produces: the `resolve`/`invoke` split ... and the invocation half".** The invocation half is
two functions, not one, and a compiled `CALL` needs the second. `invoke_call` ends at `Ended`;
everything a `CALL` does that `ExprKind::Call` does not -- the `RESULT` write and its caller-side
`>>>` at the calling clause's own indent -- is past that. Had `Op::Call` stopped at `invoke_call` and
settled `RESULT` itself, the `>>>` line would have been a second copy in the driver, which is exactly
the defect the sharing rule exists to prevent. The line should read "and the invocation half, which is
`invoke_call` plus the `RESULT` settle `CALL` alone owns".

**2. "Both routes, not one."** I read this as the *resolution* being shared by both routes, not as
compiling a native op for `ExprKind::Call` as well. The expression route reaches the identical
`resolve_call`/`invoke_call` through `Op::EvalExpr` -> `eval_call`, uncached, which is what the
Interfaces line asks for. Compiling a native op for it is a different and much larger change: it needs
the argument list to become registers, which the `>name` case forbids, and it needs a value-producing
call op, which `EvalExpr`'s instruction-slot addressing has no room for. If the plan meant the larger
change, this task did not do it and the sentence should say so.

**3. Step 1's "an external routine".** Read as `::ROUTINE`. An external Rexx *file* is Phase 7 and
this crate answers 43.1 in its place, so there is nothing to write a row against; the file has rows
for `::ROUTINE` through both spellings and for the 43.1 itself.

**4. The dispatch's "make the echo a no-op, confirm it reddens and names a program, restore"**
presupposes the promotion owns a new echo op. It does not, for the reason above. What I ran instead is
the pair in the section above: the sweep reddens for the clause echo the promotion *does* newly own,
and stays green for a shared line while the case file reddens.

---

## Two probe errors of my own, both from the memory list

**A stale binary produced a fake oracle divergence.** `target/debug/rexx-run` was left carrying the
`>A>`-removal mutation by a `cargo test --workspace` run, and a probe using it reported the oracle
emitting `>A>` where this crate did not -- for a single-argument `CALL`, which is a shape the suite
covers. I minimised it across five variants before noticing the three-argument row had passed with the
same binary. A rebuild made it disappear. All 21 case programs were then re-verified against the
oracle, both engines, with a fresh binary: 21 of 21 identical.

**The first `--no-fail-fast`-less mutation runs were unmeasured.** `cargo test --workspace` stops at
the first failing binary, so my first reading of "the case file adds no coverage for `>A>`" and "only
the driver test catches a wrong resolution" were both taken from runs where the integration binaries
never executed. Both were re-run; the table above is from the re-runs.

---

## Concerns

1. **The resolution table has no axis and no evidence.** Task 11 must not attribute any movement on
   the current axis set to it, in either direction: it executes zero times on all of them.
2. **Its correctness rests on `Interp::routines` being append-only** -- a second `::ROUTINE` for a
   bound name refuses the program with 99.903, so a name that resolves keeps what it resolved to.
   Fix round 1 replaced the stronger and unasserted "`install_directives` runs once, before the first
   clause": that happens to be true today because production loads one program, but it is not the
   property the table needs and it is not the one the code enforces. If a `::REQUIRES` or an
   external-file call ever *rebinds* a name, the append-only sentence is what stops being true.
3. **A wrong resolution on the hit path has two guards, and the dual sweep is neither.** The case file
   and the new driver test both catch it; the sweep over 10,391 programs does not.
4. **`CallSite` is `Cell`, so `Chunk` is not `Sync`, and it is the only reason.** Measured rather than
   reasoned, at fix round 1: requiring `Chunk: Sync` in a throwaway `const` makes rustc name
   `Cell<Option<Resolved>>` and nothing else, so every other field -- `PatchSlot` included -- already
   satisfies it and `Chunk` was `Sync` before this task. `PatchSlot` took the atomic as a
   forward-compatibility bet for shared routine bodies; a `Resolved` is too wide for that bet to be
   free, so this is the type that would have to change first. Stated in its doc rather than left to be
   discovered.
5. **Nothing here validates send caching**, per D24 as amended. The classic case needs no guard, which
   is the property a send cache lacks, and no conclusion about one is carried out of this task.

---

## Fix round 1

Three Minor findings, all documentation, in `5ab3028f`. Gates re-read after them:
`cargo test --workspace --no-fail-fast` **1423 passed, 0 failed**; fmt 0; clippy 0.

**Stale names after the split.** The review adjudicated three (`clause.rs:208`, `eval.rs:198`,
`eval.rs:544`) and explicitly declined to say which of the rest were false, so I swept all of them.
Twenty-two comments named `resolve_and_run_call` for work it no longer contains -- the `extra`
write-back, the frame pop, the five pieces of level state, the `>A>` line, the seal on the error path,
the argument evaluation -- and now name `invoke_call`. Two references stay and both are structural
rather than descriptive: the trap-handler call site (`run.rs:3364`) and the definition itself.
**One was false rather than imprecise**: `clause.rs` said `eval_call` reaches `resolve_and_run_call`,
which it does not, since opening the seam is exactly what let it enter the two halves directly.

**`Chunk::hints`' doc no longer counts.** "One of the two mutable things a running chunk owns" was a
count of an in-repo aggregate in prose; the argument it carries -- an op is emitted once and never
rewritten -- needs no count, and a third table would falsify it with nothing rereading the sentence.

**`CallSite`'s doc now rests on append-only-ness.** See concern 2 above for what changed and why the
stronger sentence was the wrong thing to hand the next reader.

**And the `Sync` note is now a measurement.** I verified the reviewer's claim rather than inheriting
it: requiring `Chunk: Sync` in a throwaway `const` and building makes rustc name
`Cell<Option<Resolved>>` and nothing else. The doc states that, and deliberately does **not** cite an
`assert_sync` helper by name -- no such function exists in this tree, and a comment quoting one would
read as a runnable check that is not there.
