# Task 9 re-review -- fix round 1 (`94237403`)

Scope: did fix round 1 close the nine findings, and did it introduce anything new.
The original implementation (`233fbd8b`) is not re-litigated.

Every claim below is labelled **[ran]** or **[read]**.

## Method

* Oracle runs used the mandated wrapper, three descriptors captured separately, never
  `2>&1`. All probes ran from `.../scratchpad/t9rr/probes`, a directory I `mkdir`'d for
  this review; no probe was run from the scratchpad root.
* A baseline binary at `233fbd8b` was built with `git archive 233fbd8b rust | tar -x`
  into the scratchpad (plus a symlink for the build script's `../../../interpreter`).
  **No git worktree was created and no git state was touched.**
* Tracked files were mutated 13 times. Before any mutation I copied `run.rs`, `eval.rs`,
  `trace.rs`, `trace_oracle.rs` and `coverage.rs` into the scratchpad, and every restore
  was `cp` **from those copies** -- `git checkout --` was never used. After every
  mutation the md5-of-md5s of the three sources was printed and compared against the
  pre-mutation value `464823e958451aa62d5a781e04639685`; it matched every time, and
  `git status --porcelain` is empty at the end.
* Every test run below was read for its **run count**, not only its exit status.

## Gates, re-run by me [ran]

| gate | result |
|---|---|
| `cargo test --workspace` | **989 passed / 0 failed** (summed over all `test result:` lines; zero `FAILED`) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **41 of 41**, mode STRICT |
| `cargo fmt --all --check` | exit 0, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

The release binary at `rust/target/release/rexx-run` predated the commit and was rebuilt
before any A/B.

---

## Per-finding verdict

### F1 -- **closed** [ran]

The two false comments are gone and replaced by a paragraph that states the limitation
positively. The load-bearing claim is the new unit test, and it holds:

* Both asserted transcripts are the oracle's own bytes. I ran the two programs verbatim
  through the oracle and `cat -A`'d the result: byte-identical to the `concat!` strings,
  and our own binary's stderr is byte-identical to the oracle's for both (`cmp`).
* **R1-M1** (`bind_indent` collapsed to `loop_indent`):
  `cargo test -p rexx-exec --lib task_9s_two_new_indents` -> `FAILED. 0 passed; 1 failed;
  300 filtered out`, while `--test trace_oracle` stayed `ok. 21 passed` and the corpus
  stayed `ok. 1 passed`. That reproduces the finding's own mechanism *and* shows the new
  test catches it.
* **R1-M2** (`>F>` at `indent + 2`): identical picture -- lib test `0 passed; 1 failed`,
  trace_oracle `21 passed`, corpus `1 passed`.

So "it goes red under precisely the two mutations the review used" is true as stated.

### F2 -- **closed**, with one newly-introduced adjacent defect [ran]

I wrote and ran **66 probes** of my own across the loop path, A/B'd against the
`233fbd8b` baseline and the oracle. Result: **~30 DIFF -> MATCH, 0 MATCH -> DIFF.**
Shapes covered: nested loops (inner and outer control variable written), `LEAVE`/`ITERATE`
with labels and with the control-variable name, `ITERATE` two blocks deep inside a
`SELECT`, negative `BY`, zero-trip, `FOR`, `WHILE`, `UNTIL`, `DO OVER`, `DO n`,
`DO FOREVER`, a control variable dropped mid-loop, one assigned from a stem, one assigned
a compound's value, `INTERPRET` writing it, a callee writing it through
`PROCEDURE EXPOSE`, `SIGNAL` out of the loop, `NUMERIC DIGITS 2` truncation, and the
overflow case the review named.

Confirmed specifically:

* `do ii = 1 to 3 ; ii = 10 ; end ; say ii` -> MATCH (oracle 11); base DIFF.
* Non-numeric and `DROP` bodies -> 41.1 on `("abc")`/`("II")`, MATCH; base DIFF.
* Blame attribution: fall-through -> `END`'s line, `ITERATE` -> its own line at the loop
  body's indent, `ITERATE` nested two blocks deep -> still its own line at the body's
  indent, overflow -> line 4 not line 2. All MATCH; all DIFF at base.
* `numeric digits 2` with a body write: base **hangs forever** (the saturating `current`
  never advances past 100), head MATCHes the oracle.

Two residual DIFF-on-both shapes I hit are pre-existing and unrelated: the oracle
under-indents a clause that follows a *completed* inner loop (reproduces with no control
variable write at all, so not F2's), and a missing `*-* Compiled method "+" with scope
"String".` traceback line (reproduces with no loop at all; already an exclusions row).
`TRACE()` is unimplemented (4c) and `::routine` dispatch is 4c.

### F3 -- controller's, out of scope

Noted only: `docs/.../2026-08-03-phase-4b-procedures-and-conditions.md:206` still carries
the sentence "An **internal label** call with `trace l` as its first clause emits nothing,
with or without `PROCEDURE`", with the correction added as a *following* paragraph at
`:210` rather than the sentence being corrected. That is `38b2cb7b`, not this commit.

### F4 -- **closed** [read] [ran]

`trace_oracle.rs:72`'s sentence now reads "the union across every witness must be exactly
the thirteen prefixes claimed"; `CLAIMED_PREFIXES` has exactly 13 entries. But the same
module doc gained two new stale statements -- see NEW-4.

### F5 -- **closed** [ran]

Verified by running the exact program the doc describes: under `trace l` the oracle's
whole stderr is `     6 *-*   sub:` and no value line of any kind; under `trace r` the
`>R>` line appears. Both sides MATCH now. The corrected sentence is true.

### F6 -- **closed** [read]

`eval_argument`'s older sentence now says "an expression" and explicitly defers to the
paragraph twelve lines below, which says which expression. No contradiction remains.

### F7 -- **closed** [read]

The report carries the correction (stderr, not stdout) and `phase-4-exclusions.txt`'s row
uses the hedged "stdout/stderr content difference". A commit message cannot be amended;
this is the available fix.

### F8 -- **closed** [ran]

* 10 `TRACE L` probes of my own -- fallen-through label, `CALL` target, `SIGNAL` target,
  a label inside a callee, `SIGNAL ON NOVALUE`'s handler label, a label never executed,
  `trace value 'labels'`, `trace l` set only inside a callee, `trace l` then `trace off`
  then `trace l`, `INTERPRET` between labels -- **all 10 MATCH**, all 10 DIFF at base.
  The silent half holds: `DO`, `SELECT`, `IF` between labels produce no line.
* `trace ?l` (run with stdin from `/dev/null`; without that the oracle blocks on the
  interactive prompt) differs by **exactly** the two `+++` banner lines the `TRACE ?` row
  already owns. Claim confirmed.
* **"Reduces to `all` in every mode but `L`" is true, not just asserted.** The only
  `TraceMode` values constructible anywhere in the tree are the five associated
  constants (`grep` for `TraceMode {` finds only the five definitions; `#[derive(Default)]`
  is never called). In `OFF`/`ALL`/`RESULTS`/`INTERMEDIATES`, `all || (labels && is_label)`
  is identical to `all`; only `LABELS` differs. The in-crate assertion covers
  `ALL`/`RESULTS`/`INTERMEDIATES`; `OFF` is trivially covered by the same test's
  `C/E/F/N/O` loop.
* R1-M5/M6/M7 all red (below).

Two false comments were left behind by this change -- NEW-2 and NEW-6.

### F9 -- **partially closed** [read] [ran]

The duplicated gate is kept and now documented at the site, which is what the finding
asked for. **The documenting comment is itself false** -- NEW-5.

---

## New findings

### Important

**NEW-1 (load-bearing) [ran]. The F2 re-read bypasses `NOVALUE`, so a `DROP`ped control
variable under `SIGNAL ON NOVALUE` raises a spurious 41.1 and exits 215 where the oracle
traps the condition and exits 0.**

`run.rs:5085` reads the control variable with `self.read_by_name(&name)`. `read_by_name`
(`stem.rs:144`) returns the derived name on a miss and reports **nothing** to the caller;
the crate's own NOVALUE-aware reader is `read` (`lib.rs:1466`), which returns
`(ObjRef, Novalue)` and whose callers run `novalue_check` (`eval.rs:319`, `:353`). The
oracle's `control->evaluate` is a full evaluation and does raise NOVALUE.

Measured, `signal on novalue` + `do ii = 1 to 3 ; drop ii ; end`:

```
oracle:  stdout "handler 4",  rc 0, no >V>/>>> lines on the failing re-test
ours:    stderr "Error 41.1 Nonnumeric value (\"II\")", rc 215, >V>/>>> emitted first
base:    ran three passes, printed "never", rc 0
```

`SIGNAL ON NOVALUE` is otherwise implemented and correct here -- `signal on novalue ;
zz = qq + 1` MATCHes on both base and head -- so this is a real hole at one site, not an
unimplemented feature. Not a MATCH -> DIFF (base diverged too, differently), but the
divergence is newly *shaped* by this round, and it makes the report's and
`phase-4-exclusions.txt`'s claim about this shape an overclaim: "a body that `DROP`s the
control variable reads the derived name and fails 41.1 on `("II")` ... Both match byte for
byte now" is true only when `NOVALUE` is not trapped.

### Minor

**NEW-2 (load-bearing) [read].** `rust/crates/rexx-exec/src/run.rs:5772` still says a
`Trace::Setting` letter with "nothing visible to show in this crate's scope
(`C`/`L`/`E`/`F`/`N`/`O`)" lands on `TraceMode::OFF`. After F8, `L` lands on
`TraceMode::LABELS`. False sentence, in the arm that calls `mode_from_setting`.

**NEW-3 (load-bearing) [ran].** `rust/crates/rexx-exec/tests/trace_oracle.rs:84`-`85`:
collapsing the `bind_indent` split "leaves this file green and the workspace at 986/0".
The workspace is **989** at this commit, and under that exact mutation the workspace is
**not green** -- the lib test the same commit added reports `0 passed; 1 failed`. The very
next paragraph says so ("goes red under exactly the mutation that leaves this file
green"), so the module doc contradicts itself twelve lines apart. The true statement is
"leaves this file green", full stop.

**NEW-4 (load-bearing) [ran].** Same module doc, the prefix table: the row
"`| >>> | every witness below |`" is now false -- `trace_labels.expected` contains zero
`>>>` (`grep -c` = 0), and `WITNESS_PREFIXES` correctly claims only `*-*` for it. In the
same paragraph, "Three witnesses below carry no prefix of their own" is now four:
`trace_labels.rex` is a fourth. Neither is caught by
`every_witness_still_emits_every_prefix_it_is_named_for`, which reads `WITNESS_PREFIXES`
rather than the prose.

**NEW-5 (load-bearing) [ran].** The F9 comment at `run.rs:5259`-`5266` says the pre-gate
is "the same ... shape **every other tracing site in this file uses** (`step`'s own
`Assignment` arm, the `Controlled` arm above)". Neither named site has that shape:
`step`'s `Assignment` arm builds `rendered` and `name` unconditionally (`run.rs:957`,
`:961` -- it needs them for `trace_result` and `slot_of`), and the `Controlled` arm's
`Vec`s are gated on `re_tested`, not on any tracing predicate. `tracing_intermediates()`
occurs exactly **once** in the whole of `run.rs`, at the site the comment is defending.
The finding F9 asked for a note saying why the gate is not a second decision; the note
that landed asserts a pattern the file does not have.

**NEW-6 (load-bearing) [read].** `trace.rs:96`-`109`: `TraceMode::OFF`'s doc still lists
`setTraceLabels` among the calls that collapse into it, and the paragraph added directly
below it says "**`setTraceLabels` no longer collapses into this** and *the sentence above
no longer names it*". The sentence above does still name it. The count was updated
("all five"), the list was not.

### Parkable

* The oracle under-indents any clause following a *completed* inner loop (reproduced with
  no control-variable write; the upstream `traceIndent` shape). Indent-only, pre-existing,
  covered by DEVIATION 0, and unrecorded anywhere I could find. Not this round's.
* `TRACE()` is not implemented (`rexx-exec: routine "TRACE" is not implemented (4c)`), so
  the `TRACE L` gap row's "`TRACE()`'s own reported setting" coupling argument had no
  live reader; that argument is now moot rather than wrong.

---

## Mutation evidence -- does it survive independent re-running?

**Yes. No claimed-red mutation was found green.** [ran]

Green baselines first, each with a non-zero run count:
`task_9s_two_new_indents` 1 passed / 300 filtered; `control_variable_reread` 1 passed;
`trace_labels` 1 passed; `function_call` 1 passed; `controlled_loop` 1 passed;
`call_arguments` 1 passed; `--test coverage` 9 passed; corpus 1 passed / 9 filtered.

### Round 1 -- all seven re-run, all seven red

| # | mutation | result |
|---|---|---|
| R1-M1 | `bind_indent` collapsed to `loop_indent` | lib `0 passed; 1 failed`; trace_oracle 21 passed; corpus passed |
| R1-M2 | `>F>` at `indent + 2` | lib `0 passed; 1 failed`; trace_oracle 21 passed; corpus passed |
| R1-M3 | control variable read from the loop's saved value again | trace_oracle `20 passed; 1 failed`; corpus `0 passed; 1 failed` |
| R1-M4 | `Iterate` blame not recorded | corpus `0 passed; 1 failed`; trace_oracle 21 passed |
| R1-M5 | `L` back to `TraceMode::OFF` | `trace_labels` `0 passed; 1 failed`; `trace::tests` `4 passed; 1 failed` |
| R1-M6 | `tracing_clause` = `mode.all` (L echoes nothing) | `trace_labels` `0 passed; 1 failed` |
| R1-M7 | `tracing_clause` = `mode.all \|\| mode.labels` (L echoes everything) | `trace_labels` `0 passed; 1 failed` |

R1-M6/M7 confirm the witness fails in **both** directions, as claimed.

### Round 0 -- a fresh sample of five (the earlier review re-ran M2, M4b, M9, M13)

| # | mutation | result |
|---|---|---|
| M1 (omitted-arg half) | drop `>A>` for an omitted argument | trace_oracle `20 passed; 1 failed` |
| M1 (supplied-arg half) | drop `>A>` for a supplied argument | trace_oracle `19 passed; 2 failed` |
| M3 | drop one of the controlled loop's `>>>` | trace_oracle `19 passed; 2 failed` |
| M5 | drop the `>F>` arm in `trace_intermediate` | trace_oracle `20 passed; 1 failed` |
| M8 | drop `>=>` from `bind_control` | trace_oracle `19 passed; 2 failed`; lib `0 passed; 1 failed` |
| M10 | remove `>R>` from `PREFIX_COVERAGE` | trace_oracle `20 passed; 1 failed` |

One process note: my first attempt at M1 mis-targeted the `RAISE ... ARRAY` doubled `>A>`
instead of the call site, and that mutation left `--test trace_oracle` at `21 passed`.
It is **not** a hole -- re-running it against the corpus gives `0 passed; 1 failed`, so
that site is pinned by `lang/raise_array_substitution.rex`. I record it because a
harness that had only run `trace_oracle` would have called it green.

So nine of round 0's thirteen have now been re-run by a reviewer (four by the previous
one, five by me) and all nine went red; all seven of round 1's went red. The round-0
evidence survives, and the redone round-1 work is independently confirmed.

---

## Summary

F1, F2, F4, F5, F6, F7 and F8 are closed; F9 is partially closed (the gate is documented,
but the documenting comment is false); F3 was the controller's. The mutation evidence for
both rounds survives independent re-running with no claimed-red mutation found green, and
66 probes across the loop path found **zero** MATCH -> DIFF.

The two things worth acting on are NEW-1 (the re-read bypasses `NOVALUE`, producing a
spurious 41.1 and rc 215 where the oracle exits 0 through the handler -- and making the
"matches byte for byte" claim for the `DROP` shape an overclaim) and the five false
statements NEW-2 through NEW-6, four of which sit inside the comment corrections this
round was written to make and one of which contradicts its own neighbouring paragraph.
