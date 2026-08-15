# Task 3 report: a plain `WHEN`'s condition compiles

One commit. Gates green, no oracle divergence found, four mutations measured
twice each (once mid-task, once at the final state of the commit).

## What changed

* `ir/mod.rs` -- `Op::Condition` gains `keyword: ConditionKeyword`, and the new
  `ConditionKeyword` enum carries `raiser()`, which is the whole of what the
  tag decides. The op's doc no longer says "`IF`".
* `ir/compile.rs` -- the `If` arm passes `ConditionKeyword::If`. The listed
  `WHEN`/`WHEN CASE` arm gains an inner `match`: a `When` whose condition
  `native_shape` accepts emits `push_native` + `Op::Condition`; everything else
  in that arm (a `WhenCase`, a declining `When`) keeps `Op::WhenTest`.
* `ir/drive.rs` -- the `Op::Condition` arm binds `keyword` and passes
  `keyword.raiser()` where it passed `raised_if_not_logical`. One arm for both
  keywords. `raised_if_not_logical` is no longer imported there.
* `ir/golden.rs` -- renders `keyword=IF` / `keyword=WHEN`, through a local
  `render_condition_keyword`. The helper lives here rather than on the enum
  because `golden` is `#[cfg(test)]` and a method on the enum would be dead in
  a production build.
* `run.rs` -- `chunk_node_at` gains `(InstructionKind::When { condition, .. },
  0)`; `raised_when_not_logical` becomes `pub(crate)`; four doc comments
  corrected (below).
* `ir/golden_tests.rs` -- five `Condition` lines gain `keyword=IF`; the two
  plain-`SELECT` streams are rewritten (their `WHEN`s now compile); three new
  golden tests.
* `ir/corpus_shape_tests.rs` -- `check_body` states a plain `WHEN`'s
  expectation, and `Seen::native_conditions` becomes a map keyed by keyword so
  the anti-vacuity guard fires per keyword.
* `tests/ir_dual_cases/when-conditions` -- new, four rows.
* the plan document -- Steps 3 and 4 record what landed.

`const _: () = assert!(size_of::<Op>() == 16)` is unchanged and holds. The
build was read unpiped, whole: `cargo test --workspace --no-run` after touching
`ir/mod.rs` produced 75 lines, every one of them a `Compiling`/`Finished`/
`Executable` line, no `warning:` and no `error` anywhere.

## Test counts

| point | passed | failed | ignored |
|---|---:|---:|---:|
| before (`4fa6c3e4d`, measured, not taken from the brief) | 1468 | 0 | 4 |
| after | 1471 | 0 | 4 |

The three are `a_when_condition_outside_the_native_set_stays_one_when_test`,
`a_call_in_a_whens_condition_is_addressed_at_the_conditions_slot` and
`an_absorbed_when_compiles_to_generic`. The four new case-file rows are stanzas
inside one `datadriven` file and do not move the count.

Gates, from `rust/`, each read unpiped:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, zero lines
  matching `^(warning|error)` in the output
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, 1471/0/4

## The `chunk_node_at` doc, and why it was corrected rather than mirrored

The brief asked for an honest decision. The doc claimed `chunk_node_at`'s slot
arms are a subset of `eval_chunk_expr`'s. Adding a `When` arm to
`eval_chunk_expr` to keep that true would have added an **unreachable** arm:
nothing emits an `Op::EvalExpr` for a `WHEN` slot, because a `WHEN` whose
condition declines keeps `Op::WhenTest` doing the whole job. An arm no stream
can reach is a test that cannot fail wearing a different hat.

The containment was also already the wrong invariant in the *other* direction:
`eval_chunk_expr` has a `SELECT CASE` arm that `chunk_node_at` must never have,
because a `SELECT CASE`'s expression is never offered to `push_native` and
holds no node any op names. So neither set contains the other, and the doc now
states the rule the arms actually follow:

