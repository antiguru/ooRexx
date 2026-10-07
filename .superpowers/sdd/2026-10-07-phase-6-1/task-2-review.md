# Phase 6.1 Task 2 review (3cee3e622..81a8c1bae)

Reviewed in one pass over `review-3cee3e622..81a8c1bae.diff`. The `emptyloop` wall-clock overrun is
with Moritz and is not judged here. Probes ran on the gate record's r1 binary (sha256 `0d08596f...`,
copied to `/tmp/claude-1000/p61/t2r/bin/rexx-run`). `git diff --stat 9777db504 81a8c1bae --
rust/crates/rexx-exec/src rust/crates/rexx-parse/src` is empty, so that binary is HEAD's code. Probe
sources are in `/tmp/claude-1000/p61/t2r/src/`. Oracle runs used the global wrapper from fresh
directories (`/tmp/claude-1000/p61/t2r/cmp.sh`).

### Spec Compliance

- ✅ Spec compliant for Steps 1-3 as written. Every scout c1 probe has a corpus witness in
  `rust/corpus/phase-6-1.txt:588-608`, along with trace, error-path and subclass witnesses the brief
  did not ask for. The refusal is gone from `run_loop_with_header` (diff 1422-1431) and from
  `flat_loop_start` (diff 1561-1567). Its pins are replaced in `run/tests/loops.rs:196-228`,
  `owners.rs:213,325`, `ir_recorded_cases/loop-counter-with-stem` and `scheduler/tests.rs`.
  `refusal-sites.tsv` was re-derived. COUNTER code runs only for a loop that names one: the IR slot
  is pushed only when `body.counter.is_some()` (`loop_header_plan`, diff 1166-1168), and the flat path
  enters `flat_loop_header_general` only on `PassTest::General` (diff 1705).
- ✅ The scheduler tests' replacement refusal, `Loud::immovable_reply`, is the LIMIT the brief asks
  for. The spec names "immovable REPLY" as a design-limit refusal at
  `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md:167-168`.
