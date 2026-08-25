# Task 4 review: gate table D, and the shared gate-table harness

Scope: `094fbbdd8..db312da3e`, two commits. Reviewed at `db312da3e`, working tree clean before and
after (`git status --short` empty, HEAD `db312da3e9715597cdf095294459db1f701d246e`).

**Verdicts.**

* **Spec compliance: APPROVE.** Every clause of the brief is met, including the three mutations
  disposed of as the brief specifies: 2 and 3 are run and recorded, 1 is named against Task 7 and
  not faked. The exclusivity/exhaustiveness requirement holds, `loud` is a column and never a
  verdict, `StderrComparison::Raw` reaches every row through the only comparison the harness has,
  no oracle bytes are typed into the table, and the four `.orx` shapes are derived by a scan that
  reddens structurally if any of them reaches nothing.
* **Task quality: REWORK.** Two Medium findings, both in the *shared* harness that Task 5 inherits,
  and both of the shape this plan exists to sweep for: a structural failure that is silently
  absorbed, and a verdict input that can be switched off with nothing to notice. Everything else is
  Low.

Findings: two Medium, seven Low.

---

## M1 (medium) — a probe present only as a name is silently dropped, and the table shrinks instead of reddening

`rust/crates/rexx-exec/tests/gate_table_d.rs:338-341`:

```rust
let Ok(abs) = fs::canonicalize(corpus.join(&probe)) else {
    // Already reported above as a missing probe; nothing to run.
    continue;
};
```

The comment is true only when the probe's *name* is absent from the directory listing. The
both-directions check above it (`:298-321`) compares derived paths against `fs::read_dir` entries,
and `read_dir` lists a symlink whatever its target is. So a probe whose name is present but which
cannot be canonicalised passes the set check, fails here, and the row leaves the table with **no
`Structural` pushed at all** — the one path in this file where a row disappears without a channel
that is red in every mode.

**Measured**, in a scratch copy of the tree (never in the repo):

| what I did | report mode, no env vars | `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` |
|---|---|---|
| baseline | `79 rows`, `agree: 31`, `5a: 36 rows, 10 not yet agree`, exit **0** | `gated by this run: 10`, exit **101** |
| `class__public__subkeyword.rex` replaced by a symlink to `/nonexistent/gone.rex` | `78 rows`, `agree: 30`, `5a: 35 rows, 10 not yet agree`, **no structural failure**, exit **0** | — |
| `class__mixinclass__subkeyword.rex` (a 5a row that does not `agree`) replaced the same way | — | `5a: 35 rows, 9 not yet agree`, `gated by this run: 9`, exit **101** |

The second row of that table is precisely the outcome the missing-probe message promises cannot
happen: *"the table reddens on the missing program rather than shrinking to the rows that still have
one"*. The third is worse — the gated count fell by one with nothing red, which is the shape of a
silent improvement.

**Concrete consequence.** Task 24 closes Phase 5a when `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1`
reports zero gated rows. A row whose probe is a dangling symlink contributes zero, so the close
criterion can be satisfied by removing evidence rather than by making it agree — and nothing in
either mode says so. The same `continue` is in the shared path Task 5's table C runs through, so the
hole is inherited rather than local. It is also reachable without malice: a `read_dir`/`canonicalize`
race, or a probe committed as a symlink.

**Fix.** Push a `Structural` in the `Err` arm naming the row, the path and the io error, instead of
relying on the earlier check to have caught it. The earlier check answers a different question.

---

## M2 (medium) — the three descriptor channels are recovered by matching the producer's display strings, and nothing pins them

`rust/crates/rexx-exec/tests/gate_tables/mod.rs:233-240`:

```rust
let diffs = descriptor_diffs_with(crate_side, oracle, StderrComparison::Raw);
Descriptors {
    status: diffs.contains(&"exit code"),
    stdout: diffs.contains(&"stdout"),
    stderr: diffs.contains(&"stderr"),
}
```

The producer is `tests/support/oracle.rs:608-628`, which pushes `"stdout"`, `"stderr"` and
`"exit code"` into a `Vec<&'static str>` whose documented purpose is *"which of the three observable
channels disagree"* — a display list. `gate_tables/mod.rs` is the **only** consumer in the workspace
that reads the content of those labels; `corpus.rs`, `state_builtin_oracle.rs`, `builtin_status.rs`
and `parse_version_oracle.rs` all check emptiness or print the vector. A label rename in the producer
therefore turns one boolean of the verdict function's input permanently false, and every reader that
would have noticed is looking at something else.