* `chunk_node_at`'s arms are the slots `compile` offers to `push_native`.
* A slot is in both functions exactly when it is offered to `push_native`
  **and** its declining fallback is `Op::EvalExpr`.

with one example named in each direction. The earlier wording enumerated the
four slots in the intersection, which would have rotted the moment Task 4 gives
`RETURN` a slot; that enumeration was removed before the commit.

## Oracle captures

Every capture below was taken from `.../scratchpad/oracle-cwd`, a directory
created empty and verified empty afterwards, with the program named by absolute
path in `.../scratchpad/oracle-t3/`. `ls` on the working directory after the
runs shows nothing but `.` and `..`. The wrapper, per program:

```
( cd $CWD; ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE </dev/null \
  >FILE.out 2>FILE.err ); echo $?
```

stdout, stderr and exit status were kept as three separate files and compared
separately. This crate was run the same way on each engine
(`REXX_ENGINE=tree-walker`, `REXX_ENGINE=ir`) and diffed against the oracle
channel by channel.

**The four case-file programs** (`w1`--`w4`): byte-identical to the oracle on
stdout, stderr and exit status, **on both engines**. The expected blocks in
`tests/ir_dual_cases/when-conditions` were spliced in programmatically from the
captured bytes rather than retyped, with only the program path rewritten to
`ir_dual.rs`'s own `INLINE_PATH`. Trailing blanks on the `*-*` lines are part
of what is committed.

**The two programs cited in `ConditionKeyword::raiser`'s doc**, captured so the
citation is a measurement rather than a recollection:

* `if 'x' then nop` -- rc 222,
  `Error 34.1:  Value of expression following IF keyword must be exactly "0" or "1"; found "x".`
* `select; when 'x' then nop; end` -- rc 222,
  `Error 34.2:  Value of expression following WHEN keyword must be exactly "0" or "1"; found "x".`

**Five extra three-way probes** on shapes no committed row holds, each compared
oracle vs tree-walker vs ir on all three channels. **No divergence on any
channel on any of the five**:

* a `WHEN` condition calling a `PROCEDURE` that prints, inside a counted loop,
  under `trace r` (rc 0)
* 34.2 raised at an elevated indent -- a `SELECT` inside a `DO` inside an `IF`
  branch, under `trace r` (rc 222); this is the probe that says the driver
  reading `current_value_indent` live after the condition's ops still prints
  the `>>>` in the right column
* `SIGNAL ON SYNTAX` over a 34.2 (rc 0, `caught 34`)
* `trace i` over a prefix condition, `when \0 then say 'neg'` (rc 0)
* a `CALL ON USER` handler raised from inside a `WHEN`'s condition and
  delivered at that clause's boundary (rc 0, `handler` / `v` / `after`)

## Mutations

Every run was the whole workspace under `memcap 8G cargo test --workspace
--no-fail-fast`, reading the run counts rather than only the status. Sources
were backed up with `cp`, restored from the copy, and the restore verified with
`sha256sum -c`; `rexx-run` was rebuilt after the final restore before any
further probe.

**Every mutation was run twice: once mid-task, and once again at the final
state of the commit.** The two runs agree exactly, test name for test name.

