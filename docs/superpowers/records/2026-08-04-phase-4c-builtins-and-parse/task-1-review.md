# Phase 4c Task 1 review -- boundary infrastructure and the four attributions

Reviewing `aa7b3505` against `7d8c43db`. `0786973b` (plan-document-only, by the
controller) ignored throughout.

**Spec compliance: PASS.**
**Quality: CHANGES-REQUESTED** -- two findings, both one-liners, both in the
defect class this task exists to close.

Nothing in the repository was modified by this review. `git status --porcelain`
is empty at the end, exit 0. Every mutation below was applied to a `tar`-copied
tree under the session scratchpad (`mut/repo`, 141 MB, `.git` and
`rust/target` excluded), never to a tracked file.

---

## 1. The verify block, re-run

Run from `rust/`, each status read unpiped.

| command | exit | result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1031 passed, 0 failed, across 71 test binaries |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | `mode: STRICT (the gate)`, `42 of 42 matching` |

The clippy run above is warm, which `rust/CLAUDE.md` says to treat as
provisional. So it was re-run **cold** on the pristine scratch copy with
`target/debug/.fingerprint` removed, forcing a full re-lint of all crates:
**exit 0, zero `^warning`/`^error` lines**. The green is real, not a
cached result.

All four numbers agree with the implementer's report.

---

## 2. Attack 1 -- can the harness pass while doing nothing?

**No, on the oracle side. Yes, on ours.** The invocation count is asserted and
it is a count, not a flag: `Oracle` carries an `AtomicUsize` incremented
*inside* `run`, immediately before the `Command` is built
(`tests/support/oracle.rs:153`), and `builtin_status.rs:472-484` asserts it
equals both `run.in_scope.len()` and the literal 66.

**The implementer's claim, reproduced independently.** I replaced the
per-name `measure(...)` call in `classify()` with a stub returning
`Status::Loud` and a synthesised `rexx-exec: routine "{name}" is not
implemented (4c)` message, running nothing:

```
test result: FAILED. 10 passed; 1 failed; 0 ignored
assertion `left == right` failed: the oracle was invoked 0 times for 66
in-scope builtins; ...
  left: 0
 right: 66
exit 101
```

Exactly as reported: ten of eleven green, and only the invocation count red.
Set equality in both directions, every count, the loud-names-itself check and
the known-gap check were all satisfied by a classifier that started no
process. The claim stands as measured.

**The residual hole (Finding 2).** The counter is on the oracle side only.
Nothing asserts that *our* interpreter ran. I mutated `measure` to keep
`oracle.run(&abs)` -- so the 66-invocation assertion is satisfied honestly --
and replaced `rexx_exec::run_program(path_str, text)` with a canned
`Outcome { exit_code: NOT_IMPLEMENTED_EXIT, stderr: "rexx-exec: routine
\"{name}\" is not implemented (4c)\n", .. }`:

```
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
exit 0
```

All eleven green with the executor never invoked. This is bounded -- the
moment any row is committed `implemented`, an all-`Loud` stub flips that row
and the run goes red -- but at *this* commit, where all 66 rows read `loud`,
the harness's entire positive content is "our side exits 120 naming its own
builtin", and that is exactly what the stub fakes. The task's own thesis is
that a differential harness must not be able to pass without running the
programs; the argument was applied to one of the two interpreters.

The fix is the same shape as the one already taken: count the executor-side
runs too and assert `rust_invocations == 66` beside the oracle's. A free-
function counter in `builtin_status.rs` incremented inside `measure` is not
enough (a stub could increment it); it wants to sit at the single call site
of `run_program`, the way the oracle's sits inside `Oracle::run`.

---

## 3. Attack 2 -- can it pass with wrong builtins?

**No.** `Status::Implemented` is reachable only through `diffs.is_empty()`
where `diffs = descriptor_diffs(&rust, &cpp)` and `cpp = oracle.run(&abs)`
(`builtin_status.rs:225-237`). The comparison reaches the oracle
unconditionally for every in-scope name; there is no name table anywhere on
the path from probe to status.

