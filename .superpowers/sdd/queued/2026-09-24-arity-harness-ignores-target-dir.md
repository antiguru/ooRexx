**CLOSED 2026-10-30 by Phase 9d Task 8.8b (ecce67dad).**

**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** no probe; tests/support/arity.rs:399 still joins ../../target/release/rexx where group_runner uses CARGO_BIN_EXE_rexx.

# The arity harnesses run `rust/target/release/rexx-run` whatever `CARGO_TARGET_DIR` says

Found by File split Task 3b (implementer's concern 3), 2026-09-24.

`rust/crates/rexx-exec/tests/support/arity.rs` runs the interpreter by the
fixed path `rust/target/release/rexx-run` and ignores `CARGO_TARGET_DIR`. A
test run with the target directory set elsewhere therefore exercises whatever
binary last landed in `rust/target`, of any age. Task 2's instrument 4 and its
six gates set `CARGO_TARGET_DIR` elsewhere while that binary dated from
2026-09-23, so their `collection_arity` and `introspection_arity` verdicts
tested a stale interpreter. Task 3b's gates used the default target
directory, so the arity verdicts at `a428c7596` (which contains Task 2's code)
are against the current binary.

Two earlier symptoms fit this: the performance-round agents needed both
`CARGO_TARGET_DIR` and a `rust/target` symlink for the fixtures to pass
(`clause-overhead.md`, concern 4), and the flaky
`every_unstable_row_is_really_unstable` runs the oracle, not us, which is why
it never noticed.

Fix: resolve the binary from `CARGO_TARGET_DIR` when set (or, better, use the
`CARGO_BIN_EXE_rexx-run` path Cargo provides to integration tests of the
package that owns the binary), and fail loudly when the binary is older than
the test binary. Witness: set `CARGO_TARGET_DIR` to an empty directory and
show the harness no longer runs `rust/target`'s binary.
