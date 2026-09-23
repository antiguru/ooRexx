# Land the frame arena, sound without leaking

Read `worktree-setup.md` beside this file first. Branch: `perf/frame-arena`.

## Decision already made (Moritz, 2026-09-23)

`unsafe` is granted for **one new module, `rust/crates/rexx-core/src/frame.rs`**,
for the register frame arena measured by round 1's `frame-arena` spike
(`docs/superpowers/records/2026-09-23-driver-spikes/frame-arena.md`; code on
branch `spike/frame-arena`, final code at `9c5502476`, checked control at
`8434023ce`). Record the grant the way `rust/CLAUDE.md` (lines around 103-110)
says grants are recorded: the `#[allow(unsafe_code)]` opt-in, both lists in
`crates/rexx-core/tests/unsafe_sites.rs`, and a dated line in `rust/CLAUDE.md`
naming the site and Moritz. The bar in `rust/CLAUDE.md` still applies in
full: the invariant is stated at the site and **checkable by reading
`frame.rs` alone**; the safe alternative's cost is measured (the checked
control already is, re-measure it on your code); a test fails if the
invariant breaks.

This task **lands on `plan/rust-rewrite`** after review, so it is production
code, not a spike: full gates, minimal comments per `rust/CLAUDE.md`.

## What Moritz asked for, on top of the spike

1. **The block size is configurable.** Choose where it is configured (the
   interpreter's construction/`Invocation` path is the natural place; no
   environment variables read by the interpreter to set it unless that is how
   comparable knobs already work -- look first). Default to what round 1
   measured. State the valid range and what happens outside it (refuse at
   construction, not at first use). The spike's soundness argument was
   module-local: every frame starts at least 65,536 cells before its block's
   end and indexes are `u16`. A configurable size must keep a module-local
   argument: either the size has a floor that preserves it, or the argument
   changes to one `frame.rs` can check by itself (for example a frame never
   spans its block and `frame.rs` itself bounds-checks the index type it
   hands out). A frame larger than the configured block must still work
   (e.g. a dedicated block). Your call; say which and why.
2. **Lifetimes, not leaks.** The spike leaked every block for the life of the
   process because its `RegFrame` handle is `Copy` with no lifetime tie to
   the arena. Replace that: blocks are freed when the arena is dropped, and a
   frame handle **cannot outlive the arena, checked by the compiler** --
   prove it with a `compile_fail` doctest (a handle escaping its arena must
   not compile). The known obstacle: the driver runs with `&mut self` on
   `Interp`, which owns the roots, so a handle borrowing the arena conflicts
   with `&mut self`. Designs worth considering: the arena held outside the
   borrow the driver needs; a scoped/branded lifetime (`fn with_frame<R>(&self,
   n, f: impl for<'f> FnOnce(Frame<'f>) -> R)`); interior mutability in the
   block storage so a shared borrow suffices. Pick one, and **measure it**:
   a lifetime-based design that costs instructions is a finding, not a
   failure. If no lifetime design is free, report the cheapest sound one and
   its price beside the leaking spike's figure, and land the lifetime design
   only if it is within 0.2% of the spike on `rexxcps` -- otherwise stop and
   report before landing.
3. Keep the spike's fix for trapped clauses (the `temps` watermark around each
   chunk run; `a_trap_that_resumes_does_not_accumulate_temps_frames`), and
   look again at the per-call reserve/release cost that left `dispatch` flat.

## Measurement

Common contract, axes as there, base `b5dd351d6`, **plus** the spike's own
binaries as reference points if you can rebuild them by `.text` hash. Report
base, your head, your head with checked access (the safe alternative), all
interleaved. Round 1 found the checked/unchecked pair moved `emptyloop`
(which reads no registers) by -1.03%; report `emptyloop` and say what your
pair does there.

## Gates, before you report DONE

Commit first, tree frozen until finished, then per `rust/CLAUDE.md`: fmt,
clippy `--all-targets -D warnings` (a sub-second clippy against a warm
target is provisional: re-run cold), build `--all-targets` release outside the
cap, `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release
--no-fail-fast`, then the same in debug. Expected at base: 133 binaries,
2651 passed / 0 failed / 4 ignored release, 2652 / 0 / 4 debug (your new tests
add to those); `corpus_differential` 604 of 604 STRICT. Sum per-binary tallies
from the `test result:` lines by script. Quote every status.

Do **not** merge into `plan/rust-rewrite`; the controller reviews and merges.