Verified by mutation rather than by reading. I replaced the executor's
`Failure::Loud` arm in `lib.rs` with `0` -- no stderr, exit 0, nothing
printed, which is the observable behaviour of 66 stubs that return `''`:

```
  ABBREV: committed loud, measured divergent
        differing: [stdout]
  ABS: committed loud, measured divergent
        differing: [stdout]
  ...
exit 101
```

Every row `divergent`, none `implemented`. And a `divergent` row cannot be
absorbed by a one-line edit: committing one requires `KNOWN GAP: <NAME>` in
the exclusions file. I made that guard fire by hand -- flipping `LENGTH`'s
committed row to `divergent`:

```
LENGTH is committed as divergent, which records a wrong answer rather than a
missing one. That needs a row reading "KNOWN GAP: LENGTH" in
.../phase-4-exclusions.txt, naming it and saying what is wrong.
exit 101
```

The guard is watched, not merely written. The 66-stubs defeat is closed.

---

## 4. Attack 3 -- robustness to the later task that removes the loud message

**Robust: it goes red, not green.** I replaced the `Failure::Loud` arm with a
43.1-shaped report on stderr at exit 213, which is what the harness would
observe after that task lands:

```
  ABBREV: committed loud, measured divergent
  ABS:    committed loud, measured divergent
  ...  (all 66)
exit 101
```

The old classifier keyed on the *absence* of a message and so would have
called all 66 `implemented`. This one keys on a three-descriptor match
against a live oracle run, so removing the loud path moves every unimplemented
row to `divergent`, which is a hard failure needing a `KNOWN GAP` row apiece.
Noisy if that task lands before the builtins do, but never silent. The third
defeat is closed.

One note for the plan, not a defect: whoever lands the 43.1 change should land
it after the builtins, or expect to write 66 `KNOWN GAP` rows. Worth a
sentence in that task's brief.

---

## 5. Attack 4 -- are the 66 probes meaningful?

Every row of `rust/corpus/builtin-probes.txt` read individually. 66 rows, no
duplicates, all in-scope names covered (the harness asserts both directions
before anything runs, `builtin_status.rs:303-316`). No probe nests a builtin
call, which is what makes `every_loud_row_is_loud_about_its_own_builtin` a
usable check.

Sixty-two of the sixty-six elicit a value a wrong implementation cannot
produce by accident. Four do not, and one of those is a genuine defect.

**Finding 1 (Important) -- `TIME` is an identity conversion.**

```
TIME	say time('N','12:34:56','N')
```

Input format `N`, output format `N`. Measured on the oracle: stdout
`12:34:56`, byte-identical to the argument. An implementation of `TIME` that
does nothing but `return arg(2)` matches the oracle on all three descriptors
and would be recorded `implemented`. That is precisely the failure the probe
file's own header forbids -- "it is satisfied by an implementation that exists
and computes nothing" -- and it makes the header's claim that each row "prints
a non-empty result the builtin *had* to compute" false for this row. The
implementer caught this shape once (`ADDRESS`) and missed it one row away.

Measured alternative, same shape as `DATE`'s (which converts `I` to `S` and is
correct): `say time('S','12:34:56','N')` -> `45296`. A stub echoing its
argument fails that.

**Finding 3 (Minor) -- three rows are satisfiable by a constant.** Measured
on the oracle in one run:

| row | probe | oracle | satisfied by |
|---|---|---|---|
| `VAR` | `zz = 1; say var('zz')` | `1` | `return 1` |
| `SYMBOL` | `zz = 4; say symbol('zz')` | `VAR` | `return "VAR"` |
| `GC` | `say gc('Force')` | `1` | `return 1` |

Each is one call away from the standard `ABBREV` already sets in this same
file (`say abbrev('Print','Pri') abbrev('Print','Pro')` -> `1 0`, both
branches in one row). Measured: `say var('zz') var('qq')` -> `1 0`,
`say symbol('zz') symbol('qq')` -> `VAR LIT`, `say gc()` -> `0`. Cheap to
strengthen and worth doing before the tasks that tick these rows.

