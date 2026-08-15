STATUS: DONE

# Task 9 Review: the instruction loop (commit 43e18462)

Reviewer: read-only review agent, 2026-07-31.
Range: 3a405228..43e18462, three files (`run.rs` new, `lib.rs`, `bin/rexx-run.rs`).

## Verdicts

**Spec compliance: one Critical gap, otherwise PASS.** Six of the seven
instructions match the oracle on everything probed, including the edges the
commission flagged. The seventh, `DROP`, is correct in its `Direct` form and
wrong in its `(v)` indirect form: the oracle treats the wrapper's value as a
**blank-separated subsidiary list** of validated variable symbols; the Rust
code treats it as a single verbatim name and validates nothing. Three
reachable, silent divergences, measured below.

**Code quality: PASS, clearly the strongest task so far.** The move is
faithful to the line (checked mechanically, not by eye), the temps chokepoint
did not regress, all four of my mutations were killed by exactly the right
tests (a first for this branch), and the implementer's overturning of the
dispatch's "measured fact" survives adversarial re-measurement.

## Priority 1: move fidelity after the checkout incident — CLEAN

Mechanical check, not a read-through: extracted `git show
3a405228:.../lib.rs` (1,259 lines) and tested every nonblank line for a
verbatim match in the union of the new `lib.rs` (929) and `run.rs` (1,166).
**27 lines lack a match, and all 27 fall inside the six stated edits**: the
crate-doc rewrite, an import reflow, `pub(crate)` on `run_activation` (the
one stated visibility change), the two "is Task 9's" placeholder comments
replaced by the implementations they promised (Assignment's Stem/Compound
arms, Exit-with-result), and `execute`'s stated restructure. Consequences:

- `run_activation`'s borrow-shape doc comment, both doctests, and the
  `failure_site` block with the measured INTERPRET caveat moved **verbatim**
  — a single retyped word would have put its line in the missing list.
- No item is defined in both files (name-level intersection of every
  fn/struct/enum/const: empty).
- `Loud`, `form_name`, `execute`, `run_program`, `Interp`,
  `on_interpreter_thread`, `INTERPRETER_STACK_BYTES`, `StackSpan` all stayed
  in `lib.rs` as required.

The disclosed `git checkout -- lib.rs` incident left no detectable scar.

## Priority 2: the temps-frame chokepoint — NOT REGRESSED

`step_in_temps_frame` (run.rs:447) moved verbatim: result captured in a
local, `pop_frame` unconditional on Ok and Err, doc comment intact word for
word. Both loops route through it (run.rs:236, run.rs:496). The conclusions
of `temps-frame-investigation.md` carry over unchanged to the new file
layout.

## Priority 3: EXIT's conversion — the implementer's story CONFIRMED

14 fresh probes, none a multiple of 256, oracle vs `rexx-run`, all rc equal:

```
exit 1234567891 -> 211        exit 2147483645 -> 253      exit 2147483647 -> 255
exit 2147483648 -> 0          exit -3 -> 253
exit -2147483641 -> 8         exit -2147483645 -> 0       exit -2147483647 -> 0
numeric digits 20; exit -2147483647 -> 1
numeric digits 12; exit -1234567891 -> 45
numeric digits 3;  exit 2147483647  -> 255
numeric digits 3;  exit 12345       -> 57
exit '  777  ' -> 9           exit 7.77e2 -> 9
```

The bisection behaves exactly as the rounding-at-creation story predicts:
`-2147483641` rounds *down* at DIGITS 9 and stays in range (rc 8);
`-2147483645`/`-2147483647` round *up* past `INT32_MIN` and reject (rc 0);
DIGITS 20 removes the rounding and the rejection with it. The two DIGITS-3
rows pin the conversion's independence from the active DIGITS (a conversion
using the current setting would answer 0 for both). The dispatch's original
"large rejected positive range" is definitively a mod-256 measurement
artifact; `exit_code_for`'s shape (no special casing, `whole_value` at
`ARGUMENT_DIGITS`) is right, and its width argument is mutation-pinned (M4
below).

## Critical 1: `DROP (v)` ignores the subsidiary-list semantics and validates nothing

