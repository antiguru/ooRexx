# Phase 9a final review fix report

Base `ff3d45c51`. Commits, in order:

| sha | item |
|---|---|
| `04812fa3b` | I1: `Package~findClass` and `class_in_program` read the store-backed package local |
| `b4ef50a50` | I2: a `Method`/`Routine` subclass copy gets its own annotation store |
| `33efaabe6` | M6: `.NAME` past a package local; `pkglocal` bench program |
| `99d6e56a1` | I5: extcall, pingmsg, dirread regressions removed |
| `f55f8b2d0` | I3, M3, M4, M5, M7 records |
| `28865c75c` | M1, M2 prose |
| (this commit) | this report |

## I1

`identities.rs` `package_find_class`, `environment.rs` `class_in_program` and `dot_variable` all read
the package local through `hash::directory_get`/`package_local_get` by the name's bytes.
`native_map_entry` had no other caller and is deleted. `package_find_class` no longer allocates a
local it only reads.

Witness `corpus/lang/package_local_find_class.rex` (+ `.d/lib.cls`, `.env`), in `phase-9.txt`, with a
`sourceline_oracle` expectation generated through the oracle driver. Oracle and head after the fix:
`dot: The Array class`, `findClass: The Array class`, `findClass absent: The NIL object`,
`method: The List class` (orxmethod `FindContextClass` from a method in `lib.cls`, whose package
local holds `BAR`, which is the `class_in_program` path), `routine: The Array class`,
`lib local: The List class`, rc 0. Before the fix, head printed `findClass: The NIL object` and the
method line raised 88.900, rc 168.

## I2

`object_protocol.rs` `duplicate_executable` runs `duplicate_collection_stores` on the cloned table.
Witness `corpus/lang/executable_subclass_copy_annotations.rex`: oracle and head `one 1 two 2` /
`0 2 two` / `x 1 0` / `one two`, rc 0; before the fix head printed `two 2 two 2` and `1 1 The NIL
object` on its first two lines.

## M6

`package_local_entry` keeps the local's `StoreView` (`Interp::package_local_view`), and
`package_local_symbol` remembers the last name found absent (`PackageLocalMiss`) with
`store_generation` and a new `store_writes`, which `hash::write_slot` bumps on every hash-store slot
write. A repeat with both unchanged answers absent without a probe; a `debug_assert!` re-probes on
every memo hit.

Witness `corpus/lang/package_local_symbol_writes.rex`: `.NAME` after `put`, `[]=`, `remove`,
`setMethod`, `unsetMethod`, `UNKNOWN`, `empty`, oracle-identical (13 lines, rc 0). Mutant: the
`write_slot` bump deleted, release build: five output lines go stale (`2 .PLWNAME`, `3 .PLWNAME`,
`.PLWNAME other`, `.PLWNAME` twice). Restored, `git diff --stat` on `hash.rs` back to the one-line
addition.

`bench-programs/pkglocal.rex` (one `~local` write, 2,000,000 `.stdout` reads; oracle twice and crate
`1 2000000`, rc 0) is in `callgrind.sh` PROGRAMS, `rexx-bench` PROGRAMS, the suite's AXES and the
README. `cargo test --release -p rexx-bench`: exit 0.

pkglocal, `callgrind.sh`, pin `a98238be5`:

| binary | Ir | vs pin |
|---|---|---|
| ff3d45c51 (r3) | 6,843,892,505 | +74.69% |
| I1 commit (r3) | 5,475,898,236 | +39.77% |
| I1 + view kept (r1) | 4,525,898,509 | +15.53% |
| M6 commit (r1) | 3,813,899,472 | -2.65% |
| HEAD (r3, full run) | 3,767,896,530 | -3.82% |

## I5

