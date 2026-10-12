# Phase 9a Task 6 report: hash methods on the condition object and the package tables

Base `c515bac3a`. Status: DONE_WITH_CONCERNS. Commits `cecec0830` (code, witnesses) and a second one (Deviation 8
withdrawal, `sourceline_oracle` expectations).

Probes ran from fresh empty directories under `/tmp/claude-1000/p9/t6-probe*` and
`/tmp/claude-1000/p9/t6-w-*`, the oracle under the standard wrapper with `timeout -k 5 20`, the
crate's `rexx-run` (release, `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t6-target`) under
`memcap 2G timeout 20`. `oracle-crashes.txt` was read first; entry 22 (a re-raise whose `TRACEBACK`
holds a non-string) is the nearby hazard and no probe touches `TRACEBACK`. stdout, stderr and status
were compared separately. Every witness's stdout was read on both sides.

## Per item

### 1. The condition object (`Message~errorCondition`, `CONDITION('O')`, re-raise)

Probe `t6-probe1/a.rex` (a `Message~send` whose method divides by zero, trapped by `SIGNAL ON
SYNTAX`, then `c~items`, `allIndexes`, `hasIndex`, `at`, `[]`, entry method, `isEmpty`, `hasItem`,
`index`, `allItems`, `makeArray`, `supplier`, `CONDITION('O')` identity, `~result` re-raise):
oracle rc 0, first lines `The Directory class 14` and
` PROPAGATED ERRORTEXT MESSAGE STACKFRAMES POSITION INSTRUCTION CODE RC CONDITION PACKAGE TRACEBACK PROGRAM ADDITIONAL DESCRIPTION`;
`(o == c)` is `0` (`CONDITION('O')` is a copy), `(c3 == c)` after `r = m~result` re-raised it is
`1`, `c~propagated` is `1`, `c~position` is the `~result` line. The brief's `13` was for another
shape; this shape answers 14. Ours before: rc 120, `method "ITEMS" of class "Directory" is not
implemented (Phase 9)`.

Implemented (`condition.rs`): every condition object is now built by `store_directory` (a
`Directory~new` with a hash store), its entries put in `key::order`, which the native-call path
already used. Re-raise and the `SIGNAL` trap write `POSITION`, `PROGRAM`, `PACKAGE`, `PROPAGATED`,
`INSTRUCTION` through `condition_put` (`hash::directory_put`, rooting the value across the index's
allocation), into the same object, so identity under re-raise is the object's own.
`condition_copy` is `~copy` and `condition_entry` is `~at`; their `Body::Native` branches are gone,
since nothing builds a native condition object any more. The extra probes (`t6-probe3`, `t6-probe4`)
covered `CALL ON USER` (10 indexes), `SIGNAL ON NOVALUE` (9), a `RAISE PROPAGATE` out of a routine
(14, `PROPAGATED` 1), `DO OVER` a condition object, and a write into the copy not reaching the
original (`15 14`); all identical on the three descriptors.

Witness `rust/corpus/lang/condition_object_hash.rex`: oracle rc 0, crate rc 0, stdout identical
(each path above printing), stderr identical (empty).

### 2. `.METHODS` (and `.ROUTINES`, `.RESOURCES`)

Probe `t6-probe2/a.rex`: oracle rc 0, `The StringTable class 5 0`,
` GAMMA ALPHA ZETA DELTA BETA` for `allIndexes` and the same for `DO OVER`, `1 0 The Method class`
for `hasIndex`/`at`, `5 GAMMA 1 1` for `allItems`/`supplier~index`/identity, then `.routines` and
`.resources` counts and order, `addRoutine` reaching `.routines` (` NEWR R1 Q2`). Ours before:
rc 120, `method "ITEMS" of class "StringTable" is not implemented (Phase 9)`.