| id | mutation | file present | reddened |
|---|---|---|---|
| MT1 | a `WHEN`'s `Op::Condition` emitted with `ConditionKeyword::If` | yes | **6**: `both_engines_agree_on_every_case_file`, `corpus_shape_tests::every_corpus_body_...`, and the four golden streams that render a `keyword=WHEN` |
| MT1 | same | held out | 5 -- the same minus the case file |
| MT2 | `chunk_node_at` loses its `When` arm | yes | **4**: `both_engines_agree_on_every_case_file`, `both_engines_agree_across_every_population`, `both_engines_agree_on_every_branch_shape`, `the_exempt_set_matches_the_current_failures` |
| MT2 | same | held out | 3 -- the same minus the case file |
| MT3 | the `When` arm never takes the native path (`if false && native_shape(..)`) | yes | **5**: `corpus_shape_tests` and the four golden streams. **`both_engines_agree_on_every_case_file` stays green**, which is the promotion's contract: the bytes do not move |
| MT4 | a declining `WHEN`/`WhenCase` gets an `Op::Condition` behind its `Op::WhenTest` | yes | **6**: `both_engines_agree_on_every_case_file`, `both_engines_agree_on_every_branch_shape`, `a_select_cases_own_value_outlives_the_registers_its_whens_take`, `a_traced_select_echoes_its_header_and_each_listed_when`, `a_when_condition_outside_the_native_set_stays_one_when_test`, `two_constructs_ending_at_one_instruction_release_to_the_lower_mark` |
| MT4 | same | held out | 5 -- the same minus the case file |

What each failure looked like, for the record:

* MT1, case file: the 34.2 row, stderr `Error 34.2: ... WHEN keyword ...`
  against `Error 34.1: ... IF keyword ...`.
* MT2, case file: the call row, stdout `medium\n` against `""` -- the ir engine
  loses the whole `WHEN` to `Loud::call_op_off_its_node`.
* MT4, case file: the `SELECT CASE` row, a third `>>>` after each value's pair.

**MT1 is the measurement that killed a sentence I had already written.** The
34.2 row's first comment said it was "the only thing in the workspace that
reddens" under MT1. It is not: the golden streams render the tag, so they
redden too, and so does the corpus sweep's new per-keyword guard. The comment
was rewritten to say what is true -- that the row is the only place the wrong
sub-number is seen in a program's own stderr -- before the commit.

**No row of the new file is a unique catch.** That is stated in the file, per
row, in those words. The absorbed-`WHEN` row reddened under **none** of the
four mutations and is labelled a transcript that claims nothing.

**What the extracted-program sweep does and does not see.** MT2 reddens
`both_engines_agree_across_every_population`, so the corpus does hold a call
inside a `WHEN`'s condition. MT1 does **not** redden it, so no corpus program
raises 34.2 from a condition that compiled -- which is why the keyword tag
needed a row of its own.

## The rows the brief asked for that were already in the tree

The brief's Step 4 asks for "a matching and a non-matching `WHEN`". Both
already exist and both stayed green through this task:

* `assignment-and-say`, "a SAY inside a matched WHEN's branch under trace i" --
  `select / when 1 = 1 then say 'w'`, a matching `WHEN` with a native condition
  under `trace i`, with its `>L> >L> >O> >>>` block.
* `trace-settings`, "trace turned on inside a branch, before a select" --
  `when 1 = 0 then say 'a'`, a non-matching one under `trace r`.

Writing them again would have been two more rows measuring what those two
measure. They were not written; this paragraph is the record that the
requirement was met by existing coverage rather than dropped.

## Comments corrected because this task falsified them

Each was false *after* this change, not merely stale in tone:

1. `Op::Condition`'s doc -- said "the `IF` condition value" and named 34.1 as
   the raiser.
2. `Op::WhenTest`'s doc -- said it tests "the listed `WHEN`/`WHEN CASE`"
   without qualification. It now says it is the whole job in one op, which is
   what a `WHEN CASE` needs and what a plain `WHEN` falls back to, and that a
   `WHEN CASE` never takes the promoted route because comparing against a
   `SELECT CASE`'s text is not what `Op::Condition` does.
3. `Interp::scan_when`'s doc -- gained the paragraph `eval_if_condition` has:
   the tree-walker's entry for both, and the compiled stream's only for a
   `WHEN CASE` and a declining `WHEN`.
4. `raised_when_not_logical`'s doc -- its reason why a comma list never reaches
   it cited only `eval_condition`'s `matches!`. There is now a second caller,
   so the sentence names both halves: a list arrives already `checked`, and
   `native_shape` declines `ExprKind::Logical` so no `Op::Condition` carries
   one.