Pin built from `git archive a98238be5` in `/tmp/claude-1000/p9/pin-src`, `CARGO_TARGET_DIR=/tmp/claude-1000/p9/pin-target`,
tree touched, log `/tmp/claude-1000/p9/ff1/build-pin.log` with one `Compiling rexx-exec` line;
binary sha256 `6a1ce7ae...dcb`. ff3d45c51 built the same way (`head0`). Both source and target dirs
deleted after the run. Measured by `rust/bench-programs/callgrind.sh`, which interleaves rounds
(round r starts at binary r) and subtracts libc and ld.

Causes, from function-level callgrind diffs (`/tmp/claude-1000/p9/ff1/fdiff.py`, `callers.py`):

* **extcall** (Task 9, +237 Ir per call): `Table` grew a second `FxHashMap` and a limit inline, so
  every native frame memcpy moved 40 more bytes (`push_native_frame`/`take_native_frame`/
  `pop_native_frame` memcpy +100 Ir a call); `Table::register` removed from the empty released map
  (+16 Ir, four times a call from `Activation::constants`); `register_local`'s `pop_temp_above` on
  those four constants. Fix: released handles boxed behind `Option<Box<Released>>`, created on the
  first release; `register` skips an empty map; `constants` registers `.nil`, `1`, `0`, `''` straight
  into the table (none is ever a temp a call pushed).
* **pingmsg** (Task 5): per `~start`, the StackFrame's 9th entry, a `Vec` for the entries, and map
  growth. Fix: entries are a fixed array, `NativeObject::reserve_entries` sizes the map once. What
  stays is the price of the EXECUTABLE entry: `executable_at` +61 Ir and `program_routine_object`
  +47 Ir per frame (a frame keeps its executable after its activation ends).
  Separately, libc's share moved: `FrameArena::open_block`'s `alloc_zeroed` per new activity memsets
  4.60e9 bytes-worth of instructions on the pin and 7.64e9 on ff3d45c51 (80,007 callocs); the
  table subtracts libc, so it is not in the figures. It is allocator-state dependent (a calloc served
  from reused memory is zeroed, from fresh top is not); not chased.
* **dirread** (Task 5): `native_hash_unknown` went through `array_argument`, which copied the slots
  three times and read the heap four times (`array_slots_of` twice in `array_argument_object`,
  `is_multi_dimensional_array`, `array_slots_of` again). Fix: `array_argument_object` reads an array
  argument once; `native_hash_unknown` takes the array object and reads its first slot. Also,
  I1's second caller of `directory_get` stopped it being inlined into `directory_lookup`;
  `package_local_get` is a separate copy so `directory_get` keeps one caller. Residual: the
  `directory_lookup` codegen is ~22 Ir per lookup over ff3d45c51's after these changes with no
  source change in it (line-level annotation: result moves in `core/result.rs` lines), not removed.

Per-step r1 runs, pin vs: `f1` (Table + constants + frame entries) extcall +0.28%, pingmsg -3.03%,
dirread +1.38%; `f2` (+ `array_argument` single read) dirread +0.85%; `f3` (forced inlining)
dirread +0.63%, reverted in `f4` = HEAD, dirread +0.45%.

### Full set, r3, pin vs ff3d45c51 (head0) vs `99d6e56a1` (head)

```
timeout 5400 rust/bench-programs/callgrind.sh -r 3 -j 8 -o /tmp/claude-1000/p9/ff1/cg-full \
  pin=bins/pin head0=bins/head0 head=bins/head        # exit 0, every spread under 0.01%
```

