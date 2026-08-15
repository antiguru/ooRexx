### Task 1: Re-establish the benchmark anchor

The `spike/bytecode-vm` figures are withdrawn and cannot be re-run from the tree: no bench-results file, no harness script, no build identity, and no empty-loop program in `rust/bench-programs/`. Every later task states a predicted axis movement, and without a reproducible starting figure those predictions are unfalsifiable.

**Files:**
- Create: `rust/bench-programs/emptyloop.rex`
- Modify: `rust/crates/rexx-bench/src/lib.rs` (`PROGRAMS`)
- Modify: `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` (`AXES`)
- Create: `docs/superpowers/plans/phase-4e-anchor.md`

**Two lists, not one, and each has a test that goes red if you miss it.** `rexx_bench::PROGRAMS` is what the criterion harness iterates, policed by `the_benchmark_list_accounts_for_every_program`. The suite binary's own `AXES` table is what the interleaved two-interpreter run iterates, policed by `the_axis_list_covers_every_bench_program`. The new entry is `Role::Loop`.

**Interfaces:**
- Produces: `docs/superpowers/plans/phase-4e-anchor.md`, holding per-axis ns/clause and ratio with a build identity and an oracle fingerprint. Every later task quotes it.

- [ ] **Step 1: Read the existing harness**

```bash
ls rust/crates/rexx-bench/src/ rust/crates/rexx-bench/src/bin/
sed -n '1,120p' rust/crates/rexx-bench/src/lib.rs
sed -n '1,80p' rust/crates/rexx-bench/src/child.rs
```

It already interleaves between binaries, takes per-arm minima, asserts the oracle fingerprint and reports absolute throughput alongside ratios. Reuse it. Do not write a second harness.

**It is Linux-only**, which is why the platform question stays open at the end of this plan rather than being answered by it.

- [ ] **Step 2: Write the empty-loop workload**

**The loop bound must be spelled `n = <digits>`, on its own line, exactly once.** `rexx-bench-suite.rs`'s `loop_count` strips the literal prefix `n = ` and parses the rest as a `u64`; it errors on a second such line and on none. Iterations per second is computed from it, so a program that names its bound anything else is not merely unconventional -- it fails `every_loop_axis_has_a_loop_bound`. This overrides the `n1` habit the probe constraint above establishes, which exists to dodge `x`/`b` hex-literal parsing and does not apply to `n`.

```rexx
/* The clause-dispatch floor: a loop whose body does nothing, so the
   measurement is the per-clause cost and almost nothing else. */
n = 3000000
do i = 1 to n
  nop
end
say 'done'
```

Sized so a single run is seconds rather than minutes. Check the oracle's own time before fixing `n`, from a fresh empty directory:

```bash
mkdir -p /tmp/probe-anchor && cd /tmp/probe-anchor
( ulimit -v 8388608; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/emptyloop.rex )
```

- [ ] **Step 3: Register it as an axis**

Add `emptyloop` to **both** lists named in Files above. Run the two policing tests and read their counts:

```bash
cd rust && cargo test -p rexx-bench the_benchmark_list_accounts_for_every_program
cd rust && cargo test -p rexx-bench the_axis_list_covers_every_bench_program
cd rust && cargo test -p rexx-bench every_loop_axis_has_a_loop_bound
```

A run count of zero means the filter matched nothing, not that the test passed.

- [ ] **Step 4: Measure**

Run the harness. Record, per axis: Rust ns/clause, oracle ns/clause, the ratio, and the number of interleaved passes.

- [ ] **Step 5: Write `phase-4e-anchor.md`**

It states, and a later task will be checked against it:

* the commit the Rust arm was built from, and the profile;
* the oracle fingerprint the harness asserted;
* per-axis absolute figures and ratios;
* **and, explicitly, that these are the tree-walker's figures at the start of the phase**, because after Task 3 the same binary has two arms and an unqualified "the ratio" stops having a referent.

- [ ] **Step 6: Commit**

---

## Global Constraints

* **The C++ tree at `/home/moritz/dev/repos/ooRexx/` is the oracle and is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle invocation:** `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`. Use `ulimit -v 8388608` for benchmark workloads.
* **Three programs crash the oracle; never run them:** `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **Run every oracle probe from a fresh empty subdirectory you `mkdir` yourself, with absolute paths.** The scratchpad root is on the oracle's external-routine search path and a leftover `.rex` file gets called as an external routine.
* **A symbol named `x` or `b` followed by a quoted string parses as a hex or binary literal.** Use `n1`, `cv`, `zz` in probes.
* **Use `/bin/grep -a`, never bare `grep`** -- the wrapper is ugrep with `-I` and silently skips binary files.
* **Read stdout, stderr and exit status as separate descriptors. Never `2>&1`.** Never read a cargo exit code from a pipeline.
* **Never run cargo from the repo root.** Run from `rust/`.
* **`cargo fmt --all --check`**, not `cargo fmt --edition 2024 --check`, which is an error rather than a check.
* **`cargo clippy --workspace --all-targets -- -D warnings`.** A warm target directory makes a green provisional; run from a clean target at the phase gate.
* **`cargo test <name>` exits 0 when it matches nothing.** Read the run count.
* **`cargo test --release` is a distinct gate** -- `lto = "fat"` changes behaviour, which `5253a674` recorded.
* **Mutation runs need `--no-fail-fast`**, or the suite stops at the first catcher and "nothing else caught it" is unmeasured.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
* **No em-dashes in comments; use `--`.** No counts of mutable in-repo aggregates in prose.
* **Markdown: one sentence per line, `*` bullets, first-word-only capitalisation including after a colon.**
* **Back up with `cp`, restore from the backup, verify with `sha256sum -c`.** Never `git checkout --`, never `git add -A`, never `git reset --hard`, never force-push.
* **Benchmark comparisons interleave between arms within one sitting.** Never read a comparison across two separate runs.
* **Commit first, then read the hash back with `git log`, then quote it.**