The oracle's indirect form treats the wrapper's **value as a blank-separated
list of variable symbols**, each trimmed, validated, upcased, and dropped by
its own shape. `drop_variable`'s `Indirect` arm (run.rs:559-584) upcases the
whole value and treats it as exactly one name, with no validation. Measured
divergences, all silent (no diagnostic, wrong state or missing error):

| Program | Oracle | Rust 43e18462 |
|---|---|---|
| `a=1; b=2; v='a b'; drop (v); say a; say b` | `A` / `B` (both dropped) | `1` / `2` (a slot literally named `A B` cleared) |
| `x=1; v=' x '; drop (v); say x` | `X` (trimmed, dropped) | `1` (slot named ` X ` cleared) |
| `a.='d'; a.1='one'; x=5; v='a. x'; drop (v); say a.1; say x` | `A.1` / `X` | `one` / `5` (tombstoned key `" X"` on stem `A.`) |
| `v='9'; drop (v)` | Error 31.2, rc 225 | runs clean, rc 0 |
| `v='.x'; drop (v)` | Error 31.3, rc 225 | runs clean, rc 0 |
| `w='a'; v='(w)'; drop (v)` | Error 20.928, rc 236 | (follows from the above: no validation) |

Characterization for the fix: the list is **not recursive** (a parenthesized
entry is 20.928 "Symbol expected as an indirect variable name"), mixed
shapes in one list all drop (third row), and the per-word validation raises
31.2 (digit-led) / 31.3 (dot-led) at run time. The catalogue already carries
these entries by construction (rexx-inventory generates from rexxmsg.xml).

Why every existing test still passes: the implementer's three indirect
sub-cases all use single-word, already-valid, unpadded values, where "split,
validate, resolve each" and "resolve the whole value" coincide. My q12 probe
shows how close this came to being caught by accident and missed: with
*unset* targets, `drop (v)` on `'a b'` matches the oracle coincidentally,
because a cleared slot named `A B` and two genuinely dropped variables both
leave `say a` deriving `A`. Only set values discriminate.

The `Direct` form is unaffected (q13-q17 exercised only `Indirect`; the
direct multi-target spelling `drop a b c` parses as separate `VariableRef`s
and matched the oracle, probe q7).

## Important 1: none

No Important findings. The validation misses above are folded into Critical
1 as one defect with one fix site (`drop_variable`'s `Indirect` arm).

## Minor findings

1. **`EXIT`'s result value is unrooted from the wrapper's pop to
   `exit_code_for`.** `step` pushes it as a temp inside the instruction's
   frame (run.rs:405), `step_in_temps_frame` pops that frame immediately,
   and the `ObjRef` then travels through `run_activation`'s return, `run`'s
   activation pop, and into `execute` before `to_number` reads it. Benign
   today for the reasons established in `temps-frame-investigation.md`
   (nothing allocates or collects on that path), but this is an
   **under**-rooting window, the class that becomes a real defect when
   collection lands, and it is much longer than the documented
   "unrooted from here to the caller's own push_temp" windows. Worth a
   comment now or a root before the pop when GC work starts.
2. **`shape_of` accepts names no scanner would produce** (empty string,
   blanks, digit-led) and classifies them Simple, which is what lets
   Critical 1's invalid names through to `slot_of` silently. The subsidiary
   -list fix will want validation *before* classification; noting so the fix
   lands in front of `shape_of` rather than inside it.
3. The brief is stale in two places the implementer correctly overrode with
   the dispatch's corrections (the `step` signature, the
   `tests/run_basic.rs` commit line contradicting the in-file-tests rule).
   Recorded so the next brief revision cleans them up.

## Mutation testing (scratch copy, committed state 43e18462, baseline 75+11+2 green)

