STATUS: DONE

# Temps-frame investigation: `?` early returns that skip `pop_frame`

Investigator: read-only review agent, 2026-07-31, at commit 34f2a4d6.
Commissioned before Tasks 10/11 multiply the pattern.

**RECOMMENDATION UP FRONT: document, do not restructure.** The pattern is
already healed at instruction granularity by an existing chokepoint
(`step_in_temps_frame`, `lib.rs:937`) whose doc comment names this exact
hazard, combined with `pop_frame` being watermark truncation, which makes an
outer pop subsume every leaked inner frame. Tasks 10/11 can copy the current
eval-site pattern freely: the `Flow::Goto` program-counter design means their
condition evaluations are per-instruction wrapped automatically. A `Drop`
guard cannot be written here without `unsafe` or a borrow-discipline rewrite;
the one viable code change is a closure helper, and it would buy uniformity,
not correctness. Details and one falsification of the commissioning belief
below.

## 1. Every site, as a table

`push_frame`/`pop_frame` appear at exactly seven sites in `rexx-exec`
(nothing in `rexx-num`, `rexx-parse`, or the bins; `rexx-core` only defines
them). "Skipping paths" counts every `?` or explicit `return Err` between
push and pop.

| # | Function | push | pop | Paths that skip the pop |
|---|---|---|---|---|
| 1 | `eval_prefix` | eval.rs:217 | eval.rs:245 | 4: operand eval `?` (218), `arith_operand` `?` (223), add/sub `map_err` `?` (231), `return Err(not_logical)` (239) |
| 2 | `eval_arithmetic` | eval.rs:271 | eval.rs:305 | 6: operand evals (272, 274), `arith_operand` (277), `return Err(power_exponent...)` (286), right operand (291), arithmetic `map_err` `?` (302) |
| 3 | `concat` | eval.rs:336 | eval.rs:349 | 2: operand evals (337, 339) |
| 4 | `eval_compare` | eval.rs:377 | eval.rs:412 | 3: operand evals (378, 380), `compare_decoded` `map_err` `?` (409) |
| 5 | `eval_logical` | eval.rs:432 | eval.rs:452 | 4: operand evals (433, 435), the two logical checks (439, 442) |
| 6 | `eval_logical_list` | eval.rs:489 | eval.rs:504 | 2 per element: item eval (492), element check (496) |
| 7 | `step_in_temps_frame` | lib.rs:942 | lib.rs:944 | **none**: captures `step`'s result in a local, pops unconditionally, then returns it |

So: six leaky sites, all in `eval.rs`, all the same shape; and one
deliberately non-leaky site that is the reason the six do not matter. Site 7
is not an accident: its doc comment (lib.rs:926-936) says the frame is
closed around `step` rather than inside it "because `step` returns through a
dozen `?` paths and a frame closed on only some of them is worse than none".
The crate uses the same capture-then-cleanup shape twice more: `eval`'s
depth bookkeeping (eval.rs:71-73, decremented on every exit path by
construction) and `pop_slots`, which handles the *slot* stack and does
assert balance. The temps-frame imbalance is a knowing, localised choice,
not an oversight.

## 2. What the RootSet actually does (read, not inferred)

`rexx-core/src/roots.rs`:

- `push_frame` (roots.rs:70) is `FrameId(self.temps.len())`. **It mutates
  nothing.** A frame is not an object or a generation; it is a saved
  watermark into the one `temps: Vec<ObjRef>`.
- `pop_frame` (roots.rs:74) is `self.temps.truncate(frame.0)`. Pure
  truncation to the watermark.

Consequences, each following directly from those two lines:

- **A later pop with an outer (smaller) watermark unwinds every leaked
  inner frame as a side effect.** Truncating to a lower index removes
  everything above it, including temps a skipped inner pop left behind.
  This is what makes site 7 a complete healer for sites 1-6: its watermark
  predates anything an instruction pushed.
