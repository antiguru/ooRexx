# Stem-slot resolution implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop hashing a compound's stem name to find its slot on every reference, the third and last application of the fix `2026-08-13-compound-name-resolution.md` made twice.

**Architecture:** `Plan::note_compound_name` already calls `slot_for` on the stem and discards the answer. `CompoundName` gains the slot beside the stem bytes it already carries, and the stem accessors take it the way `read_stem_at` already takes a precomputed slot.

**Tech Stack:** Rust 2024, `rexx-exec` (`plan.rs`, `stem.rs`, `eval.rs`).

## What this is aimed at, and why it is the obvious next increment

`perf record` over `samples/rexxcps.rex` at `1b11a6cad`, `REXX_ENGINE=ir`, `count=100`/`averaging=100`, binary at a fixed path, 999 Hz, bucketed by self time:

| bucket | before the compound-name plan | after it |
|---|---:|---:|
| compound-name work | 8.85% | **2.51%** |
| symbol lookup hashing | 8.61% | **9.33%** |

**The bucket that did not fall is the finding.** That plan removed the name hashing for a compound's *tail pieces* and left its *stem*: every `stem_get` and `stem_set` opens with `slot_of(stem_name)`, so `acompound.key1.loop` still hashes `ACOMPOUND.` on every read and every write.
`hash_one::<&[u8]>` is still 3.38% and `Interp::slot_of` 1.86% in that profile.

**The work is already done twice and the shape is settled.** `read_stem_at` takes an `Option<usize>` slot and falls back to `slot_of` when it has none; `CompoundName` already carries the stem's bytes; `note_compound_name` already assigns the stem a slot and throws it away.

## Global constraints

These bind the task in full.

* **The two engines must agree byte for byte**, on stdout, stderr and exit status, for every corpus program, with `MAX_EVAL_DEPTH` the one accepted divergence. **This plan changes no behaviour**: it is the same answer computed once instead of many times.
* **The oracle at `/home/moritz/dev/repos/ooRexx/` is read-only.** Every invocation wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`, from a fresh empty directory, absolute paths. Never run `select; when 1 = 0 then; when 2 = 2 then nop; end`, `say date('M','0','D')`, or any `NUMERIC DIGITS` above 1000.
* **A comment may not name the size of a set.** When a cardinality has to go, **name fewer things rather than quantify over more** -- a previous task replaced a count with a universal that was false the day it landed.
* **Do not claim more precision than you measured.** State how many runs a figure rests on, and measure the instrument's own spread before reporting a difference smaller than a percent. Entry 29 withdrew four figures for exactly this.
* **No em-dashes** in comments or markdown; use `--`. One sentence per line, `*` bullets, first-word-only capitalisation.
* **Never `git checkout --`**, never `git add -A` for a restore, never `git reset --hard`. Back up with `cp` to a private scratchpad directory, and **`touch` the file after restoring** -- `cp -p` preserves mtime, cargo then reports a successful build, and the mutated binary stays in place.
* **Run cargo from `rust/`.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`. Never read a cargo exit code from a pipeline.
* **A test row is either a measured witness or a labelled transcript.** Name the mutation, run it whole-workspace with `--no-fail-fast`, and re-run it at the final state of the commit. A mutation that corrupts a stem name can send a corpus program non-terminating, so use a per-test timeout.
* **The interactive `grep` honours `.gitignore`** and returns zero matches for gitignored paths including `.superpowers/`. Use `/bin/grep -a`; "nothing found" from it is unmeasured.

---

### Task 1: the stem's slot, resolved once

