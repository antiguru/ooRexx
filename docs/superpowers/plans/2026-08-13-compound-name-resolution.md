# Compound-name resolution implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop re-deriving at run time what one upfront pass over the body already knows about a compound variable's name -- how it splits, and which slot each piece resolves to.

**Architecture:** `Plan` gains a per-compound entry, built by the pass that already walks every compound name, and the run-time path reads it instead of splitting a string and hashing name bytes.

**Tech Stack:** Rust 2024, `rexx-exec` (`plan.rs`, `stem.rs`, `eval.rs`), `rexx-parse` (`ast.rs`).

## What this is aimed at

`perf record` over `samples/rexxcps.rex` on `5dc12a403`, `REXX_ENGINE=ir`, `count=100`/`averaging=100`, one run, 999 Hz, flat profile by self time:

| share | symbol |
|---:|---|
| 3.80% | `<core::str::pattern::CharSearcher as Searcher>::next_match` |
| 3.38% | `rexx_parse::ast::compound_parts` |
| 4.43% | `<RandomState as BuildHasher>::hash_one::<&[u8]>` |
| 3.29% | `<sip::Hasher<Sip13Rounds> as Hasher>::write` |
| 1.72% | `<Interp>::slot_of` |

Bucketed, that is about 8% of runtime splitting compound names and about 8.6% hashing variable names to find their slots.
Both are constants of the source text being recomputed per execution.

**One function holds both.** `Interp::tail_key` (`stem.rs`) calls `compound_parts` on the interned name, then for each `Tail::Variable` piece calls `read_by_name`, which goes to `Interp::slot_of(name)` and hashes the bytes.
`acompound.key1.loop` therefore costs one split plus two name hashes on every reference, and `rexxcps` references it in its innermost loop.

**The precedent is in the file this plan edits.** `Plan::static_indent` was the single largest self-time function on this same benchmark at 8.0%, and the fix was to compute it in the upfront pass and store it on `Plan`. This is that again, for a different constant.

**The slots already exist.** `Plan::note_compound_name` splits every compound name at build time and calls `slot_for` on the stem and on every variable piece. Nothing at run time is discovering anything new; it is re-deriving an answer the plan computed and dropped.

## Global constraints

These bind every task in this plan, in full.

* **The two engines must agree byte for byte**, on stdout, stderr and exit status, for every corpus program, with `MAX_EVAL_DEPTH` the one accepted divergence. This plan changes no engine's behaviour at all: it is the same answers computed once instead of many times.
* **`const _: () = assert!(size_of::<Op>() == 16)` in `ir/mod.rs` stays.** Nothing here should touch it; if something does, that is a design error to report rather than widen.
* **The oracle at `/home/moritz/dev/repos/ooRexx/` is read-only.** Every invocation wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`, run from a fresh empty directory with absolute paths. Never run `select; when 1 = 0 then; when 2 = 2 then nop; end`, `say date('M','0','D')`, or any `NUMERIC DIGITS` above 1000.
* **A comment may not name the size of a set.** Name the set; a measurement keeps its numbers. When a cardinality has to go, **name fewer things rather than quantify over more** -- a previous task replaced a count with a universal that was false the day it landed.
* **No em-dashes** in comments or markdown; use `--`. Markdown is one sentence per line, `*` bullets, first-word-only capitalisation.
* **Never `git checkout --`**, never `git add -A` for a restore, never `git reset --hard`, never force-push. Back up with `cp` to a private scratchpad directory, restore from the backup, verify with `sha256sum -c`, and rebuild afterwards.
* **Run cargo from `rust/`, never the repository root.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`. Never read a cargo exit code from a pipeline.
* **A test row is either a measured witness or a labelled transcript.** If you claim a row catches something, name the mutation, run it whole-workspace with `--no-fail-fast`, and **re-run it at the final state of your commit** -- a later hunk can add a catcher. If it is a transcript, label it and claim nothing.
* **The interactive `grep` is a ugrep wrapper carrying `--ignore-files`**: it honours `.gitignore` and returns zero matches for gitignored paths including `.superpowers/`. Use `/bin/grep -a`; "nothing found" from the wrapper is unmeasured.
* **Rebuild after restoring from any mutation.** A stale binary outlives its revert, and has manufactured convincing false divergences on this branch twice.

