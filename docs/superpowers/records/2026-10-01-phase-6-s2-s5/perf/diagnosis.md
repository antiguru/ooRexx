# Task 26 diagnosis: where S1 to HEAD added instructions on the over-budget programs

Binaries: `s1` (`1a81353e3`) and `head` (`c484f4516`) from `build.sh`. Profiles from the worktree
root's bench programs, from a fresh `mktemp -d`:

```
valgrind --tool=callgrind --dump-instr=no --compress-strings=no --callgrind-out-file=P.V.cg BIN rust/bench-programs/P.rex
callgrind_annotate --auto=no --inclusive=no P.V.cg ; callgrind_annotate --auto=no --inclusive=yes P.V.cg
```

plus `--dump-instr=yes --compress-pos=no --toggle-collect='*ops_loop_steady*'` for `fibfunc`. Self
Ir per source line, per call edge and per instruction come from parsing the `.cg` files
(`prof-scripts/lines.py`, `edges.py`, `instr.py`). Iterations: `sendloop` and `dispatch` 5,000,000
sends, `dispatchclass` 4,000,000, `fibfunc` and `fibcall` 1,719,390 calls (30 x fib(22)'s 57,313).

## Totals per iteration, head minus s1

| program | all | libc + ld-linux | gate measure (rest) |
|---|---:|---:|---:|
| sendloop | +195.1 | +76.0 | +119.1 |
| dispatch | +221.1 | +76.0 | +145.1 |
| dispatchclass | +226.1 | +93.0 | +133.1 |
| fibfunc | +99.9 | +5.0 | +94.8 |
| fibcall | +97.5 | +9.0 | +88.5 |

The "rest" column times the iteration count is the callgrind.sh delta (e.g. `dispatch`
145.1 x 5e6 = 725.5M against the table's 725,652,912). The malloc count is unchanged (malloc self
Ir 548,788,434 against 548,789,591 on `dispatch`): no new allocation per iteration.

## Where it is

Self Ir per function, per iteration, head minus s1 (non-libc; `run_activation` in s1 is
`drive_levels` in head):

| function | sendloop | dispatch | fibfunc | fibcall |
|---|---:|---:|---:|---:|
| `ops_loop_steady` | +59 | +84 | +49.5 | +48 |
| `drive_levels` vs `run_activation` | -27 | -42 | +15 | +16 |
| `flat_loop_step_top` (inlined into `ops_loop_steady` in head) | -101 | -101 | 0 | 0 |
| `begin_invoke` | +40 | +41 | | |
| `RawVecInner::try_allocate_in` (s1: inlined, calls `__rust_alloc`) | +38 | +38 | | |
| `__rust_alloc` | -18 | -18 | | |
| `Box<[u8]>::try_clone_from_ref_in` (s1: inlined) | +22 | +22 | | |
| `FrameArena::reserve` (s1: inlined) | +23 | +38 | | |
| `Vec<ParkedLevel>::push_mut` (s1: inlined) | +24 | +24 | | |
| `Vec<CallTail>::push_mut` (s1: inlined) | +19 | +19 | | |
| `resume_region`, `enter_eval_node`, `push_activation`, Activation drop, `release_method_activation`, `finish_send`, `start_from_package`, `begin_call`, `finish_function` | +4 to +7 each | | | |

Three kinds of cost, in order of size:

1. **The same work, no longer inlined.** On the send path s1 inlined the two name copies
   (`name.to_vec()` for `CallContext`, `name.into()` for `MethodIdentity`, `dispatch.rs`
   `begin_method`), the `CallTail` and `ParkedLevel` pushes and `FrameArena::reserve`. Head calls
   each as a function, and the allocations go through the generic `RawVecInner::try_allocate_in`
   (38 Ir self) instead of straight to `__rust_alloc` (about 9 Ir each). The self Ir of these
   outlined functions sums to +123 per send on `dispatch` net of `__rust_alloc`; s1 spent part of
   that inline in its callers, so the cost they add is less than that sum and was not separated.
   The send path's libc `memcpy` calls also changed (s1: three from `begin_invoke`, 57 Ir; head:
   two from `begin_invoke`, 90 Ir, plus one each from the `Box` clone and the `CallTail` push), the
   +76 libc column, outside the gate measure.
2. **`ops_loop_steady`'s entry block grew.** `fibfunc` per call: `ops_loop_steady` self +50, of which
   +44 is on line-0 (compiler-generated) instructions. Instruction-level: the block run once per
   entry (two entries per call) grew from 61 to 72 instructions, the prologue unchanged. The added
   instructions spill loop-invariant values to the stack: the addresses of five `Interp` vectors
   (`lea 0x240/0x2b8/0x870/0x8a0/0xd68(%r14)`), whose stack slots this function reads only at `Vec`
   growth calls and at `variables.rs:283`/`drive.rs:1649`, and one constant address. The Rust source of
   `ops_loop`'s entry is identical in s1 and head (`diff` of the two ranges is empty); the change
   is LLVM's register allocation of a body whose arms changed. Per clause the stepped-clause path
   went from 63 to 65 instructions.
3. **New checks on the call and send paths**, each 2 to 7 Ir per iteration on lines added since
   s1: `stack_exhausted` (`scheduler.rs:1073`, from `eval.rs:89`, `call.rs:835`,
   `dispatch.rs:3125`), `native_park.is_none()` (`call.rs:657`, `drive.rs:2258`),
   `guards.is_live()` (`dispatch.rs:3141`, `call.rs:1032`), `ActivationFlags::method`
   (`activation.rs:508`), the guard-kind match (`guards.rs:303`), `pools_in(..).set`
   (`variables.rs:112`, `dispatch` only). These carry behaviour (Error 11, parking, guards).

`dispatchclass` has the same function list as `dispatch` without the exposed-variable rows.
`fibcall` matches `fibfunc`.

## What a round can take

Kind 1 is behaviour-free: restore the inlining or remove the work (one of the two name copies).
Kind 2 is codegen of the steady loop; P19 forbids iterating on layout, and the cause is the loop
body as a whole, not one change. Kind 3 carries behaviour and stays.

Budget distance (from `cg-table.txt`): `dispatch` needs about -106 Ir per send, `sendloop` -59 per
send, `fibfunc` -62 per call, `fibcall` -28 per call, `dispatchclass` -3.5 per send. Kind 1 has
no rows on `fibfunc` or `fibcall`.

## Measured after the diagnosis

- Forcing `FrameArena::reserve` inline moves no program by more than 0.05%
  (`experiments/expA-table.txt`): the call itself is not the cost.
- Round 1 (`ba8f7c581`, one shared name per invocation) removed 49 Ir per send on `sendloop` and
  `dispatch` and 21 Ir per `fibfunc` call (`r1/cg-r1-table.txt` against `cg-table.txt`).
- Sharing the name across invocations whose name repeats (no allocation at all) removed a further
  5 Ir per send and nothing per call (`experiments/expD-table.txt`): what is left of an allocation
  in the gate measure is small, because malloc and free are libc.
- With libc kept, round 1 against base is below base on `fibfunc` (-0.18%) and `fibcall`
  (-2.00%) and above on `dispatch` (+2.19%) and `sendloop` (+1.05%).