`RANDOM` (`say random(5,5)` -> `5`) is the same shape but is *intrinsically*
weak: determinism is a hard requirement of this file and every deterministic
form of `RANDOM` is satisfiable by a constant. Recording it, not asking for a
change.

**Not defects, checked:**

* `SOURCELINE` -- `say sourceline(1)` is the probe's own text; both
  interpreters read the same canonicalised absolute path, so it is
  deterministic and genuinely self-referential.
* `TRACE` -- `trace off; say trace()` -> `O` requires reading live state; a
  constant would have to be `N`.
* `DIGITS`/`FORM`/`FUZZ` -- each sets a non-default `NUMERIC` value first, so
  a constant-default stub fails.
* `QUEUED` -- `queue 'a'; queue 'b'; say queued()` -> `2`, in-program only,
  which is what its own partial row permits.
* `DATE` -- `I` to `S` strips the hyphens; not an identity.
* `ADDRESS` -- the correction the implementer made is right, and verified:
  `say address()` alone is the platform default the exclusions file assigns
  to Phase 7. `address zork; say address()` -> `ZORK` is the 4c half. Note
  that this row currently measures the ADDRESS *instruction*, not the
  builtin: `rexx-run` on it gives `rexx-exec: ADDRESS is not implemented
  (4c)` at rc 120, so the row cannot flip until both halves land. Correct
  behaviour, worth knowing.

---

## 6. Attack 5 -- the counts

Verified directly against the committed file:

```
/bin/grep -ac "\texcluded$"     -> 15
/bin/grep -ac "\tloud$"         -> 66
/bin/grep -ac "\timplemented$"  ->  0
/bin/grep -ac "\tdivergent$"    ->  0
data rows                       -> 81
builtin-probes.txt data rows    -> 66
```

`NAMES` is 81, `EXCLUDED` is 18, `PARTIALLY_EXCLUDED` is `VALUE`/`ADDRESS`/
`QUEUED`, `wholly_excluded()` is 15, `in_scope()` is 66. The known 63 trap was
avoided, and it was avoided *structurally* rather than by getting an arithmetic
right once: `EXCLUDED`'s own doc comment says "taking the length of this list
instead gives eighteen where the answer is fifteen", `wholly_excluded()` is a
named function rather than a `- 3`, and `coverage.rs` asserts
`PARTIALLY_EXCLUDED` is a subset of `EXCLUDED` -- without which
`wholly_excluded()` would silently return 16.

---

## 7. Attack 6 -- the four attributions

