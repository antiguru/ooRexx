## Task 6: the flip

**Goal.** `5a` in `CLOSED_PHASES`, and the five gate commands green with the tables' gating arm live.

**Build.** The one line, at `rust/crates/rexx-exec/tests/gate_tables/mod.rs`. The previous plan's
Task 24 verified that `CLOSED_PHASES` is the switch it appears to be, by reddening the identical rows
two ways -- through the constant and through `REXX_PHASE_GATE=5a` -- so this task inherits that and
does not re-derive it.

**Done when** all five gate commands exit 0 with the flip committed, and a negative control is
recorded **in the shape Task 24 established**: revert one mechanism a 5a row depends on, confirm the
gate command exits non-zero, and confirm it reddens **a nameable set rather than everything**. A
control that reddens the whole table has usually broken the bootstrap and witnesses far less.

**If a sixth row appears** once the five are closed -- a row whose owning phase is 5a and whose
verdict is not `agree` -- that is a finding and the report says so. Do not re-file a row to another
phase to make the gate pass: that closes a gate row by narrowing what the gate covers, which Task 24
declined to do for `::ATTRIBUTE EXTERNAL` and this plan will not do either.

## Controller addendum, written after Tasks 1 through 5

**All five 5a rows now agree**, each verified by me directly -- not read out of a report -- against
the oracle on stdout, stderr and exit status separately, on both engines:

| row | rc | task |
| --- | --- | --- |
| `directives/attribute__external__subkeyword.rex` | 166 | 1 |
| `concepts/xscope.rex` | 159 | 2 |
| `concepts/usingcl.rex` | 0 | 3 |
| `classes/rexxinfo.rex` | 159 | 4 |
| `concepts/methna.rex` | 0 | 5 |

### Your negative control, and the trap it has to clear

The brief asks for a control that reddens **a nameable set rather than everything**. Task 5 has
already handed you a validated recipe for one, and more usefully, the reason the plan's *previous*
recipe was worthless.

**The plan's original control for `methna` could not fail.** It said to keep the as-written spelling
as the dictionary key. That is a **no-op**: `MethodDict` upcases on insert, and its lookups upcase separately in their own functions,
so changing one site leaves the row `agree` and the corpus at 264 of 264. Both sites have to stop
upcasing together -- `dispatch.rs`'s `method_name_pair` and `MethodDict::replace_method` -- and then
exactly one row reddens. That is now corrected in the plan and in gate table C's `control` string,
with the measurement.

**So: run your control and watch it fail, then invert it and watch it pass.** A control you reasoned
about is not a control. This plan has now shipped two criteria that were achievable, true-sounding
and unable to discriminate, and both were caught only by running them.

**And check what your control reddens, not just that it reddens.** A control that turns the whole
table red has usually broken the bootstrap and witnesses far less than one that reddens a row you can
name in advance. Say which rows went red and why those.

### What is already known about the switch, so you do not re-derive it

The previous plan's Task 24 established that `CLOSED_PHASES` is the switch it appears to be, by
reddening identical rows two ways -- through the constant and through `REXX_PHASE_GATE=5a`. Inherit
that; do not re-derive it.

### If a sixth row appears

The plan is explicit and I am not relaxing it: a row whose owning phase is 5a and whose verdict is
not `agree` is a **finding**, and the report says so. Do not re-file a row to another phase to make
the gate pass. That closes a gate by narrowing what it covers, which the previous plan's Task 24
declined to do for `::ATTRIBUTE EXTERNAL`, and this plan will not do either.

### Gates

`scratchpad/controller-gates.sh <sha> <logdir>` gates a committed SHA in two parallel worktrees. It
now takes an **`flock`** first -- a concurrent run refuses with exit 3 rather than queueing, after one
collided with another run and voided its G5. It does not run G3, deliberately: G4 is G3 plus
`REXX_CORPUS_GATE=1`, which only adds checks, so G4 strictly covers it. Read every status from its own
`.rc` file, never from a completion notification.
