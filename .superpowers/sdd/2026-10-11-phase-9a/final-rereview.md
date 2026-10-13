# Phase 9a final fix re-review, ff3d45c51..1f068bcca

Reviewer: scoped re-review (opus), 2026-10-13. Read-only on tree, index and HEAD; this file is the one
write. Crate binary: `rexx-run` built at `1f068bcca` into `/tmp/claude-1000/p9/ffr-target`
(`Compiling rexx-exec` in the build output; deleted after). Probes ran from fresh directories
`/tmp/claude-1000/p9/ffr{1,2,3,4}/`, crate under `memcap 2G timeout 20`, oracle under
`ulimit -v 1048576` with the oracle's `LD_LIBRARY_PATH`. Perf figures come from the fix's own raw
callgrind output, `/tmp/claude-1000/p9/ff1/cg-full/*.row` (columns: total Ir, libc Ir, ld Ir,
libc-and-ld-subtracted Ir), and `callgrind_annotate` of `extcall.{pin,head0,head}.r1.cg` in that directory.

## Verdicts

| item | verdict |
|---|---|
| I1 | addressed |
| I2 | addressed |
| I3 (records) | addressed |
| I5 | partially: under budget on `callgrind.sh`'s libc-subtracted counts; extcall still +3.05% whole process (N1) |
| M1 | addressed |
| M2 | addressed |
| M3 | addressed |
| M4 | addressed |
| M5 | addressed |
| M6 | addressed; the miss memo holds on every write path enumerated below |
| M7 | addressed |

New: Important 1 (N1), Minor 4.

## 1. M6: what the miss memo depends on, and every write path

`package_local_symbol` (`environment.rs`, the `package_local_miss` arm) answers Absent without a probe
when program, `store_generation`, `store_writes` and the name all match. `view_get`'s Absent answer
depends on: the contents store's three arrays and their slots; the method store's arrays and slots;
the `METHODUNKNOWN` pool entry; and which local `package_locals` maps the program to. Each one, and
what moves when it changes:

| write | path | moves |
|---|---|---|
| put, `[]=`, setEntry, `UNKNOWN` `NAME=`, putAll, native API `store_put` | `insert`/`store_insert`/`native_put` -> `write_slot` (`hash.rs:766-882`) | `store_writes` |
| remove, native API `store_remove` | `store_remove` -> `write_slot` (`hash.rs:957-977`) | `store_writes` (and `set_free` -> `store_generation`) |
| empty | slot clears through `write_slot` or a reinstall | `store_writes` / `store_generation` |
| growth, first store, first `setMethod` method table | `install_store` (`hash.rs:292-321`) | `store_generation` |
| free-chain head | `set_free` (`hash.rs:450-455`) | `store_generation` |
| `setMethod('UNKNOWN')` / unset | `set_unknown_method` (`hash.rs:996-1001`) | `store_generation` |
| bootstrap-built tables (`.local`, conditions, StackFrames, `.ROUTINES`) | `directory_put` -> `insert` -> `write_slot` | `store_writes` |
| a program's local created | `package_local` -> `hash::new_directory` -> `install_store` (`environment.rs:887-896`) | `store_generation` |
| `Package~copy` | `package_locals.insert(copy, local)` (`identities.rs:1237-1238`): a new `ProgramId`, so no memo entry can name it | n/a |
| `~copy` of a Directory | `duplicate_collection_stores` (`object_protocol.rs:750`) allocates fresh arrays for the copy; the original's arrays are not written | n/a |
| collector, sweep, prune | no `Body::Array` slot write outside `hash.rs:497`, `collection.rs:397` (queue store), `array/surface.rs:209` (Rexx `Array`), `object_protocol.rs:1346` (message parties) | n/a |

Inside `hash.rs` and `hash/relation.rs` every slot write is `write_slot`; no function there answers a
store's `indexes` or `items` array as a value (grep for `store.indexes`/`store.items` outside
`slot_at`/`write_slot` is empty), so a store array never reaches Rexx code as an `Array`.

Suspected gaps, one probe each (`/tmp/claude-1000/p9/ffr1/`), oracle and crate identical:

* A `MutableBuffer` index mutated after `put` (`mb.rex`): `.QQY`, `.QQY`, `The NIL object v 1`
  on both. The stored index does not follow the buffer, so a key cannot change without a slot write.
* Reaching the store's pool variables (`HASHINDEXES`, `METHODUNKNOWN`) from Rexx and writing them as an
  `Array`: `.Table~define` is 98.985 on both (`peek2.rex`); a `setMethod` method's `expose hashindexes`
  sees an unset variable on both (`run3.rex`: `set The String class`).

