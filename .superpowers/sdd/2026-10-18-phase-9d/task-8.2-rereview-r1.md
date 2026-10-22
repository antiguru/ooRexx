# Task 8.2 re-review, fix round 1

Range `9ce1d8384..ece5b7dc9`; code in `77fded0db`, records in `ece5b7dc9` (`5c028a8b1`, `7130649d8`
are controller records, out of scope). Scratch worktree under `t82-rereview-scratch/wt`, `--release`,
`CARGO_INCREMENTAL=0`. Binaries built there: head `ece5b7dc9`, round 0 `9ce1d8384`, base `35db1719c`,
mutant A (head with `install.rs:537` as `if false && self.library_programs.contains(&program)`),
mutant C (head with `run/loops.rs:2021-2023`, the flat UNTIL path's END store, deleted). After each
mutant was built the edit was reversed and `git diff --stat` came out empty. Every comparison runs the
oracle (standard wrapper) and the crate (`memcap 2G timeout 60`) on a copy in a fresh directory, with
stdout, stderr and rc compared separately and the directory path normalised.

**Overall: APPROVED.** Critical 0, Important 0, Minor 3. All eight items are ADDRESSED.

## Item verdicts

| item | verdict | evidence |
|---|---|---|
| I1 | ADDRESSED | Deviation candidate 2 (report `:125-142`) now states the tail-less PROPAGATE observable and the INTERPRET one, giving both answers. The unit test's program, run as a file: oracle `h 0 The NIL object` / `back RESULT` / `oh 0 The NIL object` / `interpreted from-interpret`; head `h 1 w1` / `back w1` / `oh 1 from-interpret` / `interpreted from-interpret`. That is the asserted string byte for byte (`run/tests/conditions.rs:1149`). `a_raise_tail_value_is_the_condition_objects_result` passes. |
| I2 | ADDRESSED | `stream_boundary_arity`: head `out=same err=same rc=same`; mutant A `out=same err=DIFF rc=same`, `Error 88 running stream_boundary_arity.rex:  Invalid argument.` against the oracle's `Error 88 running REXX:`. Under mutant A, `stream_boundary_report` still matches the oracle on all three, as the review found, so the new program is the only witness. |
| I3 | ADDRESSED | The report (`:98-110`, `:152-153`, `:163-171`) describes a race and gives both measured frequencies. My 20 sequential oracle runs, unpinned, from fresh directories: 19 x rc 214 `ec5bbbd3` and 1 x rc 214 `bbf77a8a`, both with line 3 in both reports (two interleavings). That is 20 of 20 line 3, which matches the implementer's 40 of 40 and not the review's 12/8 split. The same 20 runs under `taskset -c 0`: 20 x rc 214 `07abf1c9`, line 6 then line 3, byte-identical to head (5 of 5 head runs `07abf1c9`). The race is real. Which way it goes depends on scheduling; see Minor 2. |
| M1 | ADDRESSED | callgrind, `-B 0.5 -p "churn startup"`, pin / parent `35db1719c` / head / r0 `9ce1d8384`: churn whole 631334063 / 632309627 / 632312516 / 632509032, -libc 378751556 / 379155821 / 379155724 / 379357074. Head minus parent is +0.0005% whole and -0.00003% with libc subtracted (the report has +0.0012 / 0.0000). Round 0 was +0.0316 / +0.0531. startup: head minus parent +0.0026% / -0.0002%. `dispatch_seam` 6 passed. New-defect hunt below. |
| M2 | ADDRESSED | The INTERPRET RESULT shape is asserted by the unit test (I1 row). The `raise_result_and_sender.rex:56` mask is unchanged and covers only non-SYNTAX conditions that were not propagated, the ruled difference. Head is identical to the oracle on that file. |
| M3 | ADDRESSED | `tests/signals.rs`: the "long builtin" doc block now sits directly above `sigint_halts_after_a_long_builtin`, and `sigint_pending_at_a_syntax_trap_is_raised_at_the_label` carries only its own block. |
| M4 | ADDRESSED | `grep -n "Task [0-9]"` over `run/tests/conditions.rs`, `tests/signals.rs`, `dispatch.rs`, `dispatch/hash.rs`, `dispatch/hash/stem.rs`: no match. |
| M5 | ADDRESSED | `git diff 35db1719c ece5b7dc9 -- run/loops.rs` has three hunks only, at `:2018`, `:2168` and `:2209` (the flat paths). No hunk touches `run_repeating` (`:1452`, `:1516`). The new UNTIL witness is red under mutant C: `until_retest_blame` gives `out=DIFF`, `g 1     11 *-*         nop` against `g 1     11 *-*         end`, and `g 2     14 *-*         if i = 1` against `14 *-*         end`. Head matches the oracle on all three. `while_retest_blame` stays identical under mutant C, so each witness covers its own store. |

Tests at head, from `rust/` in the scratch worktree:

| command | result |
|---|---|
| `cargo test -j 4 --release --no-fail-fast -p rexx-exec --test dispatch_seam --test signals` | 0; dispatch_seam 6 passed, signals 42 passed |
| `cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib` | 0; 1173 passed |

Two failures in the first runs came from my setup, not the code. signals failed 19 tests because `CARGO_TARGET_DIR` held a `..` and the test's `PROGRAM` substitution no longer matched; it passed after a canonical path and a rebuild. `--lib` failed 67 tests because the scratch worktree had no `build/lib`; it passed after a symlink to the main worktree's `build`.

## New-defect hunt: the frameless UNKNOWN rewrite

The rewrite stops calling `seam::clear` for the frameless UNKNOWNs and runs each one's work directly:
`hash_unknown`, `stem_unknown`, `reference_unknown`, selected by scope (`dispatch.rs:3856-3872`). This
is the shape of the oracle's `processUnknown`. `RexxObject::processUnknown` (`ObjectClass.cpp:1002-1021`)
calls `method_save->run` with no `isProtected` check and no `processProtectedMethod`. `StemClass::processUnknown`
(`StemClass.cpp:277-281`) does `value->messageSend` and never touches the UNKNOWN method at all. So
skipping the seam's security question copies the oracle. It is not a lost check.

* `seam::clear` keeps two pieces of state: the protected question, and consuming `unasked_run`. `unasked_run`
  is set only around `begin_invoke` in `invoke_armed_executable` (`object_protocol.rs:952-954`) and
  is cleared right after, so the frameless path never sees it set.
* `frameless_unknown` now accepts any `NativeEntry` rather than `NativeBody::Run`. Every UNKNOWN row
  (`Stem` `stem.rs:823`, `Directory` `dispatch.rs:707`, `StringTable` `:933`, `VariableReference` `:964`)
  is a `Run`, so no `Begin` or `Leveled` UNKNOWN changes path. Choosing the function by scope rather
  than by the found row matters only if a native UNKNOWN is installed under one of those scopes, and
  `.stem~define('UNKNOWN', ...)` is refused with 98.985 on both sides.
* Probes, all identical to the oracle at head and at round 0 (out/err/rc): a stem forwarding a
  failing `substr` (traceback included), a `.Stem~new` object's NOSUCH, a stem holding an Array, a
  Directory entry and `setMethod` entry, a Directory `setMethod('UNKNOWN')` that raises, StringTable
  entries and `setMethod`, a Directory subclass, a VariableReference forward that succeeds and one
  that fails (traceback), TRACE R over a stem forward, and a stem holding a Directory.
* The rewrite fixes an observable (Minor 1). Probe `u4.rex`:

      m = .stem~method('UNKNOWN'); m~setProtected
      .context~package~setSecurityManager(.sm~new)
      call rt
      exit
      ::routine rt
      s. = 'abc'
      say 'len' s.~length
      say 'direct' s.~unknown('LENGTH', .array~new)
      o = .p~new
      say 'prot' o~pm
      ...
      ::class sm / ::method unknown: say 'sm' name args[1]~name; return 0
      ::class p / ::method pm protected: return 'pm-ran'

      oracle: len 3 / sm METHOD UNKNOWN / direct 3 / sm METHOD PM / prot pm-ran
      head:   same, out=same err=same rc=same
      r0:     sm METHOD UNKNOWN / len 3 / ...      (asks the manager for the frameless forward)
      base:   sm METHOD UNKNOWN / len 3 / ...      (the full UNKNOWN send asks too)

## Issues

### Critical

None.

### Important

None.

### Minor

1. **The rewrite's security-manager observable has no witness.** Head now matches the oracle on
   `u4.rex` (above). Base and round 0 both ask the manager `METHOD UNKNOWN` before `len 3`, and nothing
   in the tree checks this behaviour. `dispatch_seam` counts call sites, not what they do. Add `u4.rex`
   as a corpus program. It is red at `35db1719c` and at `9ce1d8384`, and
   identical at head.
2. **The oracle-defects entry `message-start-report-position-race`
   (`docs/superpowers/records/oracle-defects/defects.json`, controller record from `5c028a8b1`) says
   "the Task 8.2 report's '20 of 20 line 3' did not reproduce".** It does reproduce: 20 of 20 here
   unpinned, and 40 of 40 in `t8.2/r1/message-start/`. Which way the race goes depends on
   scheduling, and pinning the oracle makes it deterministic. `taskset -c 0` gives 20 of 20 line 6
   then line 3 (`07abf1c9`, the crate's bytes). Unpinned, this machine gives line 3 twice. The entry
   should give both, plus the `taskset -c 0` repro of the line-6 branch.
3. **Queue candidate (predates the task):** a security manager set in the middle of an activation is
   seen by an internal call on the crate side and not on the oracle's. This is `u4.rex` with
   `call r` / `r:` in place of the `::routine`. Oracle `len 3` / `direct 3` / `prot pm-ran`, with no
   manager call at all, because the internal call inherits the caller's settings, which were captured
   before `setSecurityManager`. Head, round 0 and base all call the manager for `UNKNOWN` and `PM`.
   This is not in `.superpowers/sdd/queued/2026-10-19-security-manager-protected-primitive.md` (closed),
   and I found no other queued item for it.

## Commands

Probes and the runner were in `t82-rereview-scratch/` (deleted after this review; the lines that decide
each finding are quoted above). The runner copied each program into a fresh directory, ran the oracle
with the standard wrapper and the crate under `memcap 2G timeout 60`, then compared stdout, stderr and
rc after removing the directory path.

| command | result |
|---|---|
| `bench-programs/callgrind.sh -B 0.5 -p "churn startup" pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run parent=<35db1719c> head=<ece5b7dc9> r0=<9ce1d8384>` | exit 0; table in M1 |
| 20 x oracle `msgstart.rex` sequential, then 20 x under `taskset -c 0`; 5 x head | tallies in I3 |
