# Re-review of fix round 1 (`977cf4b9e`)

Scope: `977cf4b9e` only (`4b836188a` excluded, per instructions). Tree left clean; a temporary
worktree at `8cdc40820` used for the "unmoved" gate check was removed after use; a one-line probe
into `dispatch.rs` used for the rooting-control re-run was restored and confirmed with `diff -q`.

## 1. Is the README now true?

Yes. Ran both commands the README documents, exactly as written, from `rust/`:

* `awk -F'\t' 'NR>1 {print $1"\t"$2}' bench-baselines/phase-5a-arms.tsv | sort -u` -- lists 13
  `(task, commit)` pairs, including `11-bisection|6f3434e88` and `12-prev|ee6ebbf64`, matching the
  suffix convention the README describes.
* The duplicate-key `awk` over all eight key columns prints `0`.

Checked the rest of the README against the file: every column in the table (`task`, `commit`, `axis`,
`build`, `scope`, `arm`, `size`, `instrument`, the four `value_*`) matches the TSV's actual header and
value sets; `scope` is exactly `{absolute, across_builds, arm_ratio, fixed, per_pass, per_pass_gap}`,
`arm` is exactly `{ir, ir-tw, ir/tw, tw}`, `size` is exactly `{-, large, small}`. The `across_builds`
`build` column does hold `from>to` shapes (`pinned>head`, `a-base-21cde29af>b-committed-f4b21eadb`,
etc.). `PINNED.md` and `phase-4e-arms.tsv` both exist where the README's opening lines say they do.
No claim in the file contradicts the TSV.

The historical number the new prose states (`312`, "between Task 12's two sittings ... and the
relabelling") also checks out: rows with `task=12, commit=ee6ebbf64` (the surviving first sitting)
number exactly 312, matching what the report says collided in full.

## 2. The performance disagreement: implementer is right

Recomputed `head / prev-c18b877e3` for `instructions:u` over every `(axis, arm, size)` directly from
each sitting's own `absolute` rows (not from the pre-rounded `arm_ratio` rows, which lose precision at
6 places). Two distinct quantities, named separately as the brief asked:

* **Largest ratio above 1, `12-prev` sitting:** `1.000000032` on `varlookup/ir/large`. This is exactly
  the review's cited figure, confirmed byte for byte -- but it is the largest value *above* 1, not the
  widest departure from 1 in either direction.
* **Widest absolute departure from 1, `12-prev` sitting:** `0.999999741` on `alloc4c/ir/large`,
  departure `2.59e-7` -- more than 8x `varlookup/ir/large`'s departure of `3.2e-8`. This is the
  implementer's figure.
* **Widest absolute departure, `12-fixround-1` sitting:** `1.000000238` on `alloc4c/ir/small`,
  departure `2.38e-7` -- also confirmed, and also the implementer's figure, correctly attributed to
  the third sitting rather than the second.

Both sittings' widest departures round to `0.0000002`-`0.0000003`, i.e. they agree with 1 to six
decimal places and disagree at the seventh -- confirming "six decimal places" as the honest claim
against the review's "seven."

**Ruling: the implementer is right.** The review's `1.000000032` is a real, correctly-transcribed
number, but it answers a different question ("widest ratio above 1") than the one the report needed
("widest departure from 1"), and on this data the two are not close: the true widest departure is
2.6e-7, not 3.2e-8.

## 3. New false statements introduced by the round: none found

Checked each claim by running or grepping rather than reading:

* **Both corpus programs changed length, both `sourceline_oracle` expectations regenerated and
  re-run.** `condition_nomethod.rex` 118->119 lines, `message_send_unknown_forward.rex` 65->66 lines;
  both `.txt` files' `count` line was bumped to match. Ran `cargo test --release -p rexx-parse --test
  sourceline_oracle`: passes. Additionally ran both corpus files against the oracle and against
  `rexx-run` on both engines from a fresh directory: all three descriptors (stdout, stderr, rc) match
  on every combination, oracle rc 0 for both files.
* **Gate table C `5a: 135 rows, 115 not yet agree`, table D `36 rows, 7 not yet agree`, both unmoved
  from `8cdc40820`.** Ran both `cargo test --release -p rexx-exec --test gate_table_c/-d --
  --nocapture` at `977cf4b9e` and, via a temporary worktree, at `8cdc40820`. Both figures are
  identical at both revisions.
* **The set-size sweep tables the whole set.** Diffed `c18b877e3..977cf4b9e` for comment lines
  containing `both`/`pair`/`two`/`three`/`twice`. Exactly two hits survive in the current tree: the
  module-doc "call pair" clause (addressed in question 4) and the "`signal on nomethod` and `signal
  on syntax` both armed" sentence, which the report explicitly says it kept because the sentence
  already names the two things individually. Every other of the 14 "was"/"now" pairs the round's
  table lists was independently confirmed present at the stated old and new text.
* **Rooting-control symptom: rc 120 "a message send to a value whose object is no longer live" from
  `receiver_kind`'s dead-handle arm, reached by `a~items`.** Removed `self.roots.push_temp(arguments)`
  (dispatch.rs:1162) and ran `cargo test --release -p rexx-exec --test collect_stress`. Result:
  `the_l0_subset_passes_again_under_collect_on_every_allocation` fails with `stress exit=120` and
  stderr `"rexx-exec: a message send to a value whose object is no longer live is not implemented
  (Phase 5)\n"` on exactly the long-name row (`AVERYLONGMISSINGMESSAGENAME`) -- no panic anywhere.
  File restored and confirmed with `diff -q` against a backup.

No false statement found in the fix round's prose.

## 4. Ruling on the pre-existing "call pair" phrase

The comment rule (no set-size language in comments this task/round writes) does **not** reach
pre-existing text merely because a line was touched for an unrelated reason. `//! \`exec_call\`'s
relation to the call pair exactly.` predates Task 12 verbatim; the diff shows it as a changed line
only because a new sentence was appended after the same period, not because its wording was authored
now. Two reasons this is the right line to draw:

* The risk the rule guards against is a headcount that can silently go wrong or drift (the
  `phase-5a.txt` fix in this same round is exactly that: "one class... and one only says" was
  factually wrong once a third class existed). "The call pair" names `resolve`/`invoke`, which D24
  fixes as *architecturally* two functions -- the surrounding untouched prose already says so
  repeatedly ("`resolve`/`invoke` pair", "are two steps, not one fused send"). There is no drift risk
  to guard against.
* Scope: a fix round answering specific review findings should not use a line's diff-attribution as
  license to rewrite unrelated pre-existing prose it happens to share a line with. That is the same
  overreach the round's own text declines for the same reason ("widening the sweep into text this
  task did not write is not mine to do").

The implementer's decision to leave it alone stands.

## File

`.superpowers/sdd/2026-08-17-phase-5a/task-12-rereview-1.md`
