# c6 comment pass: `ir/drive.rs`'s test-only counters

Source: `c6-comments.txt` (comments7.py), `c6-positional.txt`, and
`/bin/grep -rn -a -i "counter\|instrumentation"` over the crate's comments.

* (a) `counters.rs:45`, `suspend_counters`'s doc: "until
  [`resume_counters`]. `Interp::bootstrap_library` is the only caller and
  both are `#[cfg(test)]`": both moved together, the link resolves in the
  same module, and `bootstrap_library` (`lib.rs:2055`, `:2075`) is still the
  only caller, through the `ir::drive::` path the re-export keeps.
* (a) `counters.rs:53`, "The other half of [`suspend_counters`]": same.
* (a) `counters.rs:159`, "The same for [`super::Op::LoadConstant`]; see
  `CONST_BUILDS`": a plain comment; `super` is `ir` from `counters` as it
  was from `drive`, and `CONST_BUILDS` moved beside it, still directly
  above.
* (c) `counters.rs:34`, "Whether the counters above and below are
  recording": the counters moved in their order, `RUN_CHUNK_ENTRIES` above
  it and the rest below it, so "above and below" still holds within the
  file.
* The other moved plain comments ("Test-only instrumentation: how many
  ...", and "[`Interp::run_ops`] entry on this thread") describe the
  counter under them, which moved with them. `Interp::run_ops` is a plain
  comment, not a doc link.
* `drive.rs`'s `impl Interp { fn site_resolution_before_arguments }`, which
  sat between the `ARITH_HINT_SKIPS` and `CALL_SITE_HITS` blocks, stays; its
  doc names no counter. Its `#[cfg(test)] count_call_site_hit()` call, and
  every call inside the op loop, is byte-identical and reaches the counter
  through `drive.rs`'s `#[cfg(test)] use super::counters::{...}`.
* `drive/tests.rs` imports the accessors as `super::{...}`; that path
  resolves through the same import, so the test file is unedited, and its
  module doc ("What the driver did") is unchanged and true.
* `lib.rs:2067`, "the compiled engine's own counters": names no file.
* No other comment in the crate names a moved counter.
