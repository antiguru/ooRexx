# Task 3 review: 1d308cb9d..75c41b836

### Spec Compliance

- ❌ Issues found: b10's "the loop runs on that answer" path turns a shape that was loud at base into a
  silent divergence (Important 1). Everything else in c2, c3, c4, c5/c6, b1, b2 and b10 matches the brief.
- ⚠️ Deviations from the brief's wording that are correct. Check them, but none needs a fix:
  - `Loud::object_position` is kept (`lib.rs:2091` in the diff). It is still constructed at
    `run/loops.rs` `header_object_number` and `controlled_step_object`, and by DO OVER's directory refusal.
    The implementer's Concern 1 is accurate. The brief's "delete each refusal's constructor" cannot apply
    to a constructor with live callers.
  - `owners.rs` keeps an `Options` row as `InScope` (diff `:3619`) where the brief says to remove its tag.
    `tags!` matches every `InstructionKind` variant, so a row has to stay. `INSTRUCTION_TAGS.len()`
    stays 43, `EXPECTED_OUT_OF_SCOPE` is empty, and the Phase 5 count is 0 (`:3693`) as the brief asks.
  - The witness names differ from the scout probe names the brief lists (`use_arg_msg`, not
    `c_use_arg_msg`), and several b10 probes are merged into `do_header_objects` and `do_ctrl_objects`.
    Every group the brief names has a witness in `rust/corpus/phase-6-1.txt` (diff `:1001-1039`).

### Strengths

- `Interp::pool_value` and `set_pool_value` (`variables.rs`, diff `:3094-3149`) give accessors and
  `DELEGATE` a single path for simple, stem and compound names. Both old copies in `dispatch.rs` are
  gone. `direct_compound` takes the tail literally, and `attr_compound`'s `o~a.i` line witnesses that
  against the pool's `I`.
- USE LOCAL reuses EXPOSE's exposure list through the new `expose_slot` (`run.rs`, diff `:2420`), so
  GUARD WHEN watches auto-exposed names without new machinery. The dynamic-name hook in `slot_of`
  (`plan.rs`, diff `:2252`) runs only when an extra slot is created. PROCEDURE clears the auto-expose
  (diff `:2390`). `object_roots` roots the owner and scope (diff `:1400`).
- RAISE now traces `>K>` before `requestArray` runs (`condition.rs`, diff `:2753-2763`), which is the
  oracle's order. The condition object carries the converted array.
- The whole-group rewrites in `61d441781` record gaps and hide no regression. I checked how the rest
  part is built (`concurrency_tests.rs:3200-3250`): it reruns the whole group without each test that
  refuses, one at a time. TESTCONDITION01 and TEST_TRACE_OPTIONS used to refuse and were left out. Now
  they run. That explains each count change: RexxContext 354 to 357, and TRACE 117 to 118, where
  TEST_TRACE_OPTIONS' `assertTraceOutput` is the extra assertion (ootest `TRACE.testGroup:619-630`).
  The failing sets are unchanged. The new whole-group refusal, TESTCOPY01, is a later test refusing for
  a Phase 9 reason. Deleting the GUARD `DIFFERING` rows makes the derived part strict against the oracle.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **A user `+` that answers a non-canonical string is a silent divergence. At base it was loud.**
   - Where: `run/loops.rs`, `header_object_number` (diff `:2857-2868`, `return self.header_number(role,
     answer)`) and `controlled_step_object` (diff `:2968-2998`, `return self.arith_operand(answer)`).
   - What goes wrong: both convert the `+` answer to a `Number`. The oracle uses the answer as it is.
     It assigns it to the control variable unchanged and compares it with the general comparison
     operators, so a non-numeric answer is compared as a string.
   - Measured with probe `/tmp/claude-1000/p61/t3r/probes/do_plus_string.rex`. Oracle and ours are both
     rc 0 with empty stderr. Only stdout differs (`/tmp/claude-1000/p61/t3r/out/do_plus_string/`):
     - control variable whose `+` answers `'abc'`: the oracle prints `t1 end abc` (the loop ends because
       `'abc' > 3` holds). Ours gives `t1 trapped 41`.
     - initial value whose `+` answers `'abc'`: the oracle prints `t2 end abc`. Ours gives `t2 trapped 41`.
     - initial value whose `+` answers `' 2 '`: the oracle prints `t3 [ 2 ]` on the first pass. Ours
       prints `t3 [2]`.
   - A TO value whose `+` answers `'abc'` (`do_raise_trapped.rex` t4): the oracle loops until the kill
     (rc 124). Ours raises 41.1.
   - Answers like `'1.50'` and `'2.0'` agree. The difference is between the answer's text and its
     numeric value, and by reading `round_via_unary_plus` an answer longer than DIGITS should be
     affected too. That last part was not run.
   - Why it matters: R12 is about not answering wrong where the oracle answers. This task removed the
     refusal and nothing replaced it for this shape. `phase-4-exclusions.txt` now says the DO rows
     "agree" and adds "a value whose `+` answers is a number like any other" (diff `:356-386`). That
     sentence is false for these answers.
   - Fix: bind the answer object itself where the text matters (the initial assignment), or refuse
     through `Loud::object_position` when the answer's text is not its canonical number. Then delete or
     correct the exclusions sentence, and add the three cases above to `do_header_objects` and
     `do_ctrl_objects`.

