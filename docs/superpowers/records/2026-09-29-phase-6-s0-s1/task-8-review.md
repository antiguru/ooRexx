# Task 8 review: the in-loop driver for calls

Reviewed range `8d26923bb..b558488a1` (six commits), read-only. Built in my own copy:
`git archive b558488a1` and `git archive 8d26923bb` under
`$R=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t8-review/`,
trees touched, separate `CARGO_TARGET_DIR`s (`target`, `target-base`, `target-pin`), release builds.
`$R/cmp.sh BIN FILE...` runs the oracle under the standard wrapper plus `timeout 60` and BIN, each
from a fresh `mktemp -d`, and compares stdout, stderr and rc separately.

## Verdicts

- **Spec compliance: not compliant as committed, compliant after fixes I1 and I2.** Steps 3 to 5
  hold for every call a body's own call ops make. Step 2 does not: one witness never reaches two of
  the three paths it is filed for (I1), and the collect-stress witness holds no heap value in a
  register across an allocating callee (I2). A narrower Step 3 gap is I3.
- **Task quality: approve with fixes.** The driver, the parking, the frame arena extension and the
  begin/finish split are sound on every probe I ran. The findings are witness strength, one
  pinning undercount that the report states the opposite of, and a leftover second copy of two
  begin halves.

## Findings

### I1 (Important). `call_condition_three_deep.rex` exits at its RAISE; the CALL ON handler, the SELECT calls and the SYNTAX trap never run

`rust/corpus/lang/call_condition_three_deep.rex:39`: `raise user done description 'from deep'`
has neither `RETURN` nor `EXIT`, so RAISE defaults to exit and the program ends before a clause
boundary can deliver the CALL ON trap. Both sides print `y 14` and nothing else, rc 0:

```
$R/cmp.sh $R/rexx-new call_*.rex      # from rust/corpus/lang
call_condition_three_deep stdout=same stderr=same rc=0/0     # stdout is the single line "y 14"
```

So the report's row for this program ("SYNTAX three function calls deep trapped by the outermost
`SIGNAL ON SYNTAX`; a USER condition raised three `CALL`s deep trapped by the outermost `CALL ON`;
function calls inside `SELECT`") describes lines 9 to 23 and 34 to 35, none of which execute. A
regression that broke CALL ON delivery across a parked level, or a SYNTAX unwinding three parked
function levels, stays green. The brief's Step 2 witness for a trap across frames is therefore
missing (the depth-cap witnesses cover a deep SYNTAX under SIGNAL ON, not CALL ON).

Fix: append `return` to line 39 (`raise user done description 'from deep' return`) and regenerate
its `sourceline_oracle` file. Measured on that variant (`$R/src/c3d_return.rex`): identical on all
three descriptors, rc 0/0, and every path now runs:

```
y 14
handled USER DONE from deep 38
deep 2
deep 3
after trio trio u
two 14
caught 42 SYNTAX 32
```

### I2 (Important). `call_callee_allocates.rex` holds only inline values across its allocating callees

The brief asks for a collect-stress test that allocates in a callee "while the caller holds a
value in a register". In `rust/corpus/lang/call_callee_allocates.rex` every value a caller register
holds across an allocating callee is a string of at most 7 bytes or a small integer: `a || i`
(`left1`..`left3`, line 4), `copies('ab', 3)` (`ababab`, line 7), `a || 'z'` (`leftz`, line 8), and
`rt(4)`'s `4 rrrr` held across `rt(0)`, which exits before allocating. `INLINE_TEXT` is 7
(`rexx-core/src/handle.rs:25`), so each of these is an immediate handle that needs no root. A
defect that left parked register frames unscanned would not change this program's output under
collect-on-every-allocation. This is the "short strings are inline and hide it" shape.

The code itself is right: my long-string variant passes under the stress mode.
`$R/src/gc/args_long.rex` sets `a`..`d` to `copies(..., 20)` (40 to 60 bytes, heap) and holds
`a || b || i` in a register and a lent argument stack while `g` allocates 20 strings:

```
cd $R/tree/rust && REVIEW_DIR=$R/src/gc CARGO_TARGET_DIR=$R/target \
  cargo test --release -p rexx-exec --test review_stress -- --nocapture
.../args_long.rex plain_rc=0 stress_rc=0 collections=256 same_out=true same_err=true
```

(`review_stress.rs` is my scratch harness: `run_program` against `run_program_collect_every_alloc`,
stdout and stderr compared.) Oracle: identical on three descriptors.

Fix: set `a = copies('left', 4)` (16 bytes) on line 2, so every held value on lines 4 and 8 is a
heap string, and regenerate the sourceline oracle and the corpus expectation.

### I3 (Important). A call in another call's tree-evaluated argument list still recurses, unpinned; the report says it is covered

The report: "the calls that stay recursive are those inside frames already hooked (`Interpret`,
`NestedLoop`, `TreeEval`, `TreeSend`, `OpExec`, `TrapHandler`, natives)". Not so for a call nested
in the arguments of an `Op::CallExpr` or `Op::Call`: `x = f(g(a))` and `call f g(a)` compile to
`CallExpr`/`Call` (`rexx-ir` output), whose begin halves evaluate the arguments with the tree
evaluator (`run/call.rs:341` onward, from `begin_invoke_call`), and `g` runs through
`eval_call_resolved` and `run_activation` on a nested Rust frame. Measured with the `pinning`
feature (`$R/tree/rust/crates/rexx-exec/tests/review_pin.rs`, a park in the inner callee):

```
nested_arg: parks={(SysSleep, []): 1}        # x = f(g(a)), g sleeps
nested_arg_call: parks={(SysSleep, []): 1}   # call f g(a)
nested_arg_deep: parks={(SysSleep, []): 1}   # g calls h, h sleeps
```

and the native stack grows per level (`review_stack.rs`, `Outcome::stack.bytes`):

```
direct      depth=50 stack_bytes=3153     depth=2000 stack_bytes=3153
in_arg      depth=50 stack_bytes=212737   depth=2000 stack_bytes=8511937   # return f(r(n-1))
call_arg    depth=50 stack_bytes=207137   depth=2000 stack_bytes=8287937   # call f r(n-1)
builtin_arg depth=50 stack_bytes=204737   depth=2000 stack_bytes=8191937   # return abs(r(n-1))
```

So a park inside `g` is recorded with no pinned frame while a Rust frame lies between the scheduler
and the driver: exactly the tree-evaluated expression P6-4 names as pinning, and the undercount
S2's baton release would rely on. `Op::EvalExpr`, the other tree-evaluated route, is wrapped in
`PinKind::TreeEval` (`ir/drive.rs:537`); these argument evaluations are not. The per-kind probes
do not cover the shape.