5. `ir/drive/tests.rs`, `the_ir_engine_steps_a_selects_chosen_branch_from_the_chunk`
   -- said a `SELECT` "is resolved through the same `scan_when` ... either
   way". Not true for a `WHEN` whose condition compiled. Corrected to name the
   shared half that survives: both engines reach the same
   `Interp::condition_value`, whether through `scan_when` or through the ops
   `Op::Condition` ends.
6. `corpus_shape_tests.rs`'s value-expression comment -- said "A `SELECT`'s and
   a `WHEN CASE`'s evaluate through `Op::EvalExpr` unconditionally". That was
   **already false** before this task: a `WHEN CASE`'s values are evaluated
   inside `Op::WhenTest`, not `Op::EvalExpr`. It was in my hunk, so it is
   corrected rather than carried.
7. `a_select_with_an_otherwise_compiles_to_a_scan_chain_and_two_frames`'s doc
   -- three op numbers and `op_of[7]` all moved, and "The eleven instructions"
   lost its count under `rust/CLAUDE.md`'s rule while the sentence was being
   edited anyway.
8. `a_select_with_no_otherwise_scans_out_onto_its_own_end`'s doc -- gained the
   op number for the `END`'s own op, which the stream rewrite moved.

## Things I am unsure about, or that a reviewer should look at

1. **`an_absorbed_when_compiles_to_generic` catches nothing measured.** It
   reddened under MT1 and MT3, but only because the stream it pins also
   contains the *outer* `WHEN`'s compiled condition -- the same reason the two
   `SELECT` streams redden. Nothing I ran distinguishes it from them. I kept it
   because it is the only stream-level pin anywhere that an absorbed `WHEN`
   compiles to a bare `Op::Generic` with no region (`run::tests`' absorbed-
   `WHEN` tests pin behaviour, not the stream), and its doc claims only what
   the stream shows. If a reviewer reads that as coverage inflation, deleting
   it costs nothing.

2. **`InstructionKind::When` never appears under a `SELECT CASE`, and I did not
   assert it.** `instruction.rs`'s `if_instruction` maps
   `EnclosingSelect::Plain` to `When` and `EnclosingSelect::Case` to
   `WhenCase`, so a promoted `When`'s `info.case` is always `None`. The
   promoted path does not read `info.case` at all, and correctness does not
   depend on the invariant -- `scan_when`'s own `When` arm ignores `case_text`
   too, so even a `When` under a `SELECT CASE` would behave identically. I
   therefore wrote no comment claiming it and added no `debug_assert`. Naming
   it here rather than in the tree because it is a parser fact this task leans
   on for nothing.

3. **The driver reads `current_value_indent` after the condition's ops run;
   `scan_when` reads it before.** They agree because a nested activation
   restores it, which is the same reasoning Task 1 shipped for an `IF`. I did
   not take it on faith: the elevated-indent probe (a `SELECT` inside a `DO`
   inside an `IF` branch, 34.2 at the deeper indent) and the routine-call probe
   both match the oracle on both engines. Still, this is the axis where a
   future construct that moves the indent *without* an activation boundary
   would break silently.

4. **`ConditionKeyword` derives only `Clone, Copy`.** I wrote `PartialEq, Eq`
   first to match `NodePath` and `PlanSlot` nearby, then removed them after
   checking the workspace builds without: nothing compares two keywords.
   `BodyEngine` in the same file derives the same pair, so the file has both
   conventions.

5. **The rendering helper's placement.** `render_condition_keyword` is in
   `golden.rs` rather than being a method on `ConditionKeyword`, because
   `mod golden` is `#[cfg(test)]` and the method was a `dead_code` warning in a
   production build. The alternative was `#[cfg(test)]` on the method, which
   `ir/mod.rs`'s own comment about `render` argues against.