**`+++`'s owner.** `trace_oracle.rs:535` reads `("+++",
Coverage::Owned("Phase 7"))`. `WITNESSED_PREFIX_COUNT` (13) and
`OUT_OF_SCOPE_PREFIX_COUNT` (6) are untouched, as Step 5 requires; they moved
from `:551`/`:555` to `:557`/`:561` because the doc comment above them grew.
The `PREFIX_COVERAGE` doc bullet was rewritten; it previously said "4c", "two
producers" and called the `TRACE ?` row "deliberately owner-unassigned", and
all three would have been false after Steps 5 and 6.

**The six C++ citations, each read in `/home/moritz/dev/repos/ooRexx`:**

| citation | line reads | verdict |
|---|---|---|
| `RexxActivation.cpp:4468` | `traceValue(rc_trace, TRACE_PREFIX_ERROR);` | RESOLVES |
| `RexxActivation.cpp:4024` | `buffer->put(PREFIX_OFFSET, trace_prefix_table[TRACE_PREFIX_ERROR], PREFIX_LENGTH);` | RESOLVES |
| `RexxActivation.cpp:4305` | `if (inDebug() && !settings.wasSourceTraced())` | RESOLVES |
| `RexxActivation.cpp:4237` | `processTraceInfo(activity, Interpreter::getMessageText(Message_Translations_debug_prompt), ...)` | RESOLVES |
| `Activity.cpp:1496` | `RexxString *text = Interpreter::getMessageText(Message_Translations_debug_error);` in `displayDebug` | RESOLVES |
| `AddressInstruction.cpp:163` | `context->command(environment, _command, getIOConfig());` | RESOLVES |

The "one of `command()`'s two callers" claim also holds: `void command(RexxString *,
RexxString *, CommandIOConfiguration *config);` is declared at
`RexxActivation.hpp:224` and `/bin/grep -arn -- "->command(" interpreter/`
returns exactly two hits, `AddressInstruction.cpp:163` and
`CommandInstruction.cpp:89`, with no other call form. `TRACE_PREFIX_ERROR` is
`"+++"` at `RexxActivation.cpp:3570`. The message texts are at
`RexxErrorMessages.h:725-726`.

The live `+++` measurement is reproduced: under `trace e`, `address sh` and
`'exit 3'` give stderr `+++   "RC(3)"` at rc 0; under `trace n` the same
program emits nothing at all. The implementer's correction 6.2 -- that the
brief's text omitted the load-bearing trace setting -- is right, and the
committed row carries both the setting and the negative control.

**Finding 4 (Minor) -- "four producers" undercounts the emission sites by
one.** `Message_Translations_debug_error` is fetched at `Activity.cpp:1496`
*and* at `Activity.cpp:1507`, the secondary-message branch of the same
`displayDebug`, and each is followed by its own
`displayUsingTraceOutput`. Four *producers* read as four distinct paths is
defensible, and the C++ is an out-of-repo enumeration so prose is permitted
here -- but the row's own device is exhaustive enumeration, and a reader
grepping `+++` finds five call sites, not four. One clause ("`:1496` and its
secondary-message twin at `:1507`") settles it.

**The `TRACE ?` row.** "Owner unassigned" is gone; the row reads `Owner:
Phase 7, with the rest of interactive debug.` All three added claims were
re-measured independently, not read:

* stdin two lines `echo ONE` / `echo TWO`, program `say 'A'` / `pull v` /
  `say '<'v'>'`. With `trace ?r`: stdout `A` then `<>`, stderr carries both
  banner lines plus `/bin/sh: 1: ECHO: not found` twice. **`trace ?r` drains
  stdin and issues each line as a shell command, and the following `PULL`
  reads `""`.** Confirmed.
* The control: same stdin, no `TRACE` instruction -> stdout `A` then
  `<ECHO ONE>`. So "only stderr differs" is a `/dev/null`-only property.
  Confirmed.
* `RXTRACE=ON`, no `TRACE` instruction, empty stdin -> the same two banner
  lines and the same drained `PULL`, line numbers 1/2/3. Confirmed.

**The design spec.** `2026-07-30-phase-4a-executor-design.md:71` now reads
"every directive except `::ROUTINE`, which is 4c's (see the 4c plan's D-R)",
and the `QualifiedCall` row carries the carve-out note with the reason it is
unaffected (namespaces come from `::REQUIRES`, which is on the Phase 5 side).

**The D4 reasons.** All fifteen whole exclusions have a reason sentence, and
the three non-obvious ones are there. The `SETLOCAL`/`ENDLOCAL` correction is
verified: `say endlocal()` unpaired prints `0`; `s = setlocal(); say s
endlocal()` prints `1 1`. D4's "both return 1" was true only for the pair, and
the committed row now says so.

---

## 8. Attack 7 -- the implementer's five concerns

| # | claim | assessment |
|---|---|---|
| 6.1 | the `ADDRESS` probe shape the brief invites is the excluded half | **Right, and acted on.** Verified against the exclusions file's partial row. No further action. |
| 6.2 | the `+++` measurement omits the trace setting | **Right, and acted on.** Reproduced above: `trace n` emits nothing. Recording it in the file was the correct move. |
| 6.3 | "(`:124` is about the trace gate)" is wrong | **Right.** Read at the parent commit: `:99-100` is the ownership sentence; `:105-106` is the two-condition trace gate; `:123-124` is the `THREE MEASURED WAYS ... which 4c will have to meet:` heading. The brief's parenthetical was wrong and the substantive point stands. Recording it is enough; the 4c plan's D-R carries the same mischaracterisation and the controller's follow-up commit addresses it. |
| 6.4 | D4's "both return 1" is true only for the pair | **Right.** Measured above. Corrected in the file, which is where it belongs. |
| 6.5 | Step 7 describes an edit to a citation the file does not make | **Right.** `/bin/grep -an "phase-4-exclusions.txt:"` on that file returns nothing; it never cites itself by line. The chosen alternative -- naming the ownership sentence by content and recording that it has been fetched from the wrong place once -- is better than the literal instruction, and is required by `rust/CLAUDE.md`'s "a claim must not be falsifiable by the act of committing it". No action; already ratified by `0786973b`. |

All five are correct. Three changed what shipped, and each change is an
improvement on the brief rather than a deviation from it.

---

## 9. Attack 8 -- the `EXCLUDED_BUILTINS` move

`rexx-inventory/build.rs` is **untouched** by this commit (`git diff --stat`
over the crate shows `src/lib.rs` only, +82/-2). `NAMES` generation is
therefore unchanged, and the crate's module doc now distinguishes the
generated tables from the one hand-written policy list, which it did not
before.

`coverage.rs` reaches the list through `use rexx_inventory::builtins::EXCLUDED
as EXCLUDED_BUILTINS;`, so its pre-existing assertions -- every excluded name
is in `NAMES`, no duplicates, `EXCLUDED.len() == 18`, `NAMES.len() == 81`,
`in_scope == 66` -- read the same identifier over the same data and still
genuinely assert. They are not merely compiling: `the_builtin_exclusion_set_
matches_the_committed_file` fails if any name is renamed on either side, and
the `- 3` magic number is gone in favour of `wholly_excluded()`.

Of the three new assertions there, two add coverage that nothing else has:
`PARTIALLY_EXCLUDED ⊆ EXCLUDED` (without it `wholly_excluded()` silently
returns 16 for a typo'd partial name) and `wholly_excluded().len() == 15`. The
third, `in_scope().len() == names.len() - wholly_excluded().len()`, is very
nearly implied by the assertions above it -- it can only fail if `NAMES`
contains a duplicate of an excluded name. Not a defect, and cheap; noted only
because "can fail" is not "adds coverage" and this one barely can.

**Does `builtin_status.rs` add coverage over the existing suite?** Yes.
`tests/loud.rs` has no builtin-name coverage at all (`/bin/grep -an "builtin"`
on it returns nothing), and nothing else in the tree pins per-builtin oracle
values. The four new tests are not redundant with anything.

---

## 10. Every assertion, and how to make it fire

| assertion | how it fires | fired? |
|---|---|---|
| derived set ⊄ committed (missing) | delete a row | implementer: fails naming `LENGTH` |
| committed set ⊄ derived (extra) | add `ZORKOLO` | implementer: fails naming it |
| row status differs | hand-flip `loud` -> `implemented` | implementer: fails with both sides' output |
| `derived.len() == NAMES.len()` | a name leaves `BuiltinFunctions.cpp` | not fired; out-of-repo referent, cannot be forced locally |
| `excluded == wholly_excluded().len()` and `== 15` | change `PARTIALLY_EXCLUDED` | covered by `coverage.rs`'s own subset assertion |
| in-scope `== 66` | same | same |
| **oracle invocations `== 66`** | name-table classifier | **fired here**, 10/11 green, only this red |
| loud row names its own builtin | nest `c2x(...)` in a probe | implementer: fails naming `BITAND` and `C2X` |
| divergent row needs `KNOWN GAP` | commit a `divergent` row | **fired here**, fails by name with the required string |
| `mentions_as_word` both directions | unit test with `WORD`/`WORDS` | pinned in the file itself, both polarities |

Step 4.3 (delete a dispatch arm, watch a row flip `implemented` -> `loud`) is
correctly owed by Task 2: no builtin is dispatched at this commit, so there is
no arm to delete and no `implemented` row to flip. Nothing weaker was
substituted in its place, which is the right call.

---

## 11. Style and hygiene

* No `unsafe`; the workspace still sets `unsafe_code = "forbid"`.
* Zero em-dashes across all seven new or changed source and corpus files.
* No comment states where the implemented/not-implemented boundary sits in
  prose. The two literals that do (`15`, `66`) are inside `assert_eq!`, which
  is what `rust/CLAUDE.md` requires: assert the boundary or delete the
  sentence.
* `fresh_run_root` names the directory with pid and nanoseconds, so two
  concurrent runs of the binary cannot share a probe search path -- the
  scratchpad-contamination hazard, handled at the right level.
* Both interpreters get the same canonicalised absolute path, which matters
  because a raised condition's report names the program by path.
* `Oracle::run` now sets `Stdio::null()` on the child, a deliberate change
  from inheriting `cargo test`'s stdin. The strict corpus gate still reports
  42 of 42, which I re-ran and confirmed.

---

## 12. Cannot verify from the diff

* **The two-pass determinism control** (`batch2.sh`, each of the 66 run twice
  from two fresh directories, zero `NONDET-*` rows). The scripts live in the
  session scratchpad, not the repository. The property is plausible on
  inspection -- no probe reads the clock, the pid or an entropy source, and
  `DATE`/`TIME`/`RANDOM` are all pinned to deterministic forms -- and the
  harness passing twice in my own runs is weak corroboration, but the 66x2
  control itself is not reproducible from what was committed.
* **`rustc 1.97.1` and the `set_var` unsafety check** (report 2.6). Not
  re-run; nothing in the diff depends on it.
* **`pgrep -a rxapi` -> `885 rxapi`** at the time of the `RXQUEUE`
  measurement. Host state at that moment, unrecoverable.

---

## 13. Findings

1. **Important.** `TIME`'s probe (`say time('N','12:34:56','N')`) is an
   identity conversion: measured, the oracle prints its own argument back, so
   an implementation that returns `arg(2)` classifies `implemented`. This is
   the exact defeat the probe file's header forbids and makes that header
   false for this row. Fix: `say time('S','12:34:56','N')` -> `45296`.
2. **Important.** The harness passes with the executor never invoked --
   verified by mutation, 11/11 green at exit 0 with `run_program` replaced by
   a canned `Outcome` while the oracle still ran 66 times. The invocation
   count closes the oracle side only. Add the symmetric assertion.
3. **Minor.** `VAR`, `SYMBOL` and `GC` are each satisfiable by a constant;
   one extra call per row fixes all three (`var('qq')` -> `0`, `symbol('qq')`
   -> `LIT`, `gc()` -> `0`), matching the standard `ABBREV` sets in the same
   file. `RANDOM` is intrinsically weak and needs no change.
4. **Minor.** `trace_oracle.rs`'s "four producers" and the exclusions row's
   matching sentence undercount the `+++` emission sites by one:
   `Activity.cpp:1507` is a second `Message_Translations_debug_error` fetch in
   the same `displayDebug`, each with its own `displayUsingTraceOutput`.
5. **Minor.** `coverage.rs`'s new `in_scope().len()` assertion is nearly
   implied by the assertions above it; it can only fail on a duplicate in
   `NAMES`. Cheap, not wrong, recorded under "can fail is not adds coverage".
6. **Minor, for the plan not this task.** When the task that replaces the loud
   path with a real 43.1 raise lands, every still-unimplemented row becomes
   `divergent` and needs a `KNOWN GAP` row. Loud, never silent -- but that
   task's brief should say to land it after the builtins.

Findings 1 and 2 are why this is CHANGES-REQUESTED rather than APPROVED: both
are in the precise defect class the rewrite exists to close, both are
one-liners, and both are cheaper to fix now than after fourteen tasks have
ticked rows against them.