Fix: wrap the tree argument evaluation reached from `begin_call_expr`/`begin_call_tree` (the
non-leaf `eval` in `begin_invoke_call`'s argument loop, and `arguments_before_failure`) in
`pinned!(…, PinKind::TreeEval, …)`, leaving the callee entry itself unpinned; add
`("TreeEval", "x = f(g())\n…g: call SysSleep 0…")` to `FRAME_PROBES`; and delete the report
sentence quoted above. Whether nested call arguments should compile to `PushArg`/`CallArgs` so that
they too run stackless is a scope question for the controller; the brief's "native-stack depth no
longer grows per Rexx call" does not hold for these calls today.

### I4 (Important). `Op::CallExpr` and `Op::Call` keep a second copy of their begin half on the non-TOP route

Named risk 1. `CallArgs` and `CallNamed` are clean: their non-TOP arms call `run_call_args` /
`run_call_named`, which are `begin_call_args` / `begin_call_named` composed with `run_activation`
and the finish. `CallExpr` and `Call` are not:

- `ir/drive.rs:375-426` (`region_ops!`, `Op::CallExpr`): the TOP branch calls `begin_call_expr`
  (`ir/drive.rs:1777`); the non-TOP branch re-does its body inline (node lookup,
  `call_target_name`, `site_resolution_before_arguments`, `enter_eval_node`, `eval_call_resolved`,
  `depth -= 1`).
- `ir/drive.rs:1229-1280` (`Op::Call`): the TOP branch calls `begin_call_tree`
  (`ir/drive.rs:1873`); the non-TOP branch re-does resolution before and after arguments and calls
  `invoke_named_call`.

Both copies agree today (every probe here matches), but the brief's interface is "begin/finish
halves whose composition is the recursive path the pinned route keeps", and a later change to one
half (the I3 pin, for instance) must now be made twice. Failure scenario: I3's fix applied in
`begin_call_expr` only leaves every fragment and nested-loop `f(g())` unpinned.

Fix: add `run_call_expr` (`begin_call_expr`, then `complete_function`-style `run_activation` plus
`finish_function_op`) and route `Op::Call`'s non-TOP arm through `begin_call_tree` plus
`complete_subroutine`, mirroring `run_call_args` / `run_call_named`; delete the inline copies.

### M1 (Minor). `FrameArena::unpark` checks bounds, not identity

`rexx-core/src/frame.rs:276-291`: the panic guards `top >= len` and `start <= size`, which is what
the SAFETY comment needs, and that argument holds (property 2 for any `ParkedFrame`, from any
arena). It accepts a same-length frame that is not the parked one, and `park`'s identity check is
debug-only. Logical only, never unsound, and the driver pairs them LIFO. No change required;
noting that the `# Panics` text ("cannot be `parked`'s, by length or offset") is accurate as
written.

## Named risks

1. **One implementation of begin/finish.** Yes for `CallArgs`, `CallNamed`, `invoke_call`,
   `invoke_call_over`, `eval_call_resolved`: each recursive entry is `begin_*` composed with
   `run_activation` and the same finish (`finish_call`, `finish_function`, `settle_call_result`)
   the stackless resume uses. No for the non-TOP arms of `CallExpr` and `Call` (I4).
2. **Parked state (P6, P13, P14).** `call_tails`, `parked_calls`, `parked_levels` are plain `Vec`s
   on the inline `Activity`, reached by index; register frames park as `ParkedFrame` (length and
   opened flag, no borrow). The arena is reached through `Rc<FrameArena>` (`roots.rs:358`), so a
   P13 swap moves the `Rc`, not the blocks. The running level's `registers`, `temps` and `Level`
   are `drive` locals across the whole loop; no switch happens inside `drive`, so P14 holds, and
   S2's switch will have to park the running level too. `ParkedCall.header` holds
   `LoopHeaderValues`, whose one `ObjRef` (`over`) also sits in `over_register`, a parked register:
   rooted. The `object_roots` comment's claims were checked by the I2 long-string stress run.
3. **Pinning hooks.** Truthful for every call a body's own ops make (`fn`, `call`, `deep`, `rtn`,
   `hdr_to`, `in_flat_loop`, `trace`: parks with `[]`), and for `WHILE` (`[LoopHeader]`),
   `INTERPRET` (`[OpExec, Interpret]`), method bodies (`[TreeSend]` / `[TreeEval]`). Not truthful
   for calls nested in tree-evaluated call arguments (I3). No hook changed and the per-kind probes
   are unchanged and still meaningful for the kinds they name.
4. **Structural tests.** `refusal-sites.tsv`'s `missing_body` surface `body+send` to `body+ir+send`
   follows `drive` constructing `Loud::missing_body` (`ir/drive.rs:2006`). `NO_ALLOCATION_PROGRAMS`
   gains `lang/call_trace_nested.rex`, which allocates nothing (small integers and short strings
   only), as the harness requires. The frame-floor message edit is wording. No golden, invariants,
   valid or corpus-shape file changed (`git diff --stat`). None loosened.
