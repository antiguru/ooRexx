# Task 1 report -- rulings, grants, flat-loop adoption

Status: DONE_WITH_CONCERNS (gates G1-G6 all exit 0 at 4b694fe7c; concerns below are
documentation-level, none blocks)

Base: 0d911ab7c.

## Commits

| sha | kind | subject |
|---|---|---|
| 66073d0a2 | docs | Record the Phase 6 rulings and unsafe grants in the roadmap |
| 4b694fe7c | code | Adopt the flat-loop path: drop its toggle and SPIKE labels |

The queued item `.superpowers/sdd/queued/2026-09-26-flatloop-spike-comment.md` is closed by an
appended `## Closed 2026-09-29 by 4b694fe7c` section (the convention the
`2026-09-21-parse-message-target-unimplemented.md` item uses). Not committed: `.superpowers/` is
ignored in this worktree (`.gitignore:30`), so no commit carries it.

## Roadmap sections added (docs/superpowers/plans/2026-07-27-rust-rewrite.md, at 66073d0a2)

| Lines | What |
|---|---|
| 26-28 | Global Constraints, Rust floor: nightly amended for R4 (TSan gate only, installed toolchain, never a shipped build) |
| 109-111 | Section 1 index rows for D-U2, D-U3, D-U4 |
| 206-215 | D3's 2026-09-29 note: rulings (3) R2, (4) R3, (5) R5 quoted verbatim from spec 1.1, with pointers to D-U2/3/4 |
| 311-345 | `### D-U2 -- Send for the interpreter island`: module `rexx-exec/src/island.rs` (the spec names none; this block names it), lands in S4; invariant, why the compiler cannot check it, enforcement, breakage |
| 347-371 | `### D-U3 -- The signal module`: spec names no file; proposed `rexx-exec/src/signal.rs`, fixed when S4 is planned; `libc` direct (R5); async-signal-safe handler invariant |
| 373-398 | `### D-U4 -- frame.rs: one frame arena per activity`: LIFO per activity, used by Task 5; the per-arena invariant the unsafe blocks rely on (restating frame.rs's module-doc properties 1-5), and that LIFO is a logical property, not a safety one |
| 657 | Row 6: "kernel lock (per interpreter since D3's 2026-09-29 note, ruling 1: the design's baton; nothing process-wide)" |

New headings and index rows use `--`, not the existing blocks' em-dash, per the task's no-em-dash rule.

## SPIKE grep, before and after

Command: `/bin/grep -a -rn 'SPIKE\|spike' rust/crates/rexx-exec/src` (paths below shortened to be
relative to `rust/crates/rexx-exec/src/`).

Before (0d911ab7c):

```
eval/tests.rs:838:/// (`tests/spike.rs`) already establishes recurses exactly once per
ir.rs:85:    /// **SPIKE, not for commit.** The bottom of one pass of the flattened
lib.rs:78:// down to prove (Task 3's spike). Extended task by task with the branches and
lib.rs:1385:    /// **SPIKE.** The flattened `DO`/`LOOP`s the op driver has open, innermost
lib.rs:1392:    /// **SPIKE.** The innermost open flat loop, held apart from the stack of
lib.rs:1395:    /// **SPIKE.** The constructs the op driver has open, innermost last, across
lib.rs:1398:    /// **SPIKE.** Boxes a finished loop handed back, so that entering a loop
run.rs:326:    // ---- the instruction loop, which is what this spike is for ----
run.rs:598:            // `EXIT`, bare or with a result: the spike had only the bare form
run/loops.rs:93:/// **SPIKE, not for commit.** One repeating loop being driven from the op
run/loops.rs:156:/// **SPIKE.** What a header clause's own answer means: `Some(flow)` is the
run/loops.rs:170:/// **SPIKE.** The `WHILE`/`UNTIL` of the `DO`/`LOOP` at `index`, read back off
run/loops.rs:179:/// **SPIKE.** What `Interp::flat_loop_start` decided.
run/loops.rs:187:    /// Not a shape this spike drives: take the nested path, with the header
run/loops.rs:193:/// **SPIKE.** What one pass boundary decided.
run/loops.rs:1284:    /// **SPIKE, not for commit.** Sets a repeating loop up to be driven from
run/loops.rs:1287:    #[allow(clippy::too_many_arguments, reason = "spike")]
run/loops.rs:1299:        // SPIKE: the switch is a run-time one so that both arms are the same
run/loops.rs:1300:        // binary -- the per-op checks this spike adds to the driver's loop are
run/loops.rs:1440:    /// **SPIKE.** One pass boundary of the innermost flat loop: the state is
run/loops.rs:1475:    /// **SPIKE.** One pass boundary: what the body just answered, then the
run/loops.rs:1551:    /// **SPIKE.** An `UNTIL` loop's own bottom-of-pass test: `Some(flow)` is
run/loops.rs:1592:    /// **SPIKE.** Who a failing header re-test is blamed on: the clause that
run/loops.rs:1616:    /// **SPIKE.** A `WHILE` that failed, blamed on the `DO`/`LOOP` clause at
run/loops.rs:1630:    /// **SPIKE.** The header re-test, in its own clause: `Some(flow)` is the
run/loops.rs:1664:    /// **SPIKE.** [`Interp::flat_loop_header`] for a loop that carries a
tests.rs:224:/// Step 2, and the reason the spike exists in the shape it does.
ir/drive.rs:69:/// **SPIKE.** One construct this level has open: a `SELECT`'s branch, or a
ir/drive.rs:1880:                                        // **SPIKE.** Flattened when this is a
ir/drive.rs:1881:                                        // shape the spike drives: the frame goes
ir/drive.rs:2062:                // **SPIKE.** The bottom of a flattened pass, reached by the
ir/drive.rs:2159:        // **SPIKE.** `Flow::Next` settles at `next` whatever is open: `absorb`
ir/drive.rs:2200:                        // **SPIKE.** A `LEAVE`/`ITERATE` that reached this
ir/compile.rs:236:    // **SPIKE.** Indexed by instruction: for an `END` that closes a repeating
ir/compile.rs:349:                // **SPIKE.** A repeating loop's `END` carries the op that ends
ir/compile.rs:1038:            // **SPIKE.** The `END` of a repeating loop, which the flattened
```

After (4b694fe7c):

```
eval/tests.rs:838:/// (`tests/spike.rs`) already establishes recurses exactly once per
lib.rs:78:// down to prove (Task 3's spike). Extended task by task with the branches and
run.rs:326:    // ---- the instruction loop, which is what this spike is for ----
run.rs:598:            // `EXIT`, bare or with a result: the spike had only the bare form
tests.rs:224:/// Step 2, and the reason the spike exists in the shape it does.
```

Why each remaining match stays: none is a flat-loop marker. `eval/tests.rs:838` names the file
`tests/spike.rs`. `lib.rs:78`, `run.rs:326`, `run.rs:598` and `tests.rs:224` refer to Phase 4's
Task 3 instruction-loop spike (the borrow-discipline proof `run_activation` is written down to),
a historical reference with a different meaning, outside this task's marker set.

What changed per marker: `**SPIKE, not for commit.** ` and `**SPIKE.** ` prefixes deleted, the
rest of each comment untouched (no re-wrap). Two phrases reworded: "Not a shape this spike drives"
-> "Not a shape the flat path drives" (`run/loops.rs`), "shape the spike drives" -> "shape the
flat path drives" (`ir/drive.rs`). The `// SPIKE:` comment explaining the run-time switch was
deleted with the switch (`static FLAT` and the `REXX_NO_FLAT` read), since it described only the
switch. `#[allow(clippy::too_many_arguments, reason = "spike")]` could not simply go (the function
has eight parameters and G2 runs clippy `-D warnings`); it keeps the allow with the reason
`run_loop_with_header`'s allow already gives. The `Fallback` path for loop kinds the flat path does
not drive (`loop_header_plan` answering `None`; `LoopKind::Simple`/`With`) is unchanged.

`REXX_NO_FLAT` has no remaining reader in `rust/` (`git grep`); it is still named in historical
records (`phase-4f-record.md:4978`, `found-not-fixed-register.md:22`,
`2026-09-08-do-over-loop-rooting/diagnosis-and-fix.md:16`,
`specs/2026-09-08-loop-decomposition-option.md:14,69,80`), left as records of their time.

Pre-commit checks: `cargo fmt --all --check` exit 0; `cargo build -q -p rexx-exec` exit 0.

## Gates

Script: `scratchpad/p6-t1/gates.sh` (copy of `surface-6/gates.sh`, `S` pointed at `p6-t1`), run
at 4b694fe7c. `status.txt` verbatim:

```
4b694fe7c1cfe332a4ec8875e9c9f604ae3ebf63
started 2026-09-29T14:09:57+02:00
load at start 3.69 3.39 2.95 7/2542 1821396
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 14.68 11.73 6.44 6/2522 1825090 2026-09-29T14:13:04+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 6.05 8.29 6.71 5/2517 1953646
G5 debug build (test --no-run) exit 0
load G6 7.47 8.49 6.81 3/2513 1958082 2026-09-29T14:19:32+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 3.83 7.01 6.93 6/2511 2086659
4b694fe7c1cfe332a4ec8875e9c9f604ae3ebf63
finished 2026-09-29T14:25:53+02:00
```

HEAD matched at start and finish, and `git status --short` printed nothing. `Compiling lines: 0`
in G4/G6 is expected: G3/G5 built the same HEAD just before. There are no `FAILED` result lines in
`g4-test-release.txt` or `g6-test-debug.txt`. No `target*` dirs were left under `scratchpad/p6-t1/`
(G2 removes its own).

## Concerns

1. D-U2's enforcement shape (the island `Send` only inside a wrapper `island.rs` hands out by
   taking the baton) is this block's proposal, not the spec's; the block says so and defers it to
   S4's plan.