---

### Task 1: the split, computed once

**Files:**
* Modify: `rust/crates/rexx-exec/src/plan.rs` (the new table, filled by the pass that already walks these names)
* Modify: `rust/crates/rexx-exec/src/stem.rs` (`tail_key` reads the table)
* Modify: `rust/crates/rexx-exec/src/eval.rs` (the two stem-name sites, `:407` and `:457`)

**Interfaces:**
* Consumes: `Plan::note_compound_name`, which already splits every compound name and assigns each piece a slot; `rexx_parse::compound_parts`; `Tail`.
* Produces: a per-compound entry on `Plan`, addressed by the compound's own `SymbolId`, holding the stem name and the tail pieces in source order, with each piece marked constant or variable.

- [x] **Step 1: decide how the table is addressed, by measuring rather than by taste**

`by_symbol` is a `HashMap<SymbolId, usize>`, and the profile shows `hash_one::<&SymbolId>` already costing about 1%.
A `Vec` indexed by `SymbolId` costs an index and no hash, and is correct only if ids are dense from zero within one body.
**Establish which it is from `rexx-parse`'s interner before choosing**, and say in the report what you found and what you chose. Do not assume density; do not assume sparsity.

- [x] **Step 2: fill the table in the pass that already walks these names**

`note_compound_name` has the split in hand and throws it away. It needs the compound's `SymbolId` to key the entry -- check its callers, since one of them is `note_variable_ref`, and say in the report whether every caller has an id or whether only some do.
A name reached without an id is the case to handle explicitly rather than to leave to a fallback nobody named.

- [x] **Step 3: read the table in `tail_key`, keeping its behaviour exactly**

`tail_key` must still: join pieces with `.`, take a constant piece verbatim and case-sensitively, read a variable piece's *current value* through the ordinary variable path, derive an unset name's own spelling, and render with `to_text`.
Only the split moves. Slot resolution stays by name in this task -- that is Task 2 -- so `read_by_name` is still what a variable piece goes through.

- [x] **Step 4: the two stem sites in `eval.rs`** use the table's stem name rather than splitting again.

- [x] **Step 5: the fallback, named**

A body reached without a plan entry -- an `INTERPRET` fragment's own compound, whose ids belong to a different `SymbolTable` -- must still work.
`fragment_plan` translates a fragment's ids to enclosing slots through the *text*, so a fragment's compound cannot be keyed by the enclosing plan's ids.
Decide what happens there, implement it, and pin it with a case that runs an `INTERPRET` containing a compound reference. **An `INTERPRET` in a loop is the shape most likely to be wrong.**

- [x] **Step 6: prove the answers did not move**

The corpus sweep and `ir_dual` are the net. Beyond them, capture from the oracle: a compound with a constant tail, with a variable tail, with several tails, with an unset tail variable, with a tail variable whose value changes between references, a bare stem, `DROP` of a compound, and a compound built inside an `INTERPRET`.

