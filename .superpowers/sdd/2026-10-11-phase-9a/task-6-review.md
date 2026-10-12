# Phase 9a Task 6 review (BASE c515bac3a, HEAD 6dc1c956c)

**Spec verdict: PASS.** Both brief items are built and witnessed: condition objects are store-backed
Directories keeping identity under re-raise, and `.methods` (with `.routines`, `.resources` through
the same builder) are store-backed StringTables. The remaining `not_this_task` stores are listed in
the report and match the Task 8b ruling. Deviation 8 is withdrawn in Deviation 5's form as ruled.

**Quality verdict: PASS WITH ONE IMPORTANT FINDING.** The `.ROUTINES` table is now writable but its
writes do not reach routine resolution. Before this task a write was refused loudly; now it diverges
silently.

## What I ran

Binary built from HEAD (`cargo build --release -p rexx-exec --bin rexx-run`, `Compiling rexx-exec`
seen), target dir deleted afterwards. Oracle runs used the standard wrapper from fresh dirs under
`/tmp/claude-1000/p9/t6-rv-*`. I read `oracle-crashes.txt` first. Entry 22 (TRACEBACK holding a
non-string on `~result` re-raise) is the neighbour, and no witness touches TRACEBACK.

| program | oracle rc | crate rc | stdout | stderr |
|---|---|---|---|---|
| `lang/condition_object_hash.rex` | 0 | 0 | same | same |
| `lang/package_tables_hash.rex` | 0 | 0 | same | same |
| probe `rt.rex` (`.routines` writes, below) | 213 | 0 | differ | differ |
| probe `rs.rex` (`.resources~remove`, then `~resources`, `~resource`) | 0 | 0 | same | same |
| probe `d.rex` (`.directory~define('INIT',...)`, then a trapped raise) | 0 | 0 | same (98.985 both) | same |

I read the witness stdout. Every claimed path prints: `items` 14, the allIndexes order line,
`hasIndex` case sensitivity, `at`/`[]`/entry method, `CONDITION('O')` copy identity `0`, a copy write
not reaching the original (`15 14`), re-raise identity `(c3 == c)` `1` with POSITION rewritten to
the `~result` line (`20`), CALL ON USER (10 indexes), NOVALUE (9), RAISE PROPAGATE (`1 14`), DO
OVER. The package-tables witness prints the `.methods` order, `.routines` before and after
`addRoutine`, `.resources`, and a `.methods['EXTRA'] = 5` write read back.

## Focus answers

1. **GC rooting.** `condition_put` roots the value, and `directory_put` roots the index. The object
   is pushed in `reraise_condition_object` (`condition.rs:158`). `store_directory` roots the fresh
   Directory before any put. Entry values were already rooted as built (`condition.rs:357`).
   `build_package_string_table` roots the table globally before filling it and pushes each value
   before `directory_put` (`environment.rs:1095`). `package_table_entries` keeps only
   `store_indexes` ObjRefs taken from a globally rooted table, and `to_text` does not allocate on a
   string. Both witnesses are in `phase-9.txt`, and `collect_stress.rs:66` runs that list under
   collect-on-every-allocation (report: 37 passed; I did not rerun it). Temps pushed outside a frame
   at the SIGNAL re-raise site: a loop of 2000, 20000 and 200000 `~result` re-raises trapped by
   SIGNAL ON SYNTAX in a procedure ran at maxrss 20992, 26280 and 26492 kB. That is flat, so no
   growth per re-raise. Command: `memcap 2G timeout 200 /usr/bin/time -f ... rexx-run l.rex N`.
2. **Identity under re-raise** is witnessed (`(c3 == c)` 1, `(condition('O') == c)` 0), and the
   oracle agrees.
3. **Iteration order.** Every key in both witnesses is a string. No witness depends on an
   identity-keyed order.
4. **Deviation 8.** The withdrawal matches Deviation 5's form. `HASH_ORDERED_STDOUT` and corpus.rs's
   `stdout_mode` are gone (`grep -rn` over `rust/` and `docs/superpowers/plans`). The `stdout_mode`
   hits left are the `support/oracle.rs` parameter that the ruling keeps. `suite_rows.rs:377`
   already skipped `WITHDRAWN` headings, so that is pre-existing behaviour, not a change.
