# Phase 9a final review, 180e5b41e..ff3d45c51

Reviewer: final whole-branch review (opus), 2026-10-13. Read-only; nothing committed. All probes ran from
fresh directories under `/tmp/claude-1000/p9/fr*/`. "Base" below is a release `rexx-run` built at
`180e5b41e` (`/tmp/claude-1000/p9/fr-base/target/release/rexx-run`, `Compiling rexx-exec` in its log);
`git diff --stat a98238be5 180e5b41e -- rust/` is empty, so it is the perf pin's source. "Head" is
`rust/target/release/rexx-run` (built Oct 12 23:30; `rust/` unchanged since but for `corpus/suite-9d-set.txt`).
Instruction counts are `perf stat -x, -e instructions:u` (user instructions, whole process, counting
mode; repeated runs agree to 7 significant figures). They are not callgrind and not the close's
instrument; they say where to look.

Counts: Critical 0, Important 5, Minor 7.

## Important

### I1. `Package~findClass` no longer sees a package's `~local` entries (regression, silent wrong answer)

`rust/crates/rexx-exec/src/environment/identities.rs:741` reads the package local with
`native_map_entry`, which answers only for a `Body::Native` (`environment.rs:1285`). Task 8b
(`acd4aac65`) made `Package~local` a store-backed `Directory` (`hash::new_directory`,
`environment.rs:825`), so the lookup always misses. `.NAME` (`dot_variable`) was switched to
`hash::store_get`; this reader and `Interp::class_in_program` (`environment.rs:580`, the native API's
class lookup from `dispatch/library/surface.rs:544`) were not.

```
l = .context~package~local
l~put(.array, 'FOO')
say 'dot:' .foo
say 'findClass:' .context~package~findClass('FOO')
```
`/tmp/claude-1000/p9/fr2/pl.rex`: oracle and base `findClass: The Array class`; head `findClass: The NIL
object`, rc 0 on all three. No test or corpus program reaches it (no group went green to red).

### I2. A copy of a `Method`/`Routine` subclass instance shares its annotation table's store with the original

`rust/crates/rexx-exec/src/dispatch/object_protocol.rs:792` (`duplicate_executable`) copies the
annotation table by cloning its `Body` (`alloc_with(BehaviourId::OBJECT, body)`). Since Task 8b the table
is a store-backed `StringTable` (`hash::new_string_table`), so the clone holds the same hash-store
arrays; `duplicate_collection_stores` is not run on it. A base `Method` copy sends `COPY`
(`copy_method_object`) and is right.

```
m = .MyMethod~new('m', 'return 1'); m~annotations['A'] = 'one'
c = m~copy; c~annotations['A'] = 'two'; c~annotations['B'] = 'bee'
say m~annotations['A'] m~annotations~items c~annotations['A'] c~annotations~items
::class MyMethod subclass Method
```
`/tmp/claude-1000/p9/fr2/ann.rex`: oracle `one 1 two 2`; head `two 2 two 2` (the base-`Method` line agrees).
Same answer in `/tmp/claude-1000/p9/fr6/s4_methsub.rex` (oracle `x30 y 60`, head `y y 60`).

### I3. The 98.903 attribution for `samples` TEST_MAKESTRING and `scclient`/`sfclient` TEST_04, and the ruling built on it, were measured outside the harness's environment

`phase-9a-gate.md:193-195` and `:405-409`, `progress.md:175`, and the carried item at
`phase-9a-gate.md:457-461` say the three tests end in 98.903 loading `rxregexp`/`rxsock`, "Phase 10
Extension libraries", and rule that 9b's first item makes loading one a loud refusal. The
reproduction ran `rexx-run` without `LD_LIBRARY_PATH`; the suite sets it on the crate side
(`support/group_runner.rs:518-521`) and children inherit it (probe `fr4/parent2.rex`, `parent3.rex`).
With it:

* `makestring.rex` loads `rxregexp` and stops at `rexx-exec: routine "SYSTEMPFILENAME" is not
  implemented (Phase 9)`, rc 120 (`/tmp/claude-1000/p9/fr5/`, `LD_LIBRARY_PATH=.../build/lib rexx-run
  makestring.rex < in.txt`; oracle rc 0, 37 lines). That is Task 5's original attribution, which the
  gate record "corrected".