- [x] **Step 7: gates, then commit.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`.

- [x] **Step 8: measure this task alone**

`perf stat -e instructions:u` over `rexxcps` at a fixed count, before and after, one run each, reported in the task report.
Instructions rather than wall clock: entry 27 records that the wall clock on these axes moves by more than this whole plan is worth, from layout alone, while the instruction counter's arm-internal noise is at or below 0.005%.

---

### Task 2: the slot, resolved once

**Files:**
* Modify: `rust/crates/rexx-exec/src/plan.rs` (the table gains the slot per variable piece)
* Modify: `rust/crates/rexx-exec/src/stem.rs` (`tail_key` reads slots; `read_by_name`'s remaining callers)

**Interfaces:**
* Consumes: Task 1's table.
* Produces: each variable tail piece carrying the slot `slot_for` assigned it at build time, so no name is hashed to read a tail.

- [x] **Step 1: record the slot beside the piece.** `note_compound_name` already calls `slot_for` on each variable piece and discards the answer; keep it.

**Task 1 found a second filler of that table, and it assigns no slots at all.**
`Plan::bind` records the split of any compound-shaped spelling it binds, which is how a `DO` control variable that is itself a compound (`do a.i = 1 to 5`, whose tail `run.rs` re-resolves on every pass) gets an entry.
`bind` deliberately assigns no slot to the stem or to a piece, because the whole dotted name is already bound to one and adding slots for its parts would move every later slot number in the body -- a frame-layout change rather than a split.
So **an entry does not imply its pieces have slots**, and Task 1's measurement of the loop shape (`run::tests::a_compound_control_variables_tail_re_resolves_every_pass`) is where that bites.
Decide explicitly what a piece with no precomputed slot does -- an `Option` on the piece falling back to `read_by_name`, or `bind` starting to assign slots and the frame-layout consequence being measured -- rather than assuming every entry carries one.

- [x] **Step 2: read the value by slot in `tail_key`**, through the same machinery `read_by_name` reaches after its own lookup, so an unset piece still derives its own spelling and nothing else changes.

- [x] **Step 3: the three-source rule, which is where this task can be wrong**

`Interp::slot_of` resolves in three steps: the plan's names, then the activation's `extra`, then growth.
A precomputed slot is the *first* source only.
Establish and state whether a compound tail piece can ever be bound in `extra` rather than the plan -- `DROP (v)`'s run-time target and a fragment's new names are what put entries there -- and if it can, the precomputed slot must not shadow it.
**This is the one correctness question in the task**; answer it with a program that reaches the case, not with an argument.

- [x] **Step 4: the same oracle captures as Task 1 Step 6**, plus: a compound whose tail piece is introduced by an `INTERPRET`, and a compound referenced both before and after a `DROP` of its tail variable.

- [x] **Step 5: gates, then commit.**

- [x] **Step 6: measure this task alone**, same instrument as Task 1 Step 8.

---

## Measurement, after both land

`perf stat -e instructions:u` over `rexxcps` and over every `Role::Loop` program in `rust/bench-programs/`, base against head, one run per arm.
The loop axes are the control, except for `compound` and `alloc4c`.
`compound`'s `t.k` and `alloc4c`'s `tab.i` each hold a compound variable with a variable tail piece, which is exactly what this plan changes, so both are subject axes; `arith`, `emptyloop`, `strings` and `varlookup` hold no compound at all and are the control.
(An earlier version of this paragraph named `compound` as the only axis holding compound variables. Task 2 measured `alloc4c` and found it moved by 3.19% on its own, then read the program: `tab.i = i` in its inner loop. Task 1's table does not carry an `alloc4c` row, which is why the claim survived it.)
A movement on a control axis is codegen drift and belongs in the record as the cost side.

Wall clock only if the instruction counter shows a change worth trying to time, and then under the accept rule with **two** do-nothing controls -- entry 27 records four byte-identical copies of one binary spanning 3.47% because their `argv[0]` basenames differed in length, so every arm is staged at one fixed path.

Record the result as the next entry of `docs/superpowers/plans/phase-4f-record.md`, including the axes that did not move.

## Deliberately out of scope

* Replacing the default hasher. It would cut the same cost by a smaller factor and would hide whether the resolution work was removed.
* Caching a compound's *value* or its tails map. That is a different optimisation with a different invalidation story, and this project has an unmerged prototype of it on the C++ side.
* Anything about `Op` or the compiled stream. This plan is under both engines equally.