5. **Witnesses:** see the table above.
6. **Behaviour change:** see Important 1. `.methods` writes agree with the oracle (`.methods` is
   the live `unattachedMethods` table there, `RexxActivation.cpp:2867`), and `definedMethods`
   reflects a put on both sides (`1` then `2`).
7. **Prose:** no em-dashes or set sizes in the added lines. Two stale sentences are listed under
   Minor.

## Critical

None.

## Important

1. **`.ROUTINES` writes are kept but not used for CALL resolution.** In the oracle, `.ROUTINES` is
   the package's live `routines` table (`RexxActivation.cpp:2876`, `PackageClass.hpp:152`), and
   `findLocalRoutine` reads it (`PackageClass.cpp:822-832`). The crate now stores the write
   (`environment.rs` builder, `directory_put`) but resolves `CALL` elsewhere. Probe:

   ```rexx
   .routines~put(.routines~bar, 'FOO')
   call foo
   say 'after put:' result
   ...
   .routines~remove('BAR')
   call bar
   ::routine foo
     return 'foo'
   ::routine bar
     return 'bar'
   ```

   The oracle prints `after put: bar`, then at the `call bar` gives `Error 43.1: Could not find
   routine "BAR"`, rc 213. The crate prints `after put: foo` and `after remove: bar`, rc 0. The
   oracle's error is on stderr; on the crate's side the mismatch shows up in stdout and rc.

   At BASE, every hash method on the `Body::Native` StringTable went through `owns` returning false
   and then `not_this_task` (`hash.rs` `refusal_outside`), so `PUT`/`REMOVE` refused loudly. I
   derived that from reading the code at c515bac3a and did not run it. This task therefore turned a
   loud refusal into a silent wrong answer, and no witness covers it. Two fixes are possible.
   (a) Route `CALL`/`findRoutine` resolution through the store, which is a new identity rule and
   so, under the batch rule, a controller decision. (b) Until then, keep `PUT`/`REMOVE`/`[]=` (and
   `setMethod`/`empty`, if reachable) on `.routines` loud. Either way, add `rt.rex`'s shape as a
   witness. `addRoutine` already writes the table on purpose and is witnessed; whether `~addRoutine`
   makes the routine callable by `CALL` is the same question and goes into the same witness.

## Minor

1. `rust/crates/rexx-exec/tests/support/oracle.rs:492, 723, 727, 752, 757, 774` still call the
   multiset mode "Deviation 8's licence" and its controls "DEVIATION 8's control", in the present
   tense. Deviation 8 is withdrawn, so these name a licence that no longer exists. Reword them as
   the multiset mode's own controls, or point them at the withdrawn row.
2. `rust/corpus/lang/package_tables.rex:20-25` (and its `sourceline_oracle` copy) says the DO OVER
   rooting defect "reproduces with `.methods` alone", in the present tense. `.methods` now has a
   store and iterates through `MAKEARRAY`, not `hash_collection_indexes`
   (`object_operand_tests.rs:254-263` says so), so the sentence is probably false now. I did not
   run it. The report's reason for keeping it ("history") is not what the sentence says.
3. `environment.rs` `build_package_string_table`: the `native_instance` method or routine object
   stays unrooted across `attach_annotations` → `annotation_table` (`identities.rs:611`), which can
   allocate, until the `push_temp` after it. This was already the case at BASE (the old code also
   stored the value only after `attach_annotations`), so it is not introduced here. Neither witness
   has an `::ANNOTATE` on a method, so collect_stress cannot see it. Rooting the object right after
   `native_instance` closes it.
4. `environment.rs:1095-1098`: when `directory_put` fails, the half-filled table has already been
   cached in `package_tables` and registered as a global root, so the next read answers a partial
   `.methods` without error. This is unlikely, since only allocation failure reaches it.
5. The exclusions text says the store "iterates in the oracle's own bucket order" as a general
   property. It is witnessed only for the keys in the two programs and `package_writes`. String-key
   order reproduces in the oracle, so the claim is plausible, but its extent comes from a sample.