So a release build without the debug assertion answers the same as a probe on every path above.
Minor (m1) records what the assertion does not cover.

## 2. I5: behaviour, and the table

Each change, and whether it changes what is rooted, cached, or when it is invalidated:

* `Table::released` boxed behind `Option<Box<Released>>` (`rexx-api/src/handles.rs`): `register`,
  `resolve`, `release`, `clear` give the same answers; an absent box behaves as an empty map. No change.
* `Activation::constants` registers through `locals.register` and skips `register_local`'s
  `pop_temp_above` (`values.rs`). The four constants are `ObjRef::NIL`, `counted(1)`, `counted(0)`,
  `text(b"")` (`library.rs:1409-1415`), all immediate values with no heap object. The old pop could only
  remove a top temp equal to one of them. No rooting change.
* StackFrame entries as a fixed array with `EXECUTABLE` dropped when `None` (`context.rs:674-690`): the
  same entry set as the `Vec` + conditional push. `reserve_entries` only pre-sizes. No change.
* `array_argument_object`: `single(dimensions)` is `len == 1`, the negation of
  `is_multi_dimensional_array`'s `len != 1` (`value.rs:488-492`). The early return pushes the same
  temp as before. The non-array path is unchanged. No change.
* `array_argument`: slices the borrowed slots to `message_argument_count` instead of copying and
  truncating. Same vector.
* `native_hash_unknown` reads slot 0 of the array object: the old `forwarded.first()` after truncation
  is `Some(Some(x))` exactly when slot 0 is `Some(x)`. Same answers and refusal.
* `package_local_get` is a copy of `directory_get` without its `debug_assert_eq!` on view reuse (m1).

The table: every `head0 d%` and `head d%` recomputes from the quoted Ir columns to four decimals,
and the columns equal `/tmp/claude-1000/p9/ff1/cg-full.txt` (`exit 0`, spreads under 0.001%). After
`99d6e56a1`, the only non-comment changes to `rust/crates` are the M5 refusal string in three places,
so the measured binary stands for HEAD's code.

## 3. I1, I2 witnesses (stdout read)

From `/tmp/claude-1000/p9/ffr2/`, crate and oracle byte-identical apart from the rc line label, both rc 0:

* `package_local_find_class.rex` (oracle `LD_LIBRARY_PATH` on both): `dot: The Array class`,
  `findClass: The Array class`, `findClass absent: The NIL object`, `method: The List class`,
  `routine: The Array class`, `lib local: The List class`. `method:` reaches `class_in_program` through
  `orxmethod`'s `TestFindContextClass` (`surface.rs:544`) with `BAR` only in `lib.cls`'s local. So the
  witness reaches both I1 readers and `Package~findClass`.
* `executable_subclass_copy_annotations.rex`: `one 1 two 2` / `0 2 two` / `x 1 0` / `one two`. Lines 1-3
  are `Method` and `Routine` subclass copies (`duplicate_executable`); line 4 is the base `Method` control.
* `package_local_symbol_writes.rex` (M6): 13 lines identical. It never repeats a miss of one name with no
  write between, so it exercises invalidation and never a memo hit (m1).

## 4. Records

* I3: `phase-9a-gate.md` Task 5 sentence, samples paragraph and carried list now give the harness
  attribution. Re-run (`/tmp/claude-1000/p9/ffr4/`): with `LD_LIBRARY_PATH`, `makestring.rex` ends at
  `rexx-exec: routine "SYSTEMPFILENAME" is not implemented (Phase 9)`, rc 120. `table.tsv:766,768`
  RxMath 289/289 and rxregexp 33/33 crate rows, `group_runner.rs:518-521`, and
  `scclient.testGroup:86-96,194-195` match the citations. `suite-9d-set.txt:206-208` still lists the
  three tests, which the record says ("All three are in the derived 9d set").
* Gate line 48: cites `check_phase_10`, `suite_rows.rs:531-536`. Those lines are the `refusal_carries`
  arm. Correct.
* M3: `sim-gate.tsv` header points at `task-10-sim-report.md`, which is tracked.
* Deviation 32: the new wording ("did not build, or one an earlier `~subclass` already rewired") holds
  on both halves. `metaheld.rex` reproduces `XX Object 1 XX Object` on both. For the second half,
  `ffr3/metatwice.rex` (the same held class answered to a second `~subclass`) gives crate rc 120 with
  the new refusal text and oracle `XX Array 1`, rc 0. That half is not in the pinning test (m4).
