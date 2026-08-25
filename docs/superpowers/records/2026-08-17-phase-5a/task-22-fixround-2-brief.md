# Task 22, fix round 2

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-22-rereview.md`. **Nine of eleven findings
closed clean**, verified by execution. Two are not, plus a falsified neighbour and two blemishes.
Small round.

**The pattern to see before you start:** you applied the strike-do-not-replace rule in `lib.rs` and
abandoned it in `phase-4-exclusions.txt`, and the new false sentence is exactly there. The source
half of M2 is the model -- it states the loss, names its cause, cites the arm, and offers no reason
that can rot. Every clause of it was verified and holds.

## Must fix

**1. `phase-4-exclusions.txt:494`: "Two do." is false, and it is a set cardinality in prose.**
The reviewer measured eight shared libraries that load; I spot-checked and confirm `rxmath`, `rxsock`
and `orxncurses` all give 90.998 rc 166 (the library opened, the entry is missing) while `rexxapi`
and `nosuchlib` give 98.903 rc 158. The count is also wrong under the other reading: counting
`LIBRARY REXX` in contradicts the next clause of its own sentence, which says that one is compiled
in rather than loaded. **Strike the number.** "Shared libraries in this tree do load" carries the
whole argument and cannot rot. Keep the `librexxutil` measurement beside it.

**2. MINOR 1 is not closed: `dispatch.rs:1419`.** `Interp::invoke`'s own doc -- the doc of the
function whose `match` you edited -- still says "**`None` is a send that produced no value**, and
either kind can do it." There are four kinds and a third one does it: `Interp::write_attribute`
returns `Ok(None)` at `:1580`. So it is both a stale two-kind count and wrong on the facts.

Your report says you swept by running a collapsed-comment search rather than fixing the list you
were handed. **The search found exactly the four you were handed and missed the fifth**, which is
worth more attention than the fix: a sweep that returns its input is not distinguishable from no
sweep unless you check it against something. The reviewer's scan used a wider alternation
(`either kind|both kinds|the two kinds|neither kind|both invocable|native or Rexx|both entry
points`) and found this one. Correct the sentence, and say in your report what your pattern was and
why it missed.

**3. Falsified neighbour, `phase-4-exclusions.txt:483`.** "The row below matched the oracle byte for
byte at 62de43c0f and refuses now:" -- there are two rows below it now, and the paragraph beneath
already says "Those rows". The singular is wrong, and as written it extends an unmeasured historical
claim to a row first probed yesterday.

## Blemishes

* `phase-4-exclusions.txt:501` runs to 97 columns where the file wraps at about 76.
* `tests/collect_stress.rs:251` puts `directive_method_external_not_a_staged_gap.rex` out of the
  list's alphabetical order. Nothing asserts it; cosmetic.

## How to close

All five gates. This should be comments and a doc file only -- prove it the way you did last round,
forced rebuild and `.text` section hash before and after, which was the right instrument and I am
adopting it. Append to your report, and end with what a future task inherits from Task 22.
