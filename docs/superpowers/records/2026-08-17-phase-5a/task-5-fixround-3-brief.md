# Task 5, fix round 3

The re-review **accepts** round 2. All six findings ADDRESSED; the three controls re-run
independently on a rebuilt binary in a copy, all three exit 101 with the row reading `unanswered`;
every attack on the new bounds failed. Nothing behavioural is open.

What follows is four documentation corrections plus one guard. Round 3 is small; do not restructure
anything that works.

## A. LOW-MEDIUM -- the impossibility claim is false, and it fails on a premise this plan has already
been bitten by once

`gate_table_d.rs:59-62` (module doc) and `:300-302` (`refusal_answered`'s doc) both say a line
printed before the refusing directive **does not exist**, because the refusal is a translate-time or
install-time failure and both precede the program's first clause.

**Measured false for one of the seven rows.** For the `::REQUIRES NAMESPACE` row, a program reading
`say 'main'` plus `::requires 'helper.rex'` plus `::requires 'zzznofile.rex' namespace ns` exits
**213** with `helper-ran` on `stdout`. The reason is the one this plan's scope premise already broke
on once: **installing a `::REQUIRES` runs the required program.** Install time is not before Rexx
code runs; it is Rexx code running. The reviewer scoped the negative half as well: the same
construction prints nothing for the other six, at exits 166, 166, 158, 158, 231, 231.

**Keep the `stderr` bound.** It is right, and the review's attacks on it failed. What is wrong is the
reason given for it.

**How to restate it.** Do not replace one unbounded claim with another. Name the construction and its
result, the way a measurement is written: what program shape was tried, which rows printed nothing
under it and which printed something, and that the bound is on `stderr` because that is the answer
**every** one of these rows gives, not because no other answer could exist for any of them. If you
find the `::REQUIRES NAMESPACE` row could carry a `stdout` bound of its own, say so and leave it to a
task; do not build a per-row bound in this round.

## B. LOW -- "every probe here already opens with `say 'main'`" is false

`gate_table_d.rs:301` and `corpus/gate-tables/README.md:61-62` ("like the rest"). Measured, the four
`options__*` probes open with `say digits()` / `form()` / `fuzz()` instead.

**The claim only has to hold for the rows it is an argument about** -- the `ORACLE_REFUSES` seven.
Narrow it to those, verify it for those, and if it does hold for them, say so about them. Remember
that a comment may not name a set's size, so state the property and the set, not a count. If the
property is load-bearing for the bound's argument, prefer asserting it in the harness over asserting
it in prose.

## C. LOW -- a history sentence, the same shape N5 struck in this very commit

`gate_table_d.rs:296-298`: *"Measured, replacing such a row's probe with a program reading `nop` did
exactly that and took the 5a open count down by one."* That is the behaviour of the code before this
commit. The two sentences before it already carry the contract in full. Strike it, and apply the
deciding test to the neighbours while you are in there: strike the historical framing and see whether
the sentence still says the same thing about the code as it is.

## D. LOW -- three enumerations not carried through this round's own change

`gate_table_c.rs:22-24`, `gate_table_c.rs:56-59`, and `corpus/gate-tables/README.md:33` each
enumerate what a wiring row asks or what bounds a row, and each was left describing the pre-round
shape. Bring them to what the code does now.

## E. The third zero-count arm, folded in from the review's out-of-scope list

`Concept::oracle_lines` has no non-zero validation. All its committed values are non-zero today, so
nothing is wrong now -- but a committed `0` there rebuilds finding 1 exactly, and that is the third
arm of the same family after the two this round closed. Add the guard: a committed `oracle_lines` of
zero is **structural**, not a verdict.

I am ruling this in scope rather than leaving it to the final review because the round exists to
close zero-count arms and leaving the last one open is the shape that has now bitten twice.

## Verification this round owes

* Re-run the three controls once more. They are cheap and two rounds running have had one that read
  differently when run than when reported.
* For A, paste the construction you ran and its per-row result, not a summary of it.
* The five gate commands, each status read unpiped -- and note that `cargo test` piped into anything
  reports the pipe's status, not the test's.
* Report which of the round-2 claims your edits change, if any.
