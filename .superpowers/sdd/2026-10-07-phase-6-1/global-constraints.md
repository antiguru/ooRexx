## Global Constraints

* **Behaviour.** The corpus differential stays identical except where a task names the programs it
  changes (a refusal that becomes an answer, a wrong answer that becomes the oracle's); each is
  witnessed against the oracle. New corpus witnesses go in a new list `rust/corpus/phase-6-1.txt`,
  registered in Task 1 beside `phase-6.txt` in every harness list that names it (`corpus.rs`,
  `coverage.rs`, `collect_stress.rs`, `trace_oracle.rs`, `ir_recorded.rs`; find them with
  `grep -rln '"phase-6.txt"' rust/crates`). Concurrent behaviour is tested on the crate alone.
* **`unsafe`** only in `rexx-api/src/ffi.rs`, `rexx-api/src/load.rs`, `rexx-core/src/bytes.rs`,
  `rexx-core/src/frame.rs`, `rexx-exec/src/island.rs`, `rexx-exec/src/signal.rs`. No new dependency
  (the PRNG is in-crate). No `Op::Generic`; `Op::Clause` at discriminant 0; `size_of::<Op>() == 16`;
  `size_of::<Activation>() == 512` (R4).
* **Process-global state:** none beyond the live-interpreter registry, the timer thread and the P70
  test statics.
* **Performance (spec section 5).** Callgrind with glibc and ld-linux excluded
  (`rust/bench-programs/callgrind.sh`), each build in its own target directory with a `Compiling` line,
  plus wall clock (five interleaved runs, median, ±4% noise). Running totals against the 6.1 base
  `04b5f0ff8` (or the commit Task 1 starts from, recorded in the ledger) at most +0.5% beyond the noise
  band on every program; D6's branch, if paid (R7), is accounted beside that budget. Measured at the
  close of Tasks 1, 2, 5, 6 and 9 on the programs each names, cumulatively at Task 12. Over budget:
  three rounds, then stop for Moritz's ruling.
* **Oracle runs**: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
  timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` from a fresh empty directory;
  stdout, stderr and status compared separately; `rust/corpus/oracle-crashes.txt` read first (entry 10b:
  one SETLOCAL restore per process survives, termination restores included); no named semaphores and
  nothing that changes rxapi's persistent state. Concurrent programs: at least 5 runs per engine, counts
  stated.
* **Read-only**: the C++ tree, `samples/`, `build/`, `ootest/`, `oodocs/`, `testbinaries/`, `api/`.
* **Prose**: comments and docs state decisions, never a mechanism that does not exist yet, never a
  set's size, no em-dashes; a false sentence is deleted. Extent claims are derived with the command
  committed. Every witness's stdout is read to confirm each path it claims prints.
* **Shell**: no `bash -c`/`sh -c` wrappers; `rm` only with literal absolute paths, no globs (a loop that does `rm -rf $W/$n` stalls the task on a manual approval: make a fresh directory per run instead of deleting); never
  `pkill -f`; builds under `memcap 8G` with `-j 4`. **Every process that runs this crate's interpreter or its tests runs under `memcap`** (8G for a test binary, 2G for a single `rexx-run` probe), including scripts that loop over programs or ooTest groups: the collector triggers on slots, so a loop of large dead strings (ooTest Class TEST_SUBCLASSES_GC and others) grows ~1 GB/s and an uncapped run OOM-killed the whole session on 2026-10-08. Put `timeout` inside `memcap` (`memcap 2G timeout 10 rexx-run ...`), not outside: a timeout outside leaves the interpreter and any blocked child alive in the cgroup.
* **Per-task check**: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, plain
  `memcap 8G cargo test -j 4 --workspace --no-fail-fast` (debug), and `REXX_CORPUS_GATE=1 memcap 8G
  cargo test -j 4 -p rexx-exec --test corpus --test ir_recorded_oracle` (the witnesses need the strict
  mode). `whole_groups` (`REXX_CORPUS_GATE=1 ... --release --test concurrency_tests whole_groups`) runs
  at the close of Tasks 3 and 5 (they rewrite its expectation lines), Task 10 and Task 12.
* **Refusal tables**: a task that adds, deletes or relabels a `Loud` constructor re-derives
  `rust/corpus/refusal-sites.tsv` in its last step with `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p
  rexx-exec --test refusal_sites`, and from Task 7 on keeps `refusal-dispositions.tsv` green in the same
  commit.
* **A defect found and fixed** in any task lands with a crate test using explicit switch points (the
  `AtClause` style) and an assertion that the test reaches the defect's site. Full gates
  (`bggates.sh`, copied into this plan's SDD workspace from `2026-10-01-phase-6-s2-s5/p6-gates/` with
  `W` and `-j 4` adjusted) only at Task 12. Commit before any long run; the tree stays frozen until its
  status file says finished.


## Working rules for implementers

* Work in `/home/moritz/dev/repos/ooRexx-rust-rewrite` on branch `plan/rust-rewrite`, code under `rust/`.
  Commit with `git commit -F <file>`; the message ends with the two lines
  `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` and
  `Claude-Session: https://claude.ai/code/session_01GGimEAe3NWM7Rh22JoLorg`. Never `git add -A`,
  amend, force, reset, checkout, restore or stash; add files by name.
* Scratch under `/tmp/claude-1000/p61/t<N>/` (never the repository); oracle probes from a fresh
  empty directory there.
* Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md`; evidence:
  `.superpowers/sdd/2026-10-07-phase-6-1/scout-*-report.md` (probe texts in their appendices).
* No subagents. Bounded foreground waits only.
