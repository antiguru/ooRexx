# Task 6 report: the flip

**Status.** Done. `5a` is in `CLOSED_PHASES`, committed at `743dd41096182e16abf45ca10413b9371f8a016b`.
Every 5a row of gate tables C and D now gates under `REXX_CORPUS_GATE=1` alone. The four gate
commands `controller-gates.sh` runs are green at that SHA, each status read from its own `.rc` file.
The negative control was run, watched fail at exit 101 on a named row, and inverted back to exit 0.

**Base.** `71327dc46`. **Commit.** `743dd41096182e16abf45ca10413b9371f8a016b`, one line.

---

## 1. The change

`rust/crates/rexx-exec/tests/gate_tables/mod.rs:344`:

```diff
-pub const CLOSED_PHASES: &[&str] = &[];
+pub const CLOSED_PHASES: &[&str] = &["5a"];
```

Nothing else. `git status --porcelain` after the commit is empty.

**Its blast radius is exactly two test binaries.** `verdict_is_gated` is the only reader of
`CLOSED_PHASES`, and the only callers of `verdict_is_gated` are `gate_table_c.rs:1910` and
`gate_table_d.rs:787` --

```
/bin/grep -rn 'verdict_is_gated' --include=*.rs rust/crates | /bin/grep -v target
```

which returns the definition, the two `use` lines and those two call sites, and nothing else.

The doc comment above the constant already said "A phase is added here in the commit that closes it,
and from then on a regression in one of its rows is red under `CORPUS_GATE_ENV` alone." It stays true
and was not touched.

---

## 2. The flip, measured

From `rust/`, with the flip applied and before committing:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

status written to a `.rc` file and read from it: **0**. `/bin/grep -c '^failures:$'` on its stdout:
**0**. Its two `test result:` lines: `ok. 14 passed; 0 failed` and `ok. 15 passed; 0 failed`.

From that run's stderr, the two tables' own by-phase blocks carry one 5a line each:

```
  5a: 135 rows, 0 not yet `agree`        (gate table C)
  5a: 36 rows, 0 not yet `agree`         (gate table D)
```

and both tables print

```
gated by this run: 0 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
```

Counted off the report's own row lines with

```
/bin/grep -c -E '^  [a-z-]+ +loud=(yes|no) +5a ' <stderr>     -> 171
/bin/grep -c -E '^  agree +loud=(yes|no) +5a ' <stderr>       -> 171
```

The first pattern is deliberately wider than the controller's: `[a-z-]+` matches `unanswered` as
well as `agree` and `diverge-*`, because `unanswered` is not `agree` and *is* gated. The two counts
being equal is what says no 5a row is `unanswered`.

**No sixth row.** That is not read off the row lines alone. The `5a: ... 0 not yet agree` figure and
the `gated` list are built by the *same* closure -- `gate_table_c.rs:1897`'s `record`, whose
comment says "`unanswered` is not `agree`, so it counts as open and, on a gated phase, as gated" --
so a row that produced no verdict at all is counted in both. Both tables report 0, on the run that
exits 0. The five rows Tasks 1 through 5 closed are the last 5a rows there were.

### The `gated by this run` count does not move, and that is correct

The controller's dispatch said "Both tables currently print `gated by this run: 0 row(s)`. After your
flip that number should move." It does not, and the report line says why in its own text: it counts
**rows whose owning phase is closing or closed AND whose verdict is not `agree`** -- gated rows that
*fail*, not gated rows. With every 5a row agreeing, 0 is the only value a green gate can print, and
the number moving would have meant the gate was red.

The previous plan's `task-24-report.md:59`-`:60` is the same instrument read at a time when it did
move: with the flip applied there and four table-C rows plus one table-D row still open, it printed
`4` and `1`. Those five rows are the ones this plan closed.

---

## 3. The negative control

**The recipe** is the one the controller's addendum handed over, which is table C's own `control`
string for `methna`: stop upcasing a method name as it is added, at **both** sites, since either
alone is a no-op.