| Mutant | Result |
|---|---|
| M1: `shape_of` never answers Stem | KILLED by `drop_of_a_whole_stem_leaves_it_looking_untouched` + `drop_of_the_indirect_form` |
| M2: indirect compound key split at last dot, not first | KILLED by `drop_of_the_indirect_form` (the A.1.2 verbatim case) |
| M3: `NUMERIC DIGITS` with no expression a no-op | KILLED by both reset tests, including the 33.1-as-if-"9"-typed one |
| M4: `exit_code_for` at width 9 instead of `ARGUMENT_DIGITS` | KILLED by `exit_code_for_converts_the_result_the_way_the_oracle_does` (the implementer's own claimed mutation, reproduced) |

Four for four, each by exactly the test that claims the mechanism. The
falsifiability question that came back "no" in the last two reviews comes
back "yes" here. Baseline restored green after the battery; the shared
worktree was never modified.

## End-to-end agreement (oracle vs rexx-run, full stdout+stderr+rc)

q1-q10, q12 all byte-identical: rendering fixed at creation (`x=1/3` keeps
its 9-digit rendering after `NUMERIC DIGITS 3` while fresh `1/3` gives
`0.333`), indirect drop of a simple/stem/joined-dots compound, upcase of the
indirect name, FUZZ and FORM resets, the 33.1 conflict's three-line stderr
(rc 223), multi-target direct `drop a b c`, unset wrapper (`drop (y)`
derives `Y`, drops it, no error), and the empty-string wrapper (both sides
run clean, rc 0).

## Hygiene

No em-dashes or en-dashes in any of the three files (the one grep hit for
"unsafe" in lib.rs is the English word in a pre-existing comment, not a
block; no `unsafe` code anywhere in the diff). Comment style conforms
(double-hyphen asides, contracts in doc comments, reasoning at decision
points). Tests are in-file `#[cfg(test)] mod tests` per the rule. Clippy,
fmt, and the 699-test suite were confirmed by the commissioning lead and not
re-run here, per instruction.

## Not reached

- `run_fragment`'s internals beyond line-level move fidelity: it moved
  verbatim, and its logic was reviewed in its own task; not re-reviewed.
- The trace sink (`LABEL` as a *traced* no-op is untestable until Task 13
  gives tracing an observable output).
- `NUMERIC DIGITS`/`FUZZ` operand spellings beyond what the implementer and
  my probes covered (e.g. exotic-but-valid number spellings for the setting
  value); the validation itself is `rexx-num` settings territory, reviewed
  at its own task.
- Subsidiary-list corner cases beyond those measured (tabs as separators,
  a value that is only blanks): the fix should measure these; the six-row
  table above is what this review established.
- `INTERPRET` spike interactions with the seven new instructions (e.g.
  `interpret 'drop (v)'`): out of 4a's user-reachable surface, and the
  fragment loop's mechanics were unchanged by this diff.
- GC rooting audits beyond the one window called out in Minor 1.

---

## Re-review of the fix round, commits 18c0bd17 + d8aa4bc5 (2026-07-31, same reviewer)

Scope: `review-43e18462..18c0bd17.diff`, two commits, reviewed under the
assumption the round broke something. **Verdict: CLEAN. The Critical is
fixed faithfully to the oracle, the rewrite broke nothing reachable, and one
narrow test gap is noted (newline separator, measured but unpinned).**

### The lead's five skepticism items, in order

1. **Collect-then-drop is real, in the code and on the oracle.** The code
   collects every `validate_indirect_word` result into a `Vec` and only then
   loops `drop_by_name` -- structurally two passes, and the second cannot
   fail. The oracle claim re-measured with my own recovery probe
   (`a=1; b=2; signal on syntax; v='a 9 b'; drop (v)` with the handler
   printing both): prints `1` / `2`, both intact, so the oracle validates
   the whole list before dropping any of it and the implementer's probe
   design is sound. The unit test pins it the strong way -- it inspects
   `a`'s slot *state* after the raise, not output -- and mutation M1
   (interleave validate and drop) is killed by exactly that test.

2. **Space-or-tab, not `is_ascii_whitespace`, re-measured and correct --
   including the three bytes the report did not mention.** Newline
   (`'0a'x`), CR (`'0d'x`), FF (`'0c'x`) and VT (`'0b'x`) each form one
   word with their neighbours and fail as 20.928 on the oracle; tab
   separates. The newline case diffed **byte-for-byte** through rexx-run,
   raw newline inside the substitution text and all. One gap: mutation M4
   (add `\n` to the separator set) survives all 79 tests, because no unit
   test carries a newline word -- the distinction rests on this review's
   end-to-end diff, not on anything in-repo. Minor; the fix is one
   assertion (`v='a'||'0a'x||'b'` -> (20,928), additional `"a\nb"`).

3. **`a-b` is 20.928, confirmed** (`found "a-b"`, rc 236, byte-identical
   end-to-end), so the character-set-check-first generalisation holds and
   no recursion guard is needed for `(w)`.

4. **`drop_by_name`'s reuse by `Direct` broke nothing.** Direct probes
   re-run end-to-end post-fix, byte-identical: `drop a b c` (multi-target
   direct), `drop u.1` (tail), `drop x.` (whole stem). Mutation M5 (Simple
   arm a no-op) is killed by three tests spanning both Direct and Indirect
   callers. Mutation M3 (Stem arm rerouted to a slot clear) **survives all
   79 tests -- and that is an equivalent mutant, not a gap**: `stem_drop`
   is `replace_stem(name, None)`, rebinding the slot to a fresh empty stem,
   and a rebound-fresh stem and a cleared slot are observationally
   identical until something can hold a *second* reference to the old stem
   object, which nothing in 4a can (the D15a alias transcripts need
   `procedure expose`/argument passing, 4b's). Recorded so nobody reads a
   future M3-style survival as coverage rot: it becomes pinnable exactly
   when stem aliasing arrives, and should be pinned then.

5. **The empty/blank no-op test is NOT a witness that cannot fail.**
   Mutation M2 (remove the empty-word filter from `split_indirect_words`)
   kills it -- the filter is load-bearing against a panic, since
   `validate_indirect_word` indexes `word[0]` and an unfiltered split of
   `'   '` hands it empty words. It also fails the subsidiary-list test's
   leading/trailing-blank case. The implementer's "unaffected either way"
   observation is about old-code-vs-new-code on empty input (both happen to
   run clean), which is true and does not make the test vacuous against
   the mutations that matter.

### The second commit, d8aa4bc5

Pure deletion of stem.rs's blanket `#[allow(dead_code)]` and its
justifying comment, which Task 9's wiring made stale. Nothing else in the
hunk. Clippy `-D warnings` green on the result (lead-verified, and my
scratch builds under the mutations confirm the crate compiles warning-free
with the allow gone), which is itself the proof every stem entry point now
has a live caller.

### The two rulings

- **Minor 1 deferral, judged sufficient.** The c67dd343 comment on
  `Heap::collect` names the exact window (EXIT's result, wrapper pop to
  `exit_code_for`), the fix shape (a root that outlives a clause), points
  back to the site write-up in run.rs (which the diff adds, accurately),
  and tells the future collector-wirer to sweep for the same shape. That
  is the right place and the right content; the alternative (a root
  surviving frame pops) would be speculative machinery with one user.
  Agreed.
- **`run_source` through the wrapper: the right choice**, and the diff's
  new doc comment gives the honest reason (the one caller in the crate
  skipping the chokepoint is the example a future non-test caller copies).
  It also keeps the temps stack balanced on the Err path, which the new
  state-inspection test then runs against.

### Mutation summary, this round

| Mutant | Result |
|---|---|
| M1: interleave validate and drop | KILLED by the state-inspection test |
| M2: no empty-word filter | KILLED by the no-op test + the blank-collapse case |
| M3: `drop_by_name` Stem arm -> slot clear | SURVIVES: equivalent mutant in 4a (no stem aliasing exists); pinnable in 4b |
| M4: newline as separator | SURVIVES: real but narrow gap, no newline word in any unit test |
| M5: `drop_by_name` Simple arm no-op | KILLED by three tests across Direct and Indirect |

Baseline 79+11+2 confirmed green before and after; scratch copy only, the
shared worktree untouched.

### Not reached this round

- The corpus subset (lead-verified 9/26, not re-run).
- Substitution rendering for FF/VT words through rexx-run (oracle measured
  both as 20.928; only the newline case was diffed end-to-end -- the
  rendering path is byte-preserving `from_utf8_lossy`, identical for all
  control bytes in the 0x00-0x7F range).
- Whether `DO OVER` on a stem (Task 11, if scoped in) captures a stem
  reference that would make M3's equivalence collapse earlier than 4b;
  flagged in the Task 10/11 pre-flight instead.