**Measured**, in the scratch copy:

| tree | table D |
|---|---|
| unmutated | `agree: 31`, `diverge-both: 48`, `5a: 36 rows, 10 not yet agree`, gated **10** |
| crate exit status `+3` at `rexx-exec/src/lib.rs`'s single `Outcome` construction | `diverge-status: 31`, `diverge-both: 48`, `5a: 36 rows, 36 not yet agree`, gated **36** |
| the same `+3`, **plus** `diffs.push("exit code")` renamed to `"exit status"` in `oracle.rs` | `agree: 31`, `diverge-both: 41`, `diverge-stderr: 7`, `5a: 36 rows, 10 not yet agree`, gated **10** |

The third line is byte-identical in every reported count to the unmutated tree, on a crate whose exit
status is wrong on every row. And the rename alone is invisible to the gate:
`REXX_CORPUS_GATE=1 cargo test --release --workspace` with only the rename applied **exits 0**, with
the corpus at **106 of 106**.

**Concrete consequence.** `oracle.rs` is shared and later tasks touch it. One rename there — or one
added label, since `contains` is a positive test and an unrecognised label is simply not seen — makes
a whole channel of table D and table C read "agrees" forever, and the five gate commands stay green.
A row whose only divergence is exit status then reads `agree`, and Task 24 closes the phase on it.

**Fix.** Ask the producer for a typed answer (return the `Descriptors` triple, or an enum set) rather
than re-parsing its prose; failing that, assert inside `compare_raw` that every label the producer
returned is one of the three this function knows, so a fourth or renamed one panics instead of
vanishing.

---

## L1 (low) — the shared two-engine run omits the `chunks_refused == 0` check that `ir_dual.rs` carries

The brief says both crate runs go through `Invocation::with_engine` *"the way `ir_dual.rs` does it"*.
`run_on_both_engines` (`mod.rs:191-219`) does the engine selection that way but not the check that
makes it mean something: `Engine::Ir` falls back to the tree-walker for a body that does not fit the
instruction stream's index widths, counting it in `Outcome::chunks_refused`
(`invocation.rs:140-147`, `plan.rs:1037-1059`), and `ir_dual.rs:959`, `:995`, `:1434-1445` assert
that count is zero for exactly this reason. Without it, a refused body makes the "the two engines
agree" assertion a comparison of two tree-walker runs — tautologically true, with the row's verdict
still labelled a two-engine measurement.

**Measured, and latent rather than actual today:** adding
`assert_eq!((tree_walker.chunks_refused, ir.chunks_refused), (0, 0), ...)` to `run_on_both_engines`
in the scratch copy passes on all 79 probes (`14 passed`, exit 0). So no current row is affected.
Table C's probes are Task 5's and are not yet written, and the harness is the place the check belongs.

---

## L2 (low) — two of the three `FORM` probes read back a value that is already the default, so the readback cannot fail

`options__form__subkeyword.rex` and `options__scientific__value_of_form.rex` are both, byte for byte:

```
say form()
::options form scientific
```

**Measured:** `say form()` in a program with no directive at all prints `SCIENTIFIC`; with
`::options form scientific` it prints `SCIENTIFIC`. The builtin call therefore distinguishes nothing
— the row's force is acceptance of the keyword, exactly like every non-readback row in the table.
The other readbacks do discriminate: `::OPTIONS DIGITS` uses 12 against a default of 9,
`::OPTIONS FUZZ` uses 2 against a default of 0, and `options__engineering__value_of_form.rex` uses
`ENGINEERING` against a default of `SCIENTIFIC`.

The report states *"the three `FORM` rows say `form()` -- the only options in this table whose effect
a Phase-5-reachable builtin can observe"*, which is true of one of the three.

**Concrete consequence.** Both rows are 5c's and are non-`agree` today because the crate refuses
`::OPTIONS` outright, so nothing is wrong now. At 5c they will read `agree` against an implementation
that parses `FORM SCIENTIFIC` and drops it, and the probe's shape invites the reader to believe
otherwise. `::OPTIONS FORM subkeyword` is fixable — set `ENGINEERING` and the readback discriminates.
`SCIENTIFIC value-of(FORM)` is not fixable while `SCIENTIFIC` is the default, and should say so
rather than carry a `form()` that looks like evidence.

