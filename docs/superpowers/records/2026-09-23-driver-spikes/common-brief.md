# Driver-cost spikes, round 1: rules common to every spike

You are one of several spikes run in parallel, each trying a structurally
different idea against the same measured problem. Spikes are exploration: a
spike that finds nothing, measured cleanly, is a full result. A spike that
reports a win it has not controlled for is worse than no spike.

## The problem, measured (do not re-derive; cite)

Records under `docs/superpowers/records/2026-09-20-performance-items/`:

* Ours 1,014.6 retired instructions per clause on `rexxcps` against the C++
  oracle's 534.4 (same 20,000,000 clauses). Interpretation machinery is +305.67
  per clause, 63.7% of the gap (`2026-09-22-category-rollup.md`).
* The driver is `Interp::run_ops_from<const GRANTING: bool>` in
  `rust/crates/rexx-exec/src/ir/drive.rs`: an outer op loop and an inner
  region walk (`for region_op in ops {`). `run_ops_from::<true>` is 15,087
  bytes, 2,894 instructions, 42.9% ever executed on `rexxcps`, with a 1,416-byte
  frame, and 37.6% of its line-0 cost is spill/reload
  (`2026-09-22-driver-frame-pressure.md`).
* **Adding four op arms that never fire cost `rexxcps` +2.32%**, and a four-op
  difference moved `varlookup` by 342,005,773 instructions
  (`2026-09-22-store-fusion-report.md`). The cost lives in the register
  allocator's per-value choices across the whole function. Frame size is not
  an instrument (`2026-09-22-frame-instrument-falsified.md`).
* The register file is `RootSet::temps: Vec<ObjRef>`, addressed by
  `temp_at`/`set_temp` (`rust/crates/rexx-core/src/roots.rs`), each with an
  `assert!` bound. It costs 28.9 instructions per clause (2.85%). Bounds checks
  in the whole binary are 0.99% of `rexxcps` dynamically
  (`2026-09-21-bounds-check-share.md`).
* A per-clause conditional in the `Op::Clause` arm costs about 0.5% on
  `rexxcps` whatever it does (`README.md` in that directory).

## Consequence for every spike: the noise floor

Any edit to `run_ops_from` moves instruction counts by up to about 2.5% on
`rexxcps` through register allocation alone, independent of what the edit
does. **A result under 2.5% on `rexxcps` from a driver edit is not evidence
until a control separates it** (for example: the same restructuring with the
new behaviour disabled, or the new code present but unreachable). Say which
control you ran, or say the result is uncontrolled.

## Where you work

* You run in your own git worktree, based on `0ba0f3876` of branch
  `plan/rust-rewrite`. Create branch `spike/<your-spike-name>` there and commit
  to it. **Never** commit to, merge into, rebase, or push `plan/rust-rewrite`;
  never push anything anywhere.
* Your scratch directory is
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/spikes/<your-spike-name>/`.
  It is shared with other spikes' directories next to it: never write outside
  your own subdirectory, never use generic names at the `spikes/` level.
* Own `CARGO_TARGET_DIR` per build, inside your scratch directory. Never use
  another spike's target directory or `rust/target` of the main checkout.
* Do not dispatch subagents.

## Standing rules (verbatim in effect)

* The C++ tree at repo root, `samples/`, `build/`, `ootest/`, `oodocs/`,
  `testbinaries/` are read-only; `api/` headers frozen.
* `unsafe` only in `rust/crates/rexx-api/src/ffi.rs`, `src/load.rs` and
  `rexx-core/src/bytes.rs`, each block with a `SAFETY:` note -- **unless your
  own brief grants a named exception**, which then applies to that spike
  branch only. No `unsafe` in `tests/`.
* No new dependencies.
* No process-global state from interpreter code: no `std::env::set_var`,
  `set_current_dir`, no writes to fd 0/1/2. (Setting `RUSTFLAGS` or other
  environment for a *build command* is fine.)
* Never `git add -A`; never amend; never `git reset --hard`; never
  `git checkout -- <path>` on an edited file; never bare `git stash`.
  Commit messages via a file with `-F`, ending with
  `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` and
  `Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD`.
* Comments minimal; a comment never states the size of a set; no em-dashes.
* `memcap LIMIT CMD` for memory caps (never `systemd-run`, never `sudo`).
  Build outside the cap, test under `memcap 8G`.
* `grep` is a ugrep wrapper that skips binary files: use `/bin/grep -a` for
  any count.
* No `rm` with a star glob.
* Oracle runs, if you need any: `( ulimit -v 1048576;
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
  /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` from a fresh empty
  directory; read `rust/corpus/oracle-crashes.txt` first and never run its
  entries.

## Measurement contract

1. **Before editing anything**, build the base in your worktree
   (`cargo build --release -p rexx-exec --bin rexx-run` from `rust/`) and copy the binary to
   `<scratch>/bin/base-rexx-run`. Record its `.text` hash:
   `objcopy -O binary --only-section=.text BIN OUT && sha256sum OUT`. Other
   spikes do the same, so the base hashes cross-check each other.
2. **Write your prediction into the report file before you measure**, with a
   number per axis and the reasoning. A wrong prediction is useful; a missing
   one makes the result unreadable.
3. Axes: `rust/bench-rexxcps/rexxcps.rex` (the pinned copy), and from
   `rust/bench-programs/`: `varlookup.rex`, `arith.rex`, `emptyloop.rex`,
   `dispatch.rex`, `compound.rex`. **Always report `rexxcps`.**
4. Instrument: `valgrind --tool=callgrind` `summary:` minus everything
   attributed to `libc.so.6` and `ld-linux` (the `arith` spread is all
   `_int_malloc`). Two rounds per binary, base and head interleaved. Run from a
   fresh empty directory.
5. Confirm each measured binary is the build you think it is by `.text` hash.
   A clean `git status` says nothing about `target/`.
6. Correctness floor for a spike: `cargo test -p rexx-exec --release` green,
   and `corpus_differential` 604 of 604 STRICT under
   `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus corpus_differential`
   (the test prints the matching count; quote that line).
   Full six-gate runs are not required for a spike.

## Report

`<scratch>/report.md`, written as you go, prediction first. Also commit a copy
to your spike branch at `docs/superpowers/records/2026-09-23-driver-spikes/<your-spike-name>.md`.
Your final reply truncates at about 8 KB: lead with the table
(axis, base ex-libc, head ex-libc, delta %), then the control and what it
showed, then your one-sentence verdict (sticks / does not stick / inconclusive
and why), then concerns. State the spike branch name and head commit.