* `rust/crates/rexx-exec/src/dispatch.rs`, `method_name_pair`:
  `let key = written.to_ascii_uppercase();` -> `let key = written.clone();`
* `rust/crates/rexx-classes/src/method_dict.rs:161`, `MethodDict::replace_method`:
  `let key = name.to_ascii_uppercase();` -> `let key = name.to_string();`

The lookups at `method_dict.rs:215` and `:362` were left upcasing. That is what makes the mutation
mean "a name defined in lower case is not found by the message that names it" rather than "keys are
case-sensitive everywhere".

**Both files were copied to the scratchpad before mutation and restored from those copies**, never
with `git checkout --`.

### It failed, at the verdict assertion, on a row named in advance

Same command as section 2. Status from its `.rc` file: **101**. One failing test,
`concept_and_class_gate_table`; the other binary stayed at `test result: ok. 15 passed; 0 failed`.

```
thread 'concept_and_class_gate_table' panicked at crates/rexx-exec/tests/gate_table_c.rs:1994:5:
1 row(s) of gate table C owned by a closing or closed phase do not `agree` with the oracle; the
first of them are ["gate-tables/concepts/methna.rex"].
```

`gate_table_c.rs:1994` is the **verdict** assertion, not `assert_no_structural_failures`, which is
called before it and is red in every mode. So the exit status came through the channel the flip
controls and not through the structural channel that would have fired with or without it.

`REXX_PHASE_GATE` was unset for this run. `verdict_is_gated` is
`corpus_gate() && (closing_phase().as_deref() == Some(phase) || CLOSED_PHASES.contains(&phase))`,
so with `closing_phase()` at `None` the only route from a red `methna` to a non-zero exit is
`CLOSED_PHASES.contains("5a")` -- the line committed here. I did **not** run the pre-flip arm of
this (mutation applied, `CLOSED_PHASES = &[]`, expecting exit 0), because the controller's dispatch
said explicitly to inherit the previous plan's Task 24 result rather than re-derive that
`CLOSED_PHASES` is the switch. It is a two-minute rebuild if a reviewer wants it run anyway.

### What went red, and what did not

The row, from the report:

```
  diverge-both   loud=no  5a   methna Method Names   depth 1 parent xcremet provide.xml:456 methna.rex
      oracle rc=0    out="id COST\noperator-name The Method class\nadded-lowercase The Method class\nmatched-in-uppercase The"... err=""
      crate  rc=159  out="id COST\noperator-name The Method class\n" err="       *-* Compiled method \"METHOD\" with scope \"Class\".\n     9 *-* say 'added-lowercase' .cost~m"...
```

Two of the oracle's four lines, then a raise at the send that names the lower-case-defined method.
`loud=no`: this is the crate answering wrongly, not declining.

**It is a mechanism revert and not a bootstrap break**, read off the counts rather than argued. The
by-phase blocks under the control, against the green run:

| table | phase | green run | under the control |
|---|---|---|---|
| C | 5a | 135 rows, 0 not yet `agree` | 135 rows, **1** not yet `agree` |
| C | 5b | 6 rows, 6 not yet `agree` | 6 rows, 6 not yet `agree` |
| C | 5c | 1347 rows, 1213 not yet `agree` | 1347 rows, 1213 not yet `agree` |
| D | every phase | unchanged | unchanged |

One row moved in one table. 5b, 5c and the whole of table D are byte-identical.

### The wider set, since a gate command is wider than two binaries

