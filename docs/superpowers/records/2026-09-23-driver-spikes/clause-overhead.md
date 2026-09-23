# The IR's fixed per-clause cost: `nop` and `x = y`

Branch `perf/clause-overhead` from `eba87258d` (`plan/rust-rewrite`). Brief:
`round4/clause-overhead.md` in the session scratchpad. Scripts, range files and
listings: `clause-overhead-files/`.

## 0. Result

Stopped after three consecutive candidates failed to pay (c7, c8, c9); `nop`
did not reach 2x the oracle. Four of nine candidates kept. Base `.text`
`54fdd95a`, head `.text` `adaacbb0`, two interleaved rounds each
(`results-final.txt`):

| axis | base | head | delta |
|---|---:|---:|---:|
| `nop` | 10,540,100,344 | 10,040,100,248 | -4.744% |
| `assign` | 23,440,362,254 | 20,240,363,278 | -13.652% |
| `rexxcps` | 18,328,368,836 | 18,017,675,768 | -1.695% |
| `varlookup` | 16,457,902,077 | 15,108,897,888 | -8.197% |
| `emptyloop` | 9,585,886,980 | 9,460,885,172 | -1.304% |
| `arith` | 11,674,842,654 | 11,543,837,780 | -1.122% |
| `dispatch` | 20,691,083,588 | 20,556,079,038 | -0.652% |
| `compound` | 9,664,426,975 | 9,304,398,613 | -3.725% |

Per clause (section 1's method on the head): `nop` 102.1 -> **97.1** against
the oracle's 33.3 (2.92x); `x = y` 231.2 -> **199.2** against 99.3 (2.01x).

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

## 2. Candidates, one per commit, measured

Each prediction was written to `predictions.txt` before its measurement.
Each row is callgrind `summary:` minus libc and ld-linux, two interleaved
rounds, against the previous kept head, every binary confirmed by `.text`
hash (`hashes.txt`); per-clause figures by section 1's method.
The null control (base against a second copy of itself, `results-null.txt`)
moved no axis by more than 0.000%, spreads under 12,000 instructions per run.

| # | change | predicted `nop` / `assign` | `nop` | `assign` | `rexxcps` | verdict |
|---|---|---|---:|---:|---:|---|
| c1 | the debug pause folded into `TraceCache`'s byte, so `chunk_trace()` and `tracing_intermediates()` read one byte | -3 / -7 | -1.898% (-2.0) | -2.133% (-5.0) | -0.855% | kept `eb3130f71` |
| c2 | a range continues in `run_ops_from::<false>` once the procedure permission is spent; that instance stores `false` instead of taking | -7 / -7 | -4.836% (-5.0) | -2.180% (-5.0) | -0.165% (-0.179% on four further runs each) | kept `48004b4e7` |
| c3 | the loop-header accumulator declared once per `run_ops_from` | -5 / -5 | -7.093% (-7.0) | +0.455% (+1.0) | +0.157% | not kept, never committed (`c3-header-hoist.patch`) |
| c4 | `Op::Load` with a resolved slot reads it inline (`read_slot`) instead of calling `read_at` | 0 / -12 | +2.032% (+2.0) | -5.793% (-13.0) | -0.570% | kept `47a957d99` |
| c5 | an empty region skips `ops_in`, the iterator and the header | -10..-13 / +2 | -8.964% | +0.946% | +0.131% (three further runs each) | not kept (`c5-empty-region.patch`) |
| c6 | `Op::Store`'s fast path keyed on the resolved slot alone; clause-kind and target-kind tests moved to the slow path | 0 / -8 | +0.000% | -4.257% (-9.0) | -0.101% | kept `18115d7ed` |
| c7 | `read_slot` answers `Result`, the unset read and its `NOVALUE` check in one cold function | 0 / -5 | -3.984% | -1.976% (-4.0) | +0.034% | not kept (`c7-read-result.patch`) |
| c8 | the region walked by index into the cut stream instead of an `ops_in` slice iterator | -5 / -6 | -4.980% | -3.952% | +0.944% | not kept (`c8-index-walk.patch`) |
| c9 | `TraceEntry::stepped` as discriminant arithmetic | -1.5 / -1.5 | +2.988% (+3.0) | +1.482% | +0.290% | not kept (`c9-trace-entry.patch`) |

The rejected candidates were measured as uncommitted working-tree changes and
reversed from their saved patch with `git apply -R`; none reached a commit, so
there is nothing to revert on the branch.

**Attribution of the kept changes.** c1: the `nop` listing's `chunk_trace`
rows went from the movzbl/cmpb/mov/cmovne sequence to one load; the region
ops' unswitched test from four instructions to two. c2: the hot loop moved to
`run_ops_from::<false>`, the granting test and join are gone from its
listing, the take is one store. c4: `read_at` no longer appears in the
`x = y` listing (208 instructions, all in the driver); the `nop`'s +2 is two
stack moves in lines c4 did not touch, the allocation noise the brief
describes, and it is why the head's `nop` is 97 rather than c2's 95. c6:
`x = y` lost the `Assignment` and `Variable` tests (-9 per clause); `nop` and
`emptyloop`, which run no `Store`, moved by 0 and 0. `arith_small_int` and
`read_at` stay out of line in every kept binary (`nm`), so no kept delta is
an inlining flip of those helpers.

**What the rejections show.** Three candidates (c3, c5, c8) removed work from
the `nop` path as predicted and then cost `rexxcps` or `assign` elsewhere;
c7 helped both micro-axes and cost `rexxcps` +6.1M against spreads under
8,000. c9 removed two instructions of work and added three of allocation.
The per-clause path inside `run_ops_from` is where every remaining named
surplus lives, so every further candidate is a driver edit and carries the
allocation lottery the round-1 README measured.

## 3. Gates

Commit first, tree frozen until the status file says `finished`; statuses
and tallies are in section 4 of this file's follow-up commit, written from the
status file.

## 4. Concerns

1. `rexxcps` is not perfectly deterministic here: one of c2's rounds read
   4.76M above the other, and c5's head repeated it once; further runs of each
   binary repeated to within 8,000, and those figures are the ones used.
   Its `TIME` and `DATE` calls are the likely source; not measured.
2. The `nop` per-clause gain is 5.0 of 102 at the head, not the 7 c2 alone
   gave: c4's allocation change took 2 back. Remaining named surplus against
   the oracle: trace state about 27, dispatch and lookup about 24, bookkeeping
   and rooting about 26 against 17, and the empty-region walk 15 that c5 and
   c8 could remove only at a `rexxcps` cost.
3. c2 makes every activation body run its steady state in
   `run_ops_from::<false>` behind one extra call frame (1.4 KB) per body
   entry. The debug `rexx-exec` suite was green on c2 (1581 passed, 0
   failed, 604 of 604); how deep a recursion runs before the interpreter
   thread's stack gives out was not re-measured.
4. The test harness needs both `CARGO_TARGET_DIR` set to the scratch target
   and the `rust/target` symlink: with only the symlink, 24 corpus programs
   and `package_requires` compare a symlinked path against the oracle's
   canonical one; with only the variable, the arity harnesses do not find
   `target/release/rexx-run`. Neither is a code defect.