| program | pin | head0 | head | head0 d% | head d% |
|---|---|---|---|---|---|
| alloc | 20362612263 | 20388002876 | 20388007210 | +0.1247 | +0.1247 |
| alloc4c | 3106074223 | 3098435372 | 3098439260 | -0.2459 | -0.2458 |
| arith | 11478090458 | 11486756569 | 11486760903 | +0.0755 | +0.0755 |
| assign | 18381875843 | 18382083686 | 18382087938 | +0.0011 | +0.0012 |
| compound | 8836768981 | 8836976758 | 8836980906 | +0.0024 | +0.0024 |
| decloop | 2404646443 | 2407726563 | 2407730451 | +0.1281 | +0.1283 |
| decrender | 4171162427 | 4162241329 | 4162245663 | -0.2139 | -0.2138 |
| dirread | 2663580744 | 2701661181 | 2675465087 | +1.4297 | +0.4462 |
| dispatch | 20673777334 | 20663986519 | 20663990667 | -0.0474 | -0.0473 |
| dispatchclass | 15654210237 | 15242419449 | 15242423783 | -2.6305 | -2.6305 |
| emptyloop | 7735375677 | 7735583382 | 7735587716 | +0.0027 | +0.0027 |
| extcall | 7536919541 | 7960127747 | 7558131632 | +5.6151 | +0.2814 |
| fibcall | 8469931471 | 8296481190 | 8296485078 | -2.0478 | -2.0478 |
| fibfunc | 8300416378 | 8126966097 | 8126969985 | -2.0897 | -2.0896 |
| hashnew | 1588972374 | 1595699629 | 1595703588 | +0.4234 | +0.4236 |
| heapshape | 2306076207 | 2300482779 | 2300494163 | -0.2426 | -0.2421 |
| nop | 9181613985 | 9181822032 | 9181825920 | +0.0023 | +0.0023 |
| parse | 1552803132 | 1553724260 | 1553728594 | +0.0593 | +0.0596 |
| pkglocal | 3917669336 | 6843892505 | 3767896530 | +74.6930 | -3.8230 |
| sayloop | 107724146 | 107932193 | 107936081 | +0.1931 | +0.1967 |
| sendloop | 13818086084 | 13808295273 | 13808299421 | -0.0709 | -0.0708 |
| startup | 57372887 | 57580934 | 57584822 | +0.3626 | +0.3694 |
| strings | 17536542018 | 17521274959 | 17521278847 | -0.0871 | -0.0870 |
| textnum | 1150021282 | 1150229329 | 1150233217 | +0.0181 | +0.0184 |
| varlookup | 13132806145 | 13133013850 | 13133018184 | +0.0016 | +0.0016 |
| rexxcps | 17792036305 | 17792132575 | 17792136625 | +0.0005 | +0.0006 |
| pingguard | 1148196426 | 1148404225 | 1148408593 | +0.0181 | +0.0185 |
| pingmsg | 1667809923 | 1702538958 | 1617263106 | +2.0823 | -3.0307 |
| pingsem | 1171630875 | 1171838569 | 1171840067 | +0.0177 | +0.0179 |

Worst three after the fixes: dirread +0.446%, hashnew +0.424%, startup +0.369%. Every program is
under +0.5%. The binary measured is `99d6e56a1`'s; the two commits after it change comments, records
and one refusal string (M5), not code paths.

## Records (I3, M3, M4, M5, M7)

