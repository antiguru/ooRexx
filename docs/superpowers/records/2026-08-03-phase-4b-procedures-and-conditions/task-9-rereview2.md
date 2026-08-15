# Task 9 re-review -- fix round 2 (`f02c3ed3`)

Scope: did fix round 2 close NEW-1 through NEW-6 (round 1's re-review findings),
and did it introduce anything new. Task 9's earlier rounds are settled; not
re-litigated.

Every claim below is labelled **[ran]** or **[reasoned]**.

## Method

* HEAD at the start of this review was `f02c3ed30dfe5f7360d7b7c6a9f8bdf6588ff79e`
  (the commit under review), tree clean. Confirmed with `git rev-parse HEAD`
  and `git status --porcelain`.
* Oracle runs used the mandated wrapper (`ulimit -v 1048576`), three
  descriptors captured separately, never `2>&1`, from directories `mkdir`'d
  for this review under the scratchpad (`t9r2/probes`, `t9r2/probes2`,
  `t9r2/probes3`) -- never the scratchpad root.
* Tracked files were mutated 6 times, each on `run.rs` or `trace_oracle.rs`.
  Before any mutation each file was `cp`'d to `t9r2/backup/`; every restore
  was `cp` **from that copy**, never `git checkout --`. `git status --porcelain`
  and `git diff --stat` are empty at the end of this review, and every restore
  was additionally checked by `md5sum` against the pre-mutation value.
* A baseline binary at `94237403` (the commit before this round) was built via
  `git archive 94237403 rust | tar -x` into the scratchpad plus a symlink to
  `../../../interpreter` for the build script -- no worktree, no git state
  touched.
* Every test run below was read for its run count, not only its exit status.

## Gates, re-run [ran]

| gate | result |
|---|---|
| `cargo test --workspace` | **990 passed / 0 failed** (summed over all `test result:` lines) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **41 of 41**, STRICT |
| `cargo test -p rexx-exec --test assertions` | 9 passed (assertions_differential + the_exempt_set... both `ok`), 4224/4259 in the differential report |
| `cargo fmt --all --check` | exit 0, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

All match the report's claimed table exactly.

---

## Per-finding verdict

### NEW-1 -- **closed** [ran]

* The fix (`Interp::read` + `novalue_check`, replacing `read_by_name`) is in
  `run.rs:5109`-`5110`, matching the diff.
* **The core repro, run against both sides**: `signal on novalue name nv` /
  `do ii = 1 to 3 ; drop ii ; end`. Oracle: stdout `handler II`, rc 0, stderr
  empty. Our binary at `f02c3ed3`: byte-identical -- stdout `handler II`, rc 0,
  stderr empty. `diff` confirms.
* **The ordering claim -- survives, and its sharpest sub-claim survives too.**
  Under `trace i`, oracle and head produce byte-identical stderr (`diff`
  empty) whose failing re-test shows the `DO` re-echo and then nothing --
  no `>V>`, no `>>>` -- before jumping to the handler label. I then mutated
  `run.rs` to move `self.novalue_check(novalue)?` from *before*
  `trace_variable`/`trace_result` to *after* them (**R2-M2**), rebuilt, and
  re-ran the same `trace i` probe against the mutated binary:
  * **stdout: identical to the oracle** (`handler II`).
  * **rc: 0 on both**, identical.
  * **stderr: diverges** -- the mutated binary now emits two extra lines the
    oracle never produces (`>V>     II => "II"` / `>>>     "II"`) before the
    handler runs.

  So the report's claim -- "stdout and rc would still agree" and only stderr
  discriminates -- is **exactly what happened**, not merely asserted.
* **R2-M1 re-run** (revert to `read_by_name`): `cargo test -p rexx-exec --test
  trace_oracle` -> **21 passed; 1 failed**, matching the report's own number.
* **R2-M2 re-run** (the ordering mutation): same result, **21 passed; 1
  failed**, matching the report.
* Both mutations restored from the pre-mutation copy; `md5sum` matched the
  original both times; `git status --porcelain` empty afterward.
* **No performance cliff.** [ran] Baseline (`94237403`) vs head (`f02c3ed3`),
  `total=0; do ii=1 to 2000000; total=total+ii; end; say total`, release
  builds, `time`: baseline 3.64s real, head 3.49s real -- no regression; if
  anything head is marginally faster (`Interp::read`'s slot-first path is at
  least as cheap as `read_by_name`'s name lookup).
* **Adjacent-shape probes, my own, independent of the report's eight** [ran]:
  untrapped `DROP` still 41.1 on `("II")`, byte-identical stderr both sides;
  `signal off novalue` inside the body, same 41.1, byte-identical; the trap
  armed in a caller while the loop runs in a callee (`CALL`), stdout `handler
  11` rc 0 both sides. All MATCH.
* **The discarded probe (`v07`, `drop ii ; ii = 2`) -- the drop was sound**
  [ran]. Re-ran it with `timeout 5` against both the oracle and our binary:
  **both time out (rc 124)**. The mechanism is genuine, not fix-specific:
  each pass re-tests by reading `ii` (now `2`, user-assigned) and adding `1`
  to get `3`, which is `<= 3` so the loop continues; the body then does
  `drop ii ; ii = 2` again, so `current` never advances past 3 and the loop
  never terminates on *either* interpreter. Discarding it was the right call.
* One probe of mine (`DO ii OVER s.`, a stem) surfaced `rexx-exec: DO is not
  implemented`, unrelated to this fix -- `DO OVER` on a **stem** is a
  pre-existing, already-documented gap (`phase-4-exclusions.txt`'s DO-OVER
  deviation; "DO OVER on a string or a number is fine", stems are not, and my
  first attempt used the wrong shape). Re-run with `DO ii OVER 'ab'` (the
  shape the report's probes and `controlled_loop.rex`'s own witness use):
  MATCH, rc 0, stdout `never` both sides -- `DO OVER` never calls the
  re-tested `Interp::read` path at all, so a dropped control variable there is
  inert on both sides. Not a finding; my own probe error, corrected.

### NEW-2 -- **closed** [ran]

`grep -n 'C\`/\`L\`/\`E\`' crates/rexx-exec/src/run.rs` -> no hits (the old
list is gone). `grep -n 'C\`/\`E\`/\`F\`/\`N\`/\`O\`' crates/rexx-exec/src/run.rs`
-> one hit at `run.rs:5805`, the corrected list. A new paragraph immediately
below explicitly says `L` "is not in that list" and was until round 1.

### NEW-3 -- **closed** [ran]

`cargo test --workspace` at this commit: **990 passed / 0 failed** (re-run
above). `grep -rn "986"` over `rust/` and `docs/` finds no occurrence of the
stale count anywhere relevant (only unrelated numeric literals in corpus
files). The module doc now says "leaves this file green" with no workspace
claim, and the following paragraph explicitly explains why a workspace claim
would have contradicted it -- the self-contradiction NEW-3 named is gone.

### NEW-4 -- **closed** [ran]

* `grep -c '>>>' tests/trace_oracle/*.expected`: `trace_labels.expected` is
  **0**; the other twelve are `2,9,5,9,3,4,1,6,2,4,13,2` -- all non-zero.
  Matches the table row's new wording ("every witness below **except
  `trace_labels.rex`**") exactly.
* The "three witnesses carry no prefix of their own" sentence is gone,
  replaced by a bulleted list of *kinds* with no count. I independently
  derived which witnesses satisfy "carries no prefix the prose table already
  names for it": `exit_value`, `control_variable_reread`, `trace_labels`,
  `control_variable_novalue`, `controlled_loop` -- **five**, matching the
  report's own count for what the old text would have needed to say, and the
  new bulleted list names exactly these five programs across its four bullets.
* **The two "every witness" rows are now real assertions, and they can fail**
  [ran]. Mutated `WITNESS_PREFIXES` twice, restored from a copy each time,
  `md5sum` confirmed exact restoration:
  * **R2-M3** (drop `*-*` from `exit_value`'s entry): `cargo test -p
    rexx-exec --test trace_oracle every_witness_still_emits_every_prefix_it_is_named_for`
    -> **FAILED, 0 passed; 1 failed; 21 filtered out**, panic message quotes
    `["exit_value"]`. Matches the report's row exactly.
  * **R2-M4** (drop `>>>` from `exit_value`'s entry, a *second* witness with
    no `>>>`): same test -> **FAILED, 0 passed; 1 failed; 21 filtered out**,
    panic message `left: ["exit_value", "trace_labels"] right: ["trace_labels"]`.
    Matches the report's row exactly.

### NEW-5 -- **partially closed** [ran]. See NEW-FINDING-1 below.

The substantive correction is right: `bind_control`'s pre-gate is in fact the
only site of its kind in `run.rs` (verified: exactly one real *call* to
`tracing_intermediates()` in the file, at the gate itself; `step`'s own
`Assignment` arm at `run.rs:957`/`961` does build `rendered`/`name`
unconditionally, as the new comment says). **But the comment's own named
verification command no longer produces the number it claims** -- see below.

### NEW-6 -- **closed** [ran]

`trace.rs:96`-`110`: the five-name list (`setTraceOff`/`setTraceNormal`/
`setTraceCommands`/`setTraceErrors`/`setTraceFailures`) no longer contains
`setTraceLabels`, and the paragraph immediately below explicitly says
"`setTraceLabels` used to be the sixth name in that list and is not any
more," with the NEW-6 attribution. No contradiction remains; "all five" is
literally five names, counted.

---

## New findings

### Important / load-bearing

**NEW-FINDING-1 [ran]. `run.rs:5296`'s comment quotes a `grep -c` command
against itself, and the command no longer returns the number the comment
claims.** The comment (added by this round, `bind_control`'s doc, NEW-5's
fix) reads:

```
// `grep -c 'tracing_intermediates()' crates/rexx-exec/src/run.rs`
// is `1`, and it is this line; ...
```

Run literally, today, against the committed tree:

```
$ grep -c 'tracing_intermediates()' crates/rexx-exec/src/run.rs
2
$ grep -n 'tracing_intermediates()' crates/rexx-exec/src/run.rs
5296:        // `grep -c 'tracing_intermediates()' crates/rexx-exec/src/run.rs`
5301:        if self.tracing_intermediates() {
```

The comment's own text contains the literal string it is grepping for
(inside backticks, as the command it is citing), so the act of writing "grep
gives 1" added a second match and made the sentence false the moment it was
committed. The underlying claim it is defending -- that `bind_control` is the
only *call site* of `tracing_intermediates()` in the file -- is still true
(one real invocation, at `run.rs:5301`; the other hit is prose, not a call).
But the sentence as literally written ("is `1`") is false against the exact
command it names, which is the precise defect class this round's own
dispatch was written to eliminate (NEW-3 and NEW-5 in round 1's re-review
were both of this shape: a claim whose named command does not, in fact,
produce the claimed number).

**NEW-FINDING-2 [ran]. `phase-4-exclusions.txt:1148`-`1150`'s new NOVALUE
paragraph claims a cross-site pairing pattern that does not hold.** The
added text:

```
The fix is the crate's own NOVALUE-aware reader (`Interp::read`, which
returns `(ObjRef, Novalue)`) plus `novalue_check`, which every other read
site in this crate already pairs with it.
```

Every call to `self.read(code, ...)` in `rexx-exec`, found by
`grep -n "self\.read(code" crates/rexx-exec/src/*.rs`:

| site | pairs with `novalue_check`? |
|---|---|
| `run.rs:2050` (`expose_names`, `PROCEDURE EXPOSE (v)`'s indirect list) | **no** -- `let (value, _novalue) = self.read(...)`, discarded |
| `run.rs:5109` (this fix, the loop control-variable re-test) | yes |
| `run.rs:5742` (`drop_variable`, `DROP (v)`'s indirect list) | **no** -- `let (value, _novalue) = self.read(...)`, discarded |
| `eval.rs:319` (`ExprKind::Variable`, ordinary expression evaluation) | yes |

Of the three *other* read sites, only one (`eval.rs:319`) pairs with
`novalue_check`; the other two explicitly discard the `Novalue` flag with an
underscore-prefixed binding. "Every other read site... already pairs with
it" is false as stated -- it is true of the one *evaluation* site and false
of the two *indirect-name-list* sites.

This also undercuts the "LESSON" paragraph immediately below it, which says
`read_by_name` "exists for `PROCEDURE EXPOSE`-shaped lookups that must not
raise." Checked: `read_by_name`'s only other caller in the crate is
`stem.rs:120` (`tail_key`, a compound's tail-piece resolution) --
**not** `PROCEDURE EXPOSE` or `DROP`'s indirect forms, which use
`Interp::read` with the `Novalue` discarded, not `read_by_name` at all. I
measured that `tail_key`'s use is itself correct (probed `a.=0; say a.i`
under `signal on novalue`: oracle and head both print `0` and do not trap,
matching -- a tail-piece variable reference does not raise `NOVALUE` on
either side), so there is no live behavioural bug here. But the "LESSON"'s
characterization of what `read_by_name` is *for* does not match what the
crate's own code does today, compounding the same paragraph's overclaim
about pairing.

Both findings sit inside the exact paragraphs this round added to fix NEW-3
and NEW-5 -- i.e. the fifth and sixth consecutive instances of a comment
correction shipping with its own new false claim (after NEW-2 through NEW-6
in round 1, this makes two more in round 2, both in text whose stated purpose
was accuracy about counts and cross-site patterns).

### Parkable

None beyond what the report already recorded (the completed-inner-loop
under-indent and the `TRACE()` DEVIATION-0 correction), both of which I
re-read but did not re-litigate; they are round-1's parkables, carried
forward and expanded exactly where the report says.

---

## Summary

NEW-1 through NEW-4 and NEW-6 are closed and independently reproduced,
including the sharpest claim in the round (the ordering mutation, R2-M2:
stdout and rc agree with the oracle, only stderr diverges) and both "turned
into assertions" rows (R2-M3, R2-M4, both go red with the exact run counts
the report claims). NEW-5 is partially closed: the substantive correction
(bind_control's pre-gate is the file's only one of its kind) is right, but
the comment's own named verification command (`grep -c
'tracing_intermediates()' ...`) no longer returns the number the comment
states, because the comment quotes its own search string. A second,
independent new false claim was found in the same commit's
`phase-4-exclusions.txt` addition: "every other read site... already pairs
with [`novalue_check`]" is false for two of the crate's three other
`Interp::read` call sites.
