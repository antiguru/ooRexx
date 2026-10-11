# Phase 9a gate: the ooTest suite instrument

This record holds what Phase 9a measures and pins. Sections are added as the plan's tasks land.

## Perf pin

Phase 9's close measures the interpreter against the binary of the last commit before Phase 9
(`a98238be5`, "Phase 6.1: close the oracle-judged seeded gate item as not wanted"). 9a builds the
pin once and measures nothing against it. The binary lives outside the tree at
`/tmp/claude-1000/p9/pin/rexx-run`; if it is lost, rebuild it from the same commit with the
commands below and compare the sha256.

* sha256: `c97bf40a561cc46a4fa41ac64e7c8a16d42483f3a1869774a1c5e03610e49667`
* size: 58,283,928 bytes
* built 2026-10-11, release profile of `rust/Cargo.toml` at that commit (`debug = true` and its `lto`
  and `codegen-units` settings), `Compiling rexx-exec` present in the build output.

The whole commit is archived, not only `rust/`, because `rexx-inventory`'s build reads files outside
`rust/`. Run from the worktree:

```sh
mkdir -p /tmp/claude-1000/p9/pin/src
git archive a98238be5 | tar -x -C /tmp/claude-1000/p9/pin/src
find /tmp/claude-1000/p9/pin/src -exec touch {} +
cd /tmp/claude-1000/p9/pin/src/rust
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/p9/pin/target \
    memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run
cp /tmp/claude-1000/p9/pin/target/release/rexx-run /tmp/claude-1000/p9/pin/rexx-run
sha256sum /tmp/claude-1000/p9/pin/rexx-run
```

The source and target directories were deleted after the copy.