---

## L3 (low) — a new comment states the workspace lint as `forbid`; it is `deny`, and that changes the conclusion

`gate_tables/mod.rs:422`: *"reaching the same effect through `Stdio` alone would need a raw-fd
constructor this workspace's `unsafe_code = "forbid"` rules out"*.

`rust/Cargo.toml:31` sets `unsafe_code = "deny"`, and the comment block above it plus
`rust/CLAUDE.md:74-77` record the change from `forbid`, made by Moritz on 2026-08-20 for the specific
reason that `forbid` *"cannot be overridden by an inner `#[allow]`, which is exactly what an approved
module needs"*. So the sentence names a level the workspace does not set, and its conclusion —
"rules out" — is the one thing `deny` was chosen not to do. The live bar is `CLAUDE.md`'s exception
process, with `crates/rexx-core/tests/unsafe_sites.rs` asserting the set of granted sites.

The wording is copied from `corpus.rs:505` and `support/oracle.rs:46`, which carry the same stale
text, and the plan's own `global-constraints.md` repeats it — so this is inheritance, not invention.
The design decision is right; only the reason given for it is false. Consequence is small and
concrete: the next reader takes the `sh -c 'cat >&2'` child as forced by a lint rather than chosen
over a grantable exception.

---

## L4 (low) — a mutable set's size in a new comment

`gate_table_d.rs:35-36` and `corpus/gate-tables/README.md`: *"the four copies of
`phase_subset_files_on_disk`"*. Four is correct today — `corpus.rs:596`, `coverage.rs:685`,
`ir_dual.rs:1203`, `collect_stress.rs:226`, each a non-recursive `read_dir` filtered to
`starts_with("phase-")` and `ends_with(".txt")`, verified. But it counts files a later task can add
to, and it is the sentence a reader leans on to believe the new subtree adds no obligation.

The precedent in this plan splits the rule the same way: the Task 1 review's L5 declined to charge
fixed arities ("three descriptors", "the two engines"), and the Task 1 re-review's D1 charged a
mutable aggregate ("106 of 106") on the ground that it is *"a count no human will re-verify"*. This
is the second kind. Fix: "every copy of".