Not required by the plan, and run because "which rows went red" is a question about the gate rather
than about one binary. Under the same mutation:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec -p rexx-classes --no-fail-fast
```

status from its `.rc` file: **101**. Eleven failing tests across six binaries, from `test result:
FAILED` lines showing `6 failed`, `1 failed`, `1 failed`, `1 failed`, `1 failed`, `1 failed`:

```
class_and_object_match_their_recorded_own_instance_method_sets
every_prologue_mutated_class_matches_its_recorded_own_instance_method_set
every_untouched_class_matches_its_recorded_own_instance_method_set
hiding_leaves_a_tombstone_where_removal_leaves_nothing
rexxinfos_own_instance_methods_are_setup_cpps_and_carry_no_id
set_bag_relation_and_supplier_match_their_setup_cpp_derived_own_sets
the_two_library_files_install_and_answer_what_the_oracle_answers
the_l0_subset_passes_again_under_collect_on_every_allocation
run::tests::the_method_source_shapes_this_task_leaves_refuse_loudly
corpus_differential
concept_and_class_gate_table
```

Every one of them is about method-name casing, and the six `rexx-classes` ones share a single cause
that is worth writing down because it is not what the recipe's wording predicts. The built-in
registry passes method names **as written in the C++**, mixed case -- `"Abbrev"`, `"CaselessPos"`,
`"BaseClass"` -- and `replace_method` is what upcased them. With it not upcasing, a class's recorded
key set comes out as `{"Abbrev", "Abs", ...}` where the committed set is `{"ABBREV", "ABS", ...}`,
which `native_classes_wiring.rs:872` and its neighbours compare directly. **Dispatch of built-ins is
unaffected**, because the lookups still upcase, which is exactly why the gate tables' 5b and 5c
counts do not move and only the row that defines a name in lower case and then asks for it reddens.

So: nameable rather than everything, at both scopes -- one gate-table row, eleven named tests in a
two-crate run -- but "exactly one row reddens" is a statement about the gate tables, not about a gate
command. A reviewer re-running this control against the full workspace should expect eleven, not one.

### Inverted

Both files restored by `cp` from the scratchpad copies; `md5sum` equal to the copies on both, and
`git status --porcelain` then showed only the flip.

Same command as section 2: status from its `.rc` file **0**, `/bin/grep -c '^failures:$'` **0**,
`test result: ok. 14 passed; 0 failed` and `ok. 15 passed; 0 failed`, and the row back to

```
  agree          loud=no  5a   methna Method Names   depth 1 parent xcremet provide.xml:456 methna.rex
```

**Task 24's `cp -a` hazard did not apply and was not merely assumed away.** Plain `cp` was used, so
the restored files carry a current mtime and cargo rebuilt. The witness is behavioural rather than a
`Finished` line: had the mutated binary been reused, `methna` would still have read `diverge-both`.

---

## 4. The five gates

```
scratchpad/controller-gates.sh 743dd41096182e16abf45ca10413b9371f8a016b <logdir>
```

Both gate worktrees checked out the committed SHA -- `checked-out-sha` and `checked-out-sha-b` both
read `743dd41096182e16abf45ca10413b9371f8a016b`. `elapsed.seconds` is **3856**.

Each status read from its own `.rc` file, never from a completion notification:

| gate | command | `.rc` | `/bin/grep -c '^failures:$'` on its log |
|---|---|---:|---:|
| G1 | `cargo fmt --all --check` | 0 | 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 0 |
| G4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | 0 | 0 |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 | 0 |

**G3 was not run.** `controller-gates.sh` omits it deliberately: G4 is G3 plus `REXX_CORPUS_GATE=1`,
which only adds checks, a claim Task 5 verified by reading every reader of the variable. Inherited on
the controller's instruction rather than re-derived, and stated here so "five gates green" is not
read as five commands executed.

From both G4's and G5's logs, the same three figures:

```
264 of 264 matching                                    (corpus_differential, which then reports ok)
  5a: 135 rows, 0 not yet `agree`                      (gate table C)
  5a: 36 rows, 0 not yet `agree`                       (gate table D)
