# Task 1 report: the crate, the grant, and the dependency

Committed at `36e1f5b99eed6862c5e3a9ea619a6d53a8506ff5` on `plan/rust-rewrite`.

## What was created

* `rust/crates/rexx-api/Cargo.toml` -- new crate, `lints.workspace = true`, one dependency
  `libloading = "0.8.9"` with a short comment pointing at D-U1.
* `rust/crates/rexx-api/src/lib.rs` -- licence header, one-sentence crate doc, `mod ffi;` /
  `mod load;` and nothing else.
* `rust/crates/rexx-api/src/ffi.rs` -- licence header, `// D-U1, 2026-09-14` /
  `#![allow(unsafe_code)]`, one-sentence module doc ("The inbound FFI boundary: caller-supplied
  pointers into validated handles."). No `unsafe` block.
* `rust/crates/rexx-api/src/load.rs` -- same shape, `// D-U1, 2026-09-14` /
  `#![allow(unsafe_code)]`, one-sentence module doc ("The outbound FFI boundary: loading a library
  and resolving symbols."). No `unsafe` block.

The workspace picked the new crate up through `members = ["crates/*"]` with no edit to
`rust/Cargo.toml`; `cargo metadata` lists `rexx-api` without any change to that file. `rust/Cargo.lock`
gained `libloading 0.8.9` and its two dependencies (`cfg-if`, `windows-link`) plus the `rexx-api`
node -- 17 lines, nothing else moved.

## What was changed

* `rust/crates/rexx-core/tests/unsafe_sites.rs` -- the `granted` list in
  `only_the_granted_module_may_say_unsafe` is now the sorted three-element vector
  `["crates/rexx-api/src/ffi.rs", "crates/rexx-api/src/load.rs", "crates/rexx-core/src/lib.rs"]`.
  The `uses` list (`["crates/rexx-core/src/bytes.rs"]`) and the `concat!`-spelled needles are
  untouched, as required.

## Negative control (Step 4)

**Prediction, written before running anything:** Adding `#![allow(unsafe_code)]` to a third file
(`rust/crates/rexx-num/src/lib.rs`, chosen because it currently has none) will make `files_where`
return a 4-element `granted` vector that includes `crates/rexx-num/src/lib.rs`. That will not equal
the hardcoded 3-element vector in the assertion, so `only_the_granted_module_may_say_unsafe` will
fail with an `assert_eq!` panic whose printed `left` value names `crates/rexx-num/src/lib.rs`
explicitly. `the_scan_reaches_the_whole_workspace` is unrelated and will keep passing. `cargo test`
will exit 101, not a compile error.

**Outcome: all three parts confirmed.** Command: `cargo test -p rexx-core --test unsafe_sites`,
exit status 101 (captured unpiped via `>/tmp/negctrl.log 2>&1; echo $?`). Failure output:

```
thread 'only_the_granted_module_may_say_unsafe' panicked at crates/rexx-core/tests/unsafe_sites.rs:86:5:
assertion `left == right` failed: the set of `unsafe_code` opt-ins in this workspace has changed. ...
  left: ["crates/rexx-api/src/ffi.rs", "crates/rexx-api/src/load.rs", "crates/rexx-core/src/lib.rs", "crates/rexx-num/src/lib.rs"]
 right: ["crates/rexx-api/src/ffi.rs", "crates/rexx-api/src/load.rs", "crates/rexx-core/src/lib.rs"]
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

`the_scan_reaches_the_whole_workspace` reported `ok`, as predicted. The `#![allow(unsafe_code)]`
line was then removed from `rust/crates/rexx-num/src/lib.rs`; `git status --porcelain` and
`git diff crates/rexx-num/src/lib.rs` afterward show no change to that file.

## Commands run, in order, with exit status

1. `sed -n ... docs/superpowers/plans/2026-07-27-rust-rewrite.md` (reading D-U1) -- read-only.
2. `mkdir -p rust/crates/rexx-api/src` -- exit 0.
3. Wrote `Cargo.toml`, `src/lib.rs`, `src/ffi.rs`, `src/load.rs` (the last two built from
   `head -10 rust/crates/rexx-core/tests/unsafe_sites.rs` piped into the file, to avoid a
   retyping error in the licence block -- caught one such error with `diff` before proceeding).
4. `diff <(head -10 .../unsafe_sites.rs) <(head -10 .../ffi.rs)` / same for `load.rs` /
   `lib.rs` -- all three empty, exit 0.
5. `cargo metadata --no-deps --format-version 1` -- exit 0, confirmed `rexx-api` listed without
   touching `rust/Cargo.toml`.
6. `cargo fmt --all` -- exit 0, no files changed (`git status --porcelain` before/after identical).
7. `cargo clippy -p rexx-api --all-targets -- -D warnings` -- exit 0.
8. `cargo test -p rexx-core --test unsafe_sites` (before the negative control) -- exit 0, 2 passed.
9. Edited `rust/crates/rexx-num/src/lib.rs` to add `#![allow(unsafe_code)]`.
10. `cargo test -p rexx-core --test unsafe_sites >/tmp/negctrl.log 2>&1; echo $?` -- exit **101**,
    1 passed / 1 failed as predicted.
11. Reverted the edit to `rexx-num/src/lib.rs`; confirmed clean with `git diff`.
12. `cargo fmt --all --check` -- exit 0.
13. `cargo clippy -p rexx-api --all-targets -- -D warnings` (re-run, post-revert) -- exit 0.
14. `cargo test -p rexx-core --test unsafe_sites` (re-run, post-revert) -- exit 0, 2 passed.
15. `cargo clippy --workspace --all-targets -- -D warnings` (extra sanity check beyond the brief's
    Step 5, to confirm the new crate doesn't break the workspace-wide lint) -- exit 0.

## Where the brief was wrong

* **"Modify: `rust/Cargo.toml` (the workspace dependency table)."** There is no
  `[workspace.dependencies]` table in `rust/Cargo.toml` -- every existing crate (e.g. `rexx-core`)
  declares its own dependencies with inline version strings, and `members = ["crates/*"]` is a glob
  that already covers a new crate directory. `cargo metadata` confirms `rexx-api` is a workspace
  member with no edit to `rust/Cargo.toml` at all. I made no change to that file; `libloading`'s
  version is declared only in `rust/crates/rexx-api/Cargo.toml`.

## Dependency note

`libloading = "0.8.9"` resolved without a network error (the offline cache brief item held); no
unused-dependency lint fired under `cargo clippy -p rexx-api --all-targets -- -D warnings` even
though nothing in the crate uses it yet, so nothing further needs reporting there.
