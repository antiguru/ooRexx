# Task 7-M3 review: `7d9cfb9a`, `73b538e5`, `74b7f4d9`

Spec compliance **PASS**.
Quality **good**, with one false measurement comment that has to come out.

`1eb3866f` and `d6aabbfe` are the controller's and were skipped.
Every command below was run from `rust/` with stdout, stderr and exit status read separately.

## Verdict on the four priority areas

### 1. The error paths -- no defect

The labelled block has exactly one exit besides falling off its end, and that exit is `break 'region`.
Between `enter_stepped_clause` (`drive.rs:456`) and `leave_stepped_clause` (`drive.rs:830`) there is no `?`, no `return` and no `continue`: I searched the whole span for one and there is none.
So the leave runs on every path and runs once.
The two `?`s in that arm are both *after* the leave -- one on the leave's own result and one on `region`, which is the clause's failure travelling out as `ClauseOutcome::Ran(Err(_))`.
That is the same two-level shape `run_clause_region` had.

I compared every arm of the new loop against `7a7f5849`'s `run_region_ops` line by line.
All twenty arms match in order, in the value produced and in classification:

* `?` became `match { Ok(v) => v, Err(f) => break 'region Err(f) }` in `EvalExpr`, `Load`, `WhenTest`, `LoopRun`;
* `?` became `if let Err(f) = .. { break 'region Err(f) }` in `Store`'s `assign_evaluated` and `LoopHeaderValue`'s `accept_header_value`;
* `return Err(..)` became `break 'region Err(..)` for `ops_in`, `konst`, `store_op_off_its_node`, `loop_op_off_its_node` and all six `op_not_driven` arms;
* `JumpUnless`' `if !register_holds(..)? { return Ok(At(target)) }` became a three-arm `match` that keeps `Ok(true)` falling through, `Ok(false)` answering `At(target)` and `Err` propagating;
* `return Ok(RegionEnd::Flowed(..))` / `At(..)` became the same values under `break 'region`.

Nothing is dropped, nothing is reordered, nothing is reclassified.
The failure the region carries out reaches `leave_stepped_clause` as a value, so the clause's own failure site is recorded there exactly as the closure form recorded it, before the boundary runs.

Behaviourally, seven error probes (`e1`-`e7`) covering `EvalExpr`, `WhenTest`, `LoopHeaderValue`, a failure inside a `LoopRun` body, an `IF` condition and a `SIGNAL ON SYNTAX` trap all agree tw == ir == oracle on stdout, stderr and rc.
The failing clause's own `*-*` echo and the error's line number are right in every one.

### 2. The boundary's obligations -- all present, in the same order

Reading `enter_stepped_clause`/`leave_stepped_clause` against the pre-split `in_stepped_clause_with`, the order is unchanged:
clock invalidation, `>I>` decay, `printed_indent` into `current_value_indent`, `clause_line` then `enter_clause` (which is the `SIGL` line plus the fourth-site tripwire), the `Echo::Gated` echo, `temps_len` watermark, `push_frame` | `pop_frame`, the clause's own failure site, `leave_clause` (which is the `CALL ON` delivery), the boundary's failure site.
`enter_clause`'s and `leave_clause`'s bodies are the pre-split `in_clause`'s prologue and epilogue moved verbatim.

The case that has caught implementations twice was written fresh and run, not taken from the report:

```rexx
call on user zx name h
call on user zy name g
v = raiser()
say 'after' v
say 'second'
```

with `raiser` doing `raise user zx return 'V'` and `h` doing `raise user zy return 0`.
Stdout is `after V` / `G ran 4` / `second` on both engines and on the oracle -- `g` runs at the *following* clause's boundary, after the `SAY` has already printed, and `SIGL` names line 4 rather than line 3.
Under `trace r` the stderr is byte-identical across all three, with `g`'s clauses interleaved between line 5's `say` and line 6's.
`x1` is the same shape with the raising assignment as the program's last clause, and it also agrees.

`c1` (handler at a promoted assignment's boundary), `c3` (`LEAVE` naming a `SELECT` from inside `OTHERWISE`), `c4` (`SIGNAL ON NOVALUE` whose `SIGL` is the failing clause's own line, `novalue at 3`) and `x2`/`x3` all agree on all three descriptors, plain and traced.