gated by this run: 0 row(s) ...                        (twice, one per table)
```

### G2 at a phase boundary

`rust/CLAUDE.md` says a green clippy is only evidence if the linter re-examined the code, that a warm
target directory does not guarantee that, and that the lint should be run from a clean target
directory at every phase boundary. This commit closes a phase, so that was done rather than skipped.

`CARGO_TARGET_DIR` pointed at a directory created immediately beforehand, and
`cargo clippy --workspace --all-targets -- -D warnings` exited **0** there. That it really linted
from cold is read off the filesystem rather than off the `Finished` line, whose 7.14s is fast enough
to be worth distrusting: the directory holds 2030 files, every one of them stamped inside the
seven-second window between the launch and the `.rc` write, and among them
`debug/deps/libgate_table_c-*.rmeta` and `libgate_table_d-*.rmeta` -- the two targets this commit
changes. The machine has 32 cores, no `sccache`, and no `RUSTC_WRAPPER`. The directory was removed
afterwards by explicit path.

`controller-gates.sh`'s own G2 also shows `Checking rexx-exec` in `g2.log`, so even the warm run
re-linted the crate whose test target moved.

---

## 5. Findings

**1. No sixth row.** Both tables report 0 non-`agree` 5a rows on the run that exits 0, counted by the
closure that also counts `unanswered`. Nothing was re-filed to another phase, and nothing needed to
be.

**2. The `gated by this run` count stays 0, and the dispatch's expectation that it would move was a
misreading of the line.** Section 2 has the detail. Worth carrying because the line is the natural
thing a reviewer greps for to see whether the flip did anything, and on a *successful* flip it
cannot show anything. What shows the flip did something is the control's exit 101; what shows the
5a rows are gated is that they are 171 rows in the phase whose name is now in the constant.

**3. The control's set is one row in the gate tables and eleven tests in a two-crate run.** Section 3
has both. The gap is not the mutation misbehaving -- it is that `MethodDict::replace_method` is the
single point where every built-in method name gets upcased, so removing it moves the *recorded key
sets* of the built-in classes without moving any dispatch. Anyone reusing this recipe should expect
that.

**4. The `control` field's "who can first run it" half is now historical for every 5a row, and its
task numbers point at the previous plan's numbering.** `methna`'s reads "-- Task 21"; this plan's
Task 5 is what landed it, and there is no Task 21 here. The same is true of the "-- Task 7", "-- 9",
"-- 12", "-- 13", "-- 14", "-- 16", "-- 18" citations on the other 5a rows. This is systematic and
part of the table's established schema, not a slip in one row, so I did not touch it -- but with 5a
closed, that half of the field now names tasks that have all landed, which is the shape of prose
`rust/CLAUDE.md` warns rots. Retiring or rewording it across the 5a rows is a decision for the
controller, not a side effect of the flip.

**5. I looked for a defect in the plan file and did not find one.** Checked, against what this task
measured: Task 6's `Build` (the one line, correct file -- true), its `Done when` (satisfiable and
satisfied), its `If a sixth row appears` (did not fire), the global constraints' five-gate line
(true; the four-command runner is the controller's tooling decision, recorded above rather than
written into the plan), and Task 5's `Done when` sentence about the one-site no-op (not re-tested
here, since Task 5 owns it). `docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md` is unchanged
by this task. I would rather report an empty result than manufacture a correction.

**6. A method note, since it cost time and could have cost a figure.** A first attempt to wait on a
background run used `read -t 5 -u 3 x 3</dev/null` as the sleep. `read` from `/dev/null` returns at
once, so the loop spun to completion in milliseconds and printed `waited 550s` -- a duration
computed from the iteration count, not measured. No figure in this report comes from it; the
replacement is `timeout N tail -f --pid=<pid> /dev/null`, which blocks on the process itself. It is
the same shape as `cargo fmt --edition`: a command that reads exactly like the one you meant.

---

## 6. What a reviewer should re-run

```
git -C <worktree> show --stat 743dd41096182e16abf45ca10413b9371f8a016b
scratchpad/controller-gates.sh 743dd41096182e16abf45ca10413b9371f8a016b <fresh logdir>
```

and, for the control, the two one-line edits in section 3 followed by

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

expecting exit 101 and `gate-tables/concepts/methna.rex` named in the panic. Restore from a copy,
not from git.
