# The IR's fixed per-clause cost: `nop` and `x = y`

Branch `perf/clause-overhead` from `eba87258d` (`plan/rust-rewrite`). Brief:
`round4/clause-overhead.md` in the session scratchpad. Scripts, range files and
listings: `clause-overhead-files/`.

## 1. Accounting, on the base, before any code change

Base binary: `cargo build --release -p rexx-exec --bin rexx-run` at
`aef537563` (the bench-axis commit; no interpreter change from `eba87258d`),
`.text` sha256 `54fdd95a8316320498fc221796fe9b24aecdf593dd45806f9b8983351af5bec6`.

**Method.** One clause is the difference between a 100-clause and a 50-clause
body over the same 20,000-pass loop (`gen2.py`; prelude `y = 'abc'`,
`x = ''`), divided by 1,000,000. Callgrind `--dump-instr=yes
--compress-pos=no --compress-strings=no`, so every cost line carries its own
address and line and there is no subposition decoding to get wrong;
`addrtsv.py` (from `tw-vs-oracle-files/`) diffs per instruction with libc and
ld-linux excluded, and `classify.py` assigns every row to a purpose by address
range (`ranges.base.txt`). Whole-run check, `sumobj.py` `summary:` minus libc
and ld-linux: `nop` (270,684,806 - 168,576,563) / 1e6 = **102.11**, `x = y`
(528,947,973 - 297,708,628) / 1e6 = **231.24**. The per-instruction rows sum
to 102.0 and 231.0 (rows under 0.01 per clause dropped: the source-size
difference, parsed once). Every row in both listings is 1.000 or 2.000 per
clause.

**Op stream** (`rexx-ir`): a `nop` is one `Clause index end` op with an empty
region; `x = y` is `Clause`, `Load read=Simple at=1 dst=2`,
`Store index at=3 src=2`. Everything a `nop` costs is inlined into
`Interp::run_ops_from::<true>`; `x = y` adds one out-of-line call,
`Interp::read_at`.

### One `nop`: 102 instructions, against the oracle's 33.3

| purpose | ours | where (base) |
|---|---:|---|
| op fetch and dispatch: `pc >= stop`, two reloads LLVM hoisted there, op tag, jump-table range check, jump | 16 | `drive.rs:523`-`548` |
| clause lookup: `code.body.instructions.get(index)` bounds, `end` spill | 8 | `drive.rs:555`-`566` |
| procedure permission: `granting` test and its join (5), `region_procedure_permitted = take(procedure_permitted)` (3) | 8 | `drive.rs:569`, `:642` |
| trace state: `chunk_trace()` with its `debug_pause` cmov (5), `position_at` (6), the shortcut's second position test and `clause_line_override` (4), indent (4) and `instructions_traced_at_entry` (2), `stale` (2), `Echo::Gated` test and spills (4), `debugging` test (2) | 29 | `trace.rs:637`, `run.rs:4907`-`4978`, `drive.rs:606`, `:1933` |
| region walk for an empty region: `header = None` (1), `ops_in` bounds (5), iterator empty test (4), `header`'s drop check and reloads (5) | 15 | `drive.rs:661`-`665`, drop glue |
| bookkeeping both engines do: deadline countdown (2, plus 3 spills scheduled there), running activation (3), `clock_stale` + `trace_entry.stepped()` (6), line and clause index (3), `value_buffer.clear()` (1), `current_clause_line` (1), `pending_traps.is_empty()` (2) | 21 | `clause.rs:265`-`274`, `run.rs:4894`-`4968` |
| rooting: `push_frame` (2), `pop_frame` (3) | 5 | `roots.rs` via `run.rs` |
| **total** | **102** | 30 of them `%rsp`-relative |

The oracle's 33.3 (`tw-vs-oracle-files/tsv/nop.oracle.tsv`):
`RexxActivation::run` 25.26 and `RexxInstructionNop::execute` 8.0. By
`tw-vs-oracle.md`'s hand classification: trace tests 6, loop and virtual-call
layering 10, bookkeeping 17 (counter, `current`/`next`, `stack.clear()`,
`timeStamp.valid`, `clauseBoundary`, `traceEntryAllowed`).

**Our surplus, 69 per clause, named:**
* **Trace state, 29 against 6 (+23).** Every clause decides staleness, echo
  and debug, and computes indent and line, eagerly; the oracle's trace tests
  are bit tests inside `execute`. Of the 29, 4 are the `debug_pause` cmov
  inside `chunk_trace()` and its spill, re-derived each clause from two bytes
  where one would do.
* **Region walk, 15 against 0.** A clause is a region the driver iterates; for
  a `nop` that is an empty slice built with two bounds checks, plus an
  `Option<LoopHeaderValues>` initialised and drop-checked on every clause
  although only a loop header's region ever fills it.
* **Dispatch and clause lookup, 24 against about 10.** The outer op match
  reloads two spilled values and bounds-checks the instruction index.
* **Procedure permission, 8 against 0.** `run_ops_from::<true>` tests
  `granting` on every clause, though it is true only until an activation's
  first non-label clause; the take writes `false` over `false` thereafter.
* **Bookkeeping and rooting, 26 against 17 (+9).** Comparable work; the
  running-activation `Option` test and the `trace_entry` table shift are the
  difference.
* **Spills.** 30 of the 102 address `%rsp`, spread across every row above
  rather than a row of their own (`driver-frame-pressure` measured the same
  1.4 KB frame).

### One `x = y`: 231 instructions, against the oracle's 99.3

| purpose | ours | oracle |
|---|---:|---:|
| the clause as a `nop` (the empty-region test is 3 here, not 4) | 101 | 33.3 |
| non-empty region: iterator setup and its spills | 12 | -- |
| per region op (x2): iterator step 5, `tracing_intermediates()` test LLVM unswitched in front of the region dispatch 4, jump table 5 | 28 | -- |
| `Load`: arm operands and `at` decode 12, call and `novalue` test 5, `read_at` 31 (prologue/epilogue 10, running activation 3, exposure test 2, alias count 2, bounds 2, `Option` test 2, the rest moves), register write 3 | 51 | `RexxSimpleVariable::evaluate` 21 |
| `Store`: arm operands, `Assignment` target-kind test, `trace_mode().results` test, running activation, exposure test (27); `set_frame_slot`: alias count, bounds, tagged write (12) | 39 | `RexxSimpleVariable::assign` 24 |
| the oracle's `RexxInstructionAssignment::execute`, which calls both and holds the trace tests | -- | 29 |
| **total** | **231** | **99.3** |

**Surplus beyond the `nop`'s, 60:** the region machinery is 40 (12 + 28)
where the oracle's assignment instruction is 29 including its trace tests;
the read costs 30 more than the oracle's, 10 of it the out-of-line call's
prologue and epilogue and 7 the running-activation, exposure and alias tests
that the store repeats; the store costs 15 more, the same tests again.
Neither side allocates.

Of the walker's candidates, not found on the IR: no name-resolved target (the
`Store` carries its slot, as `tw-vs-oracle.md` said), no `Result` threading on
the hot exit (the `finish_plain_clause` path, `drive.rs:1925`), no line
lookup (a table load, `position_at`).
