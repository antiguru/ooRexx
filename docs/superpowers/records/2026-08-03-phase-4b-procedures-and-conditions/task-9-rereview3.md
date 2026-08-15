# Task 9 re-review -- fix round 3 (`50da3045`), scoped to comments/docs only

Scope: did fix round 3 close NEW-F1 and NEW-F2 (round 2's re-review findings)
without shipping a seventh false statement. Comments and documentation only;
no behaviour changed this round (already established by the controller, not
re-derived here).

Every claim below is labelled **[ran]** or **[reasoned]**.

## Method

* HEAD at the start of this review was `6eddd10d0536f520aed3f22606174a8bb2c5c212`
  (one commit after `50da3045`, touching only `rust/CLAUDE.md`; the commit
  under review, `50da3045`, was already in history). Tree clean throughout,
  confirmed with `git status --porcelain` before, during (after every
  restore) and after.
* `run.rs` was mutated twice for this review (a "without the pre-gate"
  benchmark build, and a bare re-read for context -- no second mutation was
  needed beyond the one benchmark edit). Before mutating, the file was `cp`'d
  to the scratchpad (`t9r3/backup/run.rs.orig`); the restore was `cp` **from
  that copy**, never `git checkout --`, and `md5sum` confirmed byte-identity
  both immediately after backup and immediately after restore.
* Oracle and probe runs used the mandated wrapper (`ulimit -v 1048576`),
  three descriptors captured separately, never `2>&1`, from directories
  `mkdir`'d under the scratchpad for this review (`t9r3/bench`,
  `t9r3/labelways`, `t9r3/letters`) -- never the scratchpad root.
* The benchmark was built `--release`, run three-plus times per
  configuration, and the two configurations' binaries were saved to fixed
  paths (`rexx-run-gate`, `rexx-run-nogate`) so later rounds could be
  interleaved without rebuilding.

---

## Per-finding verdict

### NEW-F1 -- **closed** [ran]

* `grep -c 'tracing_intermediates()' crates/rexx-exec/src/run.rs` now
  returns **1**, and the one hit is the real call at `run.rs:5309`. The
  comment no longer quotes this command or any number at all; the survey
  sentence is gone, not corrected from 1 to 2.
* The replacement is a benchmark claim: "3.13/3.13/3.15 s with this check
  and 3.22/3.22/3.23 s without it -- about 40 ns per pass." **Re-run** (see
  below): reproduced the same direction and a compatible magnitude on this
  machine, with clean separation between configurations across every run.
* Confirmed the mutation used to build the "without" side is a fair
  isolation of the claim: changing `if self.tracing_intermediates()` to
  `if true` (forcing the two `Vec` allocations every pass) produced
  **byte-identical stdout and empty stderr** against the un-mutated binary
  under `TRACE OFF`, confirming `trace_assignment`'s own internal gate
  (`trace.rs:540`, `if !self.trace_mode().intermediates { return; }`) really
  does suppress printing on its own -- the outer check only ever controls
  allocation, exactly as the comment claims.
* Also confirmed `tracing_intermediates()` (`trace.rs:466`) is defined as
  `self.trace_mode().intermediates` -- textually the same condition
  `trace_assignment` gates on, so "not a second decision" is not just
  plausible but literally the same boolean read twice.

### NEW-F2 -- **closed** [ran]

* `grep -rn "self\.read(code" crates/rexx-exec/src/*.rs` today: `eval.rs:319`
  (pairs, `(value, novalue)`), `run.rs:2050` (`_novalue`, discarded),
  `run.rs:5113` (this fix, pairs), `run.rs:5750` (`_novalue`, discarded).
  Excluding the fix's own site, that is exactly "one pairs and the rest bind
  the flag to `_`" (1 pairs, 2 discard) -- the new wording, verified against
  the current tree.
* The LESSON paragraph's "signatures" framing was checked against the actual
  signatures: `read_by_name(&mut self, name: &[u8]) -> ObjRef` (bare value,
  cannot report unset) vs. `Interp::read` returning `(ObjRef, Novalue)`. True.
