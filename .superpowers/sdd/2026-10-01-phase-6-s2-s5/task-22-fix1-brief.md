# Task 22 fix round 1: condensed from three parallel reviews

Sources (read the finding text there for detail):
- `task-22-review-timer.md` (T), `task-22-review-tsan.md` (S), `task-22-review-gate.md` (G).

## Code

1. **T1 (Medium) macOS build.** Put the timerfd path in `sync.rs` under
   `#[cfg(any(target_os = "linux", target_os = "android"))]`; other unix targets use the `poll`
   timeout (rounded up to whole ms). Show `cargo check --target aarch64-apple-darwin -p rexx-exec`
   has only the pre-existing E0308 at `rexxutil.rs:154`.
2. **T2.** Delete the `sync.rs:84-86` timer-slack sentence. **T4** (info): make the doc say a wake,
   the deadline, or an interrupted poll returns, the caller recomputes.
3. **T3.** Fix the `rexx-exec/Cargo.toml:61-68` rustix comment for all four features (`event`, `time`
   are the timer thread's poll and Linux timerfd).
4. **S1.** Replace the `hold_buffer` file handshake (`rexx-api/src/load.rs:926-928`, `lent.rs` HELD/DONE
   + `HOLDBUFFER` file) with one TSan sees (e.g. a `static` Mutex/Condvar the activity sets through a
   test routine), and delete the `hold_buffer` suppression from `rust/tsan.supp`. Show the TSan run
   clean without it.
5. **S2.** Skip the depth tests by exact path (`--exact --skip <full::path>`) so the three tests the
   substring caught (S2 lists them) run under TSan again.
6. **S3.** Add a bounded-depth (about 25 levels) pool-callback recursion test like the reviewer's probe
   (S3), asserting output, `exits == 1`, `takes == 1`, so TSan covers the nest every gate.

Each code fix: P51 bar (fmt, clippy -D warnings, `cargo test --workspace --release`), loom once if
`sync.rs`'s modelled wait changes, and the recorded TSan command rerun once at the end, clean.

## Gate record (`docs/superpowers/plans/phase-6-gate.md`, `phase-6-pinning.md`, `rust/tsan.supp`)

7. **G1.** Rerun the blocking-operations command at HEAD, replace the output block, classify every new
   site (G1 lists them). No line count in prose; the block is the artifact.
8. **G2, G3.** Correct the `ADDRESS` row per P64 and split `input.rs` out per P69 (G gives the text).
9. **G5.** Queued list: add `2026-10-05-stdin-chars-after-drain`; drop
   `2026-10-03-interpret-translation-error-traceback` (S3's); label `send-site-cache-self-customization`
   a design note or drop it.
10. **G6.** Figures without a source: the Task 22 scratch was deleted, so re-run the overshoot probe
    and the `b8ec39593`-era row run at HEAD, copy probe scripts and logs under
    `.superpowers/sdd/2026-10-01-phase-6-s2-s5/s4-close-evidence/` (git add -f), and cite them; same for
    the pinning `measured::` log and its diff command, and the TSan runs (re-run it once, cite that).
    Drop any figure you cannot re-derive. Criterion 2 / outer_context: cite G6 lines too.
11. **G7.** No counts in prose (`123 lines`, `18 measured:: tests`, `1127 in all`) unless quoted from a
    cited log beside its command.
12. **G8.** Enumerate the TSan exclusions by exact name with one reason each (after S2), and add a
    sentence that the deep pool-callback path runs bounded (S3), not at depth.
13. **G9.** Nits (a) G4/G6 cell attribution, (b) P67 "holds for one interpreter per process", (c)
    `tsan.supp` heading reference.
14. **Leave the `LEAD:` G4 lines (G4 finding) alone**: the lead fills them from
    `bg/75f8aeda8/status.txt`. Repointing criteria 1, 2 and outer_context from `52b038a80` logs to
    `75f8aeda8` logs is also the lead's.