- **No corruption is possible in the current call shape.** There is no
  frame metadata to desynchronise. The only misuse with an effect is
  popping a *stale* watermark larger than the current `temps.len()`, which
  `Vec::truncate` silently ignores (truncate never grows). No code path
  does this today; it would require holding a `FrameId` across a pop below
  it.
- **Nothing asserts balance, in debug or otherwise.** Contrast the slot
  side of the same file: `pop_slots` (roots.rs:96) and `grow_slots`
  (roots.rs:177) both `assert_eq!` on frame depth. The temps side has no
  equivalent, so an unbalanced temps frame is invisible to every assertion
  in the codebase.

## 3. Is it observable today? (the commissioning belief, falsified in mechanism)

The belief to test was: "benign in 4a because a raised condition terminates
the program, so the leaked frame is freed by process exit and no allocation
happens after it." **The verdict (benign) is right; the mechanism is not,
and the difference matters for item 4.**

- The leaked frame is not cleaned by process exit. It is truncated by
  `step_in_temps_frame`'s unconditional pop the moment the failing
  instruction's `step` returns, hundreds of lines before `execute` renders
  anything: `run_activation`'s loop (lib.rs:793) routes every instruction
  through the wrapper, and so does the INTERPRET fragment loop
  (lib.rs:986). The leak's lifetime is the unwind from the `?` to the end
  of the current instruction, full stop.
- Proof that "the program terminates" cannot be the load-bearing fact: the
  crate's own tests already continue after a raise. Every
  `unwrap_err()`-then-reuse test (`a_comma_list_element_failure_raises_34_6_not_34_901`
  evaluates twice on one `Interp`) resumes execution on an interpreter that
  just raised. Tests that call `eval` *directly* (the `eval_condition`
  helpers) bypass the wrapper entirely, so their leaks genuinely persist
  across the rest of the test. Nothing observes them because nothing can:
  no collection ever runs during execution (`alloc_with`, heap.rs:192, is
  free-list-or-append with no collect trigger; `Heap::collect`'s only
  callers are rexx-core's own tests and benches), and no assertion checks
  temps balance (item 2).
- During the unwind window itself, nothing can observe the extra temps
  either: the error path from any eval site propagates by plain returns
  through `step` to the wrapper, allocating only Rust-side strings
  (`Raised` construction), never heap objects, and `roots.iter` is only
  called by `collect`.

So: **not observable today, in program execution or in tests**, and the
reason is architectural (wrapper + truncation), not situational
(termination).

## 4. When does it stop being benign?

Checked each candidate:

- **4b `SIGNAL ON SYNTAX` (the commissioned candidate): does NOT make it
  malignant under the current architecture.** A trap transfers control to a
  label, i.e. it acts at the instruction-loop level, and the wrapper has
  already truncated the instruction's whole temps range before the
  `Failure` even reaches the loop's `Err` arm (lib.rs:793-838, where
  `failure_site` is recorded, is *outside* the wrapper). Leaks cannot
  accumulate across trapped conditions because each one is healed before
  the trap could see it. The one way 4b could break this is implementing a
  trap by catching `Failure` *inside* `step` mid-instruction and resuming
  within the same instruction; nothing in the current design suggests that,
  and the failure-site machinery already assumes resolution happens at the
  loop.
- **GC arrival (the "nearer" candidate): also does not.** The commission's
  framing that "an over-long root set is a correctness problem and not just
  a memory one" has it backwards for this defect's direction. A leaked
  temps frame *over*-roots: it keeps dead operands reachable, which delays
  reclamation but cannot corrupt anything, under mark-sweep or moving
  collection alike. The correctness hazard in a root set is
  *under*-rooting, and that lives elsewhere in this crate (the "`joined` is
  unrooted from here to the caller's own `push_temp`" windows, e.g.
  eval.rs:347, which are a different, already-documented discipline). When
  collection starts firing inside `alloc_with`, the worst this pattern
  contributes is a few extra live `ObjRef`s for the remainder of the
  current instruction.