* M7: `deviation-probes/` holds `objname-table`, `objname2`, `objname4`, `ord` and `metaheld` with
  both sides' out/err/rc. Re-run here (`ffr3/`): stdout and rc identical to the committed files for
  `objname-table`, `objname2`, `ord`, `metaheld` on both sides. Deviation 31's table equals
  `objname-table.oracle.out`. Deviation 33's `enh:` orders equal `ord.{oracle,crate}.out`. One label
  is off (m3).

## 5. The pingmsg libc concern

The figure is in the data: `pingmsg.*.row`, the same in all three rounds:

| binary | total Ir | libc Ir | subtracted |
|---|---|---|---|
| pin | 7,582,627,478 | 5,914,471,369 | 1,667,809,923 |
| ff3d45c51 (head0) | 10,773,664,932 | 9,070,779,961 | 1,702,538,958 |
| 99d6e56a1 (head) | 7,488,740,213 | 5,871,131,096 | 1,617,263,106 |

Pin to head0, libc rose by 3.156e9. That is 41.6% of the pin's whole-process total and 29.3% of
head0's. libc is 84.2% of head0's pingmsg total. At the measured HEAD the rise is gone: libc is
4.3e7 *below* the pin and the whole process is -1.24%. The concern is true of `ff3d45c51` only. As
written ("the budget does not see it, but wall time does") it reads as an open cost at HEAD, and
it is not one. That a 3e9 swing comes and goes with allocation pattern also makes pingmsg's wall time
a poor instrument.

## New findings

### N1 (Important). extcall: the native-frame `memcpy` the report names as the cause is still there, hidden by the libc-subtracted table

The report gives extcall's cost as "+237 Ir per call", with the frame `memcpy` "+100 Ir a call", and
says boxing the released map fixed it. Both figures are whole-process: (9,687,289,610 -
8,975,965,522) / 3,000,000 native calls = 237.1. But the table that shows +0.28% subtracts libc, and
`__memcpy_avx_unaligned_erms` is libc. `callgrind_annotate --tree=caller` on `extcall.*.r1.cg`, memcpy
called from `push_native_frame`/`take_native_frame`/`pop_native_frame`:

| binary | memcpy Ir from the native-frame moves | vs pin |
|---|---|---|
| pin | 1,110,000,000 (378.0M + 3x192M + 156M) | |
| head0 | 1,410,000,000 (450.0M + 5x192M) | +100 Ir/call |
| head | 1,374,000,000 (414.0M + 5x192M) | +88 Ir/call |

Head still has a fifth 192M memcpy site, `library.rs` `push_native_frame`, that the pin has none of.
Whole process, extcall is +3.045% at HEAD against the pin: total Ir 9,249,295,682 vs 8,975,965,522,
+91 Ir per call, of which libc is +2.52e8 (+84 Ir per call). The other whole-process deltas at HEAD
are all under +0.51% except pkglocal and pingmsg, which improved (dirread +0.504%, the next highest).
So I5 is closed on the ruled instrument (`callgrind.sh`, libc subtracted), not on the one I5 was raised
with (`perf stat` whole process, +6.03%). The report's "Fix" sentence is not supported by its own data.
This is the same blind spot as the report's concern 3, and the report does not mention it for
extcall. Not re-ruled here; the controller decides whether the budget instrument stands.

### Minor

* **m1.** The memo's only correctness check is the `debug_assert!` on a hit
  (`environment.rs` `package_local_symbol`), and no recorded gate runs it. The witness never repeats
  a miss (section 3), `pkglocal` runs release, and the report's checks are all `--release`.
  `package_local_get` (`hash.rs:1740-1755`) also drops `directory_get`'s `debug_assert_eq!` that a
  reused view equals a fresh `read_view`. That assertion is what guards the cached `package_local_view`.
* **m2.** `class_in_program` (`environment.rs:593-597`) matches only `Ok(Found)` from
  `package_local_entry`. It drops an `Err` (e.g. an `UNKNOWN` method on a package local that raises)
  and an `Owed` entry, and falls through to `.local`. `package_find_class`
  (`identities.rs:741-748`) propagates both. Not probed.
* **m3.** The README says `objname-table.rex` is "Deviation 31's table, one line per row". Its fourth
  line is an internal routine (`r: procedure`), but the table row reads "the same literal in a
  `::ROUTINE`". `objname4.rex:10,25-26` (`::routine other`) witnesses the `::ROUTINE` row, oracle `long`.
* **m4.** `a_metaclass_new_answering_no_new_class_is_loud` (`dispatch/tests.rs:1858`) pins
  "did not build" (`.array`, `.Foo`) but not "a subclass already took". `metatwice.rex` above shows the
  refusal fires.
