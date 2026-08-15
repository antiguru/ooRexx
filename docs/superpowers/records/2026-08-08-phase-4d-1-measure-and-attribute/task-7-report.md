# Task 7 report: the allocator diagnostic

Status: DONE.
BASE `9ce83f14`. Commit `e743e05e`. Deliverable: an addition to
`docs/superpowers/plans/phase-4d-attribution.md` ("Task 7 -- the allocator-swap diagnostic",
between C6 and C7).
The swap itself was never committed: it was an uncommitted edit, backed up with `cp`, measured,
profiled, then restored from the backup and verified byte-identical.
Working tree clean before and after.

## What was done

`#[global_allocator]` in `rust/crates/rexx-exec/src/bin/rexx-run.rs` was pointed at `mimalloc`
0.1.52 (`libmimalloc-sys` 0.1.49, its bundled v3), added as a normal dependency in
`rexx-exec/Cargo.toml`.
No `unsafe` was written in this crate's own code -- the swap is a dependency plus one `static`
declaration, and `cargo build --offline --release` succeeded under the workspace's
`unsafe_code = "forbid"` without an exception, because the `unsafe impl GlobalAlloc` lives inside
the `mimalloc` crate, outside this workspace's lint.
Both edited files and `Cargo.lock` were backed up with `cp` before editing and restored with `cp`
afterward; `sha256sum -c` confirmed all three byte-identical to the backups, and the rebuilt
`rexx-run` is `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967` -- the baseline
binary the committed baseline and Task 6 were both measured against.
Never `git checkout --`.

Five repetitions per axis, base and mimalloc-swapped binary interleaved, same wrapper as Task 6's
prototypes (`ulimit -v 8388608`, fresh empty directory, `/dev/null` stdin, `date +%s.%N`).
All ten runs per axis exited 0; stdout was stable within each side and identical across the swap on
every axis.

| axis | base median | mimalloc median | change | C6's glibc self-time share |
|---|---:|---:|---:|---:|
| `varlookup` | 5.2260 s | 5.2883 s | **+1.2%** | 4.6% |
| `compound` | 6.6706 s | 6.2322 s | -6.6% | 30.0% |
| `strings` | 9.2406 s | 8.6737 s | -6.1% | 37.7% |
| `alloc4c` | 2.2800 s | 1.8611 s | **-18.4%** | 39.5% |
| `arith` | 3.0940 s | 2.8925 s | -6.5% | 34.6% |

`alloc4c` -- the axis with the largest win and the highest C6 share -- was profiled a second way
(`samply record --save-only`, 1 kHz, `pollard` with `expand_inlines`, `unsymbolicated_pct` 0.37%),
on the mimalloc binary, to see where the recovered time actually went.
Summing every `mi_*` function's self time gives 16.8%, plus 0.7% for `madvise` mimalloc still
issues through libc -- about 17.5% self, against the base binary's 39.5% in the glibc allocator
family on the same axis.
mimalloc's own bookkeeping is a little over half as expensive per call as glibc's there, and that
still only buys an 18.4% wall-clock win, because the per-call cost inside mimalloc itself and the
surrounding hashing/copying machinery (`memcpy`, `memcmp`, SipHash, `hashbrown` probing) do not
shrink when the allocator changes.

## The fork, and which side it fell on

**Recovers little, not most.** Reading each axis's win against its own C6 self-time share as the
loosest possible ceiling ("if every byte of that self time vanished and nothing else grew"): 46.6%
recovered on `alloc4c`, 22.0% on `compound`, 18.8% on `arith`, 16.2% on `strings`, and a net loss on
`varlookup`, the axis with the least allocator involvement.
Never "most" on any axis, and the fraction shrinks as the share shrinks.

**This is the expected outcome, not the informative one, and it is reported as such.** The brief
said to expect this and to flag if the result contradicted it; it did not.
The cost this measurement bounds is allocation *count*, not allocator *quality*, and the
profiling on `alloc4c` shows the mechanism directly: mimalloc halves its own per-call self-time
share on that axis and still leaves the axis at 1.86 s rather than something close to the oracle's
1.18 s, because nothing about switching allocators removes a call, a hash, or a copy.
D1's pre-registered side byte-arena -- removing calls rather than serving them faster -- remains
the candidate this measurement does not replace.

## Feasibility

The parity gate names Linux and macOS.
This project's CI, `.github/workflows/{unix,windows,bsd}.yml`, names five platforms for the C++
oracle (this crate has no CI of its own yet): Linux (`ubuntu-24.04`), macOS (`macos-15`, arm64),
Windows (`windows-2022`), FreeBSD 14.2, OpenBSD 7.8.

**Checked, from this machine: Linux.** `cargo build --offline --release` compiled `mimalloc` via
`libmimalloc-sys`'s `cc`-crate build script -- no `cmake`, no network fetch beyond the local
registry cache.

**Unchecked: macOS, Windows, FreeBSD, OpenBSD.** Not built on any of them from here.
`libmimalloc-sys`'s `build.rs` branches by name on `target_env == "msvc"` and
`target_vendor == "apple"`, and the vendored `c_src/mimalloc/v3/readme.md:35-36` claims ports to
"Windows, macOS, Linux, WASM, various BSD's", with `prim.h`/`prim.c` carrying `__FreeBSD__`,
`__OpenBSD__`, `__NetBSD__` and `__DragonFly__` branches by name.
That is upstream's own claim and this crate's build script targeting those platforms by name, not
a build performed on them.
Recorded as unchecked rather than inferred.

## No optimisation adopted

The swap was reverted before this commit closed and is not proposed here.
Adoption of any allocator is a 4d-2 decision against the parity gate, informed by this bound, not a
Task 7 decision.

## Verification

All read unpiped from `rust/`, after the swap was reverted and the tree confirmed clean.

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings`, `CARGO_TARGET_DIR` pointed at a freshly created directory | exit 0, 61 `Compiling`/`Checking` lines, 0 `warning`/`error` lines on stderr |
| `cargo test --offline --workspace --no-fail-fast`, mimalloc still swapped in | exit 0, 1317 passed, 0 failed |
| `cargo test --offline --workspace --no-fail-fast`, after revert | exit 0, 1317 passed, 0 failed |
| `git status --short` before commit | one line, the attribution doc |
| source hashes against the pre-swap backups (`Cargo.toml`, `rexx-run.rs`, `Cargo.lock`) | all three `OK` |
| `rust/target/release/rexx-run` sha256, after revert and rebuild | `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`, the baseline binary |

## Constraints honoured

The swap never entered the baseline: it is a runtime behaviour change, backed up with `cp` and
restored with `cp`, never `git checkout --`.
No `unsafe` was introduced; the workspace's `unsafe_code = "forbid"` held throughout.
Every timed run used a fresh empty directory, `/dev/null` on standard input, and
`ulimit -v 8388608` on both binaries; stdout, stderr and exit status were read as separate
descriptors.
No cargo exit status was read from a pipeline -- each was captured to a file and the shell's own
`$?` read immediately after the direct invocation.
No `git add -A`, no `git reset --hard`, no force-push.
Every comparison was interleaved within one run; no figure here is a comparison across two separate
runs.