- **4a's remaining tasks (9-11): nothing reaches it sooner.** IF, SELECT,
  DO and LOOP compile to `Flow::Goto` program-counter jumps handled by
  `run_activation`'s loop (the `Flow` match, lib.rs:839-843), not to nested
  interpretation inside one `step` call. Every condition re-evaluation of
  every loop iteration therefore happens under a fresh
  `step_in_temps_frame`. The wrapper's coverage is not merely preserved by
  Tasks 10/11's design; it is how that design already works.
- **The genuinely unhealed shape, for the record:** any future non-test
  code that calls `eval` in a loop *outside* the wrapper and continues
  after an `Err`. Today only test helpers do this. If 4b's built-in
  function dispatch or condition machinery ever evaluates expressions
  outside instruction context, that call site inherits the obligation the
  wrapper currently discharges.

## 5. Fix shape

Candidates, judged on whether they can actually be written in this crate:

- **A `Drop` guard: cannot be written here without `unsafe` or a
  restructure.** The guard must hold `&mut RootSet` (or `&mut Interp`) to
  pop in `Drop`, and the body between push and pop calls
  `self.eval(...)`, which needs `&mut self` at the same time. Two live
  `&mut` borrows: does not compile. Escapes are a raw pointer in the guard
  (`unsafe`, forbidden here without strict need, and this is not strict
  need) or moving `RootSet` behind `RefCell`, which relaxes the whole
  crate's borrow discipline to fix a non-defect. Not writable, so not a
  candidate regardless of elegance.
- **Threading `match` instead of `?` through six functions with up to six
  exit paths each:** mechanical noise at every site, and the next task adds
  the seventh site wrong. No.
- **A closure helper** `fn with_temps_frame<T>(&mut self, f: impl
  FnOnce(&mut Self) -> Result<T, Failure>) -> Result<T, Failure>` that
  pushes, runs `f(self)`, pops unconditionally, returns the result. This
  compiles (one `&mut self` at a time), matches the shape site 7 and
  `eval`'s depth bookkeeping already use, and would make sites 1-6
  balanced on every path. It is the only viable *code* change. But it must
  not be sold as a correctness fix: today it changes nothing observable,
  and its real value is insurance for a hypothetical future caller outside
  instruction context.
- **Document (recommended).** Two sentences, two places: on
  `RootSet::pop_frame`, that truncation-to-watermark semantics are
  load-bearing (an outer pop must subsume leaked inner frames, so pop must
  never become stricter, e.g. assert-on-imbalance, without first balancing
  every eval site); and in `eval.rs`, that a `?` between push and pop is
  deliberate and healed by `step_in_temps_frame`, so Tasks 10/11 should
  copy the pattern without guilt and future non-instruction callers of
  `eval` must provide their own balancing point. Optionally, one debug
  tripwire in `step_in_temps_frame`: on the `Ok` path assert the temps
  length equals the frame watermark, which catches a *success*-path leak
  (the one shape the wrapper currently masks silently); needs a small
  `temps_len()` accessor on `RootSet`. Cheap, and it turns the invariant
  from prose into a check.

**Recommendation: document now (two comments), take the debug tripwire if a
commit is being scheduled anyway, adopt the closure helper only if Tasks
10/11's author wants one pattern instead of two. Do not attempt the guard.**

## Method note

Everything above is from reading the committed tree at 34f2a4d6
(`roots.rs`, `heap.rs`, `eval.rs`, `lib.rs` around both instruction loops)
plus greps for every `push_frame`, `pop_frame`, `collect`, and non-test
`eval` caller. `lib.rs`/`run.rs` are mid-rewrite by another agent; if that
rewrite moves instruction execution off `step_in_temps_frame` or stops
routing an instruction loop through it, items 3-5 must be re-checked, since
every conclusion here leans on that chokepoint.