* `read_by_name`'s only caller besides its own definition is
  `stem.rs:120`, inside `tail_key` (a compound's tail-piece resolution) --
  confirmed by `grep -rn "read_by_name(" crates/ --include=*.rs`. So "the
  first's other caller is not the one it named" (i.e., not
  `PROCEDURE EXPOSE`) is true.

### Sweep-found fix 1 (`run.rs`, "goes through `read_by_name`") -- **closed** [ran]

* Walked the paragraph's own history across the three commits:
  * `94237403` (round 1): the actual code used `self.read_by_name(&name)`
    (confirmed: `git show 94237403:...run.rs | grep read_by_name` finds it
    at the call site), and the prose matched.
  * `f02c3ed3` (round 2): the actual code changed to
    `self.read(code, *control)` (the NEW-1 fix), and the *second* paragraph
    below was updated to say "`read`, not `read_by_name`" -- but the
    *first* paragraph still said "the read below goes through
    `read_by_name`", now false against the code three lines below it.
  * `50da3045` (round 3, this commit): the first paragraph now says "the
    value below is read out of the variable pool" (no reader named) plus a
    footnote pointing at the paragraph below and naming exactly this history
    ("this one said `read_by_name` until round 2 changed it and left the
    sentence behind, which is the same defect twice on one arm").
* This is an accurate historical claim, confirmed by reading all three
  versions directly rather than trusting the report's account of them.

### Sweep-found fix 2 (`phase-4b.txt`, "four of the others set no trace at all") -- **closed** [ran]

* Enumerated all 11 programs listed in `rust/corpus/phase-4b.txt` and
  grepped each `.rex` file (case-insensitive, whole file, not just
  line-start) for any `TRACE` clause:
  * No trace clause at all: `interpret_dynamic.rex`,
    `interpret_error_echo.rex` (one prose hit for the word "trace" in a
    comment, not an instruction), `call_procedure_expose.rex`,
    `use_arg_forms.rex` -- **exactly four**.
  * `trace r`: `call_return.rex`, `call_expression.rex`, `signal_forms.rex`,
    `condition_traps.rex`, `push_queue.rex`, `loop_retest_blame.rex` -- six.
  * `trace i`: `raise_array_substitution.rex` -- the subject program itself.
* "Four of the others set no trace at all" is exactly right, counted from
  the files, not from the corpus-file's own prose.

---

## Seventh-instance check: **no**

I audited every added sentence in the diff (`review-f02c3ed3..50da3045.diff`,
13 KB, 3 files) for a countable claim, an "every other X", a claim about a
mutable repo aggregate, or a claim falsifiable by its own commit. None found.
Specifically checked and confirmed true by running or reading source:

* `grep -c 'tracing_intermediates()'` -> 1 (the comment no longer quotes it).
* The four `self.read(code, ...)` call sites and which pair with
  `novalue_check` (1 pairs, 2 discard, matching "one... the rest").
* `read_by_name`'s only other caller is `tail_key` in `stem.rs`, not
  `PROCEDURE EXPOSE`.
* `trace_assignment` carries its own `intermediates` gate (read the function).
* `tracing_intermediates()` is textually `trace_mode().intermediates`.
* The `read_by_name` history across all three commits (round 1 introduced,
  round 2 changed the code but not this sentence, round 3 fixes the sentence).
* `phase-4b.txt`'s "four of the others set no trace" (counted from the
  eleven `.rex` files directly).

Every one of these checks out. The one sentence carrying a residual
call-site count ("one pairs and the rest...") is phrased without a specific
number that could go stale in the same way ("the rest" adapts to any count
>= 1, unlike "986" or "grep -c ... is 1"), and the report's own framing
(pairing is a *per-site decision*, not a census) does not depend on it being
exactly two -- it is supporting detail, not the load-bearing claim. I do not
think this is a new instance of the defect class; it is weaker sourcing than
ideal but not false, and not falsifiable by its own commit.

---

## Benchmark re-run [ran]

Release build (`cargo build --release -p rexx-exec --bin rexx-run`),
`do ii = 1 to 2000000 ; total = total + ii ; end` under `trace o` (explicit
`TRACE OFF`), two fixed binaries built from the pre-mutation and mutated
(`if true` instead of `if self.tracing_intermediates()`) trees, alternated
round-by-round to control for drift:

| round | with gate (s) | without gate (s) |
|---|---|---|
| 1 | 3.546 | 3.632 |
| 2 | 3.553 | 3.617 |
| 3 | 3.564 | 3.626 |
| 4 | 3.518 | 3.610 |
| 5 | 3.527 | 3.641 |

With-gate mean 3.5416 s (range 3.518-3.564), without-gate mean 3.6252 s
(range 3.610-3.641) -- **the ranges do not overlap** across all five rounds.
Delta: 0.0836 s / 2,000,000 passes = **~41.8 ns/pass**, matching the
comment's claimed "about 40 ns per pass" closely. (An earlier, non-interleaved,
3-run-per-side pass gave a noisier ~92 ns/pass; the interleaved 5-round
comparison above is the one I trust, and it lines up with the comment.)
Absolute wall-clock times differ from the report's (mine ~3.5 s vs. the
report's ~3.13 s) -- expected, different machine/load -- but the per-pass
**delta** is the load-bearing number, and it reproduces. The pre-mutation
file was restored and `md5sum`-verified identical afterward; `git status
--porcelain` was empty at the end.

**Verdict: the benchmark number is real. The gate is worth keeping on the
evidence given.**

---

## Taxonomy stress test (task item 4): fourth label route found, no tenth letter

### "All nine of `check_trace_setting`'s accepted letters" -- **holds, no tenth letter** [ran + reasoned]

* Read `TraceSetting::parseTraceSetting` directly
  (`interpreter/execution/TraceSetting.cpp:135`-`220`, the oracle source,
  read-only): the `switch` has cases for exactly `?` (a toggle, not a
  letter), `A`, `C`, `L`, `E`, `F`, `N`, `O`, `R`, `I` -- nine letters -- and
  a `default` that returns the offending byte as an error. Exhaustive by
  construction; there is no tenth case anywhere in the function.
* Empirically probed the live oracle with 17 other letters
  (`D S T U V W X Y Z B G H J K M P Q`) as a bare `TRACE` setting: **all
  17 raised Error 24.1** ("TRACE request letter must be one of \"ACEFILNOR\";
  found ...") with no exceptions.
* No tenth letter exists. This claim is sound, and the primary source
  (the C++ switch statement itself) settles it more strongly than any
  amount of testing could.

### "All three ways a label is reached" -- **a real fourth way exists; not part of this round's diff** [ran]

`trace.rs:120`'s doc comment ("measured across all three ways a label is
reached") and `trace_labels.rex`'s header ("the three ways a label is
reached all echo... fallen through, a CALL target (SUB)..., and a SIGNAL
target (THERE)") are **pre-existing text from Task 9 review round 1 (F8)**,
not touched by this round's diff (`review-f02c3ed3..50da3045.diff` touches
only `phase-4-exclusions.txt`, `phase-4b.txt`, and `run.rs`; `trace.rs` and
`trace_labels.rex` are untouched). It does not bear on the "seventh false
statement in this commit" question, but the controller's task asked me to
stress the taxonomy here specifically, and it found something:

Probed the oracle with an internal *function call* reaching a label under
`TRACE L` (a fourth syntactic route, distinct from `CALL sub` as a
statement -- this crate's own docs elsewhere treat `ExprKind::Call` as a
genuinely separate construct from `InstructionKind::Call`, Task 4 vs.
Task 3, with its own separate corpus witness):

```
trace l
say 'before'
x = sub()
say x
exit
sub:
procedure
say 'in the callee'   -- (probe variant without this line used for the run below)
return 42
```

Oracle stderr: `     6 *-*   sub:` -- the label echoes exactly as it does
under `CALL sub`. This crate's own release binary produces **byte-identical**
stdout and stderr for the same program (diffed both descriptors, both
empty). So:

* **No behavioural bug** -- this crate already handles the fourth route
  correctly (`eval_call` shares the same nested-activation machinery
  `InstructionKind::Call` uses, per `eval.rs`'s own module doc, so the label
  echo falls out of the same code path for free).
* **But the exhaustiveness claim is incomplete.** "Three ways" enumerates
  fallthrough, `CALL` (statement), and `SIGNAL`; it omits the internal
  function-call expression form, which is a fourth syntactic route this
  crate treats as its own construct everywhere else (separate task, separate
  AST node, separate corpus file `call_expression.rex`). `trace_labels.rex`'s
  witness likewise does not exercise this route.
