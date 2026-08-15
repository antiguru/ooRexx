### Task 2: The interleaved two-interpreter harness, and the baseline

**Files:**
* Create: `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`
* Modify: `docs/superpowers/plans/perf-baseline.md`

- [ ] **Step 1: Read what already exists before writing anything**

`rust/crates/rexx-bench/src/bin/rexx-time.rs` already times an arbitrary command over N runs after M warmups and reports a median. **Reuse it rather than reimplementing timing.** Its interface is `rexx-time [--warmup N] [--runs N] <command> [args...]`.

`perf-baseline.md` already holds the oracle's Phase 0 wall times. **Do not reuse them.** They were taken at a different commit of the oracle and the phase's own rule is that a baseline is measured at gate time.

- [ ] **Step 2: Write the harness**

`rexx-bench-suite` runs, for each axis: the oracle then this crate then the oracle then this crate, alternating, for N pairs -- **not all of one side then all of the other**. Frequency drift across minutes on this 32-core part exceeds several of the effects being measured, and alternating makes drift common-mode.

It reports per axis:

* **wall time**, median and spread, both sides
* **iterations per second**, both sides, taken from the program's own loop count, which is exact
* **the ratio**, oracle over rust for throughput measures
* the **fixed per-process offset** as its own line, measured by timing a program whose body is `nop`, so a change to startup is visible rather than distributed across every axis

The axis list is a **literal constant in the source**, asserted against the contents of `rust/bench-programs/` so that an axis cannot silently disappear. This is the device `tests/corpus.rs` uses for its subset files; copy that shape.

- [ ] **Step 3: Pin N, the statistic, and the tool, in the source**

Adopt Global Constraints `:39`'s definition of "slower": the point estimate falling outside the C++ baseline's confidence interval on the slow side. Ratios are reported because they are legible; **the gate is decided on interval overlap**, so the harness must emit an interval, not only a median.

- [ ] **Step 4: Fingerprint the oracle**

Record `size`, `mtime` and `sha256` of `/home/moritz/dev/repos/ooRexx/build/bin/rexx` into the report. The project declined a corpus-harness fingerprint deliberately, on the grounds that rebuilds are rare and the user's own; that reasoning does not carry to a phase whose entire output is ratios against that binary, measured once here and again at the end of 4d-2.

- [ ] **Step 5: Run it, commit the report**

Both sides under `ulimit -v 1048576`, stated in the report. Release build both sides; the oracle is a CMake `Release` build and `rust/Cargo.toml` now pins `[profile.release]`.

Record the `arith` result explicitly against **Phase 2's outstanding parity debt** (`d1-decision.md:76`, recorded at 1.22 times and warned to be a lower bound). This is that debt's scheduled re-measurement.

---

