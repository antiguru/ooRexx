# Task 2 review -- the slot, resolved once (`a93bfb548`, record entry `df3c34087`)

Reviewed against `task-2-brief.md`, `task-2-report.md` and `review-b3e8cf4d2..df3c34087.diff`, with `task-1-review.md` as the prior.
Everything below was run by me, in detached worktrees at `df3c34087`, `b3e8cf4d2` and `cb4f27d1c` under private `CARGO_TARGET_DIR`s and a private backup directory, with `ootest` and `rust/corpus-l1` symlinked in.
No file in `/home/moritz/dev/repos/ooRexx-rust-rewrite` was edited; every source mutation was applied inside the worktree, restored from a `cp -p` backup, verified with `sha256sum -c`, `touch`ed and rebuilt.

## Verdicts

* **Spec compliance: PASS.** All six steps done. The one correctness question is answered correctly, I reproduced its program, and no instrument I could build finds a behaviour change. The stem half and the `bind` half are left alone, as the brief allows and the report states.
* **Code quality: PASS WITH CHANGES.** One measurement in the record entry is stated to a precision the instrument does not have, and its percentage restatement in the report is wrong by a factor of ten at the low end. No code change is required for either. The code itself is right: the `Option` is handled at every site, the type carries the distinction rather than a comment, and the fallback is one function with the fast path.

## Findings, most severe first

### 1. A reproducible panic and an oracle divergence, found while adjudicating the `exec_procedure` concern -- NOT this task's, and not caused by it

`PROCEDURE` as the first instruction of a `::ROUTINE` is accepted by this crate and raises 17.1 on the oracle.

```rexx
call myrout
say 'after'
exit
::routine myrout
procedure
say 'in routine'
return
```

Oracle: `Error 17.1 ... PROCEDURE is valid only when it is the first instruction executed after an internal CALL or function invocation.`, rc 239.
This crate at `df3c34087`, **both engines**: `panicked at crates/rexx-core/src/roots.rs:409 ... grow_slots on a frame that is not the top one`, rc 101, nothing on stdout.

The chain: `Activation::routine` sets `entered_by_call: true`, so `exec_procedure`'s `first_instruction && entered_by_call` guard admits it; `exec_procedure` then `push_slots` a second frame onto an activation whose `owns_frame` is already `true`, and `invoke_call`'s return path pops exactly one. The leaked frame leaves the caller's frame off the top, and the caller's next `slot_of` miss (here, `RESULT` inside `invoke_named_call`) trips `grow_slots`' assertion.
The `caller.extra = resolved` write is on this same path and wipes the caller's run-time bindings there, but the frame leak dominates and is reached first.

Nothing in `b3e8cf4d2..df3c34087` touches `run.rs` or `activation.rs`, so this is pre-existing and independent of Task 2. It wants its own task.

### 2. The record entry's four control-axis figures are stated more precisely than the instrument supports

Entry 28: "Task 2 moved the four axes holding no compound variable by **43 to 903 instructions** ... `strings` -48, `varlookup` +43, `emptyloop` -903, `arith` +505".

I reran all four, `perf stat -e instructions:u`, `REXX_ENGINE=ir`, every arm staged at one fixed path, two runs per arm interleaved. Taking each arm's minimum:

| axis | `b3e8cf4d2` | `df3c34087` | my difference | entry's |
|---|---:|---:|---:|---:|
| `strings` | 44,928,588,979 | 44,928,589,208 | **+229** | -48 |
| `varlookup` | 44,840,637,596 | 44,840,637,038 | **-558** | +43 |
| `emptyloop` | 27,150,610,924 | 27,150,610,418 | **-506** | -903 |
| `arith` | 20,403,950,597 | 20,403,949,898 | **-699** | +505 |

(One `strings` `b3e8cf4d2` run came back 51,000,456 instructions high -- external interference -- and is excluded; every other run of all seven axes is used.)

Same-binary, run-to-run spans in the same sitting: `varlookup` head **1,035**, `arith` head **802**, `arith` t1 **524**, `emptyloop` head **301**. So the instrument's own resolution on these axes is about a thousand instructions -- larger than three of the four figures the entry reports, and the sign of `strings` comes out opposite in my runs.

