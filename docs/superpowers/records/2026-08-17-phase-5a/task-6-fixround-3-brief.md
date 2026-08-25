# Task 6, fix round 3

Round 2 verified strongly. All six findings ADDRESSED. **The blast-radius attack found no over-fire**
across roughly forty programs -- no stem anywhere, stem on the right, stem receiver failing past its
own conversion in every operand shape, surrounding-clause errors after the operator succeeded, and a
leak hunt over every caller that could swallow the `Err`. The five witnesses are byte-exact on both
engines. The control was verified more strongly than the mutation: the **pinned** `rexx-run-15a1ffa98`
differs from the oracle on all eleven programs by exactly the frame line, stdout and rc identical.
And the duplication question came back clean -- one shared helper at `eval.rs:1112` with a three-line
adapter at each site, which is not the finding.

Three things follow.

## 1. Comparison is the same unmet class, and it was closed on a premise I gave you

Measured by the reviewer and re-measured by me: `numeric digits 1`, `s. = '9.9E999999999'`,
`say s. > 1` is rc 214 on the oracle and opens
`       *-* Compiled method ">" with scope "String".`; both engines omit it. The same holds for `<`,
`>=`, `<=`, `=` and `\=`, and for a stem receiver whose **right** operand overflows. Strict `==` and
`>>` are correctly frameless, and concatenation genuinely cannot raise -- that half is confirmed.

`compare_numbers` at `eval.rs:1381` does answer `Err`, so your report's "checked rather than assumed"
sentence about comparison is false.

**The false premise is mine, not yours.** I wrote it into the plan at Task 6's prerequisite check --
that comparison needs only a string and so cannot fail -- from probes that all used a **non-numeric**
operand, where comparison falls back to string comparison. Two numeric operands convert like any
arithmetic operator. I have corrected the plan at `be81ce689`; read that paragraph before you start,
because it now states the rule you are implementing against: **the families split by whether the
operator needs a number, not by whether it is arithmetic** -- arithmetic always, comparison when both
operands are numeric, concatenation never.

**Ruling: wrap it.** The helper exists and this is a fifth adapter, not a mechanism. Give the six
operators witnesses on the same terms as the others -- one program is enough if it covers the shared
path and you say which operators reach it and how you checked, rather than one program each for its
own sake. Extend the control to them.

If wrapping `compare_values` turns out to change the frameless cases -- strict comparison, the
non-numeric fallback, `1 > s.` -- stop and report that precisely; those are measured frameless today
and must stay so.

## 2. Two false sentences about performance

"nothing on any path that succeeds" and "Nothing this round's diff added runs on any path `arith`'s
benchmark exercises". `arith_general` runs `if result.is_err()` unconditionally, and
`bench-programs/arith.rex` is all `arith_general` work. The measurement agrees with the code and not
with the sentence: across the single commit `211763aaa..c351fa473`, arith went from `1.000000` with
min equal to max on all four rows to `1.001006`-`1.001294`, again min equal to max.

Sub-threshold, so nothing is gated on it. Say what is true: the added work is one predicate on every
arithmetic result, and attributing the remainder between that branch and code layout needs the
do-nothing control, which you have not run and do not need to.

## 3. Prose, four instances and one comment that misnames its own subject

* `eval.rs:1053` still says `arith_left_operand` shares `blame_stem_forwarded_operator`, contradicting
  the paragraph added below it.
* `eval.rs:973` "exactly as before" is historical framing, and slightly false: a non-stem receiver now
  reaches the call on any `Err`.
* `eval.rs:972` "all three past ..." is a new set-size phrase, of exactly the shape finding 2 struck
  last round.
* The report's performance table breaks its own "(min..max) where they differ" convention twice --
  `alloc4c/ir` min `0.999999`, and `arith/tw` dropping large's `1.001036`. That is finding 5's shape
  inside finding 5's own round.
* `operator_frame_stem_divide_by_zero.rex` and its `sourceline_oracle` twin say "the divisor's own
  arithmetic overflow". The divisor is `0`; what raises is the division's zero check. Name what
  actually happens.

**Recorded, not fixable:** `c351fa473`'s message says the five witnesses join "the eleven already
there". Six were there. It goes in the report's own record rather than in a history rewrite.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* The comparison witnesses on both engines against the oracle, and the frameless cases re-measured
  after the change: strict `==` and `>>`, the non-numeric fallback, and `1 > s.`.
* The control extended, with its transcript.
