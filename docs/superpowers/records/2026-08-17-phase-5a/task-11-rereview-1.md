# Task 11 fix round 1 -- scoped re-review

Reviewing `f92e00b86..fefe33597` against the two blocking findings only.

## B1: CLOSED

`eval.rs:670`-`684` (`eval_cold`'s doc) and `value.rs:679`-`687`
(`string_value_text`'s doc) were checked figure by figure against
`rust/bench-baselines/phase-5a-arms.tsv`'s `11-bisection` rows (344 rows,
`task` column tab-separated, read with `/bin/grep -a`), computing every
subtraction independently rather than trusting the report's table:

* `per_pass tw strings`: a=9096.519093, b=9168.518788, c=9129.518712,
  d=9139.518738, e=9129.518361.
* `per_pass tw varlookup`: a=1761.000061, b=1776.999966, c=1773.000081,
  d=1766.000074, e=1773.000054.

Results: B-C = 39.000076 / 3.999885 (eval.rs's "39" and "4" for the `List`
arm), B-D = 29.00005 / 10.999892 (value.rs's "29" and "11" for the trace
gates), E-A = 32.999268 / 11.999993 (eval.rs's new "33 and 12" residual
sentence). All four figures now in the two comments are exactly the marginal
subtraction each sentence claims to be. Both comments state explicitly that
the two causes are not additive and point at the TSV rows rather than
asserting a total; "costs both nothing" is gone. No stray "38" or "43"
remains anywhere in either file (`/bin/grep -a` for both).

The one correction inside the round (38 -> 39 in `fefe33597`) is the only
figure that changed and it now matches the table; nothing else needed a
second correction.

## B2: CLOSED

Verified by running, fresh directory, both engines:

* `additional (1,2)` under `raise syntax 93.900` and `array (1,2)` are now
  both byte-identical to the oracle on both engines (rc 163, `Error 93.900:
  1.`) -- the divergence the original finding measured is gone.
* `additional (1,,3)` / `array (1,,3)` (rc 216, hole substitutes empty) and
  `additional ('R',,'X')` / `array ('R',,'X')` (rc 216, `in invocation of R`)
  are byte-identical pairwise and to the oracle, on both engines.
* The fix is contained to the arm it claims: `run.rs`'s `Interp::array_slots`
  / `array_slots_of` (`value.rs`) expose an array's existing `Body::Array`
  slots without a new value kind or heap layout; `run.rs:4570`-`4610` gates on
  `raise.condition == SYNTAX` and `array_slots_of(value)`, building the same
  hole-preserving substitution list the pre-existing `ARRAY` arm builds. No
  new mechanism.
* The refusal the rewritten sentences (`lib.rs:1737`, `owners.rs:151`) now
  describe -- "an `ADDITIONAL` value under `SYNTAX` that is a class object or
  one of the interpreter's own" -- matches what runs: `additional (.array)` is
  rc 120 here (`a class object as a RAISE ADDITIONAL value is not implemented
  (Phase 5)`) against oracle rc 158 `98.939`, and `additional (.environment)`
  is rc 120 here against oracle rc 163 `INPUTOUTPUTSTREAM.` -- both exactly as
  the report states.

## Question 3 ruling: the report's "no in-crate test" claim is wrong

The report says the object-valued `ADDITIONAL` refusal has "no differential
row and no in-crate test either -- it is asserted only by the corrected
comments and by the last negative control." The first half is right (no
differential row is possible here: the oracle's own bytes for this shape are
158/163, not this crate's 120, so it can never be a byte-identical row). The
second half is false: `eval.rs:3669`, `an_object_as_a_raise_syntax_substitution_is_loud`,
already exercises exactly this shape --  `additional (.array)` and `additional
(.environment)`, asserting `(120, "")` and that stderr contains `"a RAISE
ADDITIONAL value"` -- on both engines. It predates this task's fix round
entirely (present at `f92e00b86`, the diff's base commit, added at `e5cce254`
2026-08-16, before Task 11 went to review) and was untouched by any of the
three commits under review.

I confirmed it is load-bearing rather than coincidental: mutating
`run.rs:4585`'s `slots.is_none()` to `slots.is_some()` (so the refusal fires
on the array case instead of the non-array case) turns this exact test red
(`FAILED`) alongside reddening the corpus gate to 151 of 152 (restored
afterward, `diff -q` confirmed clean, `git status` clean).

**Ruling: not a finding.** The concern in the report is unfounded -- an
instrument for this refusal already exists and does fail if the refusal's
gating breaks. The report's claim that no in-crate test exists is itself
inaccurate and should be corrected in the report (it is not a defect in the
code or the comments, which are both true as written).

## Question 4: the three new corpus programs

`lang/raise_additional_array.rex`, `lang/raise_array_spelling.rex`,
`lang/raise_keyword_object_traces.rex` are all in `corpus.rs`'s
`RAW_STDERR_COMPARISON` (lines 306-308) and `phase-5a.txt` (lines 337-339),
each with a `sourceline_oracle/*.txt` counterpart. Run fresh, all three are
byte-identical to the oracle on stdout, stderr and exit status on both
engines (rc 216, 216, 223 respectively, as the writeup states) -- all three
are untrapped and traced, matching the writeup's account of why the first two
attempts (a trapped ladder) were rewritten.

Mutated `run.rs:4514` (the condition-keyword trace line) back to the pre-fix
shape -- `let traced = self.to_text(value).to_vec();` instead of
`self.string_value_text(value)` -- rebuilt, and ran the STRICT corpus gate:
it reddens to 151 of 152, the sole mismatch named as
`lang/raise_keyword_object_traces.rex` with exactly the expected wrong
rendering (`"SYNTAX" => "1\n2"` instead of `"SYNTAX" => "an Array"`). Restored
`run.rs` from a pre-mutation copy, confirmed with `diff -q` (no output) and
`git status --short` (clean), and rebuilt the workspace before finishing.

## Aside: a pre-existing bug adjacent to this fix, not introduced by it

While probing B2's "byte-identical" claim I found a real divergence at
`raise syntax 40.4 array (1,(2,3))` (a nested array as an `ARRAY` element):
oracle and this crate agree the outer report is `maximum expected is an
Array.`, but the `ARRAY` arm's own per-element rendering
(`run.rs`, the loop below the `raise.additional` block, `let rendered =
self.to_text(value).to_vec();`) renders the nested array as `2\n3` instead of
`an Array`, where the new `ADDITIONAL`-as-array path (which uses
`string_value_text`, not `to_text`, per item) gets this case right. That
`to_text`-per-item line is untouched by this diff (unchanged since Task 9) and
is not something this fix round introduced -- it is a pre-existing bug in the
`ARRAY` arm's own rendering, out of this re-review's scope, and not a
regression from the fix. Flagging only because I found it while verifying B2;
not treating it as a finding against this fix round.
