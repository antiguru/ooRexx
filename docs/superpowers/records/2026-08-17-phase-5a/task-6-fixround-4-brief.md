# Task 6, fix round 4 -- prose only, and one rule decides almost all of it

**The mechanism passes.** The re-review ran 48 probes plus the 12 corpus programs on both engines
against the oracle and found **no over-fire and nothing regressed**, including the implicit
comparisons -- `IF`, `SELECT`/`WHEN`, `DO WHILE`, `DO UNTIL`, an `IF a, b` comma list, and
`DO i = 1 TO 3 WHILE` -- which all emit the frame on the oracle and match byte for byte. A counted
loop's own termination test is frameless and *structurally cannot* fire, because it uses
`controlled_within_wide` (`run.rs:8916-8925`) rather than `compare_values`. The control was verified
twice, the corpus went 117 to 118 with zero removed, and the new witness is byte-exact on the oracle
and both engines.

**Do not change behaviour in this round.** Nothing below asks you to.

## The rule that decides NEW-1 through NEW-4, and the report defects too

**An enumeration in prose goes stale at the next commit that adds a member, and nothing tells you.**
Three rounds running, this task has corrected a count or a list and had the correction itself go
stale or come out false in the same round. So the fix is not a better count. **Delete the
enumeration.** Say what the thing is and what makes something a member; do not list the members, and
do not say how many there are.

Two things this does not forbid. A **measurement** keeps its numbers -- a benchmark ratio, an error
number, a byte count you took by running something. And a list the **code itself enforces** is fine,
because the compiler or a test fails when it goes stale; prose is not that.

## In the tree

* **NEW-1, moderate.** `corpus/phase-5a.txt:214-215` says "the **six** non-strict comparison operators
  that reach `compare_values`'s numeric path". Measured, **ten** do: `>` `<` `>=` `<=` `=` `\=` `\>`
  `\<` `<>` `><`, all rc 214 with the frame. Banned as a set size and false as a count, in the round
  that was fixing a set-size phrase. Delete the enumeration; name the property that puts an operator
  on that path.
* **NEW-2, minor.** `phase-5a.txt:213-214`, "the other **four** adapters". True today, still a set
  size, and it becomes false at the sixth.
* **NEW-3, minor.** `corpus/lang/operator_frame_stem_compare_overflow.rex:6-9` and its
  `sourceline_oracle` twin say the program "stands for the family" and then omit four of its members.
* **NEW-4, moderate.** `eval.rs:1102-1106`: `blame_stem_forwarded_operator`'s doc enumerates its own
  callers -- "... and `header_number` does the same for the other receiver". This round added a fifth
  caller at `eval.rs:1323` and left the list as it was. Accurate at the base commit, incomplete at the
  head. This is the enumeration that will rot fastest, because every future adapter adds to it.
* **NEW-5, minor.** `eval.rs:973-977`: the replacement for "exactly as before" is itself historical --
  "now reaches", and "the removed inline call inside `arith_left_operand`" names code that is not in
  the tree. Apply the deciding test: strike the historical framing and see whether the sentence still
  says the same thing about the code as it is.

## In the report

`.superpowers/sdd/2026-08-17-phase-5a/task-6-report.md`. It is git-ignored (`.gitignore:30`), so
nothing prevented editing it, which matters for the first one.

* **NEW-6.** Round 3 recorded finding 2 as fixed. **The two false sentences are still there,
  verbatim and unmarked, at `:436` and `:724-725`**; the corrected version exists only at
  `:836-850`, and the round-3 text says `:436` *"said"* of a line it never edited. Correct the
  sentences where they are.
* **NEW-7 and NEW-8.** The sitting **table** matches the TSV cell for cell, all 24 -- that part is
  right. The **prose about the table** is false in three places: `:845-846` "again min equal to max on
  all four rows" (`arith/ir/small` is min `1.001283`, max `1.001284`, in this round's sitting *and*
  round 2's); `:947-948` "the same four values to six decimal places" (that cell's median moves
  `1.001283` to `1.001284`); `:941` "Only `alloc4c/ir/small` differs from an exact 1.000000".

  **Ruling: delete the summarising prose rather than correct it.** The table is right and is the
  record. Keep only the substantive conclusion, which survives and is worth keeping -- `arith` did not
  move because `arith.rex` exercises `arith_general` and `apply_prefix` and not `compare_values` --
  and let anyone who wants per-row numbers read the rows. **Three rounds have now put a false sentence
  on top of a correct table.** The sentence is the defect, not the arithmetic in it.

## Also record, do not fix

The re-review's out-of-scope list, in the report's own gap record, all confirmed identical on the
pinned pre-task build: `s. = .nil` then `say s. > 1` **silently answers `1` at rc 0** where the oracle
raises `97.1` -- worse in shape than the `+` sibling's `41.1`-versus-`97.1`, because that one is loud;
`eval.rs:3272`'s pre-existing "all four of"; and the `.environment` and `.array` stem defaults taking
the declared Phase-5 loud refusal.

## Verification this round owes

* The five gate commands, each status read **unpiped** -- a command piped into `grep` or `tail`
  reports the pipe's status, not the command's.
* Nothing else. No new sitting: this round changes no code. Say so rather than running one.
* Before you report, re-read every sentence you wrote in this round and ask of each one whether it
  names a size, lists members, or describes what the code did before. That is the check the last three
  rounds each passed and then failed.