* I3: re-run here. From a fresh directory with `makestring.rex`, `rxregexp.cls` and `in.txt`,
  `LD_LIBRARY_PATH=.../build/lib memcap 2G rexx-run makestring.rex < in.txt`: rc 120, `rexx-exec:
  routine "SYSTEMPFILENAME" is not implemented (Phase 9)`; without the path: 98.903 loading
  `rxregexp`. `cd.rex` (the reviewer's) through a `bin/rexx` link to `rexx-run` with the library
  path: `rexx-run: scclient: No such file or directory (os error 2)`. The oracle side of `cd.rex`
  (finds `scclient.rex`) is the reviewer's run, not repeated. The test group's `"rexx" prefix ||
  "server"`/`"client"` and `'cd' samplePath` lines are `ootest/ooRexx/samples/scclient.testGroup:86-96, 194-195`.
  `phase-9a-gate.md` Task 5 sentence, the samples paragraph (with the command), and the carried list
  corrected; `extensions/rxregexp/rxregexp` and `rxmath/RxMath` crate rows in `suite-close/table.tsv`
  are 33/33 and 289/289.
* M3: carried list's sim-gate item dropped; `sim-gate.tsv` header points at
  `task-10-sim-report.md` for the Phase 9a re-derivation. M4: line 48 states `check_phase_10`'s
  D3 requirement.
* M5: refusal text now "a metaclass NEW answering a class Class~new did not build or a subclass
  already took"; Deviation 32, the constructor's doc, the call site's comment, the disposition row
  and `a_metaclass_new_answering_no_new_class_is_loud` changed with it. `refusal-sites.tsv`
  re-derived (`REXX_REFUSAL_SITES_REFRESH=1`): only `lib.rs` line numbers moved. New probe
  `metaheld.rex`: oracle and crate `XX Object 1 XX Object`, rc 0.
* M7: `docs/superpowers/records/2026-10-11-phase-9a/deviation-probes/`: `objname-table.rex`
  (Deviation 31's table; oracle `long / a String / long / long / hundred / a String`, crate rc 120),
  `objname2.rex`, `objname4.rex`, `ord.rex` (Deviation 33's `enh:` line: oracle `MANGO UM ALPHA QUUX
  BETA ZETA FOO`, crate `ALPHA BETA FOO MANGO QUUX UM ZETA`; both then rc 159 at line 8, 97.2), and
  `metaheld.rex`, each with both sides' out/err/rc and a README with the commands. The reviewer's
  `ordm.rex` was not kept: its class is not public, so it raises 97.1 on both.

## M1, M2

Deleted the false examples (Package~local, ~annotations, TraceObject) from `owns`, `not_this_task`,
`Primitive::Directory`/`StringTable`, `is_hash_collection`. `.TraceObject~new~allIndexes` answers
(it would refuse through `not_this_task` if the object were `Body::Native`), so it is store-backed
too. ` -- ` replaced at `options.rs` (take_defaults), `dispatch/hash.rs` (owns),
`dispatch/collection/supplier.rs` (drain_supplier), `lib.rs` (method_objects); `git diff ff3d45c51 --
rust/crates | grep '^+' | grep ' -- \|—'` is empty.

## Checks

At `28865c75c` (the code and records this report describes; the report commit adds only this
file). All foreground, `CARGO_INCREMENTAL=0`, `memcap 8G`, `-j 4`, release, default `rust/target`:

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib` | 0 (1087 passed) |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 (29 passed, 1 ignored; `1022 of 1022 matching`) |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test collect_stress` | 0 (37 passed) |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test refusal_sites --test refusal_dispositions --test phase_ten_reasons --test closed_phases --test owners --test introspection_arity --test method_bodies` | 0 (5, 3, 5, 8, 6, 26, 23 passed) |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `cargo test --release -p rexx-exec --test suite` (ungated) | 0 (50 passed) |
| `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/class/Method,base/class/Routine cargo test --release -p rexx-exec --test suite` | 101: 3 of 3 red, verdicts and table rows identical to `suite-close` (Package 77/80, TEST_NEW01 TEST_PROLOG failed, TESTRECLAIM01 timeout; Method and Routine 54/56 and 58/60, the two 9c rexxc tests) |
| `cargo test --release -p rexx-api` | 0 |
| `cargo test --release -p rexx-exec --test release_local_reference` | 0 (3 passed) |
| `cargo test --release -p rexx-bench` | 0 |

## Concerns

* `package_local_miss` is valid only while every hash-store content write goes through
  `hash::write_slot` and every pool write bumps `store_generation`. The witness and its mutant show
  the bump is load-bearing for the paths tested; a future writer that bypasses `write_slot` would
  answer stale. The debug assertion catches it in debug builds only.
* dirread's residual +0.45% is near the +0.5% budget and partly codegen in `directory_lookup` that I
  did not attribute to a source change.
* pingmsg's libc share (calloc zeroing in `FrameArena::open_block`) rose by ~3.0e9 instructions
  pin to ff3d45c51; the table subtracts libc, so the budget does not see it, but wall time does.
* I4 (yaml GUARD) is queued to 9d by the controller and untouched here. `suite-9d-set.txt` still
  lists the three samples tests, as derived; their owners are 9b and 9c.