* `scclient`/`sfclient` TEST_04 run the server and clients as `rexx scserver` / `rexx scclient` after
  `address '' 'cd' samplePath`. Head: `rexx-run: scclient: No such file or directory (os error 2)`, rc 2;
  oracle finds `scclient.rex` (`fr4/cd.rex`). The cause is the front end's program-name resolution
  (9c's `rexx`), not a library.
* Extension libraries load under the harness today: `suite-close/table.tsv` has the crate passing
  `extensions/rxmath/RxMath` 289/289, `extensions/rxregexp/rxregexp` 33/33,
  `extensions/rxsock/socketClass` 20/23, `rxunixsys/SysUnix` 27/28. The ruled refusal would turn those
  passes into refusals and would not fix any of the three tests.

### I4. Pure-Rexx `yaml.cls` reaches a compiler GUARD; the Phase 10 Extension group row hides it

`suite-close/table.tsv:772`: `extensions/yaml/yaml` crate TEST35_DIRECTIVES and
TEST47_MULTILINEFLOWCOLLECTIONS blocked by `a compiled a pass boundary with no loop open op has no driver
arm` and `a compiled Store op has no driver arm`. `yaml.cls` has no `LIBRARY` requirement; the refusal
is `Loud::op_not_driven` (`run/loops.rs:1860`), whose `refusal-dispositions.tsv` row is GUARD
("driver invariant: every op the compiler emits has a driver arm"). Five lines reproduce it, with the
oracle's `build/bin/yaml.cls` beside them (`/tmp/claude-1000/p9/fr3/y47.rex`):

```
p = .Yaml~new
yaml1 = '---' || "0A"x || 'key: [a,' || "0A"x || '  b, c]' || "0A"x
doc1 = p~parseString(yaml1)
say doc1["key"]~items
::requires 'yaml.cls'
```
Oracle `3` rc 0; head and base `rexx-exec: a compiled a pass boundary with no loop open op has no
driver arm` rc 120. Pre-existing at `180e5b41e`; 9a exposed it (baseline: crate started 0 yaml tests).
Under `::options trace a` the refusal fires at `blockMap`'s `do while` end after `mapValue` →
`flowSeq` returns (`fr3/tr/out.txt`). The suite gate is green on the group by its `group` rows, the 9d set
excludes rowed tests, and the gate record files it as a refusal "in a Phase 10 Extension group" with
no owner, so nothing owns a core driver defect.

### I5. Three bench programs are over the Phase 9 budget against the pin's source, from tasks that measured nothing on them

Base vs head, every program of `callgrind.sh`'s set but `heapshape`/`rexxcps` (`/tmp/claude-1000/p9/fr7/perf.txt`):
`extcall` +6.03%, `pingmsg` +2.85%, `dirread` +1.23%; everything else within +0.33% or improved
(`dispatchclass` -3.58%, `fibcall` -2.80%). Bisected over binaries built at each task's base
(`/tmp/claude-1000/p9/fr-bis/measure.txt`, `fr-bis/bins/`):

| program | owning task | delta | that task's own measurement |
|---|---|---|---|
| extcall | Task 9 main (`7d7036e3a`, `ccd4d4f02`; bin `6e0395fdf` vs `5d12347c0`) | +5.95% | none; `progress.md:159`'s +0.0754% is the fix round alone |
| pingmsg | Task 5 (`c515bac3a` vs `6a8c461fd`) | +2.74% | none |
| dirread | Task 5 | +1.11% (T4 +0.14%, T8b +0.16%) | none |

The budget is measured at the Phase 9 close (ruled), so this is a pointer, not a re-ruling; but
`progress.md:145`'s premise that each task stayed under +0.5% is false for Tasks 5 and 9. Commands:
`/tmp/claude-1000/p9/fr-bis/build.sh SHA...`, `/tmp/claude-1000/p9/fr-bis/measure.sh BIN...`.

## Minor

* **M1. False prose after Task 8b's store-backed tables.** `dispatch/hash.rs:160-162` ("a `Directory`
  built on `NativeObject`'s map, such as `Package~local`, has no store"), `hash.rs:189`, `dispatch.rs:1517`
  (`Primitive::Directory` "such as `Package~local`"), `dispatch.rs:1520` (`Primitive::StringTable` "an
  `~annotations` table"), `run/loops.rs:829`. Both are `Body::Instance` with a store since `acd4aac65`
  (`environment.rs:825`, `identities.rs:596`). I1 and I2 are this prose's failure mode in code.
* **M2. ` -- ` in comments added in the range:** `options.rs:371`, `dispatch/hash.rs:162`,
  `dispatch/collection/supplier.rs:87` (`lib.rs:1428` is deferred, `progress.md:167`).
* **M3. Stale records about the sim gate.** `phase-9a-gate.md:451-452` carries "`sim-gate.tsv`'s stale
  rest rows for Method and METHOD"; `0ad23af25` removed every rest row (`grep -c rest
  rust/corpus/sim-gate.tsv` matches only the header). `sim-gate.tsv`'s header still sends the reader to
  `phase-6-1-gate.md` for calibration, which no longer holds the re-derived rows (they are in
  `task-10-sim-report.md`).
* **M4. `phase-9a-gate.md:48`** "A Phase 10 refusal row's check accepts a refusal naming Phase 10" is
  the Task 3 interim; since Task 4 `check_phase_10` requires the D3 reason (`suite_rows.rs:531-536`).
* **M5. Deviation 32's and `Loud::metaclass_new_reused_class`'s "a class it did not just build"**
  (`lib.rs:528-537`, `phase-4-exclusions.txt` Deviation 32). The check is `ClassGraph::is_cloned`
  (built by `Class~new`, not yet reparented), not "just built": a class `Class~new` built earlier and held
  elsewhere is rewired, as on the oracle (`fr4/meta3.rex`: both `XX Object 1 XX Object`, rc 0). The
  behaviour is right; the text over-claims.
* **M6. Every `.NAME` lookup allocates once any package local exists.** `environment.rs:532` +
  `package_local_entry` (`:761`) build a string and push a temp per lookup. `fr1/a.rex` (`~local`, then
  2,000,000 `.stdout`): base 4.056e9, head 7.458e9 instructions (+84%); without `~local` +0.1%. No
  bench program touches `~local`.
* **M7. Deviations 31 and 33 cite probe output not in any committed record** (`objname2`/`objname4`
  table; `ord.rex`'s two orders). Re-run here, Deviation 31's table reproduces on the oracle
  (`fr9/on.rex`).

## Hot-path list (item 4)

Per-clause/send/call/allocation work added in the range, with the figure the task recorded and, where
taken here, the base-vs-head instruction delta.

| path | function(s) | task | task's figure | here |
|---|---|---|---|---|
| per native call (library) | `Table::register` (`released.remove`), `Host::register_local` (`pop_temp_above`), `root_native_answer`, `collect_under_stress`, `NativeFrame::temps_floor` | 9 | +0.0754% extcall (fix round only) | extcall +6.03% (+5.95% at Task 9 main) |
| per `~start` / message send | likely `context::build_frame_with` → `executable_at` per frame (the spawner's caller frames), `started_message` | 5 (+8b) | none (8b: pingmsg +0.46% vs `823db3ab0`) | pingmsg +2.85% (+2.74% at Task 5) |
| per Directory `UNKNOWN` send | `native_hash_unknown` → `array_argument` → `array_argument_object` (temp push, conversion check); also `native_stem_unknown`, `native_reference_unknown` | 5 | none | dirread +1.11% at Task 5 |
| per `.NAME` | `dot_variable` `package_locals.is_empty()`; with a local, `package_local_entry` allocates (M6) | 8b | dirread +0.146% | dirread +0.16% at 8b; +84% with a local |
| per return | `RootSet::add_global` as `NameMap` (`root_exit_value`) | 8b | fibcall -2.05%, dispatchclass -2.58% | fibcall -2.80%, dispatchclass -3.58% |
| per hash put/remove/empty/setEntry/`[]=` | `note_routines_write` → `package_routines_program` linear scan of `package_tables` | 6/8b | none (deferred `:96`) | `fr2/dput.rex` +0.18%; +0.21% more after 400 `Package~copy` |
| per hash collection `new` | `native_capacity_init` → `initialize_store` → `holds_store_class` (`is_a` over `store_holders`) | 8 | hashnew +0.41% | hashnew +0.33% |
| per heap-string text read / conversion | `redirect_of` → `instance_redirect`, `reference_redirect` (out of line); `try_text`, `text_len_inner`, `to_number` instance arms; `classify_string_conversion` → `instance_conversion` | 8 | strings -0.17%, rexxcps -0.085% | strings -0.08% |
| per String message-path native | `base_string`/`subclass_text` in `numeric_receiver`, `string_arithmetic_operator`, `concatenation_or_this`, `native_string_op_not`, `string_d2x_d2c`, `native_string_make_string`; `string_changestr` pre-scan | 8 | as above | — |
| per send to `.nil` / per SUPER | `nil_behaviour` arm in behaviour lookup; `super_scope_in` `scope == NIL` compare | 8b | +0.0000% send programs; -0.05..-0.07% | dispatch -0.05% |
| per array string value | `heap.object_name` check (`value.rs:408`) | 8b | — | — |
| per collection | `Heap::prune_names` (when names exist); store-backed annotation tables traced | 8b | decloop +0.12%, parse +0.13% | decloop +0.13% |
| per Rexx `CALL` resolution | `package_routine_lookup` `routines_written.is_empty()`; when set, `find_written_routine` (Vec lineage, linear `native_entry`) | 8b | — | — |
| per message entry remove / instance entry read | `remove_native_entry` computes `message_entry_scope` before matching; `native_entry` instance arm | 5 | — | — |
| per raised SYNTAX condition | `build_condition_object_from`: store-backed directory, `sort_by_key` with linear `position` key; `build_frame_with` → `executable_at` | 6, 5 | none (deferred `:86`) | `fr2/cond.rex` (50,000 trapped 41.1): 3.820e9 → 6.789e9 (+78%); `RAISE USER` loop +0.3% |
| per program start | `install_store` for annotation tables, `builds_in_place` per `::CLASS`, 9a's own startup | 6, 8b | startup +0.141% | startup +0.28%, sayloop +0.19% |

## Passes done

1. **Stale derived state.** Re-derived `suite-9d-set.txt` from `suite-close/tests.tsv` by its header
   command: identical; diff against the first derivation matches the gate record's lost/gained list.
   Recounted row files per owner/reason at HEAD: equal to the gate record's table. Re-ran the gate
   record's table/verdict/regression commands: all figures match (22630/248/125, 340/62, 16 red→green,
   0 regressions). Checked `sim-gate.tsv` against `task-10-sim-report.md` (M3). Checked rowed tests the
   crate fails or is blocked on for non-Phase-10 refusals (I4). Re-ran the samples attribution under
   the harness's environment (I3).
2. **Checks blind to their subject.** Read `l3_core_suite`, `judge`, `the_row_files_hold_only_checked_rows`,
   `check_row`/`check_phase_10`, `refusal_carries`, `phase_ten_reasons`, `release_local_reference`,
   `instance_copy_memory`, the DIFFERING cross-check's early return. Ran `release_local_reference`
   (3 passed; RelLoop timed directly: head 32172 KB at 2,000,000 vs base 430776 KB, so it measures its
   subject). The four refusal rows' texts equal the captured refusals in `suite-close/tests.tsv`.
   Blind spot found: a Phase 10 `group` row makes every crate outcome in the group invisible,
   including GUARD refusals (I4).
3. **GC rooting.** Read `copy_package`, `copy_package_tables`, `copy_method_object`,
   `clone_executable`, `native_as_instance`, `native_enhanced`/`method_table_pairs`/`drain_supplier`,
   `new_subclass`, String/Stem subclass constructors and copies, `rexx_routine_executable`,
   `Heap::names`/`prune_names`, `handles::Table::release`/`resolve_live`, `register_local`,
   condition construction. Six probe programs (`/tmp/claude-1000/p9/fr6/s*.rex`: package copies with
   written `.ROUTINES`, enhanced and metaclass `NEW` loops, String/Stem subclass ops and copies,
   Method/Routine/Message subclass copies, stack-frame executables and package locals, object names)
   run identically plain and under collect-on-every-allocation (temporary test file, deleted;
   `git status` clean), 64 to 986 collections each. No rooting defect found.
4. **Hot-path drift.** Read every per-path change; measured the bench set base vs head and bisected
   the three over-budget programs across task bases (table above, I5). No callgrind run.
5. **False prose.** Grepped added lines for ` -- ` and em-dashes (M2); checked comments touching
   store-backed tables (M1), the gate record's figures against `close-run3` logs (all cited counts
   match: 1019 of 1019, suite.rs 50 passed, phase_ten_reasons 5, release_local_reference 3,
   concurrency_tests 53/1, 264.57 s), Deviations 31-33 against probes (M5, M7).
6. **Spec conformance.** Walked spec section 3's list against the gate record's witness table and
   `refusal-dispositions.tsv` (notify GUARD row, SUPPLIER DEVIATION row present); section 7's 9a
   edits (`PHASES`/`OPEN` carry "Phase 9"; `ReleaseLocalReference` row delivered). The close-command
   criterion rests on the controller's flake ruling (`progress.md:172`), still with Moritz. I3 is the
   one gate-record claim the records do not support.
