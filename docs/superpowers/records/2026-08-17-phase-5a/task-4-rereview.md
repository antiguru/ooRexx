# Task 4, fix round 1 -- re-review

Scope: `45a10c7fe..0d4a63feb`, one commit (`0d4a63feb`, "Give an unresolvable probe a channel, and
the verdict a typed input"). Reviewed at `0d4a63feb`, repository clean before and after
(`git status --short` empty, HEAD `0d4a63febe5089115602521250c7cdd5a72cd23e`). Scoped re-review: the
spec-compliance half was APPROVE in round 1 and is not reopened.

**Verdict: APPROVE.** Nine of nine findings closed, every control fires, all five gate commands exit
zero with the corpus at 106 of 106. One new Low and three nits, all recorded below; none of them
weakens a fix this round made.

---

## One line per finding

| finding | status | what I ran to say so |
|---|---|---|
| **M1** -- an unresolvable probe leaves the table with no channel | **CLOSED** | dangling symlink now gives exit **101** and a structural naming the row, the path and the io error; with the fix reverted in a scratch tree the same symlink gives 78 rows, no structural, exit **0** |
| **M2** -- the verdict recovers its channels from display strings | **CLOSED** | the combined mutation now reports `diverge-status: 31` and `5a: 36 rows, 36 not yet agree`, where before the fix it was byte-identical to unmutated; the rename alone moves no verdict |
| **L1** -- `chunks_refused == 0` missing from the shared two-engine run | **CLOSED** | passes on all 79 probes; fires with the quoted message when the ir arm's count is raised at the crate's `Outcome` site; the reasoning checks out against `run.rs`, `plan.rs` and `invocation.rs` |
| **L2** -- two `FORM` probes read back the default | **CLOSED** | measured at the oracle: `form()` answers `SCIENTIFIC` bare, `ENGINEERING` under the new directive; the `SCIENTIFIC value-of(FORM)` probe drops its readback and says why |
| **L3** -- the stale `forbid` sentence | **CLOSED** | `/bin/grep -ra 'unsafe_code = "forbid"' crates/` matches nothing; all three copies say `deny`, and each of the four claims they now make is true |
| **L4** -- a mutable set's size in a comment | **CLOSED** | "every copy of" in `gate_table_d.rs`'s module doc and in `corpus/gate-tables/README.md`; no other cardinality left in the changed files |
| **L5** -- the identical-probe account was short | **CLOSED** | re-derived by `md5sum` over the 79 probes: the groups the report names are exactly the groups that exist |
| **L6** -- "the live instances" understates the acceptance-only property | **CLOSED**, with a new gap of its own (N2 below) | the report now states the property for every `agree` row |
| **L7** -- the `::ROUTINE EXTERNAL` boundary | **CLOSED** | recorded, not decided: the row is still filed under `7`, and the consequence is written in `owning_phase`'s doc beside the arm and in the probe's own first comment |

**Every existing caller of the changed shared file still means what it meant.** `corpus.rs:393`,
`state_builtin_oracle.rs:500`, `builtin_status.rs:242` and `parse_version_oracle.rs:145` all take the
`Vec<&'static str>` and either test it for emptiness or `join(", ")` it -- no consumer of label
*content* is left in the workspace (`/bin/grep -ra 'contains(&"' crates/` finds only
`trace_oracle.rs`'s unrelated prefix filters and the new test's own assertions). `ir_dual_oracle.rs`
does not use this function at all (`use support::oracle::{Oracle, did_not_finish, locate}`), and
`input_oracle.rs` carries a documented local copy of the comparison that this change does not touch.
`labels()` pushes in the same order the old function did, so no report's text moves. All four
harnesses ran and passed under both corpus gate commands.

---

## New defects

### N1 (low) -- a genuinely missing probe now gets a second structural failure that says something false

The `Err` arm's message is *"the probe is listed in the directory but cannot be resolved"*. The
missing-probe check above it pushes a `Structural` but does not remove the row from `rows`, so the
loop still reaches `fs::canonicalize` for that row, which fails with the same `os error 2` a dangling
symlink gives. **Measured** (scratch tree, `class__public__subkeyword.rex` deleted outright):

```
structural failures, which are red in every mode and are not verdicts `REXX_CORPUS_GATE` could relax:
  gate-tables/directives/class__public__subkeyword.rex: a row has no probe program. ...
  gate-tables/directives/class__public__subkeyword.rex: the probe is listed in the directory but
  cannot be resolved: No such file or directory (os error 2). ...
```

**Concrete consequence.** Deleting or renaming a probe is the commonest structural failure this
table has, and is the one the brief's own mutation 3 exercises. From now on it reports as two
failures, the second of which asserts the file is listed in the directory when it is not -- sending
the reader after a symlink or a permission problem that does not exist. The one fact that
distinguishes the new arm's case from the old check's is the one fact the message states without
checking. The pattern is also what Task 5's table C will copy.

**Fix, two lines:** `on_disk` is still in scope at that point -- either guard the push with
`on_disk.contains(&probe)`, or drop the clause and say "the probe path cannot be resolved: {error}".

### N2 (low, report-only) -- L6's replacement sentence is short by one, in the same shape L5 was

The report's L6 section says *"`::OPTIONS DIGITS`, `::OPTIONS FUZZ` and, since L2, `::OPTIONS FORM`
are the exceptions in principle -- their readback does discriminate"*. **Measured**: the probes that
carry a readback rather than `say 'main'` are `options__digits__subkeyword.rex`,
`options__fuzz__subkeyword.rex`, `options__form__subkeyword.rex` **and
`options__engineering__value_of_form.rex`** -- the last is the row `::OPTIONS ENGINEERING
value-of(FORM)`, whose probe is byte-identical to the `FORM` one after L2's change and which the
round-1 review named as discriminating in its own L2 paragraph.

The sentence reads as an exhaustive list of exceptions, and it contradicts the L5 paragraph two
sections above it, which correctly says a `<keyword> subkeyword` row and its `value-of(<keyword>)`
row are exercised by the same clause. **Consequence:** the handover's reader counts the rows whose
probe would notice an accept-and-ignore implementation and gets one fewer than there are, and
concludes that `value-of` rows never discriminate. `all three are 5c rows that do not agree today`
is true of the fourth as well, so only the enumeration is wrong.

### Nits

* **`gate_table_d.rs`'s new comment mis-describes one of its own examples.** It says a probe *"whose
  bytes cannot be reached"* is present in the listing, passes the set comparison and "would leave the
  table here". **Measured**: `chmod 000` on a probe canonicalises fine and panics later at
  `gate_tables/mod.rs:192` with *"cannot read ...: Permission denied (os error 13)"* -- it never
  reaches the `Err` arm. Still red, so nothing is at risk; the example is simply not this arm's.
* **`run_on_both_engines`'s doc still says "The assertion is unconditional and names the program"**
  (`gate_tables/mod.rs:163-166`) with two assertions now under it. Not false, but the function's
  contract as stated at the top no longer mentions the `chunks_refused` half, which is the half a
  reader of Task 5's table C would need.
* **`corpus.rs:509` is 93 characters** in a doc block otherwise wrapped at 71-78; the new text was
  joined to the old tail without rewrapping. `cargo fmt` does not rewrap doc comments, so the gate
  cannot see it.

### Outside Task 4's scope, but worth one line

`.superpowers/sdd/2026-08-17-phase-5a/global-constraints.md` still reads *"The workspace sets
`unsafe_code = "forbid"`"*. The plan itself was corrected at `45a10c7fe`
(`docs/superpowers/plans/2026-08-17-phase-5a.md:367`) and all three in-tree copies are fixed, but the
attention lens each dispatch hands its agent is a separate copy and still carries the sentence the
round was told to stop propagating.

---

## Every control and mutation I ran, and what it returned

All mutation work was done in a copy of the tree under the session scratchpad, laid out as a fake
repository root with the read-only siblings symlinked (`interpreter/` and the rest) so the build
scripts resolve; the `.git` link was removed so no git command could reach the real repository. The
repository itself was never edited: `git status --short` is empty and HEAD is still `0d4a63feb`. The
scratch copy was verified byte-identical to the repository (`diff -rq` over
`crates/rexx-exec/tests`, `corpus/gate-tables` and `crates/rexx-exec/src/lib.rs`) after every
mutation was reverted, and again at the end.

**Baselines**

1. `cargo test --release --test gate_table_d` in the scratch copy -- exit **0**, `15 passed`,
   `79 rows`, `agree: 31`, `diverge-both: 48`, `loud: 48`, `gated by this run: 0`. (Round 1's
   baseline was `14 passed`; the extra one is `labels_name_exactly_the_channels_that_differ`.)

**M1**

2. `class__public__subkeyword.rex` replaced by a symlink to `/nonexistent/gone.rex`, no environment
   variables -- exit **101**, `14 passed; 1 failed`, and the structural failure
   *"gate-tables/directives/class__public__subkeyword.rex: the probe is listed in the directory but
   cannot be resolved: No such file or directory (os error 2). The row for ::CLASS PUBLIC subkeyword
   therefore has no program to run, which is structural -- it is never a row the table drops"*. Row,
   path and io error all named, as ruled.
3. **The negative control for it**: the `Err` arm replaced by the old `else { continue; }`, same
   symlink -- `78 rows`, `agree: 30`, `5a: 35 rows, 10 not yet agree`, **no structural failure**,
   `15 passed`, exit **0**. This reproduces round 1's measurement exactly and is what shows the
   redness in control 2 comes from the new arm rather than from anything else in the round.
4. The same probe **deleted** rather than symlinked -- exit **101** with **two** structural failures,
   the second of which is false. This is N1.
5. The same probe at `chmod 000` -- exit **101**, but through a panic at `gate_tables/mod.rs:192`
   (*"cannot read ...: Permission denied (os error 13)"*), not through the `Err` arm. This is the
   first nit.

**M2**

6. The review's combined mutation: crate exit status `+3` at `rexx-exec/src/lib.rs`'s `Outcome`
   construction **and** `labels.push("exit code")` renamed to `"exit status"` -- `diverge-status: 31`,
   `diverge-both: 48`, `5a: 36 rows, 36 not yet agree`, exit **101**. Before the fix this was
   byte-identical to the unmutated tree. The mutation's exit-status half is now seen through the
   rename, which is the whole of M2.
7. The rename **alone** -- `agree: 31`, `diverge-both: 48`, `5a: 36 rows, 10 not yet agree`:
   **the verdict does not move**, which is what a display-label rename must not do. The run exits
   **101** all the same, because `labels_name_exactly_the_channels_that_differ` catches the rename;
   the report's "inert" is about the counts and is true of them. Worth stating plainly, since M2 was
   about a rename being invisible: it is no longer invisible.
8. **Make the new test fail**: the `exit code` label dropped from `labels()` altogether --
   `cargo test --release --test gate_table_d` exit **101** and `cargo test --release --test corpus`
   exit **101**, both on
   `support::oracle::tests::labels_name_exactly_the_channels_that_differ`, panicking at
   `support/oracle.rs:710` with *"DescriptorDiff { stdout: false, stderr: false, exit_code: true }
   left: false right: true"*. `directive_option_gate_table` itself still passes under that mutation,
   which is the point of the typed producer: the verdict no longer depends on the rendering, and the
   rendering has its own catcher for the emptiness test `corpus.rs` does.

**L1**

9. The assertion passes on all 79 probes at the unmutated commit (control 1).
10. Made to fire by adding one to the ir arm's `chunks_refused` at the crate's `Outcome` site --
    exit **101** with *"a body of .../annotate__attribute__subkeyword.rex did not run on the engine
    it was attributed to: the ir arm refused 1 bodies to the tree-walker and the tree-walker arm
    counted 0, where it compiles nothing to refuse"*, which is the report's quoted message.
11. **The reasoning, checked against the code rather than accepted.** `run.rs:1364` guards the
    `chunk_for` call with `matches!(self.engine, Engine::Ir)`, so the tree-walker arm never compiles
    and its counter cannot move -- the comment's last sentence is exactly right. `plan.rs:1037-1059`:
    `chunk_for` returns `None` and increments `chunks_refused` when `ir::compile` errors, and
    `run.rs`'s `if` then falls through to the tree-walking loop. So a refused body does make the
    engines-agree assertion above it a comparison of two tree-walker runs.
    `invocation.rs:143-147` documents the same. `ir_dual.rs:1434-1445` checks both arms' counts, so
    *"`ir_dual.rs` checks the same count for the same reason"* holds.

**L2**

12. At the oracle, standard wrapper, fresh empty directory, three descriptors never merged:
    `say form()` alone -- rc 0, stdout `SCIENTIFIC`; with `::options form engineering` -- rc 0,
    stdout `ENGINEERING`; with `::options form scientific` -- rc 0, stdout `SCIENTIFIC`. The
    `FORM subkeyword` readback now discriminates and the `SCIENTIFIC value-of(FORM)` one provably
    cannot, which is what its new comment says.
13. The three probes this round changed, run on both interpreters from a fresh empty directory:
    `options__form__subkeyword` (crate rc 120 refusal / oracle rc 0 `ENGINEERING`),
    `options__scientific__value_of_form` (crate rc 120 / oracle rc 0 `main`),
    `routine__external__subkeyword` (crate rc 120 / oracle rc 158, `Error 98`). None killed, none
    hung under `timeout -s KILL 10`; each still exercises its row's keyword at its row's position,
    and each row's verdict is unchanged.

**L3, L4, L5**

14. `/bin/grep -ra 'unsafe_code = "forbid"' crates/` -- no match. The three copies at
    `corpus.rs:506`, `gate_tables/mod.rs:451` and `support/oracle.rs:46` all say `deny`. Each claim
    they make checks out: `rust/Cargo.toml:31` is `deny`; `rust/CLAUDE.md:74-80` is the per-site bar;
    `crates/rexx-core/tests/unsafe_sites.rs` exists and asserts both the opt-in set and the
    `unsafe`-block set; and `rexx-core/src/lib.rs:23` carries the one granted `#[allow(unsafe_code)]`,
    which is what makes `deny` load-bearing.
15. `/bin/grep -ai 'four' ` over the changed files -- only `gate_table_d.rs:467`, a **report string**
    over the shape enumeration, which round 1 considered and placed within its fixed-arity precedent.
    Not reopened.
16. `md5sum` over all 79 probes, grouped: `ALL`/`SYNTAX value-of(ALL)`; six
    `<condition>`/`SYNTAX value-of(<condition>)` pairs (error, failure, lostdigits, nostring,
    notready, novalue); `NUMERIC`/`INHERIT value-of(NUMERIC)`; `FORM`/`ENGINEERING value-of(FORM)`.
    Exactly the groups the report's L5 names, including the one L2 created.

**L6, L7**

17. Probes whose first executable line is not `say 'main'`: `options__digits__subkeyword.rex`,
    `options__engineering__value_of_form.rex`, `options__form__subkeyword.rex`,
    `options__fuzz__subkeyword.rex`. The report names three of the four. This is N2.
18. `owning_phase` still files `("::ROUTINE", "EXTERNAL")` under `7` and the table still reports it
    as `7`, so the boundary was recorded and not decided. Its new paragraph's claims check out: row
    identity is `(directive, keyword, position)` (`gate_table_d.rs:90-121`), the probe carries
    `'LIBRARY zzznolib zzzr'` rather than `LIBRARY REXX`, `::METHOD EXTERNAL` is indeed filed `5a`,
    and the row file is `corpus/docs/directive-options.txt`.

**The headline count, re-derived rather than read off the report**

19. **10 of the 36 rows owned by 5a do not read `agree`, out of 79 rows in the table.**
    **Predicate:** `owning_phase(row) == Some("5a")` and
    `verdict(compare_raw(crate_side, oracle)) != Verdict::Agree` -- the crate and the oracle differ
    on at least one of exit status, `stdout` or `stderr`, with `stderr` compared byte for byte and
    with the crate side being the outcome both engines produced. Derived by parsing the report's own
    row lines and grouping them independently of its summary block: 79 row lines, `36 5a / 2 5b /
    38 5c / 1 7 / 2 deferred-parse-error-rendering`, and within 5a `26 agree / 10 diverge-both`. The
    ten are the five `::ANNOTATE` rows, `::ATTRIBUTE EXTERNAL`, `::CLASS INHERIT`,
    `::CLASS METACLASS`, `::CLASS MIXINCLASS` and `::METHOD EXTERNAL`. Unchanged from round 1, and
    **M1's fix does not move the row population**: the row set comes from
    `corpus/docs/directive-options.txt`, which this commit does not touch, and no committed probe
    fails to canonicalise.

**The gate commands, in the repository, unmutated, each with its own exit status**

20. From `rust/` at `0d4a63feb`: `cargo fmt --all --check` -- **0**;
    `cargo clippy --workspace --all-targets -- -D warnings` -- **0**;
    `cargo test --release --workspace` -- **0**;
    `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- **0**, corpus **106 of 106**;
    `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` -- **0**, corpus
    **106 of 106**.
21. `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` -- **101**, corpus
    **106 of 106**, *"gated by this run: 10 row(s) whose owning phase is closing or closed and whose
    verdict is not `agree`"*. Expected red: `CLOSED_PHASES` is empty until Task 24, so this command
    is a counter and not a gate. Not reported as a defect.
22. The oracle-invoking harnesses that read the changed function all ran under the corpus gate
    commands and passed -- `state_builtin_oracle.rs`, `builtin_status.rs`, `parse_version_oracle.rs`,
    `input_oracle.rs`, `ir_dual_oracle.rs` and `corpus.rs` each appear in the runner's output with no
    failure.
23. `git diff --stat 45a10c7fe 0d4a63feb -- 'rust/crates/*/src/'` is empty, so no performance sitting
    is owed, as the report says. Every mutation that touched `src/lib.rs` was reverted and verified
    byte-identical against the repository afterwards.

---

## What I could not check

* **Whether `owning_phase` files each row under the right phase.** Nothing in the tree checks it, by
  design; L7 narrows the one row where the boundary runs inside a row, and the other seventy-eight
  rest on a human reading a diff. Unchanged from round 1.
* **The oracle-did-not-finish channel**, still unfired: making it fire needs a probe that hangs or
  crashes the oracle, which this project forbids committing, and I did not build one outside the
  repository either.
* **`StderrComparison::Raw` still has no runtime witness**, which the report says itself. I did not
  re-run round 1's `Normalized` flip, having no reason to doubt a measurement nothing in this round
  touches.
* **Whether `compare_raw`'s three-field copy maps the right field to the right name.** M2's ruling is
  met -- the values are typed and no text is parsed -- but the adapter from `DescriptorDiff` to
  `Descriptors` is still a hand-written field-by-field copy, and a swap in it (`status: diff.stdout`)
  would survive every test here. The residual risk is much smaller than what M2 closed: a swap
  permutes the labels a divergent row is reported under and cannot turn a divergence into `agree`,
  because `verdict` returns `Agree` only when all three are false. Worth knowing when Task 5 adds a
  second caller.
* **`DescriptorDiff::any()` and its `Default` derive have no caller outside the new test.** Neither is
  wrong; both are surface the round added and nothing in the crate reads.
* **Whether the pinned benchmark binary still reproduces its sha256.** Out of scope -- this round
  lands nothing in `src/` -- and I did not rebuild at `15a1ffa98`.
