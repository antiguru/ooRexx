# Phase 9a Task 6 re-review (6dc1c956c..9e8da28de)

Verdicts
* I1 (.ROUTINES writes): addressed. Built HEAD (`Compiling rexx-exec` seen), target dir and probe dir deleted.
* Minor 1 (oracle.rs Deviation 8 wording): addressed. `grep -rn "Deviation 8\|DEVIATION 8"` over rexx-exec/tests and src is empty.
* Minor 2 (package_tables.rex sentence): addressed. Paragraph deleted in the corpus file and the sourceline copy; the copy with its `count` line stripped is byte-identical to the corpus file; count 101 -> 92 matches the 9 deleted lines (1 blank + 8 text); the comment now ends `agrees byte for byte too. */` and nothing else in the file mentions seven bytes, the workaround or the found-defect record.
* Minor 3 (rooting): addressed as "no change needed". `native_instance` does `push_temp(object)` (environment.rs:1212), called inside the loop's frame at environment.rs:1078; `push_temp(value)` follows at ~1096 and the frame pops after the loop. True.

(a) Mutator coverage. Probes (rexx-run, release, memcap 2G timeout 20), each followed by `call foo` over `::routine foo`/`::routine bar`:
* rc 120 with the Phase 9 refusal: `send('PUT')`, `sendWith` of PUT, SETENTRY, REMOVEENTRY, EMPTY, REMOVEITEM, `[]=`, REMOVE, PUTALL, `FOO=`; `~putAll(directory)`, `~putAll(stringtable copy)`, `~foo =`, `~unknown('FOO=',...)`, `~~put`, and `remove` after a `DO OVER`.
* Silent, `after: foo`, and correct (no write): `merge`, `removeAll`, `replace`, `alternate`, `hashcollection`, `setMethod`/`unsetMethod` (StringTable lacks them, so they fall to `unknown`, which reads), `.routines~copy~put(...)` (a copy, not the table), `supplier`, `items`.
* `Package~addRoutine('FOO', .routines~bar)` is allowed and then `call foo` answers `bar`: intended (ruling and the test's adjacent-success list).
* No gap found. Reflective `~method('PUT')~run` shapes I tried failed with Error 97 on both sides' private-message rule before any write, so they did not test it; the guard sits on the native bodies themselves, which any such route reaches.

(c) Replacement text consistent with regenerated expectation: yes (see Minor 2).

New findings
* Minor: `refuse_routines_write` runs `is_package_routines_table`, a linear scan of `package_tables`, on every Directory/StringTable `put`, `[]=`, `remove`, `empty` etc. (hash.rs `store_put` 1245, `store_entry_write` 1794, `native_put` 1619). The map is empty or tiny for ordinary programs, so the cost is small, but it is a per-write cost on a hot path in a phase that measures nothing against the pin. Check `package_tables.is_empty()` first, or measure at Phase 9 close.
* Minor: the report's `Concerns` says `Error 35.1 ... &1` is pre-existing for `call .routines~put(...)`; unverified here, harmless.
No Critical or Important findings.
