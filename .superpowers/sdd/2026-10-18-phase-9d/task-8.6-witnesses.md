# Task 8.6 close: two Deviation witnesses (tests only)

Context: Phase 9d Task 8.6 of the ooRexx Rust rewrite (checkout /home/moritz/dev/repos/ooRexx-rust-rewrite,
crates under rust/). Moritz ruled two new Deviations; each needs a test pinning this crate's answer. No
interpreter code changes. Read rust/CLAUDE.md, then rust/crates/rexx-exec/tests/stdout_write_errors.rs
(its harness spawns the `rexx` binary).

1. A standard descriptor closed when `rexx` starts is open on /dev/null here (Rust's runtime reopens
   fds 0-2 before main); the oracle answers `1 ERROR:9 Bad file descriptor` to every write. Add a test to
   stdout_write_errors.rs that starts `rexx` with stdout closed (spawn `/bin/sh` with `exec "$REXX" prog
   >&-`, the binary path passed in the environment, so no unsafe pre_exec is needed) and asserts that
   `lineout('STDOUT', 'a')` answers `0` and `stream('STDOUT', 'd')` `READY:` (report what it answers on
   stderr). Name: a_standard_output_closed_at_start_is_dev_null.
2. APPEND keeps O_APPEND on the write-only open: the shape of
   docs/superpowers/records/2026-10-18-phase-9d/t8.6/r2/probes/a3.rex (a file with mode 222 holding `y`,
   `open('write append')`, lineout w1, a command `echo wcmd >> file`, lineout w2, close) leaves the file
   `y|wcmd|w1|w2|` here; the oracle gives `y|w1|w2|`. Add the test beside the existing APPEND witnesses
   (find them: corpus/lang/stream_append_held.rex is a corpus program; this one cannot be a corpus program
   because it differs from the oracle, so make it a Rust test in rust/crates/rexx-exec/tests/, in an
   existing stream test file if one fits). Run the file read back after chmod 644 so the test can read
   it; clean up. Name: an_append_stream_on_a_write_only_file_writes_at_the_end.

Run both tests 3 times, `cargo fmt --all --check`, `cargo clippy -p rexx-exec --all-targets -- -D warnings`
(CARGO_INCREMENTAL=0, CARGO_TARGET_DIR=/tmp/claude-1000/t86-wit-target, memcap 16G; delete the target dir
at the end). Doc comment per test: one sentence stating the property and that the oracle differs (no
Deviation number; the controller adds the numbers to phase-4-exclusions.txt). Commit only your test
files, staged by exact path. Never go idle on a background command. No subagents. Reply with: status,
commit, the two test paths::names, test output summary.
