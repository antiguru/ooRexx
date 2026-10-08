# Task 3 fix round 1 re-review: 75c41b836..d9192ef4a

Ours: `rexx-run` built in release from a worktree at `d9192ef4a`, its own target dir (both since removed).
Oracle: the standard wrapper. Each probe ran from its own empty dir, both engines in the same path, through
`/tmp/claude-1000/p61/t3rr/cmp.sh`. Probes are in `/tmp/claude-1000/p61/t3rr/probes/` and outputs in
`/tmp/claude-1000/p61/t3rr/out/<probe>/`.

### Finding Verdicts

- **Important b10: a user `+` answering a non-canonical string diverged silently** -- ADDRESSED.
  `accept_object_header` keeps each answer as it is (diff `:1154-1185`), and `ObjectControlled` binds
  the `+` answer unchanged (diff `:1738-1754`). The first review's probes now agree on all three
  descriptors: `do_plus_string.rex`, `do_plus_trace.rex`, `do_raise_untrapped_ctrl.rex` and
  `do_raise_untrapped_hdr.rex` (rc 163 both). In `do_raise_trapped.rex`, t4 loops until the kill on both
  sides (rc 124 both). The exclusions sentence is corrected (diff `:185-189`). Witness:
  `do_object_control` t3 and t9. But see New Breakage 2, a different divergence in the same comparison.
- **Ruled in: object control values and DO OVER `.context~package~local` match the oracle; delete
  `Loud::object_position`** -- ADDRESSED.
  - `four.rex` runs the four shapes from the first round's Concern 1: `.environment to 5`, `.local to 3`,
    `1 to .local`, and a control set to `.local` with no TO. Identical on all three descriptors, rc 159
    both.
  - `object_position`, `value_name`, `over_target_gap`, `header_object_number` and
    `controlled_step_object` have no reference left in `rust/` (grep run before the worktree was removed).
  - DO OVER package local runs (`do_over_package_local`). With several keys its order is sorted where
    the oracle uses hash order (`over_pl.rex`: stdout differs in order only, rc 0 both). That is
    Deviation 8's licence (`phase-4-exclusions.txt:1379`). `over_env.rex` and `over_local.rex` are
    identical.