Everything else numeric in the three new files is fixed arity ("five cells", "a cube of eight
points", "the two engines") or an explicit measurement, and is within that precedent.

Weaker, same family: `corpus/README.md` and `corpus/gate-tables/README.md` both carry
*"204 programs, 0 divergences, exit 0"*. **I reproduced it** —
`./target/release/rexx-diff --cpp <oracle> --rs <oracle> --corpus corpus` prints `204 programs, 0
divergences` and exits 0 — and it is introduced as a measurement, which the constraint licenses. It
stops describing the tree the moment Task 5 lands table C's probes; the load-bearing half of the
sentence is "0 divergences", not the count.

---

## L5 (low) — the report's account of the identical-probe pairs is short by two, and the two it misses are L2's

The report says *"**Two pairs** of rows have textually identical probes ... The same holds for each of
the six condition options against `SYNTAX value-of(that option)`"*, which accounts for the `ALL` pair
and the six `CONDITION` pairs.

**Measured** by `md5sum` over the 79 committed probes, the pairs sharing content are: `ALL`/`SYNTAX
value-of(ALL)`; the six `CONDITION`/`SYNTAX` pairs; `options__form__subkeyword` /
`options__scientific__value_of_form`; and `options__numeric__subkeyword` /
`options__inherit__value_of_numeric`. The last two are a third kind the sentence does not reach — and
the FORM one is exactly the pair L2 is about, so the gap in the prose is the gap in the analysis.
Report-only; no code changes.

---

## L6 (low) — "the live instances" understates how much of table D is acceptance-only

The report's *"What the table cannot see"* names the accept-and-ignore hazard and then says
*"`::ATTRIBUTE DELEGATE` and `::METHOD DELEGATE` are the live instances"*. The property holds for
**every** `agree` row this table has. None of the 36 5a-owned probes instantiates or invokes
anything; each is `say 'main'` plus directive clauses, so its three descriptors are the same whether
the keyword's effect is implemented or the keyword is parsed and dropped.

**Measured, and the limitation is forced rather than chosen.** I wrote the obvious discriminators —
`.k~new~m` against `::method m abstract`, an outside call against `::method m private`, `o~at = 5`
against `::method at attribute`, `.k~new` against `::class k abstract` — and ran both interpreters on
each. The oracle answers rc 163 (`Error 93`), rc 159 (`Error 97`), rc 0 `5`, and rc 158 respectively;
this crate answers rc 120 `rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)` on
all four. `.k` itself resolves identically on both sides (`The K class`), but `.k~id` is already rc
120. So no stronger probe is writable at this commit, which is the same reason the brief gives for
relocating mutation 1 to Task 7.

**Concrete consequence.** Task 24's close criterion is that every 5a row in both tables reads
`agree`. On this table, 26 of the 36 5a rows already do, and each of those is a parse-acceptance
check. That is fine if and only if table C carries the semantic half and the handover says so in
those terms; a reader of this report would reasonably conclude that only the two `DELEGATE` rows are
in that position.

---

## L7 (low) — the `::ROUTINE EXTERNAL` row pins a phase boundary Task 22 is being asked to draw

`owning_phase` files `("::ROUTINE", "EXTERNAL")` under `7`, justified by the plan and by the probe
choosing `'LIBRARY zzznolib zzzr'`. The plan's Task 22 says both *"`::ROUTINE EXTERNAL` naming a real
shared library stays Phase 7's"* **and**, in the same paragraph, *"say which of the three `EXTERNAL`
forms this task moves and which it does not"*.

A row's identity here is (directive, keyword, position), so the two `::ROUTINE ... EXTERNAL` forms
share one row. If Task 22 concludes that `::ROUTINE r EXTERNAL 'LIBRARY REXX name'` moves into 5a,
that behaviour has no row in table D at all: the row that exists is Phase 7's and is never gated,
and the 5a form is outside the denominator. Not wrong today — the report already flags the mirror-image
conflict on `::METHOD EXTERNAL`, whose crate-side owner string still says `(Phase 7)` while the row is
filed 5a — but the `::ROUTINE` half is unflagged and is the one where the boundary runs *inside* a
row rather than beside it.

---

## Two things I checked and am recording as not defects

* `constant_sending_an_own_private_class_method` (`orx.rs:468`) indexes `line.tokens[1..]`, which
  panics on a `::CONSTANT` clause with no operands. Neither bootstrap file has one, and a panic is
  loud rather than silently green, which is the right side of the line this plan cares about.
* `agree` and `loud` cannot co-fire in practice — it would need the oracle to write a `rexx-exec: `
  line — though the predicate is independent as the brief requires. The independence is real and
  witnessed: mutation C below moves `loud` from 48 rows to 2 while all 48 rows stay non-`agree`.

---

## The two verdicts on the brief's own requirements, stated against what I ran

### Exclusivity and exhaustiveness: the argument holds, and is stronger than the report claims

`verdict` (`mod.rs:163-174`) is a `match` on `(differs.status, differs.stdout, differs.stderr)` with
eight literal arms, no `_`, no guards.

* **Exhaustive over the cube** — a missing point is `E0004` and does not compile. Not merely
  asserted: I enumerated all eight points from outside the function and every one lands in a cell.
* **Mutually exclusive** — the report's reason (a duplicated arm is `unreachable_patterns`, an error
  under `-D warnings`) is true but is not the load-bearing one, and the report says so itself: a
  duplicated arm could not break exclusivity anyway, because `verdict` returns *one* `Verdict`. Two
  cells cannot hold the same point regardless of how the arms are written. That is the argument.
* **No precedence question** — with no wildcard and no guard, no arm's meaning depends on which arms
  precede it, so reordering the eight changes nothing.
* **The check is not decoration.** `the_verdict_function_partitions_the_descriptor_cube` catches the
  one failure a `match` cannot state — a cell with an empty preimage. Ran the falsifying mutation
  (`(false, false, true) -> DivergeBoth`): the test fails naming `diverge-stderr`, exit 101.

**The caveat that matters is M2**, and it is about the other side of the function: the cube is
partitioned correctly, but the map from a real comparison onto the cube goes through
`diffs.contains(&"exit code")`, and that can be switched off with nothing to notice.