**The comparison is not vacuous.** I repeated the report's control on my own probe set: with the `Op::Clause` arm made loud on entry, **every one of the eleven** probes turns rc 120 on the IR arm and is unchanged on the tree-walker.
So each reaches a promoted clause, and the agreement is agreement about the path this task changed.
`drive.rs` was restored from a `cp` copy and its `sha256sum` matches the pre-control file.

### 3. The sharing rule -- holds, and I tried to falsify it

Structurally: `enter_clause`, `leave_clause`, `enter_stepped_clause` and `leave_stepped_clause` each have exactly one definition.
`in_clause` (`clause.rs:415`) and `in_stepped_clause_with` (`run.rs:4593`) are three lines each over that pair, and `drive.rs:456`/`830` is the only other caller of either half.

Behaviourally, two perturbations placed *inside* the shared bodies:

* `enter_clause`'s `current_clause_line = line` -> `line + 1` (`clause.rs:479`). The two engines stayed **byte-identical** on `c2` and `c4` while both diverged from the oracle, and the suite went **1380 passed / 31 failed**.
* `leave_stepped_clause`'s clause-failure `record_failure_site` disabled (`run.rs:4753`). Both engines degraded **identically** to `0 *-* <no failing clause recorded>` on all five error probes, both diverging from the oracle, and the suite went **1378 passed / 33 failed**.

Both halves therefore have one implementation that both engines run, and a recorded expectation catches a change to it.
Every file was restored from a `cp` copy and `sha256sum`-verified, and the tree is clean.

One caution for whoever repeats this: perturbing `current_value_indent` in `enter_stepped_clause` **does** split the engines.
That is not a second boundary implementation -- it is the documented local-vs-field hand-off `Op::TraceClause` relies on (`drive.rs:536-545`), which the perturbation desynchronises by construction.
A perturbation of a value the two engines legitimately consume by two routes is not a test of the sharing rule.

### 4. GC rooting -- unchanged

`impl ClauseValue for RegionEnd` is untouched by the diff, so what a promoted clause roots across a delivered handler is decided by the same code as before.
`leave_clause` still pushes `value.rooted()` *after* `pop_frame`, in that order.
The register frame is reserved by `run_ops`' caller and outlives the whole enter/leave pair, so the region's registers sit below every watermark the clause takes -- the `temps_at_entry`/`pop_frame` pair moved into `SteppedClause` without changing which frame it truncates to.
`LoopHeaderValues::over` is an `ObjRef`, but every writer of it is `temp_at(registers, ..)`, so it aliases a rooted register exactly as it did when the local lived in `run_region_ops`.
The debug tripwire is intact and ran: all dev-profile runs are with `debug_assertions` on.

### 5. `74b7f4d9`'s replacement -- sound

`compile::assert_region_ops_name_their_clause` exists (`ir/compile.rs:1044`) and `debug_assert_names_the_clause` is the per-op debug half, so both citations hold.
The replacement carries no figure and states only the mechanism, which is what the situation admits.
Nothing under `crates/` still matches `10 and 8`, `of the 19`, `nine slots`, `run_clause_region` or `run_region_ops`.

## The claim I was asked to attack: the 10 instructions per promoted clause

**No semantic difference is hiding in the rebuild.**
The new loop skips nothing, defers nothing and makes nothing conditional that the pre-split path did unconditionally -- that is the op-by-op comparison in section 1, plus the arm's prologue and epilogue.
The one work-quantity change in the arm goes the other way: `7a7f5849` fetched the clause's instruction twice per promoted clause on the `GRANTING` path (once in the arm, once in `run_clause_region`), and the rebuild fetches it once (`drive.rs:384`).

The sign matters and it is worth being explicit about.
The rebuild is **slower** than the spike by 10 instructions per clause, so a "does less work" explanation would have to apply to the *spike*, not to what landed.
Since the rebuild demonstrably does everything `7a7f5849` did, any omission that would explain the shortfall has to be on the spike's side, and no patch of the spike survives to check.
So the report's conclusion is not contradicted by anything readable in the diff, and the alternative it was offered against -- the rebuild quietly doing less -- is now closed by inspection rather than left open.
The register-pressure mechanism itself remains unverified.

