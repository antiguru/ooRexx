# Phase 9b close: dirread perf fix

Code commit `f3efac9a1`. Status: dirread back under budget, every program passes `-B 0.5`.

## Cause

`d~a` on a Directory reaches `native_hash_unknown`, which calls `array_argument_object` on the
forwarded argument array. At afc3b895b that function was out of line: 51 Ir self per call
(200001 calls, +10.2M), 12 of them prologue and epilogue on the `fn` line, for a value that is
already a single-dimensional array. `d['A']` does not reach it.

## Fix

1. `array_argument_object` is `#[inline]` and handles only the already-an-array case (heap read,
   dimension check, `push_temp`). Conversion and refusal moved to an `#[inline(never)]`
   `converted_array_argument`. Rooting and every refusal are unchanged.
2. `native_hash_unknown` uppercased its name with `to_ascii_uppercase`, a second allocation after
   `to_vec`. It now uppercases in place and pops a trailing `=`, so one malloc and free per send go.

## dirread, whole process, one round each (`-r 1 -p dirread`, scratchpad)

| binary | Ir | vs pin |
|---|---|---|
| pin a98238be5 | 3130118068 | |
| afc3b895b (close log) | 3146330145 | +0.5180% |
| fix 1 only (inline split) | 3142715056 | +0.4024% |
| fix 1 + 2 (`f3efac9a1`) | 3125236537 | -0.1560% |

Per send: fix 1 saves 18 Ir (3.6M / 200001), fix 2 saves 87 Ir (17.5M / 200001).
`array_argument_object` no longer appears as a function in the profile.

## Full set, 3 rounds interleaved, `-B 0.5`, pin vs `f3efac9a1`

Commands (from `rust/`), both exit 0:

    memcap 12G bench-programs/callgrind.sh -o .../perf-fix/a -B 0.5 -p "alloc ... fibfunc" pin=<pin> fix=<fix>
    memcap 12G bench-programs/callgrind.sh -o .../perf-fix/b -B 0.5 -p "hashnew ... pingsem" pin=<pin> fix=<fix>

`<fix>` is `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b-close-perf-fix`, byte-identical (cmp) to
`cargo build --release --bin rexx-run` at `f3efac9a1`. Tables: `perf-fix/callgrind-a.log`,
`perf-fix/callgrind-b.log`.

| program | before (afc3b895b) | after (f3efac9a1) |
|---|---|---|
| alloc | +0.2272 | +0.2535 |
| alloc4c | -0.2078 | -0.2086 |
| arith | +0.0880 | +0.0880 |
| assign | +0.0020 | +0.0020 |
| compound | +0.0351 | +0.0351 |
| decloop | +0.1310 | +0.1308 |
| decrender | -0.1725 | -0.1725 |
| dirread | +0.5180 | -0.1560 |
| dispatch | -0.0664 | -0.2472 |
| dispatchclass | -3.5609 | -3.9384 |
| emptyloop | +0.0047 | +0.0048 |
| extcall | -13.2913 | -13.3316 |
| fibcall | -3.4351 | -3.4488 |
| fibfunc | -3.5383 | -3.6133 |
| hashnew | +0.3215 | +0.3246 |
| heapshape | -0.1405 | -0.1408 |
| nop | +0.0040 | +0.0039 |
| parse | +0.0690 | +0.0690 |
| pkglocal | -2.2364 | -2.6188 |
| sayloop | +0.1925 | +0.1934 |
| sendloop | -0.1004 | -0.1004 |
| startup | +0.2928 | +0.2886 |
| strings | -0.0799 | -0.0799 |
| textnum | +0.0284 | +0.0285 |
| varlookup | +0.0026 | +0.0028 |
| rexxcps | -0.0316 | -0.0328 |
| pingguard | +0.2695 | +0.0262 |
| pingmsg | -1.2200 | -1.2128 |
| pingsem | +0.1034 | +0.1643 |

Worst three after: hashnew +0.3246, startup +0.2886, alloc +0.2535.

Pin-side figures move between runs for some programs (alloc pin 22831665035 then 22825678004,
dispatch pin 22065711518 then 22105710248) with zero within-run spread, so a delta under about
0.2% on those is not attributable to this change.

## Checks, at f3efac9a1's tree

- `cargo fmt --all --check`: exit 0
- `cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0
- `cargo test --release -p rexx-exec --lib`: 1116 passed, 0 failed
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: exit 0, 29 passed, 1 ignored
- `memcap 8G cargo test --release -p rexx-exec --test collect_stress`: exit 0, 37 passed
- Probe, pin vs fix, getter, setter, mixed-case `~unknown`, StringTable setter and the no-value
  setter refusal: stdout, stderr and rc 120 identical.