**`loud` is never a verdict.** `is_loud` (`mod.rs:249-254`) reads only `Outcome`: this crate's
`NOT_IMPLEMENTED_EXIT` **and** a `rexx-exec: ` line. Nothing about the oracle enters it, `verdict`
never sees it, and it is reported and counted as its own column. Measured co-firing today:
`diverge-both` ∧ `loud` on 48 rows; under mutation C, 46 rows non-`agree` and **not** loud.

### Structural versus verdict: both fire, red in both modes — with M1's exception

Every structural channel was made to fire, all with `REXX_PHASE_GATE` **and** `REXX_CORPUS_GATE`
unset, all exit 101:

| channel | what I did | what it printed |
|---|---|---|
| a row with no probe | moved `class__public__subkeyword.rex` aside | *"a row has no probe program. A row without one is structural and is never skipped: the table reddens on the missing program rather than shrinking..."* |
| a probe no row names | copied one to `class__nosuchkeyword__subkeyword.rex` | *"a probe program no row names, so nothing runs it"* |
| the two engines disagree | appended `!` to `stdout` only when `interp.engine` is `Engine::Ir` | *"the two engines disagree on .../annotate__attribute__subkeyword.rex, which is a structural failure and not a verdict"*, with both engines' three descriptors |
| a row no phase arm covers | deleted `"::ANNOTATE"` from `owning_phase`'s 5a arm | six rows, each *"no arm of `owning_phase` covers this row, so nothing owes it an `agree`"* |
| a shape the scan cannot reach | narrowed shape 1's `INHERIT` tail from `> 1` to `> 2` | *"the scan reached no occurrence of this shape in either bootstrap file"* |
| a verdict cell that cannot fire | mapped `(false, false, true)` to `DivergeBoth` | *"the `diverge-stderr` cell has an empty preimage over the descriptor cube"* |

The structural assertion is called unconditionally at `gate_table_d.rs:543`, before the gated one at
`:546`, and after `emit_uncaptured` — so the report prints and *then* the table reddens, which is what
the delete-a-probe run shows. **M1 is the one row missing from that table**: a probe that cannot be
canonicalised has no channel at all.

### `StderrComparison::Raw` on every row: no row escapes it

`compare_raw` (`mod.rs:233-240`) is the harness's only comparison and passes `Raw` unconditionally;
there is no second call site, no branch and no per-row opt-in, so "every row" is a property of the
shape rather than of a list that could be short. The single `TRACE` row (`::OPTIONS TRACE
subkeyword`, `::options trace i`) is not a corpus program and is in no `phase-*.txt`
(`/bin/grep -c "gate-tables" corpus/phase-*.txt` → 0 in all four), so it owes no entry in
`corpus.rs`'s `RAW_STDERR_COMPARISON`.

**What that check could not see.** Flipping `Raw` to `Normalized` in the scratch copy changes
**nothing** today: `agree: 31`, `diverge-both: 48`, `5a: 36 rows, 10 not yet agree`, gated 10 — the
same numbers as the unmutated tree, because every row whose `stderr` differs also differs on another
channel. So the property has no runtime witness on this commit and rests on review, which is what the
report claims for it and is honest. It will acquire one the first time a row's only divergence is a
trace indent.

### The table types no expected oracle bytes: holds

Every row's oracle column is `oracle.run(&abs)` on the run that reports it (`gate_table_d.rs:344`),
and `expect_exit_code()` is reached only after `did_not_finish` is checked (`:345-356`). My own
sweep of the three new files: `/bin/grep -anE '[0-9]{3,}'` returns only the copyright year and two
document dates in comments; a search for a quoted literal containing `Error`, `rc=<digit>` or
`main\n` returns nothing in any of the three files. The only literals that touch a run's output at
all — `"rexx-exec: "` and `" is not implemented"` — are about **this crate's** own refusal path, and
the exit status they are paired with comes from `rexx_exec::NOT_IMPLEMENTED_EXIT` rather than a typed
120.

### The four `.orx` shapes are derived, not asserted

The scan reached, on my run: `StreamClasses.orx:115`; `StreamClasses.orx:371`;
`StreamClasses.orx:548` and `:549`; `CoreClasses.orx:151` through `:162`. I read each at the file:

