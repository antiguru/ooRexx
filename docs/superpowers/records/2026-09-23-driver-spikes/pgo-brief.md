# Spike: pgo -- profile-guided optimisation of the shipping build

Read `common.md` beside this file first; it binds you.

## The idea

The measured cost is the register allocator's choices across one 15 KB
function whose arms are 57% never executed on `rexxcps`, and the compiler has
no idea which arms are hot. Profile-guided optimisation tells it. No source
change at all, so the noise-floor rule in `common.md` does not apply the same
way: the source is fixed and only the build changes. Nobody has tried PGO or
any profile-driven layout on this tree.

## Doing it

* `-Cprofile-generate=<dir>` build, run a training set, merge with an
  `llvm-profdata` whose LLVM version matches `rustc --version --verbose`'s
  `LLVM version` (candidates: `/usr/bin/llvm-profdata-21`, and the nightly
  toolchain's `~/.rustup/toolchains/nightly-*/lib/rustlib/*/bin/llvm-profdata`;
  check the version, a mismatch errors or silently drops profiles), then
  `-Cprofile-use=<merged>.profdata` build. Keep `lto = "fat"`,
  `codegen-units = 1` as the release profile has them. Pass flags through
  `RUSTFLAGS` on the build command; do not edit `Cargo.toml` or add a
  `.cargo/config` for the measured builds.
* **Two training sets, reported separately, because training on the benchmark
  measures memorisation:**
  * `held-out`: the programs in `rust/corpus/` (the differential corpus; skip
    any listed in `rust/corpus/oracle-crashes.txt`, and any that hangs or needs
    input -- cap each run with `timeout`), **excluding every measured axis**.
  * `in-sample`: the six measured axis programs themselves. This is the upper
    bound, not a result.
* If `-Cllvm-args=-pgo-warn-mismatch` or similar reports dropped profiles, say
  so; a profile that did not apply measures nothing.

## What to measure, beyond the common contract

* The common callgrind table for base, `held-out`, and `in-sample`.
* **PGO mostly changes layout and branch direction, which callgrind's
  instruction count cannot see.** So also measure cycles: `perf stat -e cycles
  -x,` one event per invocation (this sandbox has been measured to have one
  PMU slot; asking for several at once can silently multiplex), at least five
  runs per binary, interleaved, on `rexxcps` and `varlookup`. If `perf` refuses
  or reports `<not counted>`, retry once with a probe run before and after, and
  say exactly what it printed; do not conclude the counter is unavailable from
  one failure. Report instructions and cycles side by side, and say plainly if
  they disagree in sign.
* The driver's frame size and `run_ops_from::<true>` instruction count before
  and after, as a description only (they are not an instrument).
* Correctness floor from `common.md` on the `held-out` build.

Spike name: `pgo`.