#### Minor (Nice to Have)

1. **Concern 3's silent divergence is unmasked and recorded only in this task's report and scout A's
   section 4.** The divergence: `call on any`, then `raise novalue return` in a callee. It was loud
   before c4. It runs now, and our handler fires where the oracle prints only `back`. It shares a root
   with `raise lostdigits return`, which already diverged at base. Put it where the project looks:
   an exclusions known-gap row, or a queued item.
2. **Concern 5:** the hand-edited `refusal-sites.tsv` row at `8f080d2fa` was never checked by building
   that commit, so the history may not bisect cleanly there. The tree at HEAD is regenerated, so
   nothing is wrong now.
3. **Concern 2:** `the_s2_rows_of_the_derived_list_in_both_modes` (TRACE_TraceObject
   TEST_TRACEOBJECT_COLLECTOR) fails, and the implementer reproduced the failure at base. It is in the
   gate record, but the brief only names the `whole_groups` filter. Give it an owner before Task 5's
   `whole_groups` close, so that a red gated file is not normalised.
4. **Observation, not this task's defect.** GUARD WHEN on a compound tail is never woken on the oracle.
   It hangs until the kill in all 3 runs of `ctl_method.rex` (the tail is set by an EXPOSE method) and
   all 5 runs each of `attr_guard.rex` (`o~a.b = 1` through the new accessor) and `attr_guard_stem.rex`
   (`o~s.[1] = 1`). Ours wakes and answers `woke 1` in every run. A simple-name control,
   `ctl_simple.rex`, wakes on both sides in 3 of 3 runs. So the compound-attribute-under-GUARD risk adds
   nothing new: the oracle blocks for ever, which is the case `oracle-crashes.txt` entry 7 describes,
   with nothing to match. It may deserve a line there.
5. **Observation, oracle crash.** FORWARD ARGUMENTS over an instance whose `MAKEARRAY` answers a List
   is SIGSEGV rc 139 on the oracle (`req_nonarray.rex`, second line, one run). Ours answers 98.946.
   It is a candidate for `oracle-crashes.txt` and must not be added as a witness.

### Checks run (oracle wrapper from the global constraints, from a fresh empty dir per run; ours is `/tmp/claude-1000/p61/t3/bin/head-rexx-run`, built at `652826b54`; script `/tmp/claude-1000/p61/t3r/cmp.sh`)

- USE LOCAL, first touch by DROP, including `drop (lst)` of a name that is only dynamic (`ul_drop.rex`):
  identical. Both drop the object's variables.
- USE LOCAL, first touch through a compound tail: an unlisted `k` exposed through `a.k`, a listed `k`
  that stays local, and `value('A.K')` / `call value 'Q.1'` (`ul_tail.rex`): identical.
- USE LOCAL with `procedure expose v` and `procedure expose w` in nested internal calls (`ul_proc.rex`):
  identical.
- USE LOCAL after REPLY, with a dynamic name and a listed local (`ul_reply.rex`): identical in 5 of 5
  runs.
- b10, an operator raising inside a loop:
  - trapped, in the control variable and in a header value (`do_raise_trapped.rex` t1, t2): identical
    lines.
  - untrapped, in the control variable (`do_raise_untrapped_ctrl.rex`, rc 163) and in BY
    (`do_raise_untrapped_hdr.rex`, rc 163): identical on all three descriptors, traceback included.
  - `trace r` over a user `+` in the header and in the control variable (`do_plus_trace.rex`): identical.
- RAISE ADDITIONAL where MAKEARRAY answers a string (`req_nonarray2.rex`): identical, 98.939.
- `::ATTRIBUTE` on a compound or stem under GUARD: Minor 4.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The groups are built cleanly on shared helpers, and the witnesses and whole-group
rewrites hold up under the named-risk probes. One fix remains: b10's numeric-answer path changed a loud
refusal into a measured silent stdout divergence, and the exclusions record now calls it agreement.