* `:115` — `::CLASS 'InputOutputStream' public MIXINCLASS Object INHERIT InputStream OutputStream`
* `:371` — `::CLASS 'StreamSupplier' public subclass 'Supplier'`
* `:546`/`:547` — `::method getSeparator private class external "LIBRARY REXX file_separator"` and
  its path-separator twin; `:548`/`:549` — `::constant separator (.File~getSeparator)` and its twin.
  The scan's hits are the two `::CONSTANT`s, which is right: the shape is a property of a `::CONSTANT`
  clause and the two private class methods are what make them hits. The brief's `:546`-`:549` is the
  whole construct.
* `:151` — `::method string_cls_alnum; use strict arg; return xrange("alnum")`, and eleven more
  through `:162`.

`orx.rs` holds no line number, class name or shape location, and a shape reaching zero is
`Structural` (verified above).

**The usage column's own measured claim, independently reproduced.** `orx.rs`'s module doc says that
with the operand rule off, *exactly one* hit is a keyword's operand rather than a subkeyword. I ran
it: forcing `consumed = 0` in `uses_at_an_option_position` changes exactly one cell of the whole
report — the `::CLASS CLASS` row gains `core=CoreClasses.orx:3974`, which is
`::class "Singleton" mixinclass class public`. Every other row's usage column is byte-identical.

### The headline number, and the predicate I used

**10 of 36 rows owned by `5a` do not read `agree`, out of 79 rows.**

* *not `agree`* — `verdict(compare_raw(crate_side, oracle)) != Verdict::Agree`, i.e. the crate and
  the oracle differ on at least one of exit status, `stdout` or `stderr`, with `stderr` compared byte
  for byte, and with the crate's answer being the one both engines agree on.
* *owned by 5a* — `owning_phase(row)` returns `Some("5a")`.

Re-derived by running, not read off the report:
`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --test gate_table_d` exits **101** with
`gated by this run: 10` and names the ten. The by-phase block reads `5a: 36 rows, 10 not yet agree`;
`5b: 2, 0`; `5c: 38, 35`; `7: 1, 1`; `deferred-parse-error-rendering: 2, 2` — 79 rows, 48 not
`agree`, matching `diverge-both: 48` and `loud: 48`. `loud` and non-`agree` are the same rows:
grouping the report's row lines gives `agree loud=no` on 31 and `diverge-both loud=yes` on 48, with
no other combination.

The row file has 79 non-comment lines, each with five tab fields, and the 79 derived probe paths are
distinct — so no two rows share a probe and the both-directions check pins the count exactly.

---

## Every control and mutation I ran, and what it returned

All mutation work was done in a copy of the tree under the scratchpad, with the repo-root siblings
(`interpreter/`, `ootest/`, `samples/`, …) symlinked so the build scripts and oracle-reading tests
resolve. **The repository was never edited**; `git status --short` is empty at the end and HEAD is
still `db312da3e`. The scratch copy was verified byte-identical to the repo (`diff -rq` over
`crates/rexx-exec/tests`, `corpus/gate-tables` and `crates/rexx-exec/src/lib.rs`) after every
mutation was reverted.

**Baselines**

1. `cargo test --release --test gate_table_d` (repo) — exit **0**, `14 passed`, `79 rows`,
   `agree: 31`, `diverge-both: 48`, `loud: 48`, `gated by this run: 0`.
2. The same in the scratch copy — identical, which is what licenses the mutations below.
3. `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --test gate_table_d` (repo) — exit
   **101**, `gated by this run: 10`, panic at `gate_table_d.rs:546`.

