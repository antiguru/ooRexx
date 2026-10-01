# Task 8 fix round 1 report

`$S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t8-fix`.
`$S/cmp.sh BIN OUT FILE...` runs the oracle (standard wrapper plus `timeout 60`) and BIN, each from a
fresh `mktemp -d`, and compares stdout, stderr and rc separately.

## I1 `460034fb4`

`call_condition_three_deep.rex:39` ends `raise user done description 'from deep' return`;
`sourceline_oracle/call_condition_three_deep.txt` regenerated with the module-doc driver on a
scratch copy of the file.

`$S/cmp.sh $S/target/release/rexx-run $S/w1 corpus/lang/call_condition_three_deep.rex`:
`stdout=same stderr=same rc=0/0`. Oracle stdout, read:

```
y 14
handled USER DONE from deep 38
deep 2
deep 3
after trio trio u
two 14
caught 42 SYNTAX 32

```

CALL ON handler (`handled ...`), the SELECT calls (`two 14`), the SYNTAX trap from `one(0)`
(`caught 42 SYNTAX 32`, the empty `condition('D')` line) all run. Stderr empty on both.
`cargo test --release -p rexx-parse --test sourceline_oracle`: 1 passed, exit 0.

## I2 `47f70e30b`

`call_callee_allocates.rex`: `a = copies('left', 4)` (line 2), `copies('ab', 6)` (line 7),
`copies('rr', i)` in `rt` (line 28), so every value held in a register or pending argument across an
allocating callee is longer than 7 bytes: `a || i` (17), `a || 'x'` (17), `abababababab` (12),
`a || 'z'` (17), `4 rrrrrrrr` (10, held across `rt(0)`). Sourceline oracle regenerated as for I1.

Same `cmp.sh` run: `stdout=same stderr=same rc=0/0`; stdout
`36 leftle` (three times), `abababababab / 11`, `leftleftleftleftz / 5`, `4 rrrrrrrr none`, `2 rrrr`.
`memcap 8G cargo test --release -p rexx-exec --test collect_stress`: 32 passed, exit 0
(`call_callee_allocates.rex` is in `phase-8.txt`, which the stress subset reads).

## I3 `8428ca61b`

The non-leaf argument evaluation in `begin_invoke_call`, `invoke_builtin_call` and
`arguments_before_failure` (`run/call.rs`) runs under `pinned!(…, PinKind::TreeEval, …)`; callee entry
is not pinned. `FRAME_PROBES` gains three `TreeEval` cases: `x = f(g())`, `call f g()`, `x = abs(g())`,
each with `g` calling `SysSleep`.

Negative control: the three cases with the `call.rs` change reverse-applied
(`git apply -R`), `cargo test -p rexx-exec --features pinning --test concurrency_tests --
measured::a_park_under_each_frame_kind_records_it`: exit 101,
`"TreeEval: {(SysSleep, []): 1}"` three times. With the change, `-- measured::a_ measured::an_`:
4 passed, exit 0.

Feature off, the change compiles to nothing: `.text` sha256 of the release `rexx-run` is
`807c1340…45878` at `c929acecf` and at `8428ca61b`.

The report sentence naming the calls that stay recursive is deleted from `task-8-report.md`. These
calls still recurse on the Rust stack.

## I4 `c929acecf`

`Op::CallExpr`'s non-TOP arm calls `run_call_expr` (`begin_call_expr`, then `run_activation` and
`finish_function_op`), `Op::Call`'s calls `run_call_tree` (`begin_call_tree` plus
`complete_subroutine`); the inline copies are gone. The comment above `Op::Call` that named
`invoke_named_call` is replaced.

Paths reached: `gdb` hit counts on `$S/p4/nontop.rex` (interpret `x = f(g(2))`, interpret
`call f g(3)`, interpret `call nosuch g(1)`, nested loops with `f(g(...))` and `call f g(j)`):
`run_call_expr` 1, `run_call_tree` 2. `$S/cmp.sh` on it: `stdout=same stderr=same rc=213/213`.
`cargo test --release -p rexx-exec --lib`: 860 passed, exit 0. `REXX_CORPUS_GATE=1 memcap 8G cargo
test --release -p rexx-exec --test corpus`: `661 of 661 matching`, 29 passed, exit 0.

Callgrind `-r 1` against the build of `47f70e30b`: fibcall `8592742612` -> `8592742716`, fibfunc
`8279742732` -> `8279742836`, sendloop +276, nop +104 (`$S/cg-i4`), about 100 instructions per
run. The archived-tree measurement below has fib identical between `t8r3` and `fix1`.

## M1

No change.

## Performance

`4b0128186`, appended to `docs/superpowers/plans/phase-6-perf.md` `## Task 8` `### Fix round 1`
(builds, shas, command). `callgrind.sh -r 3`, exit 0, no SPREAD, delta against base `1754a3b5a`:

| program | t8r3 d% | fix1 d% |
|---|---:|---:|
| fibcall | +2.9603% | +2.9603% |
| fibfunc | +3.6404% | +3.6404% |
| sendloop | +1.1201% | +1.1201% |
| extcall | -0.9761% | -0.9761% |
| rexxcps | +0.6277% | +0.4957% |
| dispatch | -1.1204% | -1.1204% |

I4 does not raise Ir on fib: fibcall and fibfunc retire the same count on `t8r3` and `fix1`.

## Gates

GATES_PLACEHOLDER

## Concerns

- `.superpowers/` is ignored by git, so the `task-8-report.md` deletion and this report are not in
  any commit.