The report measured arm-internal spread for `rexxcps`, where the effect is 2.61%, and did not measure it for the four axes where the claimed effect is one part in a billion -- which is the only place it decides anything.
**The conclusion is right and I reproduce it**: Task 2 leaves the four control axes at the same op stream, and the drift belongs to Task 1 (below). What should change is the statement: a bound ("under about a thousand instructions, which is the instrument's own resolution on these axes") rather than four signed movements.

### 3. The report's percentage restatement of that range is wrong by ten at the low end

"they move by between 43 and 903 instructions out of twenty to forty-five billion, which is 0.000001% to 0.000003%."
43 / 44,840,637,921 = 9.59e-10, which is **0.0000001%**, not 0.000001%. The top of the range checks out (903 / 27,150,611,005 = 3.33e-8 = 0.0000033%).
This is in `task-2-report.md` only; entry 28 carries the instruction counts without the percentages, so the repository document is not affected.

### 4. `note_compound_name`'s new inline comment quotes profile shares without naming the commit they were taken at

"measured by `perf record` over `samples/rexxcps.rex` ... hashing a `&[u8]` was 4.43% of self time with SipHash's own `write` at a further 3.29% and `Interp::slot_of` at 1.72%."
Those are the plan's own figures at `5dc12a403`, before either task. The plan and entry 28 both name that commit; the comment does not, and it sits in the function that removes the work the numbers measure -- so a reader who re-profiles at head will not find them. Add the commit, as the plan and the record both do.

### 5. No test pins the other build order for the same symbol

`a_control_variable_does_not_take_the_slots_off_a_compound_already_seen` covers `say v.i` then `do v.i`, where `bind`'s new `get_or_insert_with` is what preserves the slots. The reverse order -- `do v.i = 1 to 2` then a body reference -- rests on `note_compound_name`'s *unconditional* `self.compounds[id.index()] = Some(entry)` overwriting `bind`'s slotless entry, and nothing asserts it.
Both orders end with the slotted entry, so no answer is at risk and no output can see it; it is the same performance property the existing test exists for, in the other direction.

### 6. A stray line wrap in the new doc comment

`plan.rs`, `build_records_a_compounds_split_under_the_compounds_own_id`: "... while `DD.JJ`'s and `V.I.7`'s carry / a number. The / numbers are spelled out ...". A one-word line, an edit artifact. Cosmetic.

## The five checks

### 1. The correctness question -- PASS, reasoning verified and the program reproduced

**The `do za.zi` case, reproduced.** I instrumented `Interp::slot_of` in a worktree to print each growth into `extra` and each hit against it, built release, and ran `do za.zi = 1 to 3 / nop / end` from a fresh empty directory on both engines:

```
PROBE grow-into-extra "ZI" slot 1
PROBE grow-into-extra "ZA." slot 2
PROBE hit-extra "ZI" slot 1
PROBE hit-extra "ZA." slot 2
   ... repeating, once per resolution, for the rest of the loop
```

Byte-identical on `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`. So the report's claim is exactly right: a compound `DO` control variable's tail piece **is** bound in `extra` and not in the plan, and the `Option` is required rather than defensive.

**The no-shadow argument is sound, and I checked it two ways rather than reading it.**
The argument: a `Some` slot lands on a piece only through `Plan::slot_for`, and `slot_for`'s only effect is `self.names.entry(name).or_insert(next)` -- so the piece's name is in `plan.names` at exactly that number. `Interp::slot_of` reads `activation.plan.slot_of(name)` first and `extra` only on a miss. Therefore for a slotted piece `extra` was already unreachable before this task existed. I read both functions and the argument holds verbatim.

* **The shadow sweep, reproduced.** A build that prints whenever a name found in the plan *also* has an `extra` entry, run over every `rust/corpus/**.rex` and every `rust/bench-programs/*.rex` on both engines -- **160 runs, zero shadow lines**. The probe site was shown live by making the same line unconditional: it fires 10,000,501 times on `compound.rex` alone and twice on a three-line program, so the sweep executed the branch millions of times and never found one.
* **A stronger probe than the shadow one, and it is the one that settles the task** -- see check 3.