**Mutation 2 (the brief's), re-run three ways**

4. Append `!` to the crate's `stdout` at `rexx-exec/src/lib.rs`'s single `Outcome` construction, both
   engines — all 31 `agree` rows become `diverge-stdout`; `5a: 36 rows, 36 not yet agree`. Under the
   phase gate: `gated by this run: 36`, exit **101**, naming all 36.
5. Crate exit status `+3`, both engines — all 31 become `diverge-status`; `5a: 36 rows, 36 not yet
   agree`; `loud` falls from 48 to 2 while all 48 stay non-`agree` (the two survivors are the
   parse-error rows, whose refusal returns through a site the mutation does not touch).
6. Append `!` to `stdout` **only** when the engine is `Ir` — the engines-disagree structural
   assertion fires first, exit **101**.

Mutations 4 and 5 are what actually witness `diverge-stdout` and `diverge-status` on real rows: the
unmutated table produces only `agree` and `diverge-both`.

**Mutation 3 (the brief's), both directions**

7. `class__public__subkeyword.rex` moved aside, **no env vars set** — exit **101**, structural
   failure naming the missing program; `13 passed; 1 failed`.
8. A probe copied to `class__nosuchkeyword__subkeyword.rex`, no env vars — exit **101**, *"a probe
   program no row names, so nothing runs it"*.

**Mutation 1** — not run, correctly. Its discriminator is `.M~baseClass`; measured, this crate
answers rc 120 on `.k~id`, so `~baseClass` is equally out of reach and the check would do the same
thing whether or not the claim were true. The task named it against Task 7 and did not fake it.

**The structural channels not covered by the brief's three**

9. `owning_phase`'s `::ANNOTATE` arm deleted — six structural failures, exit 101.
10. Shape 1's `INHERIT` tail narrowed from `> 1` to `> 2` — the shape reaches nothing, structural,
    exit 101.
11. `(false, false, true)` remapped to `DivergeBoth` — the cube test fails naming `diverge-stderr`,
    exit 101.

**Controls that found the two Medium findings**

12. `class__public__subkeyword.rex` replaced by a dangling symlink, no env vars — **exit 0**,
    `78 rows`, `agree: 30`, no structural failure. → M1.
13. `class__mixinclass__subkeyword.rex` (5a, non-`agree`) replaced by a dangling symlink, under the
    phase gate — `gated by this run: 9`, exit 101, no structural failure. → M1.
14. `diffs.push("exit code")` renamed to `"exit status"` in `support/oracle.rs`, **with** the `+3`
    exit-status mutation — `agree: 31`, `gated by this run: 10`: identical to the unmutated tree on a
    crate that is wrong on every row. → M2.
15. The rename alone, `REXX_CORPUS_GATE=1 cargo test --release --workspace` — **exit 0**, corpus
    **106 of 106**. → M2: no gate command sees it.

**Controls that returned "no change", which is itself the result**

16. `StderrComparison::Raw` flipped to `Normalized` — `agree: 31`, `diverge-both: 48`, gated 10:
    unchanged. The Raw property has no runtime witness today.
17. `assert_eq!((tw.chunks_refused, ir.chunks_refused), (0, 0))` added to `run_on_both_engines` —
    passes on all 79 probes. L1 is latent, not actual.
18. `consumed = 0` in `uses_at_an_option_position` — exactly one cell of the report changes, the
    `::CLASS CLASS` row at `CoreClasses.orx:3974`. Confirms the module doc's measured claim.

**Verification, in the repository, unmutated**

19. The five gate commands, each with its own exit status, from `rust/` at `db312da3e`:
    `cargo fmt --all --check` → **0**; `cargo clippy --workspace --all-targets -- -D warnings` →
    **0**; `cargo test --release --workspace` → **0**;
    `REXX_CORPUS_GATE=1 cargo test --release --workspace` → **0**, corpus **106 of 106**;
    `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` → **0**, corpus
    **106 of 106**.
20. `rexx-diff --cpp <oracle> --rs <oracle> --corpus corpus` — `204 programs, 0 divergences`, exit 0.
    The README's measurement reproduces.
21. All 79 probes run through the release `rexx-run` under `timeout -s KILL 10` from a fresh empty
    directory: none killed (no rc 124, no rc 137). All 79 through the oracle under the standard
    wrapper (`ulimit -v 1048576`, `LD_LIBRARY_PATH`, `timeout -s KILL 10`, three descriptors never
    merged): all finished normally, seven with a Rexx error status (166, 231, 166, 158, 213, 231,
    158) and none killed.
22. `git diff --stat 094fbbdd8 db312da3e -- 'rust/crates/*/src/'` is empty — no sitting is owed, as
    the report says.
23. The two cross-reference rows re-run against the oracle: both `Error 25`, sub-numbers **25.901**
    (`::CLASS`, `found "CLASS"`) and **25.926** (`::RESOURCE`, `found "LIBRARY"`), rc 231 each, and
    this crate answers the same two sub-numbers at rc 120 on one line. The report's claim that they
    differ only in rendering holds, and `phase-4-exclusions.txt`'s 2026-08-20 note does say in terms
    *"Nobody owns it. It is not Phase 5 work."*
24. The four copies of `phase_subset_files_on_disk` read `corpus/` non-recursively and filter to
    `starts_with("phase-") && ends_with(".txt")` — verified at all four, and no probe path appears in
    any `phase-*.txt`.
25. `Engine` has exactly the two variants the harness runs, and neither new file reads `REXX_ENGINE`
    except in the doc comment explaining why it does not.

---

## Which probes I sampled

**Mechanically, all 79.** A script re-derived each row's probe path by the same rule as
`Row::probe_path`, opened the file, and required (a) the row's directive to appear, (b) the row's
keyword to appear as a bare word, and (c) for a `value-of(X)` row, the keyword to appear immediately
after `X`. **Problems: none.** The 79 rows derive 79 distinct paths, so no probe serves two rows.

**By reading, in full:** `class__inherit__subkeyword.rex`, `class__mixinclass__subkeyword.rex`,
`class__metaclass__subkeyword.rex`, `class__class__subkeyword.rex`, `class__abstract__subkeyword.rex`,
`annotate__class__subkeyword.rex`, `annotate__package__subkeyword.rex`,
`method__external__subkeyword.rex`, `attribute__external__subkeyword.rex`,
`routine__external__subkeyword.rex`, `method__delegate__subkeyword.rex`,
`attribute__get__subkeyword.rex`, `attribute__set__subkeyword.rex`, `method__attribute__subkeyword.rex`,
`options__digits__subkeyword.rex`, `options__fuzz__subkeyword.rex`, `options__form__subkeyword.rex`,
`options__scientific__value_of_form.rex`, `options__engineering__value_of_form.rex`,
`options__trace__subkeyword.rex`, `requires__library__subkeyword.rex`,
`requires__namespace__subkeyword.rex`, `resource__end__subkeyword.rex`,
`resource__library__subkeyword.rex` — and the remaining 55 in a single dump.

**Can any pass for a reason unrelated to its row?** Two answers.

* *Structurally, no.* Each probe's directive clause carries its keyword at its position, and the
  probe path is derived from the row, so a probe cannot be attached to the wrong row.
* *Semantically, most cannot tell acceptance from effect* — which is L2 and L6, and is forced by
  what the crate can do today rather than chosen. The one place a probe's failure is attributed to a
  neighbouring keyword is `class__inherit__subkeyword.rex`, whose refusal message names
  `::CLASS MIXINCLASS`; that coupling is intrinsic, since a class can only `INHERIT` a mixin.

---

## What I could not check

* **Whether `owning_phase` files each row under the right phase.** Nothing in the tree checks it, by
  design, and I read the assignment against the plan's handover section rather than against a
  running thing. The four arms I traced to text — 5c for `::OPTIONS`/`::RESOURCE`/`::REQUIRES`/
  `::ROUTINE`'s option surface, 5b for `DELEGATE`, 7 for `::ROUTINE EXTERNAL` naming a real shared
  library, 5a for the `::ANNOTATE` *installation* with only the `ROUTINE` readback handed to 5c —
  all match. L7 is the one place I think the granularity, not the assignment, is wrong.
* **The oracle-did-not-finish channel.** Making it fire needs a probe that hangs or crashes the
  oracle, which this project's rules forbid committing; I did not construct one outside the repo
  either, because the predicate (`did_not_finish` on `Termination`) is the same one `corpus.rs` and
  Task 1 already exercise.
* **Whether the pinned binary still reproduces its sha256.** Out of scope — this task lands nothing
  in `src/` and owes no sitting — and the report's own section 9 flags that the staleness test's
  second half is already non-empty at the base for reasons that predate Task 4. I did not rebuild at
  `15a1ffa98`.
* **Whether table D's 5a rows will still read `agree` once the crate implements the keywords'
  effects.** No probe here can see that, and no probe here could have been written to see it at this
  commit (L6's measurements). It is table C's half.
* **`rexx-diff`'s cross-implementation invocation over `corpus/`.** I ran the self-test the READMEs
  cite (`--cpp X --rs X`) and reproduced it. I did not run `--cpp <oracle> --rs <rexx-run>`, which
  the README says now reports these probes' divergences; nothing in a test runs it, so it gates
  nothing either way.
* **Concurrency and long-run behaviour of the in-process crate runs.** `run_on_both_engines` is
  unbounded by construction, as its own doc says; the stand-in is the by-hand `timeout -s KILL 10`
  sweep, which I re-ran (control 21) but which is a check on the probes that exist rather than a
  bound on the ones Task 5 will add.
