### Task 7: rename `{name}/mod.rs` to `name.rs`, where that is what it means

Requested by Moritz on 2026-08-15. The tree has exactly four `mod.rs` files, and **only two of them
are in scope.** Confirm the set yourself with `find crates -name mod.rs` before starting.

#### In scope

* `crates/rexx-exec/src/builtin/mod.rs` becomes `crates/rexx-exec/src/builtin.rs`
* `crates/rexx-exec/src/ir/mod.rs` becomes `crates/rexx-exec/src/ir.rs`

Both keep their sibling directory. `name.rs` beside `name/` is the 2018 convention and is what
`run.rs` beside `run/tests.rs` will already look like once Task 6 lands.

#### Out of scope, and this is the point of the task rather than an omission

* `crates/rexx-exec/tests/support/mod.rs`
* `crates/rexx-parse/tests/gate_walk/mod.rs`

**These must NOT be renamed.** Measured by the controller: `mod support;` is declared by seven test
binaries (`builtin_status.rs`, `corpus.rs`, `input_oracle.rs`, `parse_version_oracle.rs`,
`state_builtin_oracle.rs`, `trace_indent.rs`, `trace_oracle.rs`) and `mod gate_walk;` by
`rexx-parse/tests/tiling.rs`. Renaming them to `tests/support.rs` and `tests/gate_walk.rs` would
still resolve as modules, **and** cargo would additionally auto-discover each as an integration-test
target, compiling the helpers standalone as a test binary that exists for no reason. The `mod.rs`
form under `tests/` is the idiom that prevents exactly that.

Record that reasoning where a future reader of those two files will find it, so the next person
applying this convention does not undo it.

#### Steps

1. Confirm the four-file set and the eight declaring sites above. Report anything that has moved.
2. Rename the two `src/` files with `git mv`, so the history follows.
3. Build. Nothing else should need editing: `mod builtin;` and `mod ir;` resolve to either spelling.
   If any other file needs a change, stop and report what and why before making it.
4. Leave a note at the two `tests/` files saying why they keep `mod.rs`.
5. Gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --release --workspace`, the corpus gate under `REXX_CORPUS_GATE=1`, and
   `cargo doc --no-deps` read for warnings, since a module path changing is exactly what breaks an
   intra-doc link.
6. Report the test count before and after, binary by binary. A rename that silently drops a test
   target is the failure this task must not produce.

---