`exec_procedure` does hold the same rule explicitly (`if plan.slot_of(&name).is_none()`), as reported, and `Activation::routine`/`Activation::nested`/`Activation::new` all start `extra` empty, so nothing else can seed a plan name into it.

### 2. Task 1's invariant, and the `Option` handling -- PASS

`Plan::bind` still assigns no slot to the stem or to a piece; its only `slot_for` is the one for the whole dotted name. So an entry does not imply its pieces carry slots, and the code says so in the type: `TailPiece::Variable { name, at: Option<usize> }` is a named field on a struct variant, and `join_tails` cannot read a piece without deciding what `None` means. That is the right shape, and it matches `read_stem_at`/`read_at`, which already carry `at: Option<usize>` for the same relationship.

Every site that constructs or matches the variant, from a whole-tree search: `CompoundName::split` (constructs `at: None`), `note_compound_name` (`*at = Some(self.slot_for(name))`), `join_tails` (reads), and three test sites. There is no fourth reader and no `unwrap`.

The two fillers now agree on the *outcome* whichever order the pass reaches them in: `note_compound_name` overwrites unconditionally, `bind` uses `get_or_insert_with`, so the slotted entry wins from either side. Both entries hold the identical split because both split the identical spelling under the identical `SymbolId`, which the new test checks rather than assumes (`assert_eq!(controlled.control, id)`).

### 3. Behaviour preservation -- PASS, by the equivalent of Task 1's technique