5. **Witnesses against the oracle.** All six identical on stdout, stderr and rc
   (`$R/cmp.sh $R/rexx-new call_*.rex`, rc `call_signal_out_of_callee` 3/3, others 0/0). Two are
   weaker than claimed (I1, I2). Error 11: 9999 callee levels by `CALL`, by internal function and
   by `::ROUTINE`, on `8d26923bb` and `b558488a1` alike (`$R/src/depth_*.rex`, `.local` counter);
   the oracle reaches 27317 (`CALL`) and 17295 (`::ROUTINE`), which the witnesses rightly do not
   print. Untrapped recursion: rc 245 on both builds. `oracle-crashes.txt` entry 20 reproduced
   twice (oracle rc 139, stdout `start`; this crate `start`, `caught 11 SYNTAX`, empty line, rc 0).
   Further probes, all identical to the oracle (`$R/src/b1`, `$R/src/b2`): calls in every loop
   header position, `DO OVER`, `WHILE`/`UNTIL`, `SELECT`/`IF`/`PARSE VALUE`, `INTERPRET`,
   `NUMERIC`/`ADDRESS` restore, `SIGL`, 44.1 names (`nov()`, `'NOV'()`, `rt()`, `'rt'()`, EXIT
   without value), CALL ON delivered at the resumed boundary (`x = f() + 1`, `do i = 1 to f()`),
   EXIT from a function inside a loop, SYNTAX from a callee inside open `SELECT` and loop frames,
   `LEAVE`/`ITERATE` after calls, nested loops, method bodies calling labels, `TRACE R`/`TRACE I`,
   PROCEDURE after a call in the first clause. Pre-existing differences seen on both builds:
   `length(copies(f('ab'), f(3)))` (the report's `min(3, f())` divergence) and a
   `do ... end; procedure` blame line (`proc5.rex`).
   Suites in my copy, release: `ir::drive` 18 passed, `rexx-core` `frame` 8 passed,
   `collect_stress` 32 passed, `REXX_CORPUS_GATE=1 --test corpus` 29 passed with `661 of 661
   matching`, each exit 0.
6. **Perf record.** `phase-6-perf.md` `## Task 8` has binary shas, `.text` sizes, the build and
   callgrind/wallclock commands, loads, and the tables for t8, t8r1, t8r2, t8r3; every figure the
   report quotes agrees with them (fibcall +2.96%, fibfunc +3.64%, sendloop +1.12%, rexxcps +0.63%;
   wall +9.61%, +10.56%, heapshape +4.09% with base2 +1.57%; round deltas). The report's "not
   measured on the final build" icache gap, measured here on `b558488a1` against `8d26923bb`,
   `perf stat -x, -e <event> rexx-run bench-programs/fibfunc.rex`, three runs each:

   | | instructions | L1-icache-load-misses | cycles |
   |---|---:|---:|---:|
   | base | 8.932e9 | 56.8M to 59.7M | 2.354e9 to 2.377e9 |
   | new | 9.054e9 (+1.4%) | 74.0M to 75.1M (+28%) | 2.591e9 to 2.638e9 (+10%) |

   So the wall gap is front end. `perf record -e L1-icache-load-misses -c 20000` puts 8.4% of the
   new binary's misses in `resume_region` (the second `region_ops!` expansion, which fib's `+` after
   each call runs from), 3.1% in an out-of-line `Vec<ParkedCall>::push_mut`, and memmove at 11.5%
   against 5.3%. No obvious cheap fix: the second expansion is what the brief's "no per-clause
   resume test" costs, and the report's measured variants already cover moving `park_call` and the
   header boxing. Not failed on, per the brief to this review.
7. **`git checkout-index -f` on `lib.rs`.** `rexx-exec/src/lib.rs` is unchanged across the whole
   range; `rexx-core/src/lib.rs` has only the `ParkedFrame` export. Round 3's commit touches only
   `run/call.rs`, and each round's message claim is present in its diff (round 1: `ops_loop_steady`,
   `resume_call`, builtin shortcut in `begin_call_over_pushed_args`, `Plan::last_chunk`; round 2:
   `running_level` memo first; round 3: `begin_call` taking `&[Option<ObjRef>]`, library and
   external entries `to_vec`). No lost intended change is visible.

Rules: `unsafe` added only in `frame.rs`; `size_of::<Op>() == 16` assert intact (`ir.rs:61`); no
`Op::Generic`; no em-dashes in added lines.

Target dirs deleted after the run.