2. D-U3's file name `rexx-exec/src/signal.rs` is proposed; the spec names no file.
3. The roadmap's unsafe section still says "Expect exactly two candidates over the whole project",
   stale since `bytes.rs`/`frame.rs` and now D-U2/D-U3; not edited (outside this task's list).
4. `frame.rs`'s 2026-09-23 grant (and `bytes.rs`'s) had no Section 1 block before; D-U4 is the first
   roadmap record touching `frame.rs`.

## Fix round 1

Commit 9d863ccc5 (docs only, `docs/superpowers/plans/2026-07-27-rust-rewrite.md`; no gate run, per
the controller).

- **Important 1** (Global Constraints, "Expect exactly two candidates"): replaced with a sentence
  that names the granted sites by record: the modules the D-U blocks name, plus `rexx-core`'s
  `bytes.rs` and `frame.rs` granted at their `#[allow(unsafe_code)]` (`frame.rs` re-granted by
  D-U4), with `crates/rexx-core/tests/unsafe_sites.rs` (checked: exists) as the in-tree enumeration.
  No set size. "*candidate* ... not an exemption" and the D5 sentence are kept.
- **Important 2** (D-U2): "`ObjRef` is `!Send` and `!Sync`" -> "`ObjRef` becomes `!Send` and
  `!Sync` (spec section 5) before this grant lands" (today `pub struct ObjRef(u64)`,
  `rexx-core/src/handle.rs:47`).
- **Minor 1**: "The same ruling adds three more" -> "The same rulings continue".
- **Minor 2**: D3's Evidence paragraph gains "(Resolved 2026-09-29 by ruling 2 of the dated note
  above: both, with nightly admitted for the TSan gate only.)". The line's existing em-dash is
  pre-existing text, not added.
- **Minor 3**: D-U3's Question adds that the oracle checks only SIGHUP's previous action
  (`:136-139`) and this design checks each, a licensed divergence (spec section 4).
- **Minor 4**: both module names are marked the same way, "named here; the spec leaves it to the
  plan", in the D-U2 and D-U3 blocks and in both index rows.