- **the_s2_rows TEST_TRACEOBJECT_COLLECTOR: bisect, then a fix or an expectation with its reason** --
  ADDRESSED.
  - The bisect and its reason are in the gate record (diff `:216-225`).
  - Named risk: whether the reordering is a legal interleaving. It is. `b=b` runs on method B's REPLY
    continuation and `pp(...)` on TEST's, two separate activities.
  - On the oracle, `coll.rex` (the resource's class, standalone, `/tmp/claude-1000/p61/t3rr/coll/`)
    gave rc 0 in 10 of 10 runs, with 3 distinct stderr orders across those runs (`md5sum o*.err`). The
    two continuations' lines interleave differently from run to run.
  - Ours gives one order per mode, stable in 5 of 5 runs each: `b=b` comes after PP in normal mode and
    before it in `every` mode.
  - The allowance is wider than an interleaving: see Minor 4.
- **Records: exclusions known-gap row; oracle-crashes 28 and 29** -- ADDRESSED.
  - The row's program (`anyrow.rex`) reproduces: the oracle prints `back`, ours prints `D=[] NOVALUE`
    then `back`, rc 0 both. The LOSTDIGITS form (`anyrow2.rex`) diverges the same way.
  - Entry 28's program as written is SIGSEGV rc 139 on the oracle (2 of 2 runs here). Ours is 98.946,
    rc 158.
  - Entry 29's program as written is killed at the timeout on the oracle (rc 137, 1 run here). Ours
    prints `woke 1`, rc 0.
  - See Minor 1 for one quoted output that is wrong.

### New Breakage in the Fix Diff

1. **Critical: a numeric loop whose control becomes an object loses a heap TO or BY to the collector.
   The release binary panics (rc 101) on a plain run.**
   - Where:
     - `object_control_from_numbers` (`run/loops.rs:2628`) builds the TO and BY objects as temps inside
       the switching pass's frame. That frame is popped at `loops.rs:2440`.
     - After that, only `ObjectControl` holds them, and `Activity::object_roots` reaches it only through
       `flat_top`/`flat_loops` (`activity.rs:542`).
     - At every later pass boundary, `flat_loop_step_top` takes the loop out of `flat_top`
       (`loops.rs:1794`). `object_control_advance` (`loops.rs:2605`) then sends `+` and the TO
       comparison without pushing `ctl.to` or `ctl.by` as temps.
     - A user `+` that allocates can therefore collect them, and the next send finds them dead.
   - Measured, `dead2.rex`: `do n = 1 to 123456789012.5 by 1.25`. The body sets `n = .k~new(10)` once
     `n > 3`, and leaves after 20000 passes. The `+` and `>` methods allocate a small array.
     - Oracle: rc 0, `end k10 20001`.
     - Ours: rc 101, `panicked at crates/rexx-exec/src/dispatch.rs:1660:9: a live value`, nothing on
       stdout. 3 of 3 runs for `dead3`.
   - Variants pin the cause to TO and BY:
     - `dead7.rex` (heap TO, `by 1`): panics.
     - `dead3.rex` (`to 99999`, `by 1.25`): panics in 3 of 3 runs.
     - `dead6.rex` (`to 99999`, `by 1`, neither on the heap): identical to the oracle.
   - Under collect-on-every-allocation (a local test added to the worktree copy only, `stress.log`), the
     same program fails on the first pass after the switch: `rexx-exec: a message send to a value whose
     object is no longer live`. In that same run, the two loops whose header holds an object survive with
     a heap TO (`.t~new(4)`) and BY (`.b~new(2)`). Their answers are temps in the DO instruction's
     region.
   - The witnesses miss it. `do_object_control` t2 and t8 and `do_object_control_trace` switch only with
     TO and BY small enough to be inline, so the corpus stress run cannot see this.
   - This is the named risk "a control value that changes type mid-loop".
2. **Important: the TO comparison and BY's direction test compare against the value `'1'`, where the
   oracle tests for the object `TheTrueObject`.** A `>`, `<` or BY `<` that answers a computed `'1'`
   ends or reverses the loop here, and does not on the oracle.
   - Where: `is_true_object` (`run/loops.rs:2943`) is `answer == LOGICAL_TRUE`, and `LOGICAL_TRUE` is
     `ObjRef::inline_byte(b'1')` (`eval.rs:33`). Any one-byte `'1'` string matches it. The oracle's
     test is `== TheTrueObject` (`DoBlock.cpp`, `checkControl`, and `DoBlockComponents.cpp:173`).
   - Measured, `gt_one2.rex`: a `>` answering `'1' || ''`, `left('12', 1)`, `d2c(49)`, `1~string`,
     `.true~copy` or `.true~string`, with `to 3 for 3`.
     - Oracle: 3 passes each, rc 0.
     - Ours: 0 passes each, rc 0, stdout only.
   - `by_dir2.rex`: BY's `<` answers `left('12', 1)`.
     - Oracle: ascending. One `>` is sent and the loop ends.
     - Ours: descending. 4 passes, each sending `<`.
   - `gt_one.rex` (a literal `1`, `'1'`, `0 + 1`, a comparison, `.true`) agrees. So the literals happen
     to differ from the inline value, and only computed answers diverge.
   - This is a silent stdout divergence on the path this round added. No witness or exclusions row
     covers it.

### Minor

1. The exclusions known-gap row says the LOSTDIGITS form prints `h LOSTDIGITS`
   (`phase-4-exclusions.txt:3268`). That row's program with LOSTDIGITS prints `D=[] LOSTDIGITS`
   (`anyrow2.rex`).
2. `over_snapshot`'s comment still calls the sorted walk "the one collection whose order is this
   crate's" (`run/loops.rs:831`), and `hash_collection_indexes`' doc still says "A `StringTable`'s
   indexes". Since `is_hash_collection` (`:819`) admits a native `Directory`, both are now incomplete.
   Deviation 8's WHY paragraph (`phase-4-exclusions.txt:1411`) also names only `StringTable`.
3. `by_message_first_pass` blames a first-test failure on the DO clause, and that is witnessed only
   by `do_object_control` t1 (trapped). `err_first.rex` (untrapped, traceback) and `wu.rex` (WHILE,
   UNTIL, FOR 0, a labelled LEAVE and ITERATE, DIGITS 3) agree on all three descriptors, so nothing is
   wrong. The untrapped traceback has no witness.
4. The `TRACE_INTERLEAVES` widening (`concurrency_tests.rs`, diff `:1855-1875`) compares sorted stderr
   lines, so it also accepts reordering within one activity, not just between the two continuations.
   It applies to one row only, and that row fails against the oracle in both modes anyway (97.1, 2
   assertions).

### Checks run (all oracle and ours unless noted, identical on all three descriptors unless noted)

- Type change mid-loop, the named risk:
  - `mixed.rex`: object to number to object, a heap TO object, FOR, a number loop with an object TO, and
    a switch in the body.
  - `mixed2.rex`: `trace r` over an object whose `+` answers numbers, a switch with BY 2, and a switch
    then a self-assignment.
  - `trace_switch.rex`: `trace r` across a switch.
  - `switch.rex`: TO and BY text kept across a switch: the oracle passes `[0.50]` and `[3.50]`, and so
    does ours.
  - All identical. The rooting failure in New Breakage 1 needs heap TO or BY and an allocating `+`,
    which none of these has.
- `label.rex`, a labelled object loop with ITERATE and LEAVE by label, and a labelled switch: identical,
  and identical again under `REXX_SWITCH_MODE=every`.
- `reply_loop.rex`, an object loop after REPLY: identical (1 run).
- `by_dir.rex`, BY's `<` answering a literal `'1'`: identical.
- Report checks: the fix report names `cargo fmt`, clippy, the workspace tests, both corpus gates and
  the gated `concurrency_tests` file, with exit statuses at `150611a38`. Not re-run.

### Out-of-Scope Observations

- Ours omits the `>I>`/`<I<` method entry and exit lines that the oracle prints under `trace all` in
  `coll.rex`, across REPLY continuations: 14 lines missing from the multiset. This is the REPLY trace gap
  already queued at Task 12 (Task 2 ruling).

### Verdict

**Fix round:** Findings remain open. All four findings are addressed, but the fix introduced two
defects: Critical New Breakage 1 (an unrooted TO or BY after a control switches to an object, which
panics with `a live value`) and Important New Breakage 2 (`is_true_object` matches any `'1'` rather than
the true object).