Implemented (`environment.rs` `build_package_string_table`): the table is `hash::new_string_table`
and each entry is put with `directory_put`, value rooted first. The three kinds share that one
builder, so `.ROUTINES` and `.RESOURCES` became store-backed with `.METHODS`; the brief names
`.methods` only, and this is the same constructor, not a widening of the method set. Readers and
the writer that matched `Body::Native` on these tables now use the store: `package.rs`
`package_table_entries` (store walk) and `~addRoutine` (`directory_put`), `context.rs`'s routine
executable lookup (`native_entry`, which reads either body).

Witness `rust/corpus/lang/package_tables_hash.rex`: oracle rc 0, crate rc 0, stdout identical,
stderr identical (empty).

Both witnesses are registered in `rust/corpus/phase-9.txt`.

## `not_this_task` sites left

None of the call sites in `dispatch/hash.rs` and `dispatch/hash/relation.rs` was deleted: each is
still reached. `Body::Native` `Directory` and `StringTable` receivers remain outside this task:
`Package~local` (`environment.rs` `package_local`), `~annotations` tables
(`identities.rs` `annotation_table`), and the security manager's info directory
(`security.rs` `security_arguments`). Measured: `say .context~package~local~items` is oracle rc 0
`0`, crate rc 120 `method "ITEMS" of class "Directory" is not implemented (Phase 9)`;
`.k~annotations~items` is oracle `1`. Making those store-backed is the same change for three more
constructors and is reported, not done, under the mechanical-batch rule. The unit test
`a_native_row_on_a_receiver_of_another_class_refuses_its_type`, which used a started message's
`errorCondition` as its store-less `Directory`, now uses `.context~package~local`.

## Refusal tables

`REXX_REFUSAL_SITES_REFRESH=1 memcap 8G cargo test --release -j 4 -p rexx-exec --test
refusal_sites`: exit 0, `rust/corpus/refusal-sites.tsv` unchanged (`git diff --stat corpus/` lists
only `phase-9.txt`). One `Loud::unknown_receiver` site in `context.rs` ("a package table this
crate did not build") is deleted; the constructor stays.

## Deviation 8

`lang/package_writes.rex` now agrees with the oracle byte for byte: its tables print in the store's
order, which is the oracle's. `corpus.rs`'s control
`the_sorted_stdout_licence_covers_an_ordering_difference_and_nothing_else` fails because of it
("agrees on stdout byte for byte, so it does not need Deviation 8"); it is `HASH_ORDERED_STDOUT`'s
only entry.

Controller ruling A, applied in the second commit: Deviation 8 is withdrawn in
`docs/superpowers/plans/phase-4-exclusions.txt` in Deviation 5's form (number and heading kept, one
paragraph, body removed). `HASH_ORDERED_STDOUT`, `stdout_mode`,
`the_sorted_stdout_licence_covers_an_ordering_difference_and_nothing_else` and
`the_multiset_stdout_mode_is_selected_for_exactly_the_licensed_list` (which asserted the list
non-empty) are deleted from `tests/corpus.rs`, which now passes `StdoutComparison::Raw`;
`support::oracle::stdout_multiset` and its controls stay. The licence lines in `phase-5c.txt` are
dropped. `suite_rows::deviations` skips a `WITHDRAWN` heading, and `--test suite` passes.

`corpus/lang/package_tables.rex`'s "this crate's sorted walk" paragraph is rewritten in place in
both it and `crates/rexx-parse/tests/sourceline_oracle/package_tables.txt`, keeping the line count
(101). The expectation was then regenerated with the module comment's oracle driver and is identical
to the hand edit. The paragraph below it (a DO OVER rooting workaround citing a record that
reproduced with `.methods`) is history about a native table and was left alone.

**A check missed in the first commit**: `cecec0830` added two `corpus/lang/*.rex` programs without
their `sourceline_oracle` expectations, so `cargo test -p rexx-parse --test sourceline_oracle` was
red there. Both expectations are generated with the oracle driver in the second commit; exit 0.

## Group verdicts