- ⚠️ Cannot verify from the diff: spec exit criterion 5 (`loose-ends.md:253-260`) needs
  TRACE_TraceObject TEST_CALLER_STACK_FRAME_REPLY_START and TRACE TEST_TRACE_LABEL_WITH_FORWARD
  either to pass or to have a reason in the gate record naming a cause outside 6.1's items. The
  causes are recorded in the `concurrency_tests.rs` notes and the report, but not in
  `phase-6-1-gate.md`'s `## Task 2`. The controller should also decide whether the FORWARD/REPLY
  trace gap (probe p7 below) falls under a 6.1 item (D1's REPLY work) or outside 6.1. The queued
  item `2026-10-02-do-with-over-refusal` still has to be closed (report Concern 4).

### Strengths

- The oracle's ordering is reproduced and witnessed. COUNTER is reset to 0 before any header
  expression runs (`Op::Const` plus `LoopHeaderValue{Counter}`, diff 694-702). `do_counter_trace.rex`
  shows `C => "0"` read inside TO. It is assigned after WHILE and before the body. WITH's target
  is echoed, then `SUPPLIER` is sent before FOR, and INDEX then ITEM are bound before FOR is
  consulted. These rules match `WithLoop::setup`/`checkIteration`
  (`interpreter/instructions/DoBlockComponents.cpp:332-403`) and `DoBlock::setCounter`
  (`DoBlock.cpp:122-131`).
- `do_with_errors.rex` covers each failure the oracle raises (97.1, 98.994 including a `NIL`/non-supplier
  answer, 34.906). `do_with_supplier_subclass.rex` shows the message protocol.
- `PassTest` keeps loops with no counter on the old inlined header. Spelling out `in_counted_clause` at
  diff 1727-1741 is equivalent to the closure form: I checked it against `clause.rs:234-244` and
  `:371-391`, and the failure path blames the same way.
- The rewritten whole-group lines (4d46b76b0) record gaps that are now visible. They do not hide a
  regression. Checks below.

### Named-risk checks

| risk | check | result |
|---|---|---|
| LEAVE / ITERATE / SIGNAL with COUNTER and WITH | `p2.rex`: a WITH loop left by a trapped SYNTAX, entered twice through `call t`; an inner WITH loop doing `iterate i` on its outer COUNTER loop; `leave i` from an inner COUNTER loop | identical to the oracle on stdout/stderr/rc, normal and `REXX_SWITCH_MODE=every`: `h 2 2 x` twice, `3 2 4 y`, `1 2 1 2` |
| TRACE | `p3.rex`, `trace r`: COUNTER+WITH+UNTIL, COUNTER+repeat+WHILE with ITERATE, `loop counter ... forever` + LEAVE, COUNTER+TO/BY/FOR | stderr byte-identical to the oracle (55+ lines, COUNTER/WITH/UNTIL/WHILE `>K>` order and indents) |
| REPLY inside the loop body | `p4.rex` (REPLY from inside a COUNTER+WITH loop in a method, then a second COUNTER loop on the continuation), `p5.rex` (REPLY in a COUNTER repeat loop); 5 runs each | normal mode identical to the oracle 5 of 5 each. `every` mode prints `got ...` after `body 2` 5 of 5. That is a different legal interleaving of main and the continuation, and counter/index/item values agree |
| stem OVER with a COUNTER, tail added mid-loop | `p6.rex` | identical (`2 2`) |
| `run_repeating` reachability | read `ir/drive.rs:1607-1641` (the only caller of `run_loop_with_header`) and `grep -rn Fallback` (only construction: `run/loops.rs:1573`, `LoopKind::Simple`, labelled) | unreachable by construction. The only `Fallback` is a labelled simple block, and `run_loop_with_header`'s `Simple` arm returns (`run/loops.rs:960-1000`) before `run_repeating` is called. No program reaches it. See M1 |
| whole-group rewrites hide a regression | TRACE LABEL_WITH_FORWARD: `p7.rex` is the test's `test_forwarded3.rex` resource run with no loop at all. TraceObject VARIABLE/COLLECTOR: `p8.rex` collects trace objects and counts them with a plain `do t over` and with `do counter c t over` | p7: oracle 12 `>I>/<I<` lines and ours 10, 5 of 5 runs, with no loop involved. p8: ours collects 0 trace objects where the oracle collects 15, and the plain and COUNTER loops agree with each other (`c = m` is 1). Both gaps are independent of loops, so the rewritten lines record gaps that are now visible |

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

**I1. COUNTER and DO WITH leak one rooted temporary per value per pass, for the life of the loop.**
`run/loops.rs:727` (`count_pass`: `push_temp(value)`) and `run/loops.rs:2509` (`with_advance`:
`push_temp(value)` for INDEX and for ITEM) push onto the activity's temps stack on every pass, and
nothing pops them before the loop ends. The flat header step (`enter_clause`/`leave_clause`) opens no
temps frame. The controlled loop's own per-pass bind does this correctly: it brackets the bind with
`push_frame()`/`pop_frame(pass)` (`run/loops.rs:2264`, `:2394`). Measured with `/usr/bin/time -f %M` on
the binary directly:

| program | ours | oracle | control |
|---|---:|---:|---:|
| `m1b.rex`: `do counter c 30000000; end` | 255,900 kB | 20,060 kB | `do i = 1 to 30000000` (m5) 19,316 kB |
| `m1c`/`m1`/`m1b`: 0.3M / 3M / 30M passes | 22,560 / 44,588 / 255,900 kB | | linear, about 8 bytes per pass |
| `m1n.rex`, `m1w.rex` (body `nop`; WHILE form) | 255,680 / 255,700 kB | | a body clause does not release it |
| `m8.rex`: `do with index i item v over` an endless Supplier subclass, LEAVE at 2M | 51,340 kB | 20,884 kB | the same supplier stepped by hand (m9) 19,084 kB |

Besides the stack's growth, every INDEX/ITEM value stays rooted and uncollectable until the loop
exits. A `DO WITH` over a large collection, or a long `DO COUNTER ... FOREVER` service loop, grows
without bound where the oracle stays flat. Fix: bracket `count_pass`'s and `with_advance`'s binds
in `push_frame`/`pop_frame` the way `loop_advance`'s controlled arm does. Under the global
constraints this needs a crate test. The existing `temps_len()` pattern fits:
`run/tests/conditions.rs:417-430`. Assert `temps_len()` after N passes equals it after 2N, for COUNTER and for WITH.

#### Minor (Nice to Have)

- **M1.** `run_repeating` and `run_loop_with_header`'s non-`Simple` arms are dead (table above). This
  task added more code to them: the `counter` parameter and the `count_pass` call at
  `run/loops.rs:1302`, and the `LoopKind::With` arm calling `with_state` (diff 1454-1458). Nothing
  tests that code, and nothing can, so whether it is right rests on reading alone. Deleting it, or
  replacing those arms with an `unreachable!` that names `flat_loop_start:1573`, is a separate
  decision, as the report says. Recommended before more features are threaded through it.
- **M2.** `with_advance` recomputes `control_slot(code, name)` and `shape_of(...)` for INDEX and ITEM
  on every pass (`run/loops.rs:2515-2516`). `LoopCounter` caches the same pair once at entry
  (`:325-331`). Caching them in `WithState` removes a name scan and a slot lookup per variable per
  pass.
- **M3.** `HeaderRole::OverFor`'s doc (`run/loops.rs:384-389`) describes only
  `DO name OVER expr FOR expr`. It is now also DO WITH's FOR (`loop_header_plan`, diff 1202-1206;
  `header_expr_for`, diff 1225).
- **M4.** The report's "The oracle calls the base class's natives directly" is half true.
  `SupplierClass::loopAvailable`/`loopNext`/`loopIndex`/`loopItem`
  (`interpreter/classes/SupplierClass.cpp:147-160, 196-209, 233-246, 284-297`) call natives only when
  `isBaseClass()` is true. For a subclass or an enhanced instance they send the message. The
  conclusion still holds: a primitive Supplier's methods cannot be replaced, so always sending is
  observably the same. Only the report's sentence is affected, not the code.
- **M5.** The rewrapped comments leave overlong lines: `run/loops.rs` FlatLoop comment (diff
  1633-1634) and `phase-4-exclusions.txt` (diff 265).

### Assessment

**Task quality:** Needs fixes

**Reasoning:** Behaviour matches the oracle on every witness. It also matches on the reviewer's
probes for LEAVE, ITERATE, SIGNAL, REPLY, TRACE and stem OVER, and the rewritten whole-group lines
are gaps that are now visible, not regressions. The per-pass temp push in `count_pass` and
`with_advance` makes COUNTER and DO WITH loops grow memory linearly with the number of passes (13x
the oracle's peak at 30M passes) and keeps every bound value alive. That needs a frame per pass and
a crate test before this task can be trusted.