* This is exactly the shape the report's own concluding paragraph predicted
  ("exhaustiveness over a language is a weaker guarantee than a transcript,
  and if a seventh instance is going to come from anywhere it is one of
  those two"). It found the right spot; this review just confirms a hit.

---

## NEW-F1's deletion (task item 5): mostly clean, one true-but-uncounted casualty

**Confirmed [ran]:** nothing false and load-bearing was removed. The deleted
paragraph's two substantive claims were:

1. "It is the only pre-gate of its kind in this file" -- true (per
   `rereview2.md`'s own verification, not re-derived here), but decorative:
   nothing in the code depends on this being true, and it was the exact
   claim whose evidence (the self-quoting `grep -c`) was the defect.
   Correctly deleted rather than restated with a number.
2. "`step`'s own `Assignment` arm builds its `rendered` unconditionally
   because `trace_result` and the write both need it." **Verified true**
   [ran]: `run.rs:957` builds `rendered` unconditionally, before the match
   on target kind, and it feeds both the unconditional `trace_result` call
   (`run.rs:958`) and each arm's `trace_assignment` call -- unlike
   `bind_control`'s site, where the only consumer of `name`/`rendered` is
   the `intermediates`-gated `trace_assignment`, so gating the build there
   is safe. This sentence was true, not falsifiable by its own commit (it
   states a simple, stable code-structure fact, closer to the "immutable"
   end of the taxonomy than the "mutable aggregate" end), and it was the
   one piece of context that explained *why* bind_control's site differs
   from the sibling the earlier, false version of the comment had wrongly
   generalised from. It is not restated anywhere in the new comment.

This is not a false statement removed, and not a correctness risk -- the new
comment's "the compiler and the profiler answer it" stance covers a reader
who wants to re-derive this by inspection. But the task asked specifically
whether the deletion removed something true and load-bearing, and by a
strict reading it removed one true, mildly load-bearing explanatory fact
alongside the false claim it was originally cited to support. Marking this
**parkable**: real, minor, not urgent, and arguably the right trade given
the round's own stated goal (delete decoration, don't try to save the true
parts of a paragraph built to support a false claim).

---

## Summary

* **NEW-F1: closed.** Survey deleted, not corrected; benchmark re-run and
  reproduces (~42 ns/pass here vs. ~40 ns/pass claimed, clean separation
  across 5 interleaved rounds).
* **NEW-F2: closed.** Enumeration re-run against the current tree; "one
  pairs, the rest discard" and the signature-based LESSON both check out.
* **Both sweep-found fixes: closed.** `read_by_name`'s history traced across
  all three commits and matches the claimed timeline; `phase-4b.txt`'s
  "four of the others set no trace" is exactly right, counted from the
  eleven `.rex` files.
* **Seventh false statement in this commit: no.** Every added sentence
  checked out against the running code or direct history. `rust/CLAUDE.md`'s
  new rule (`6eddd10d`, technically the commit after the one under review)
  was also read for the same defect and found clean -- its counts match
  what earlier rounds already established.
* **Benchmark: confirmed real**, ~40 ns/pass order of magnitude, clean
  separation, no overlap across 5 interleaved rounds.
* **Fourth label route: found.** Internal function call (`x = sub()`) is a
  fourth syntactic way to reach a label under `TRACE L`, omitted from
  `trace.rs`'s "three ways" claim and from `trace_labels.rex`'s witness.
  No behavioural bug (this crate matches the oracle byte-for-byte on it
  already) -- a documentation/witness-completeness gap in **pre-existing**
  text, not in this round's diff. **Parkable**, and it is the concrete
  instance the report's own risk flag predicted.
* **Tenth letter: not found.** Nine is exhaustive, confirmed by reading the
  oracle's own `switch` statement and by probing 17 other letters live.
* **NEW-F1's deletion: mostly clean.** One true, non-falsifiable, mildly
  informative sentence (the `step`-Assignment-arm contrast) was deleted
  along with the false claim it supported, and is not restated elsewhere.
  **Parkable** -- not a correctness issue, not a new false statement, just
  a minor loss of explanatory context.