**What would settle it**, none of which I may run: a disassembly diff of `run_ops::<false>` against a reconstructed spike; or, cheaper, `perf stat -e instructions:u` on a build with the four halves forced `#[inline(never)]`, which separates "inlined into a bigger live set" from "extra work" without needing the spike back.

## Gates, re-run here

| gate | result |
|---|---|
| `cargo test --workspace` | exit 0, **1411 passed, 0 failed, 4 ignored** |
| `+ REXX_{CORPUS,ASSERTIONS,BIF,KEYWORD}_GATE=1`, `--no-fail-fast` | exit 0, **1411 / 0 / 4**, all four banners read `mode: STRICT` |
| `cargo test --workspace --release --no-fail-fast` | exit 0, **1411 / 0 / 4** |
| `+ all four gates`, release | exit 0, **1411 / 0 / 4**, all four `mode: STRICT` |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 (warm target dir -- see unsettled) |
| `ir_dual` sweep | 9 tests, includes `both_engines_agree_across_every_population` |

`nm --print-size` on the release binary reads `run_ops::<false>` at 0x29e7 = 10,727 bytes and finds no `run_clause_region` or `run_region_ops` symbol, matching the report and `74b7f4d9`'s "0x29e7 either side".
The report's headline table matches `bench-baselines/phase-4e-arms.tsv` to every digit it quotes, and `emptyloop`'s regression is reported as a result rather than omitted, which is what the brief required.
No `unsafe`, no em-dash in any added comment, no new count of a mutable in-repo aggregate -- the module doc's old "Two things remain expressible" count was in fact *removed* in favour of an uncounted list.

## Findings

* **Important -- `rust/crates/rexx-exec/src/ir/drive.rs:452-455`.** The comment reads "Measured on `bench-programs/varlookup.rex`, this shape runs the compiled arm **71 instructions per pass below** the tree-walker where the closure form ran it 81 above." −71 is 7-M2's *spike*. This build measures **−51**: `bench-baselines/phase-4e-arms.tsv` records `7-M3 7d9cfb9a varlookup split per_pass_gap ir-tw instructions:u -50.999974`, and `7d9cfb9a`'s own commit message says "gap +81 -> -51". The report's whole "where the 20 instructions went" section exists because the −71 did *not* reproduce, so this is a figure attributed to code that does not produce it -- the same defect `74b7f4d9` removed one commit later from a comment twenty lines below. The "81 above" half is correct. Fix: quote −51, or drop the figure as `74b7f4d9` did.
* **Minor -- `rust/crates/rexx-exec/src/ir/drive.rs:1160`.** Reflowing `debug_assert_names_the_clause`'s doc left `/// index-bearing op` as a two-word orphan line.
* **Minor -- `7d9cfb9a`'s commit message.** Its cycle figures (`varlookup` 1.01065 -> 1.00731) are from the first sitting; the baseline file records the second (1.011185), and the report discloses that the two differ by about a point. The message is immutable and the file is what Task 11 will read, so nothing needs doing to the message -- but the report's disclosure is the only thing reconciling them, and it should survive into the next brief.
* **Pre-existing, not this task -- traced handler indent at a `DO` header boundary.** A `CALL ON` handler delivered at a promoted loop header's boundary has its own clauses indented two spaces less than the oracle indents them (`13 *-*   h:` against `13 *-*     h:`). Both engines agree; only the oracle differs. I rebuilt `7a7f5849`'s three files and reproduced the identical bytes, so `7d9cfb9a` did not introduce it, and I found nothing recording it in the phase's SDD. Per this tree's own rule about corrections going where the next reader sees them, it belongs in the text of whichever task owns trace fidelity, not in this review alone.

## Unsettled

* **Whether the 10 instructions per promoted clause is codegen or a spike-side semantic difference.** Closed on the rebuild's side by the op-by-op comparison above. The spike's side cannot be checked without reconstructing it. Settled by a disassembly diff against a rebuilt spike, or bounded by an `#[inline(never)]` build measured on `instructions:u`.
* **`clippy` was green here from a warm target directory.** The report claims a clean `CARGO_TARGET_DIR`; this tree's own rule says a same-session green is provisional. I did not re-run it clean.
* **`emptyloop`'s cycle ratio under the spike's layout**, which the report already names as unanswerable across two denominators. Nothing here changes that.
