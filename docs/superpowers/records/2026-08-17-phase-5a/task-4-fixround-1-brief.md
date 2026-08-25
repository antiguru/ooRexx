# Task 4, fix round 1 — rulings on the review

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-4-review.md`. **Spec compliance: APPROVE** — every
clause met, mutations 2 and 3 run and recorded, mutation 1 correctly **named** against Task 7 and not
faked, the exclusivity/exhaustiveness requirement holds, `loud` is a column and never a verdict,
`Raw` reaches every row, no oracle bytes are typed into the table, and the four `.orx` shapes are
derived by a scan that reddens structurally if any of them reaches nothing.

**Task quality: REWORK.** Two medium, seven low. **Both mediums are in the shared harness Task 5
inherits**, and both are the shape this plan exists to sweep for: a structural failure silently
absorbed, and a verdict input that can be switched off with nothing to notice. Fixing them now costs
one round; fixing them after Task 5 costs two tables.

---

## M1 — a probe that cannot be canonicalised leaves the table with no `Structural` at all.

`gate_table_d.rs:338-341`'s `else { continue; }` carries the comment *"Already reported above as a
missing probe"*. **That is true only when the probe's *name* is absent from the directory listing** —
and `fs::read_dir` lists a symlink whatever its target is. So a probe whose name is present but which
cannot be canonicalised passes the set check, fails here, and **the row leaves the table through the
one path in the file with no channel that is red in every mode.**

The reviewer measured it in a scratch tree: replacing a probe with a symlink to a nonexistent path
gives `78 rows`, **no structural failure, exit 0** — the exact outcome the missing-probe message
promises cannot happen. Worse, doing it to a 5a row that does not agree gives `5a: 35 rows, 9 not yet
agree`, **gated count down by one with nothing red**, which is the shape of a silent improvement.

**Concrete consequence, and it is the reason this is not Low: Task 24 closes Phase 5a when the gated
count reaches zero. A row whose probe is a dangling symlink contributes zero.** The close criterion
can be satisfied by removing evidence, and neither mode says so.

**Ruled: push a `Structural` in the `Err` arm**, naming the row, the path and the io error. Do not
rely on the earlier check — **it answers a different question**, and saying so in the comment is half
the fix.

## M2 — the verdict function recovers its channels by matching the producer's display strings.

`gate_tables/mod.rs:233-240` reads `diffs.contains(&"exit code")` and friends out of a
`Vec<&'static str>` whose documented purpose is *"which of the three observable channels disagree"* —
**a display list**. This is the only consumer in the workspace that reads those labels' content;
every other checks emptiness or prints them. So a rename in the producer turns one boolean of the
verdict function's input **permanently false**, and every reader that might have noticed is looking
at something else.

Measured, and this is the line that decides it: with the crate's exit status wrong on **every** row
*and* `"exit code"` renamed to `"exit status"` in `oracle.rs`, table D reports counts **byte-identical
to the unmutated tree** — and `REXX_CORPUS_GATE=1 cargo test --release --workspace` with the rename
alone **exits 0 with the corpus at 106 of 106.**

**Ruled: ask the producer for a typed answer** — return the `Descriptors` triple or an enum set —
rather than re-parsing its prose. If that is genuinely out of reach in this task, the fallback is an
assertion inside `compare_raw` that **every label the producer returned is one this function knows**,
so a fourth or renamed label panics instead of vanishing. `contains` is a positive test and an
unrecognised label is simply not seen; that asymmetry is the defect.

**Make the fix fire**: rename a label in a scratch copy and confirm the harness now reddens.

## L1 — **Ruled: fix.** The shared two-engine run omits the `chunks_refused == 0` check `ir_dual.rs`
carries. Task 5 inherits this path too.

## L2 — **Ruled: fix.** Two of the three `FORM` probes read back a value that is already the default,
so **the readback cannot fail**. A probe that cannot distinguish success from doing nothing is
decoration; give each a value the default is not.

## L3 — **Ruled: fix the comment. The plan was the source and I have already corrected it** at
`45a10c7fe`. The workspace lint is **`unsafe_code = "deny"`** (`rust/Cargo.toml:31`), not `forbid`;
Moritz made that change on 2026-08-20 **precisely because `forbid` cannot be overridden by an inner
`#[allow]`, which is what an approved site needs.** So *"rules out"* is the one thing `deny` was
chosen not to do. The design decision in your comment is right; only its stated reason is false. The
live bar is `rust/CLAUDE.md`'s per-site exception process, with the granted set asserted by
`crates/rexx-core/tests/unsafe_sites.rs`. **`corpus.rs:505` and `support/oracle.rs:46` carry the same
stale sentence** — fix those two as well, since you are in the file and they are the same claim.

## L4 — **Ruled: fix.** A mutable set's size in a new comment. Name the set.

## L5 — **Ruled: fix.** The report's account of the identical-probe pairs is short by two, and the two
it misses are L2's.

## L6 — **Ruled: fix by stating it.** *"The live instances"* understates how much of table D is
acceptance-only. Say what the table does and does not exercise, in the shape the plan asks for
elsewhere: what a check could not see, said rather than implied.

## L7 — **Ruled: carry it forward, do not decide it here.** The `::ROUTINE EXTERNAL` row pins a phase
boundary **Task 22 is being asked to draw**. Record the row's dependency on that decision where Task
22's author will meet it — the row's own evidence column or the table's header — rather than settling
a boundary this task does not own.

---

## Verification

The five gate commands, each with **its own** exit status; corpus **106 of 106**.

**Report the 5a non-`agree` count again after the fixes, with its predicate** — it is 10 of 36 today
and M1's fix may change the row population. If it moves, say why.

`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` is now **live and expected to exit non-zero** — I measured it
at 101 against your commit while the unphased run exits 0. **That red is the design, not a
regression**: `CLOSED_PHASES` is empty until Task 24. Report the gated count; do not try to make it
zero.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-4-report.md` under "Fix round 1".

**Return only:** status, commit SHAs, one line per finding, the non-`agree` count with its predicate,
whether M1's and M2's fixes were seen firing, and anything you could not close.
