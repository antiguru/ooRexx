# Task 23 implementer brief (condensed from the S5 audit findings)

Requirements: `task-23-brief.md` (the plan's task text) plus what follows. The plan's Global
Constraints section (`docs/superpowers/plans/2026-10-01-phase-6-s2-s5.md`, read only that section)
binds every step. Spec: `docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md` section 9
criteria 4-7, sections 2.3-2.5, 5.

## Criteria 4 and 5: build from `s5-find-audit.md` (evidence `s5-evidence/audit/`)

1. **P71.** The committed inventory command is the audit's R1 pattern:
   `/bin/grep -a -rn 'unsafe impl\|thread::spawn\|thread::Builder\|thread::scope\|Arc<\|Mutex<\|Condvar\|Atomic\|thread_local!' rust/crates`.
   Run it at your final head, commit its output under
   `docs/superpowers/records/2026-10-01-phase-6-s2-s5/` (create the dir if absent), and make the
   criterion 4 table cover every hit group (the audit's table C is the starting point; re-check line
   numbers at your head).
2. **Static assertions A1-A4** exactly as the audit's section "What the implementer must build",
   each with its control where the audit gives one (A3's control: the same assertion on a `Send`
   type fails; show it once in your report, do not commit a failing control). Check clippy on A3.
3. **P72, A5.** Seal `Islanded`'s payloads: a crate-private `IslandPayload` trait implemented for the
   two instantiations (`NonNull<Interp>`, `(Box<OffBaton>, ThreadContext)`), and
   `unsafe impl<T: IslandPayload> Send for Islanded<T>`, with the SAFETY comment naming why each
   payload is sound. Show a third payload is a compile error (report only).
4. **P73.** Amend spec section 5's "refuses an object handle" sentence and section 2.5's
   non-test `thread_local!` sentence to what holds (R2, R3), citing P52 and P56. Minimal wording.
5. **P74.** The criterion 5 row states the `NativeState::Pointer(*mut c_void)` caveat in one clause.

## Criterion 6: sharing fraction (Task 23 Step 3)

A feature-gated counter in `rexx-exec` (feature name of your choice, off by default; zero cost when
off, which a `.text` hash of a release `rexx-run` with the feature off before and after shows). Each
object tagged with the last activity that resolved it; objects touched by more than one activity
counted. Run over the corpus and ooTest (`ootest/` is read-only; use the existing runners), record
the figure with its command and log under the records dir.

## Criterion 7: ping-pong benchmark (Task 23 Step 4)

Under `rust/bench-programs/`: message round trip, semaphore post/wait, GUARD WHEN handoff. Compare
against the oracle (oracle command below), recorded, not gated. Wall clock: quote the command and
machine load beside the figures; run each side interleaved, several times.

## Gate record

Start `## S5` in `docs/superpowers/plans/phase-6-gate.md` (after the S4 section) with subsections
for criteria 4, 5, 6, 7, each quoting its evidence and command. Do not touch the S4 section.

## Checks (P51)

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (and with your
feature on), `cargo test --workspace --release`, `cargo test -p rexx-core --doc`. No full gates.

## Rules

- unsafe only in ffi.rs, load.rs, bytes.rs, frame.rs, island.rs, signal.rs. No new deps beyond loom
  and libc (rustix features OK). No mutable globals beyond the registry, the timer thread and P70's
  test statics. No Op::Generic; Op::Clause stays discriminant 0; `size_of::<Op>() == 16`.
- Read-only: the C++ tree, samples/, build/, ootest/, oodocs/, testbinaries/, api/.
- Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty dir; no named semaphores; 30 runs before a claim about a concurrent program.
- Git: commit with `git commit -F <file>`, message ending with the two lines
  `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` and
  `Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD`. Add files by path; never
  `add -A`, amend, force, reset, checkout, restore, stash. No push.
- Shell: never `bash -c`/`sh -c` wrappers in tool commands (they block on approval); script files
  run as `bash /abs/path.sh`. rm only literal absolute paths, no globs. Never `pkill -f`. Long runs
  in the foreground or wait on a PID; never wait for a notification. Scratch and target dirs under
  `/tmp/claude-1000/p6-t23/`; `memcap 8G` on cargo test; delete scratch at the end.
- Prose: minimal, no em-dashes, no set sizes or counts in prose unless quoted from a cited log
  beside its command, nothing forward-looking.
- No subagents.

## Report

Write `task-23-report.md` (same dir) as you go. Return: status (DONE / DONE_WITH_CONCERNS / BLOCKED /
NEEDS_CONTEXT), commits, one-line test summary, concerns.