```sh
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/tmp/claude-1000/p9/t6-suite/table.tsv \
  REXX_SUITE_GROUPS=base/class/Message,base/directives/METHOD,API/oo/METHOD,base/class/Package,base/bif/CONDITION,base/keyword/RAISE,base/keyword/SIGNAL,base/directives/ROUTINE,base/directives/RESOURCE \
  timeout 590 memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Exit 101 (6 of 9 groups red, as each is in the baseline). Against
`docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/tests.tsv`, the crate column, no test
is newly not passed. Changed:

| group | test | baseline | now |
|---|---|---|---|
| base/directives/METHOD | TESTPACKAGE | blocked: `ITEMS` of `StringTable` (Phase 9) | passed |
| base/directives/RESOURCE | TEST_RESOURCE_DEFINED | blocked: `ITEMS` of `StringTable` | passed |
| base/directives/ROUTINE | TESTDOTROUTINES | blocked: `ALLINDEXES` of `StringTable` | passed |
| base/directives/ROUTINE | TESTPACKAGE | blocked: `ITEMS` of `StringTable` | passed |
| base/class/Message | TEST_{REPLY,SEND,START}WITH_NOT_ARRAY | blocked: `MAKEARRAY` (Task 5) | passed |
| base/class/Package | TEST_PACKAGE_FINDPROGRAM_REXXTRY | failed | passed (the 9d set's known case) |

Two `ROUTINE` rows changed only in refusal text (`Phase 10` to `Phase 10 Rxapi`, Task 4). Verdicts:
`base/class/Message`, `base/directives/RESOURCE`, `base/directives/ROUTINE` green (baseline red);
`base/directives/METHOD` red on TESTDELEGATE only; `base/class/Package` red on TESTRECLAIM01 (the
60 s hang, Task 9's), TEST_NEW01, TEST_PACKAGE_LOCAL_GET_EMPTY, TEST_PROLOG, as before;
`API/oo/METHOD`, `base/bif/CONDITION`, `base/keyword/RAISE`, `base/keyword/SIGNAL` red on the
baseline's own tests.

## Checks (each read unpiped)

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0 (log shows
  `Checking rexx-exec`).
* `memcap 8G cargo test --release -j 4 -p rexx-exec --lib`: first exit 101, 1083 passed, 1 failed
  (`a_native_row_on_a_receiver_of_another_class_refuses_its_type`, see above); after the fix, that
  test alone exit 0, and the whole binary re-run: exit 0, 1084 passed.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -j 4 -p rexx-exec --test corpus --test
  collect_stress --test refusal_sites --no-fail-fast`: exit 101. `collect_stress` 37 passed,
  `refusal_sites` 5 passed, `corpus` 28 passed 1 failed: the Deviation 8 control above.
* After the ruling: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 4 -p rexx-exec -p
  rexx-parse --all-targets -- -D warnings` exit 0; `memcap 8G cargo test --release -j 4 -p
  rexx-parse --test sourceline_oracle` exit 0; `REXX_CORPUS_GATE=1 timeout 590 memcap 8G cargo test
  --release -j 4 -p rexx-exec --test corpus --test suite --test refusal_dispositions --no-fail-fast`
  exit 0 (corpus 27 passed 1 ignored, suite 50, refusal_dispositions 3).
* `memcap 8G cargo test --release -j 4 -p rexx-exec --test coverage --test trace_oracle --test
  ir_recorded --test closed_phases --test owners --no-fail-fast`: exit 0 (21, 41, 28, 8, 6 passed).

## Concerns

* `RAISE ... RESULT` (not valid on the oracle either) reports `Error 25.907: ... found "&1".` on
  the crate where the oracle substitutes `"RESULT"`: an unsubstituted insert, pre-existing, not in
  this task's scope. Program: `raise user foo description 'd1' result 'rr'`.
* Condition objects are now built by a `Directory~new` send and store puts where they were a map
  write. No benchmark was run; 9a measures nothing against the pin.
* The debug-profile corpus run was not made (beyond the 5-minute per-task budget).