**Files:**
* Modify: `rust/crates/rexx-exec/src/plan.rs` (`CompoundName` gains the slot; `note_compound_name` keeps what it already computes)
* Modify: `rust/crates/rexx-exec/src/stem.rs` (the accessors take it)
* Modify: `rust/crates/rexx-exec/src/lib.rs` (`Code` hands the stem's name and slot back together)
* Modify: `rust/crates/rexx-exec/src/eval.rs` (the compound read path passes it)
* Modify: `rust/crates/rexx-exec/src/run.rs` (the compound write, the controlled loop's own read, and `DROP` of a tail)
* Modify: `rust/crates/rexx-exec/src/parse_template.rs` (a compound `PARSE` target's read)

(An earlier version of this list put the compound **write** path in `eval.rs`. It is not there: `assign_expr_target` is `run.rs`'s, and it is what `tab.i = i` goes through on both engines. Two more callers hold an entry as well, `run.rs`'s controlled-loop re-test and `drop_variable`, and a compound `PARSE VAR` target in `parse_template.rs`. Corrected here after reading `stem.rs`'s callers, which is what Step 2 asks for.)

**Interfaces:**
* Consumes: `CompoundName { stem, tails }` and `Plan::compounds` from the previous plan; `read_stem_at`'s existing `Option<usize>` convention.
* Produces: the stem's slot on the same entry, and `_at` forms of the stem accessors that take it.

- [x] **Step 1: put the slot where the split already is.**

`note_compound_name` calls `slot_for(stem)` and discards it. Keep it, on `CompoundName`, as an `Option` for the same reason the tail pieces carry one: `CompoundName::split` is entered by a fragment with no plan at all, and `Plan::bind` records a split for a compound `DO` control variable **deliberately assigning no slots**. An entry existing does not imply it carries slots, and that must stay visible in the type rather than remembered in a comment.

- [x] **Step 2: give the accessors the `_at` form.**

`read_stem_at` is the pattern: take `Option<usize>`, use it, fall back to `slot_of(name)` when it is `None`. **Every `slot_of(stem_name)` in `stem.rs` is in scope** -- find them by reading the file rather than from a list in this plan, and say in the report how many there were and which you changed. A site you leave by choice is a finding to state, not an omission.

- [x] **Step 3: pass the slot from the callers that have an entry.**

`eval.rs`'s compound read path already looks the entry up for the stem name; the write path is its sibling. A caller with no entry passes `None` and behaves exactly as today.

- [x] **Step 4: the three-source question, which is the only place this can be silently wrong.**

`Interp::slot_of` resolves through the plan's names, then the activation's `extra`, then growth; a precomputed slot is the first source only.
The previous plan established that a tail piece **can** be bound in `extra` -- `do za.zi = 1 to 3` grows `ZI` into it -- and that a precomputed slot cannot shadow one, because a slot only reaches a piece via `slot_for` putting its name in the plan's map.
**Establish the same for a stem, by running rather than by analogy.** Construct a program where a stem name reaches `extra`, or show it cannot; either answer is a finding, and "it cannot happen" without a program is the failure mode this project has been bitten by repeatedly.

- [x] **Step 5: prove the answers did not move.**

Build the accessor-level check the previous plan's reviews used: `assert_eq!(precomputed slot, the full three-source resolution)` inside the accessor, run the whole workspace, then **invert it** to prove the probe fires. A dead probe reporting zero firings looks exactly like a live one.

- [x] **Step 6: oracle captures.** A stem read, a stem write, a bare stem, `DROP` of a stem, `PROCEDURE EXPOSE` of a stem, a stem whose tail changes between references, a compound inside an `INTERPRET`, and a compound `DO` control variable -- the last two being the entries that carry no slots.

- [x] **Step 7: gates, then commit.**

- [x] **Step 8: measure.**

`perf stat -e instructions:u` over `rexxcps` and every `Role::Loop` program, base against head, arms staged at **one fixed binary path** -- four byte-identical copies of one binary once spanned 3.47% on wall clock here purely because their `argv[0]` basenames differed in length.
**`compound` and `alloc4c` are subject axes, not controls**: `t.k` and `tab.i = i` are compounds, and the previous plan's own text got this wrong until a measurement caught it.
Measure the instrument's spread on each axis before reporting any difference under a percent.
Then re-profile `rexxcps` with `perf record` and report the hashing bucket, since the whole reason for this task is that the bucket did not fall last time.

---

## Measurement, after it lands

Record the result as the next entry of `docs/superpowers/plans/phase-4f-record.md` -- **appended, never by editing an earlier entry**, which entry 29 exists to restate.
Include the axes that did not move and the cost side if there is one.

## Deliberately out of scope

* Replacing the default hasher. If the stem was the last name-keyed lookup on the hot path, the bucket should fall without it, and doing both at once would hide which one worked.
* The compound `DO` control variable's own tail pieces, which still resolve by name because the whole dotted name holds the slot; the previous plan's report has the argument and its correction.
* Caching a stem's tails map or its values. Different optimisation, different invalidation story.