Task 1's review asserted the cached split equals a freshly computed one. The slot half's equivalent is to assert the **precomputed slot equals the full three-source resolution**, which is exactly the claim the whole task rests on, and it subsumes the committed tripwire (which only compares against the plan's own map).

Inside `Interp::read_by_name_at`, in the `Some(slot)` arm:

```rust
let fresh = self.slot_of(name);
assert_eq!(slot, fresh, "REVIEW PROBE: ...");
```

`memcap 8G cargo test --workspace --no-fail-fast`: **1479 passed, 0 failed, 4 ignored, zero firings**, across the corpus differential, the `ootest` population sweep, the dual-engine case files and every unit test.
**Negative control**: inverted to `assert_ne!`, the same probe fires **18 times over 18 failing tests** in `rexx-exec` alone, so it is live and not vacuous.

Everything after the slot in `read_by_name_at` is the one body `read_by_name` always had -- a set slot yields its value, an unset one derives the piece's own spelling through `self.text(name)` -- and `join_tails` still renders with `to_text` and joins with `.`. `read_by_name` is now literally `read_by_name_at(name, None)`, so the two paths cannot drift.

**Gates in my worktree**, each read unpiped: `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0, re-run with the whole `debug/.fingerprint` directory removed so every crate including `rexx-exec` was genuinely re-linted; `memcap 8G cargo test --workspace --no-fail-fast` **1479 passed, 0 failed, 4 ignored** -- the reported count exactly, two above the brief's 1477 baseline.

**Oracle, independently re-captured.** All **twelve** stanzas of `tests/ir_dual_cases/compound-names` extracted from the committed file and run from a fresh empty directory at absolute paths under `ulimit -v 1048576`: every one matches the recorded transcript byte for byte, rc 0, empty stderr. The two new stanzas also match the oracle byte for byte under this crate on **both** engines, run directly rather than through the harness.

### 4. The record entry `df3c34087` -- PASS WITH CHANGES

Every number checked against the diff, against the report and against my own runs.

* Hashes: base `cb4f27d1c`, Task 1 `b3e8cf4d2`, head `a93bfb548` -- all three correct, and the record entry is a second commit precisely so its `commit` field names one that exists.
* The seven-row combined table is internally consistent to the last digit (each percentage recomputes from its own pair), and I reproduce all three subject axes:

| axis | entry: base | entry: head | entry | mine |
|---|---:|---:|---:|---:|
| `compound` | 37,163,096,047 | 27,800,230,124 | -25.19% | **-25.20%** |
| `alloc4c` | 9,392,724,256 | 8,456,718,073 | -9.97% | **-9.97%** |
| `rexxcps` | 28,594,847,258 | 25,865,205,907 | -9.55% | **-9.56%** |

* The split-by-task table (-17.70%/-9.12%, -7.19%/-2.61%, not measured/-3.19%) reproduces: mine are Task 1 -17.69%/Task 2 -9.12% on `compound`, -7.14%/-2.60% on `rexxcps`, -3.19% on `alloc4c`. Compounding the two halves recovers the combined figure on both axes (-25.20%, -9.55% against -25.19%, -9.55%).
* "All of the +0.04% to +0.60% above belongs to Task 1" -- **confirmed**. My `cb4f27d1c`-to-`b3e8cf4d2` figures are +0.0360%, +0.1845%, +0.3148%, +0.5968%, and the `b3e8cf4d2`-to-head figures on the same four axes are hundreds of instructions. The claim is right; the four signed figures behind it are finding 2.
* "twenty to forty-five billion" -- correct; the four span 20.4G (`arith`) to 44.9G (`strings`).
* Every `rexxcps` arm self-calibrated to `100 x 100` on my runs too, checked on each arm's own `Averaged:` line, so all arms did the same work. `compound` and `alloc4c` stdout is byte-identical across all three arms.
* The cause section's profile shares match the plan's table at `5dc12a403`.
* "A compound's *stem* still resolves by name at every reference (`stem_get`/`stem_set` open with `slot_of`)" -- verified in the source: `stem_get`, `stem_set`, `stem_drop_tail`, `stem_assign` and `stem_drop` all open with `self.slot_of(stem_name)` and none is touched by the diff.
* What the entry does not claim is correctly bounded: no wall-clock figure, and no wall-clock run was taken.

The one change wanted is finding 2.

### 5. The plan's correction -- PASS

The plan's measurement paragraph now reads "The loop axes are the control, except for `compound` and `alloc4c`", names `t.k` and `tab.i` as the subject axes and `arith`/`emptyloop`/`strings`/`varlookup` as the control, and carries a parenthetical saying what the earlier version claimed and how it was caught. That is the right disposition -- corrected in place with what was removed and why.

I checked the underlying facts rather than the prose: `bench-programs/alloc4c.rex`'s inner loop is `tab.i = i`, `compound.rex`'s is `t.k = t.k + 1`, and in `arith`, `emptyloop`, `strings` and `varlookup` the only period anywhere in the file is inside a comment. `samples/rexxcps.rex` does reference `acompound.key1.loop` in its innermost loop, as entry 28's disposition says.

**No other claim in the plan or the entry still rests on the old assumption.** The only other occurrences of "only" in the plan are about the `Vec` density argument and the three-source rule, neither of which touches axis roles.

## The four concerns, adjudicated

### "The new tripwire adds no coverage" -- correct, and I reproduced both halves

| run | mine | reported |
|---|---|---|
| M4 (`note_compound_name` stores `slot_for(name) + 1`) | **1459 passed, 20 failed, 4 ignored**, 19 distinct tests, tripwire panicked **18** times | 20 failures across 19 tests, 18 panics |
| M4' (M4, tripwire deleted) | **1459 passed, 20 failed, 4 ignored**, the identical 19 test names, **0** tripwire panics | the same 20 failures |

Both rows reproduce to the digit, and the two failing sets are name-for-name identical, so the tripwire really is the only thing M4' removes. The failing set spans the compound unit tests in `plan.rs`, `stem.rs` and `run.rs`, the case-file harness (`both_engines_agree_on_every_case_file`), the differentials (`corpus_differential`, `bif_assertions_differential`, `keyword_assertions_differential`), the population sweep (`both_engines_agree_across_every_population`), the collector tests and `the_exempt_set_matches_the_current_failures` -- so a wrong slot is already loud in every instrument this crate has before the assertion is consulted.

**Accepted as stated.** The report says plainly that the tripwire catches nothing the suite would otherwise miss and is kept for *where* it fails rather than *whether*, which is the honest disposition and the one this project's own rule asks for. Keeping it is defensible on the same ground `Op::Load`'s `at` already carries one: the invariant it names -- entry and activation from one plan -- is held by construction and by nothing typed, and a violation would otherwise surface as a wrong byte in a differential rather than as a named premise at `join_tails`.

### "A compound `DO` control variable's tail still resolves by name every pass" -- accept the decision; the stated reason is slightly overstated

The decision is right and the scope call is right: it is a frame-layout change, its cost is not knowable without its own measurement, and folding it into a change whose whole claim is that behaviour did not move would be wrong.

The reason as written -- that it would "change what `Code::slots`, `Op::Load`'s `at` and `PROCEDURE EXPOSE`'s alias indices all mean" -- overstates it. All three are derived from the same `Plan`, so a consistent renumbering keeps them consistent; what actually changes is the *numbers*, not the meanings. The real costs are two, and one of them is not in the report: every body's frame grows by one slot per distinct stem prefix and tail-piece name, and `ir`'s op stream carries slot indices as `u16`/`u32` with `ChunkTooLarge` refusing a chunk that exceeds them -- so a large body could cross the width and fall back to the tree-walker, which the dual harness sees as a nonzero `chunks_refused`. That strengthens the decision rather than weakening it.

### "The stem half is untouched" -- confirmed

`stem_get`, `stem_set`, `stem_drop_tail`, `stem_assign` and `stem_drop` all still open with `self.slot_of(stem_name)`, `CompoundName::stem` carries no slot, and the diff to `stem.rs` is three hunks, all inside `join_tails`/`read_by_name`/`read_by_name_at`. `note_compound_name` still calls `slot_for(&entry.stem)` and still discards the answer, which is exactly the remaining work entry 28 names.

### `exec_procedure`'s `caller.extra = resolved` -- **not a real defect**

On the path the oracle permits -- `PROCEDURE` as the first instruction after an internal `CALL` -- the replacement cannot lose anything, and the reason is structural rather than incidental:

* `invoke_call`'s `Entered::Label` arm sets `callee.extra = caller.extra.clone()`.
* `PROCEDURE` is legal only as the callee's *first executed instruction*, so nothing has run in the callee since that clone, and the caller is suspended and cannot have changed its own map.
* `exec_procedure` only ever *adds* to that map before the write-back: `expose_names` and the `slot_of` in the binding loop insert, and nothing removes.

So `resolved` is a superset of `caller.extra`, and assigning it is merging. The eager write-back is also necessary, not merely harmless: the callee sets `owns_frame = true`, and `invoke_call`'s return path deliberately skips the `extra` move-back for an owning callee, so a name grown while resolving a computed `expose (v)` would otherwise be stranded.

The replacement *would* wipe on the `::ROUTINE` path, where `Activation::routine` starts `extra` empty -- but that path is finding 1, it is an unrelated pre-existing bug, and it panics before the wipe can be observed. **Fix finding 1; leave this line alone.**

## Runs behind this review

* Whole workspace at `df3c34087`, my worktree: **1479 passed, 0 failed, 4 ignored**; `fmt` exit 0; `clippy -D warnings` exit 0.
* Behaviour probe (`assert_eq!` precomputed slot against the full three-source resolution) whole workspace: 1479 passed, **zero firings**; inverted, 18 firings over 18 tests.
* Shadow sweep, plan-name-also-in-`extra`, 80 programs x 2 engines: **160 runs, zero shadows**, probe site shown live at 10,000,501 executions on one program.
* `do za.zi = 1 to 3` under an instrumented `slot_of`: growth into `extra` plus a hit per resolution, identical on both engines.
* Oracle, fresh empty directory, absolute paths, `ulimit -v 1048576`: all twelve `compound-names` stanzas re-captured, all matching byte for byte at rc 0 with empty stderr; stanzas 11 and 12 also run on both engines against those captures.
* Mutations, whole workspace, `--no-fail-fast`, restored from backup, `touch`ed and rebuilt after each: M4 (1459/20/4, 18 tripwire panics) and M4' (1459/20/4, 0 panics, identical names).
* `perf stat -e instructions:u`, `REXX_ENGINE=ir`, three arms (`cb4f27d1c`, `b3e8cf4d2`, `df3c34087`) staged at one fixed path `.../t2rev/bench/armbinary`, two interleaved rounds per axis over seven axes, run from a fresh empty directory.
* `PROCEDURE` inside a `::ROUTINE`: oracle 17.1 at rc 239, this crate a `grow_slots` panic at rc 101 on both engines.
