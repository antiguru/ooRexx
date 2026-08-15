# Task 16 Step 4 report: the exclusions file

Status: **ALREADY DONE BY ANOTHER AGENT.** No work of mine was needed and none
was committed. `docs/superpowers/plans/phase-4-exclusions.txt` exists, is
tracked, and was committed as `7aff84f4`, "Write the Phase 4 exclusions and
deviations file".

What I did instead, since the deliverable existed: **verified it** rather than
either rewriting it or accepting it. Four of its claims are independently
reproduced below, one of which is a measurement that appears nowhere in the
spec. Two review points follow, neither blocking.

## How I found out

`Write` refused the path because the file already existed, which is the tool
working correctly. Reading it first, then `git log -- <path>`, showed
`7aff84f4`. Working tree clean for that file.

Had I not been made to read before writing, I would have silently replaced
another agent's committed work with my own draft. Worth noting as a near miss
rather than a triumph: the instruction to read before overwriting is what
caught it, not judgement on my part.

## The file is correct on the point I was warned about

The team lead flagged the one thing the spec does not spell out: 11.1 at the
evaluation-depth limit is **parity** on the parenthesis and call axes and a
**deviation** only on the flat-term axis, and asked whether it could be stated
cleanly in one row.

The committed file already resolves it, and resolves it the same way I had
concluded independently before reading it. Its deviation row 2 separates:

* **Parity** on the parse-side axes, naming both oracle cliffs and
  `MAX_EXPR_DEPTH` raising the same 11.1 on one shared budget.
* **Deviation** on the flat-term axis, where the oracle prints at 100,000 and
  dies at 150,000 with rc 139 and no condition.

One row, two clearly separated clauses, with the sentence "Both use 11.1 and
only the second is a difference" doing the work. So the answer to the lead's
question is: **one row is enough, and it is already written that way.** No
amendment needed.

The file goes one better than I would have, by adding the second-order point
that a limit *above* the oracle's cliff is worse than one below, since it
widens the window where we succeed and the oracle segfaults.

## What I verified

Everything here was re-run by me today, wrapped as
`( ulimit -v 1048576; build/bin/rexx FILE )` where it touches the oracle.

**The builtin table citation and the derived count.**
`interpreter/expression/BuiltinFunctions.cpp` line 3042 is
`pbuiltin LanguageParser::builtinTable[] =`, and the table holds **exactly 81**
`&builtin_function_*` entries. All eighteen names the file lists — the fifteen
whole exclusions plus `VALUE`, `ADDRESS` and `QUEUED` — are present in it, each
checked by exact match. So `81 - 15 = 66 in scope, three of them partial`
derives correctly.

**The stem iteration order, which is the claim worth checking.** The file
states the oracle yields `1 B 3 2 ZZ 10` for tails inserted `1, 2, 3, 10, ZZ,
B`. That specific sequence is **not in the spec** — D15a gives `1 10 2 3 B ZZ`
as what a `BTreeMap` produces, as a contrast, and never states the oracle's own
order. So the file is asserting something new, which is exactly the shape of a
claim that drifts. Measured:

```rexx
q.1 = 'a'; q.2 = 'a'; q.3 = 'a'; q.10 = 'a'; q.ZZ = 'a'; q.B = 'a'
do i over q.;  out = out i;  end
say strip(out)        ->  1 B 3 2 ZZ 10
```

**Correct**, exactly as written, and now reproduced rather than inherited.

**`ADDRESS()`'s platform default.** `say address()` with no `ADDRESS`
instruction prints `sh`. Correct.

**The corpus consequence.** "No corpus program may contain `DO OVER` on a
stem" holds today: nothing in `rust/corpus/` or `rust/corpus-l1/` does. The
file's own warning about grepping carefully is well placed and I walked into
it — `do number over 1.2, -0.003` in `corpus-l1/json_test_number.rex` matches a
naive "over a dotted thing" pattern and is the **list** form, not a stem.

**The parse-side numbers** — [39,900, 39,950] parens, [34,500, 34,760] calls,
1,150-1,200 prefix chains, 331 parens and 341 calls on a default thread — all
match what I measured in Tasks 3c's review and 3d, including my re-measured
331/341 rather than the superseded 337/349. Someone folded those in correctly.

## Two review points, neither blocking

**1. The file hardcodes exit code 120, and Task 12 owns that number.** Line 29
says "exit code 120". That is correct today (`NOT_IMPLEMENTED_EXIT: i32 = 120`
in `rexx-exec/src/lib.rs:69`, verified), but the constant's own doc comment says
**Task 12 confirms the final value**, and this file is a gate artifact that
nothing recompiles. If Task 12 picks a different code, this goes stale silently
and the gate text starts describing a failure mode that no longer occurs. That
is the same carried-forward-number class that produced two wrong figures
elsewhere today.

Cheapest fix, if wanted: say "the not-implemented exit code, `NOT_IMPLEMENTED_EXIT`,
which is outside 157..253 where `256 - major` lives" and let the constant be the
single source. The band matters to the argument; the specific integer does not.

**2. The third section is a real improvement and also an unpinned surface.**
"KNOWN GAPS -- neither excluded nor deviated" is a genuinely good addition: a
divergence with no owner is a third status and conflating it with either
section would be wrong. But the gate asserts the set of the *other two*
sections, so this one can be edited freely by the phase being gated — which is
the precise failure mode the set assertion exists to prevent, reintroduced one
section down. Not urgent, since nothing in it is a claim of compliance, and the
right answer may well be "leave it editable on purpose". Worth a deliberate
decision rather than an accident.

## One thing outside this task that I noticed and did not touch

`git status` shows an untracked file at `interpreter/interpreter`, inside the
tree the global constraints declare read-only. Almost certainly a stray build
artifact rather than a modification, and deleting things in that tree is not
mine to do, so it is flagged and left alone.

## What I did not do

No commit. Nothing to commit: the deliverable existed and my verification
changed no file. The only artifact of this task is this report.
