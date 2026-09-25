# Phase 8 surface, Task 1 -- `.environment` and `.local` answer the whole `Directory` surface

BASE `d7eafeb54`. Written first and appended as the work goes.

Probe harness: `scratchpad/surface-t1/ab.sh PROBE` runs the oracle under the standard wrapper and
`rust/target/release/rexx-run` from fresh empty directories, three descriptors, and prints a
verdict line. Probes live in `scratchpad/surface-t1/probes/`.

## Step 1 -- the blocker, by running

The blocker the Phase 5 records name is found: **membership, then order**.
`docs/superpowers/records/2026-08-17-phase-5a/task-23-report.md:364` ("Attempt 2") kept `Directory`
refusing where `StringTable` iterates because "`.environment` and `.local` are modelled as a subset of
the oracle's ... `do e over .local` iterates **ten** entries on the oracle and **none** here", and
5h Task 4 kept the two `Body::Native` on top of that. Neither holds as a reason today, and the
candidates below are each measured.

### (a) iteration order -- real, and reproducible by the store

Oracle, `say` of `.environment~allIndexes` (`probes/order_env.rex`), identical across runs:
`INPUTOUTPUTSTREAM ALARM ENDOFLINE ... BUFFER LOCAL`, 69 items; `.local`'s is `SYSCARGS INPUT
TRACEOUTPUT DEBUGINPUT STDOUT OUTPUT STDERR STDIN STDQUE ERROR`. Crate at BASE: rc 120,
`method "ALLINDEXES" of class "Directory" is not implemented (Phase 5)`.

`NativeObject`'s entries are a `std` `HashMap` (`rexx-core/src/body.rs:608`), randomly seeded per
process, so it cannot reproduce any order. The store `hash.rs` gives `.Directory~new` can:

* **`LOCAL` is last because it is not in the contents.** `Setup.cpp:1781` installs it with
  `TheEnvironment->setMethodRexx`, so it is a method-table entry and `DirectoryClass::allIndexes`
  appends it after the contents.
* **The contents sit at 69 buckets.** `scratchpad/surface-t1/buckets.py` takes the string hash
  (`RexxString::getStringHash`, which `hash.rs`'s `string_hash` already is) of every observed name but
  `LOCAL` and asks which bucket counts from 1 to 4999 make the observed order non-decreasing in
  `hash % B`: only 1 and 69. 69 is one growth from the default 17 (`HashCollection::expandContents`
  doubles `capacity()`, which is the total of 34 slots, and `calculateBucketSize(68)` is 69). For
  `.local` the same test gives 1 and 17, the default.
* **Confirmed on the oracle, including growth** (`probes/env_geometry2.rex`): a `.Directory~new(68)`
  filled in the observed order, with `setMethod('LOCAL', ...)`, answers `allIndexes` identical to
  `.environment`'s, and stays identical after each of 200 program-added entries is put into both,
  across the growth that forces. `.local`'s observed order put into `.Directory~new` answers
  identically at the default size (`probes/env_geometry.rex`). So the free chain's history does not
  leak into order: the fullness test is a count, and the count is history-independent.

Verdict: not a blocker. Fill the store at the oracle's geometry in the oracle's walk order.

### (b) `.local`'s minted names and `STDQUE` -- real, needs a representation

Crate at BASE, after bootstrap (a temporary in-crate test, removed): `.local` holds **no** entries
until a minted name is demanded; `.environment` holds exactly the oracle's names (none extra, none
missing, `LOCAL` as an ordinary entry). Oracle collision chains at 17 buckets make insertion order
observable in two buckets: `DEBUGINPUT` before `STDOUT` (bucket 10) and `STDIN` before `STDQUE`
(bucket 13). The crate's mint builds all three streams before any monitor, so it would put `STDOUT`
before `DEBUGINPUT` -- a wrong order even once minted.

`STDQUE` is `.RexxQueue~new('SESSION')` (`CoreClasses.orx:1007`), Phase 10's. Any read that
answers or compares its item is Phase 10's; reads of indexes and counts are not (the oracle's
`allIndexes`, `items`, `hasIndex('STDQUE')` do not touch the queue).

Verdict: not a blocker. Mint at the end of the bootstrap, as `LocalServer~initInstance` does before
any program, into slots already laid out in the oracle's order; keep `STDQUE`'s slot owed.

### (c) `EnvironmentModel::unbuilt` never shrinks -- real, a live defect

The model is built at `CoreClasses.orx:55`, before the `.orx` classes install, and the map is
computed then. The temporary test lists `ALARM`, `VALIDATE`, `PROPERTIES`, `STREAM`, `FILE` and the
rest of the `.orx`-installed names as `Environment:Phase 5` after the bootstrap, while all of them
answer. Measured consequence (`probes/unbuilt_c2.rex`): `.object~subclass('K')~defineMethods(.environment)`
refuses `a directory whose entries this crate does not fill is not implemented (Phase 5)` where the
oracle raises 93.974; `.local` refuses naming Phase 10.

Verdict: not a blocker for this task; the representation must derive "owed" from state, not from a
snapshot.

### (d) the D45 chokepoint -- not touched by message sends

Oracle, a `Routine` with an auditing security manager (`probes/secmgr.rex`): `.environment~at`,
`.local~hasEntry`, `.environment~allIndexes~items`, `.local~items` and `.environment~supplier` show
**no** checkpoint of their own; the only events are the `LOCAL`/`ENVIRONMENT` pair from resolving
the `.environment` or `.local` symbol itself, and `LOCAL NAME = STDOUT` for `.stdout`. So sends to
the directory objects are plain `Directory` methods, and the seam stays on the `.NAME` lookup path
(`directory_lookup`) and the write path (`set_directory_entry`), which is what
`tests/environment_seam.rs` counts.

Verdict: not a blocker.

### (e) `.NAME` cost -- bounded to names no class search answers

`dot_variable` asks `installed_class`, `imported_class` and `rexx_package_class` (a cached hit for
every library and native class) before `directory_lookup`, so **no class reference reads the
directories in this crate**; the cost falls on `.environment`, `.local`, `.endOfLine`, `.stdout`,
program-added entries, **and every `.NAME` that misses both directories** -- `.rs`, `.line`,
`.context`, `.methods` and an unresolved name all search both before `rexx_variable` (corrected in
fix round 1; the review measured that path, item 4 below). `rexxcps` names no dot symbol at all (`bench-rexxcps/rexxcps.rex`), so it
moves only if startup or a shared hash path moves.

Base figures, callgrind instruction counts on `rexx-run` at `d7eafeb54` (copied to
`scratchpad/surface-t1/bench/rexx-run.base`, sha256 `b8106b3f...b856ab`):

| program | base Ir |
|---|---|
| `bench/dotname.rex` (six directory-reaching `.NAME`s per pass, 200,000 passes) | 2,320,121,695 |
| `rexxcps.rex` | 21,196,129,434 |
| `bench/startup.rex` (`say 1`) | 122,985,045 |

After-figures are in Step 1e below, taken once the representation exists.

### (f) anything else the records say

* The membership/order note above (5a Task 23 attempt 2).
* 5a Task 11 (`task-11-report.md:377`): `.environment~nosuch` answers `.nil` through `Directory`'s
  `UNKNOWN`; the store path's `UNKNOWN` answers the same way for a `.Directory~new`.
* 5a Task 21 C2: the `unreadable_collection` refusal for `~defineMethods` is `unbuilt`'s, see (c).

### (g) found while probing, not in the brief's list

* **A frozen behaviour handle.** `new_instance` stores the class's behaviour handle at creation
  (`dispatch.rs:4189`), and the directories would be created during the bootstrap. Whether that
  handle goes stale once `CoreClasses.orx` finishes is a question for a run, asked in Step 3.
* **Other `Body::Native` directories share the defect and are out of this task's scope**: a
  condition object (`signal on syntax` then `condition('O')~hasIndex('CODE')`, oracle `1`, crate rc
  120 naming Phase 5), a package's `~local` (`.context~package~local~items`, same), and the
  security manager's info directories (`security.rs:131`). The ooTest framework reads condition
  objects; this is recorded for Step 5 and the handover, not fixed here.
* **A `.Directory~new` divergence in code this task touches**: `allIndexes`, `makeArray` and
  `DO OVER` run every `setMethod` method to build their answer (`probes/dir_method_side.rex`: the
  crate prints `ran M` three times, the oracle never). `DirectoryClass::allIndexes` appends
  `methodTable->allIndexes()` and runs nothing. Index-only reads are needed anyway so `.local`'s
  `allIndexes` does not read `STDQUE`'s item, so this is fixed here.

**Conclusion of Step 1: no blocker makes every representation wrong.** The historic blocker was
membership plus order; both are reproducible.

## Step 2 -- the representation, chosen before code

**`.environment` and `.local` become the store `hash.rs` gives a `.Directory~new` instance**: a
`Body::Instance` of `Directory` from `new_instance`, with the store installed in its pool, string-value
keys, and `owns` answering true for them. Every `Directory` method `hash.rs` answers then answers for
them through the same code a program's own directory runs. `owns`'s `Body::Native` early return
stays, for the out-of-scope native directories in (g).

* **Geometry and order are laid out when the model is built.** `.environment`'s store is built at
  the oracle's size (capacity 68 -> 69 buckets) and every name of the oracle's list is inserted in
  the oracle's walk order: the registered value if the crate has one, otherwise an **owed
  placeholder**. `LOCAL` goes into the method-table half. `CoreClasses.orx`'s own
  `.environment~put(class, name)` then replaces placeholders in place, which never moves an entry,
  so no re-lay is needed. `.local` is built at the default size with every oracle name inserted in
  the oracle's walk order as a placeholder. `ORACLE_ENVIRONMENT` and `ORACLE_LOCAL` are re-stated
  in the oracle's `allIndexes` order.
* **`LOCAL`'s method-table entry holds the `.local` object itself**, and a method-table item that
  is an object rather than a method id answers that object when "run". That is
  `ActivityManager::getLocalRexx` for an interpreter with one instance, and keeps `items` 69,
  `allIndexes` ending in `LOCAL`, `supplier`/`allItems` running it, and `setMethod`/`unsetMethod`/
  `put` on `LOCAL` behaving as they do on the oracle's method entry.
* **Owed placeholders replace `EnvironmentModel::unbuilt`.** One placeholder object per owner
  (Phase 5, Phase 10), rooted by the model. Every store read that answers or compares an item
  refuses on a placeholder with `Loud::environment_entry(index, owner)`; `.NAME` refuses with
  `Loud::environment_symbol`; a write replaces it and an `empty` drops it. Reads of indexes and
  counts answer. "Owed" is therefore state, which fixes (c): the walkers' refusal asks whether the
  table still holds a placeholder.
* **`.local` is minted at the end of `Interp::bootstrap_library`**, through `set_directory_entry`
  (the write chokepoint) and `local_route` (the read one), so no new seam call and no handle leaves
  the seam. `STDQUE` stays owed. The on-demand mint in `directory_entry` and `hash_entry_read`
  goes, since nothing is demanded before a program runs.
* **Index-only reads stop running methods** (`allIndexes`, `makeArray`, `DO OVER` through
  `makeArray`), per `DirectoryClass::allIndexes`.
* **The `.NAME` lookup reads the store through `DirectoryClass::get`'s order** (contents, method
  table, unknown method): `.local` is read with `get` and `.environment` with `entry`, which is `get`
  of the upper-cased name (`PackageClass.cpp:1144`, `:1158`; `InterpreterInstance.cpp:903`).
* The seam module is unchanged; `directory_scope`/`env_seam::which` stays as the cheap gate in front
  of the placeholder scan.

A cheaper shape was considered and rejected: keeping `Body::Native` with an ordered map would be a
second hash-store implementation that has to reproduce bucket chains, growth and the method-table
half, all of which `hash.rs` already does and is differential-tested for.

## Build log (appended as it went)

* Representation built as Step 2 says. First release build: `probes/order_env.rex` (both
  directories' `allIndexes`), `probes/secmgr.rex` and `probes/dir_method_side.rex` are byte-identical
  to the oracle.
* (g) the frozen handle, run: `probes/behaviour.rex` asks `hasMethod` for every name
  `.Directory~methods` lists at `Directory`, `MapCollection`, `Collection` and `Object` scope, on a
  `.Directory~new` and on each directory: none is missing on either. The in-crate test below asserts
  the stronger property, same `(scope, MethodId)` per name.
* Draft witnesses `scratchpad/surface-t1/witness/{environment,local}_directory_surface.rex` are
  byte-identical to the oracle on the first run, rc 0, three descriptors.

## Step 5 -- the framework walked to its next stop (not fixed)

Run against the work-in-progress binary (`scratchpad/surface-t1/bench/rexx-run.wip1`, the
representation above, before tests were updated) from a scratch copy of `ootest/`
(`scratchpad/surface-t1/walk/ootest`), each side from a fresh directory, `LD_LIBRARY_PATH` the
oracle's `build/lib` on both (`librxregexp.so` NEEDs only `libstdc++`, `libgcc_s`, `libc`).

* **L2 walk step 4 passes.** The driver `say 'ooTest.frm loaded, version' .ooTest_Framework_version`
  plus `::requires "<copy>/ooTest.frm"`, run beside copies of `rxregexp.cls` and `OOREXXUNIT.CLS`:
  oracle and crate both rc 0, `ooTest.frm loaded, version 1.0.1_4.0.0`, stderr empty. The stop at
  `ooTest.frm:49` is gone.
* **Setup note for the single-group form.** With the scratch copy, the oracle itself stops at
  `ooTest.frm:76` (`43.901 Could not find file "rxregexp.cls"`): the L2 walk's run found it from
  the repository layout. A copy of `extensions/rxregexp/rxregexp.cls` (same bytes as
  `build/bin/rxregexp.cls`, `cmp` clean) placed in the copy's root fixes that for both sides. Before
  that fix the two sides' 43.901 reports already differed by one traceback line: the oracle prints
  `79 *-* retCode = 'worker.rex'(arguments)`, the crate does not.
* **The next stop.** `testOORexx.rex <copy>/ooRexx/extensions/rxregexp/rxregexp.testGroup`: oracle rc
  0, `Tests ran: 33`, `Assertions: 11741`, `Failures: 0`, `Errors: 0`. Crate rc 120, stdout empty,
  stderr `rexx-exec: method "COPY" of class "Object" is not implemented (Phase 5)`. Located by a
  second copy with `::options trace i` appended to `worker.rex`: the last clause is
  `worker.rex:1074` `originalCommandLine = cmdLine~copy`, `CMDLINE` a `String` (the group file's
  path), inside `CommandLine~setAllDefaults` from `CommandLine~init`. One-line probes against the
  crate: `'abc'~copy` refuses the same way; `~copy` on an `Array`, `Set`, `Bag`, `Table`,
  `Directory`, `StringTable`, `IdentityTable`, `Relation`, `Queue`, `List`, `CircularQueue`,
  `MutableBuffer`, `.object~new` and `.environment` all answer.
* **Who owns it.** `Object~copy` on a `String` receiver: the refusal names Phase 5, which is closed.
  Per ruling S4 this is Task 8's first step to measure and route; it is not fixed here.

## What the first test run found, and what changed for it

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --no-fail-fast` on the first build:

* **`lang/directory_index_refusals.rex` and `lang/required_string_method_argument.rex` went red.**
  The store path took an index by its text; the oracle's `StringHashCollection::validateIndex` is
  `stringArgument(index, "index")` (`classes/support/HashCollection.cpp:1112`), so `.nil` or a
  `Directory` as an index is 88.909 and an object with a `makeString` is keyed by what that answers.
  That was already a divergence for a `.Directory~new` (the `.environment` path hid it by using the
  required-string conversion). Fixed for every string-keyed collection: `at`/`[]`, `put`/`[]=`,
  `hasIndex`, `remove`, and the entry family plus `setMethod`/`unsetMethod` (whose C++ is
  `stringArgument(name, "index")` too, `DirectoryClass.cpp:482`, `:539`). Both programs
  byte-identical after.
* **`collect_stress`'s L0 subset panicked** (`a live value`) under collect-on-every-allocation:
  the mint stored `SYSCARGS`'s array and each monitor through `set_directory_entry`, which allocates
  the index text before the value is in the store. Both are now pushed as temporaries inside the
  mint's root frame.
* **In-crate tests that asserted the old representation**, each re-measured against the oracle
  before it was changed: `.environment~request('ARRAY')` (oracle answers, now the crate does; the
  row moved to a condition object, which still refuses), `.environment + 1` (oracle `The NIL object`
  rc 0, and so does the crate now; row removed), `do i = .environment to 5` (oracle 97.1, the crate
  now refuses with the same text a `.Directory~new` gets), `do e over .environment`/`.local` (oracle
  iterates; rows replaced by a package's `~local` and a condition object, both native directories
  that still refuse, plus `.environment`'s count of 69 as an adjacent success), and
  `.K~defineMethods(.environment)` (no longer an unreadable collection; row removed, the `.local` row
  stays because `STDQUE` is owed).
* `corpus/refusal-sites.tsv` re-derived: `environment_entry` moved to the send surface; its row is
  filled `diverges`, `yes`, `directory entry`, witness `.local['STDQUE']`.
* **`gate_table_c` fails, and did at BASE.** Its gated rows are `File`, `Stream` and
  `StreamSupplier` instance rows (Phase 7) that are `unanswered`; the three probe programs raise the
  same rc 163/163/159 with the same first traceback line on `rexx-run.base` and on the new binary.
  Not this task's.

## Step 1e -- the `.NAME` cost, after

callgrind, same programs, from `scratchpad/surface-t1/runs/t`:

| program | base Ir | first build | after the store-read changes |
|---|---|---|---|
| `bench/dotname.rex` | 2,320,121,695 / 2,309,515,905 | 4,993,512,161 | 3,736,105,149 |
| `bench/startup.rex` | 122,985,045 / 122,983,799 | 123,715,075 | (below) |

The base figure moved by 0.5% between two runs of the same binary: `NativeObject`'s map is a
`std` `HashMap` with a per-process seed. The first build spent the difference in `read_store`
(five pool lookups per store read, and `hash_scope` upper-casing and allocating `"Table"` on every
call). Three changes to the shared store path, which every mapped collection takes: the `Table`
scope is held in `ObjectModel`; a store read is one pass over the pool for both halves and the
unknown method (`ScopePools::entries`, new in `rexx-core`); the `.NAME` lookup probes by the name's
bytes without building an index. `dotname` stays **1.62x** base: per directory-reaching `.NAME`,
about 1,200 more instructions. No class reference pays it (Step 1e above).

## Step 3 -- witnesses

* `corpus/lang/environment_directory_surface.rex` -- every method `Directory~methods(.Directory)`
  lists, sent to `.environment`: `allIndexes`, `makeArray`, `DO OVER`, `supplier` (index and item,
  all 69 pairs) and `allItems` as ordered output; the entry family's upper-casing; `setEntry` with
  no value; program-added entries and where they land in the order; `setMethod`/`unsetMethod` and
  the `.NAME` they make; `remove('ARRAY')` and a later `.array`; `removeItem`, `remove('LOCAL')`,
  `init`, `empty`.
* `corpus/lang/local_directory_surface.rex` -- the same surface on `.local`, the index and count reads
  first (before any `.output` use and after), then `STDQUE` replaced and the item reads.
* `corpus/lang/directory_index_string_value.rex` -- `validateIndex` on a `.Directory~new` and a
  `.StringTable~new` for every index-taking method (88.909 for `.nil` and a `Directory`), an index
  keyed by its `makeString`, and `Table` accepting `.nil` as the adjacent success. At BASE the crate
  answered 14 of the 16 rows (`rexx-run.base`); the oracle raises all 16.
* Each filed in `corpus/phase-8.txt` with a `sourceline_oracle` companion generated by the module's
  driver from a scratch copy of the file (`scratchpad/surface-t1/srcg/`), body checked against the
  file by `diff`.
* In-crate, `environment.rs`: `every_directory_method_answers_on_both_directories` takes the names
  from the registry's own `Directory` table at test time, asserts each resolves on both directories
  to the method a new `Directory` resolves to (the frozen-handle question, (g)), and runs a send of
  each on both without a not-implemented refusal; a name with no call in the test panics.
  `the_bootstrap_leaves_the_oracles_names_in_the_oracles_order` asserts both `allIndexes` equal the
  oracle lists and that `STDQUE` is the only owed entry.
* In-crate, `dispatch.rs`: `a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud` pins
  the Phase 10 refusal on every item-reading route to `STDQUE` (`[]`, `entry`, `UNKNOWN`,
  `allItems`, `supplier`, `hasItem`, `index`, `removeItem`, `remove`, `DO OVER` of `allItems`, and
  `.stdque`), with the index and count reads and the post-replacement item reads as the adjacent
  successes.

## Step 4 -- controls (predictions written before running)

**Control A, the directories refused again.** `owns`'s `Body::Native` early return is still in the
tree (it now guards the native directories), so restoring it changes nothing; the equivalent
mutation is an early `return false` in `owns` for `.environment` and `.local`. Prediction:
* corpus: `environment_directory_surface.rex` and `local_directory_surface.rex` red (rc 120 at the
  first store-family send). `directory_index_string_value.rex` green. Any corpus program that
  *writes* into either directory by a send (`put`, `[]=`, an entry assignment) red, because the
  fallback `hash_entry_write` needs a `Body::Native`; programs that only read by `[]`, `at` or
  `UNKNOWN` green.
* in-crate: both new `environment.rs` tests red; the `STDQUE` test red, and not as a refusal: the
  fallback read hands out the placeholder, `say .local['STDQUE']` printing its rendering at rc 0.

**Control B, the insertion order reversed** (`ORACLE_ENVIRONMENT` and `ORACLE_LOCAL` iterated
backwards in `build_environment`). Prediction:
* corpus: `environment_directory_surface.rex` red on the `indexes`, `makeArray`, `over`, `supplier`
  and `order`/`restored` lines (buckets of the 69 hold up to four names); `local_directory_surface.rex`
  red on its `indexes`/`makeArray`/`over`/`after`/`replaced`/`supplier`/`order` lines (buckets 10 and
  13 collide). No other corpus program red.
* in-crate: `the_bootstrap_leaves_the_oracles_names_in_the_oracles_order` red;
  `every_directory_method_answers_on_both_directories` green.

## Step 6 -- `"Phase 5"` is not added to `closed_phases.rs`

`/bin/grep -rln -a --include=*.rs '"Phase 5"' rust/crates/*/src` after this change names
`environment.rs`, `redirect.rs`, `lib.rs` and `run.rs`, so the condition does not hold. What still
names it, by `/bin/grep -rn -a --include=*.rs '"Phase 5"' rust/crates/*/src`, and who it should
name:

| site | what refuses | owner it should have |
|---|---|---|
| `environment.rs` `build_environment`, the first `owed` placeholder | an `.environment` name or a minted `.local` name the crate has not built. Reachable only without a completed library bootstrap: `the_bootstrap_leaves_the_oracles_names_in_the_oracles_order` asserts none is left after it | the library bootstrap; no phase, since a program never sees it |
| `redirect.rs` `open_named_stream`, `run.rs` (the same `.STREAM` refusal) | `Stream` before `StreamClasses.orx` installs | the same: bootstrap-only |
| `lib.rs` `Loud::receiver_class`, `operator_operand`, `object_position`, `native_method` | a send, an operator, a header position, a resolved primitive with no body -- the default owner every such refusal carries, whatever the receiver. The framework's next stop (`'abc'~copy`, Step 5) and the native directories in (g) are this row | per receiver; Task 8 routes the framework's |
| `lib.rs` `method_from_source`, `object_method`, `method_body`, `setup_method` | a method source that is neither a string nor an array (`.K~defineMethods(.environment)` now reaches it for its class values, where the oracle raises 93.974), `setMethod`/`unsetMethod` on a receiver with no dictionary here, a directive body this crate cannot run, a bad argument to a setup-only method | unassigned |
| `lib.rs` `library_source`, `required_source` | an embedded `.orx` or a `::REQUIRES` file that does not parse | unassigned |
| `lib.rs` `expose_receiver`, `use_local_in_a_method` | `EXPOSE` whose receiver is neither a class object nor an instance, `USE LOCAL` as a method's first instruction | unassigned |
| `lib.rs` `instruction_owner`'s `InstructionKind::Options` arm | the owner the split table gives an `OPTIONS` instruction | unassigned |

The `closed_phases.rs` doc comment's example -- "a send to an unimplemented `Directory` method is
one" -- is still true for the native directories and is left as it is.

**Control A, run.** `REXX_CORPUS_GATE=1 cargo test --profile mutation -p rexx-exec --test corpus`:
**0 of 548**, every program rc 120 `a message send to a value that is not a hash collection`. The
three in-crate tests red. **The corpus prediction is falsified in its extent**: the library
bootstrap itself writes into `.environment` (`CoreClasses.orx:65`, `.environment~put(class,
name)`), so refusing the directories fails every program before its first clause, and "programs
that only read stay green" could not be observed. The in-crate reds are therefore not evidence
either. A narrower control follows.

**Control A2, the directories refused once the bootstrap is over** (the same early `return false`,
gated on `!interp.library_bootstrap`). Prediction:
* corpus: `environment_directory_surface.rex` and `local_directory_surface.rex` red;
  `directory_index_string_value.rex` green; `directory_at_and_put.rex` red (it `put`s into
  `.environment`, and the fallback write needs a `Body::Native`); any other program that writes into
  either directory by a send red; programs that only read entries by `[]`, `at` or `UNKNOWN` green.
* in-crate: `every_directory_method_answers_on_both_directories` red (a refusal on the first name);
  `the_bootstrap_leaves_the_oracles_names_in_the_oracles_order` red at its `ALLINDEXES` send; the
  `STDQUE` test red with `say .local['STDQUE']` printing the placeholder's rendering at rc 0.

**Control A2, run.** Corpus **539 of 548**. Red, against the prediction:
* confirmed: `environment_directory_surface.rex` and `local_directory_surface.rex` (rc 120 at
  `ITEMS`); `directory_index_string_value.rex` green; `directory_at_and_put.rex` red. The other
  writers red as predicted: `required_string_method_argument.rex`,
  `environment_local_shadows_the_environment.rex`, `environment_directory_entry_method.rex`,
  `package_namespace.rex` (`.local~zzznsboth = ...`), `library_package_retried.rex`
  (`.environment~put(...)`), each rc 120 `a message send to a value that is not a hash collection`.
* **falsified**: `directory_string_keys.rex` red with no write. Its `.environment~local~class~id`
  goes through `UNKNOWN`, and the fallback read sees only the contents half, not `LOCAL` in the
  method table, so it answers `.nil`'s class. "Programs that only read stay green" was too wide.
* in-crate, all three red: `the_bootstrap_leaves...` at its `ALLINDEXES` send as predicted; the
  `STDQUE` test with `(0, "an entry owed by Phase 10\n", "")` for `say .local['STDQUE']`, the
  placeholder handed out, as predicted; `every_directory_method_answers_on_both_directories` red,
  but not at a refusal on the first name as predicted -- at the test's own setup write
  `l['STDQUE'] = 'q'`, which the fallback refuses.

`hash.rs` restored from the scratch copy, sha256 equal to the copy taken before the control.

**Control B, run.** Corpus **546 of 548**: exactly `environment_directory_surface.rex` and
`local_directory_surface.rex`, stdout only. Line labels that differ, oracle against
`target/mutation/rexx-run` built from the mutated tree: `.environment`'s `indexes`, `makeArray`,
`over`, `order`, `restored` and 42 of the 69 `supplier` lines; `.local`'s `indexes`, `makeArray`,
`over`, `after`, `replaced`, `order` and 4 `supplier` lines. In-crate:
`the_bootstrap_leaves_the_oracles_names_in_the_oracles_order` red,
`every_directory_method_answers_on_both_directories` green. All as predicted.
`environment.rs` restored from the scratch copy, sha256 equal.

## Step 1e -- interleaved, final code

callgrind instruction counts, `rexx-run.base` (`d7eafeb54`) and `rexx-run.wip4` (this task's source
before formatting; the final release build counts `dotname.rex` at 3,735,331,821), alternated base/after/base/after, each program in turn
(`scratchpad/surface-t1/bench/interleaved.txt`):

| program | base run 1 | after run 1 | base run 2 | after run 2 | after / base |
|---|---|---|---|---|---|
| `rexxcps.rex` | 21,196,306,078 | 21,206,676,964 | 21,203,165,006 | 21,206,061,518 | 1.0003 |
| `startup.rex` (`say 1`) | 122,981,285 | 123,574,416 | 122,986,865 | 123,574,132 | 1.0048 |
| `dotname.rex` | 2,313,533,887 | 3,735,324,005 | 2,314,320,067 | 3,735,333,369 | 1.614 |

`rexxcps` moves within the spread two runs of the same base binary show (0.03%). Startup pays the
mint the bootstrap now does before every program, where `say 1` used to pay it at its first `SAY`,
plus the laid-out directories. `dotname` is the concern: six directory-reaching `.NAME`s per pass
cost 61% more instructions. A class reference does not reach this path.

Step 5 re-run on the final release build (`bench/rexx-run.final`): step 4 rc 0 `ooTest.frm loaded,
version 1.0.1_4.0.0`; the single-group driver rc 120, stdout empty, the same `COPY` refusal.

## Hygiene

* A temporary in-crate probe (`zz_stress_subset`, removed) ran every corpus program with
  `Invocation::none()`, so the programs' own fixture files landed in `rust/crates/rexx-exec/`
  (47 files and a `work/` directory, all timestamped 04:06, the probe's run). They were removed by
  the explicit list `git status --porcelain --untracked-files=all` gave; the tree held none at BASE.
* `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` fails on this
  tree, **and on BASE**: the same probe, added to a `git archive d7eafeb54` export built in its own
  `CARGO_TARGET_DIR` (`scratchpad/surface-t1/base-src`, log `gates/base-stress.log`), panics
  `a live value` on `lang/condition_object_syntax.rex` and `lang/security_manager_result.rex` under
  collect-on-every-allocation, the same two programs this tree panics on. Not this task's; the mint's
  own unrooted values were fixed (above) and are not among them.

## Readings before the commit

The test rows ran before four comment-only edits (the split comments in `dispatch.rs`, the route
comment in `hash.rs`, two doc comments in `environment.rs`); `fmt` and `clippy` ran after them.

| command | exit | figures |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | re-checked `rexx-core`, `rexx-api`, `rexx-classes`, `rexx-exec` |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --test refusal_sites --test environment_seam` | 0 | corpus **548 of 548**; 5 and 3 passed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --lib -- environment::tests dispatch::tests::a_directory_entry dispatch::tests::a_conversion eval::object_operand_tests run::tests::the_refusals_this_task` | 0 | 22 passed |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 | 1 passed |
| `cargo test --release -p rexx-core` | 0 | |

The whole touched-crate run (`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec -p rexx-core -p
rexx-parse --no-fail-fast`) is started after the commit; its status file is named in the message to
the controller.

## Committed, and the run after it

Commit `6bf1401fd50216af8cb153d35a07f1b9f8ad4321` (read back with `git log -1`).

`REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-core -p rexx-parse
--no-fail-fast` at that commit, tree untouched until the status file said `finished`
(`scratchpad/surface-t1/gates/final-status.txt`, log `gates/final.log`): **exit 101**, 70 test
binaries `ok`, 2 `FAILED`, corpus **548 of 548**. The two are the BASE failures above:
`collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` (`a live value`)
and `gate_table_c`'s `concept_and_class_gate_table` (the gated Phase 7 `File`/`Stream`/
`StreamSupplier` rows). For `gate_table_c` the evidence is that its three probe programs answer
identically on `rexx-run.base`; the test binary itself was not run at BASE.

## Status: DONE_WITH_CONCERNS

1. `.NAME` of a directory-reaching name costs 1.61x base in instructions (`dotname.rex`); class
   references do not reach it and `rexxcps` is within noise.
2. Native `Directory`s (condition objects, `Package~local`, security argument directories) still
   refuse the store-family methods; the framework reads condition objects.
3. `collect_stress` and `gate_table_c` fail at BASE and here; neither is this task's.
4. The framework's next stop is `worker.rex:1074` `String~copy` (Phase 5-named refusal), for Task 8.

---

# Fix round 1

Review: `task-1-review.md` (verdict: needs fixes). BASE for this round `6bf1401fd`. Appended as the
work goes.

## Item 1 -- `VALUE(name, new, '')` under collect-on-every-allocation

Oracle and crate agree on the ordinary run (`surface-t1/fix1/value_env.rex`, IDENTICAL:
`.ABCDEFGHIJKLMNOP`, `new`). New row `values_environment_write_roots_the_old_value_before_the_stores_allocation`
in `tests/collect_stress.rs`, three rows modelled on the compound one: the read-only form, a write to
a name already present, and the failing shape. **Prediction before running at `6bf1401fd` with no
fix:** the third row panics `a live value` (the reviewer's backtrace), failing the test; the first
two rows alone would pass.

**Run at `6bf1401fd`, no fix.** Red, `a live value`. **The prediction was falsified in one part**:
the second row as first written (`value(name, 'one', '')` then `value(name, 'two', '')`) also
panicked, because its first call is the failing shape itself -- the name is absent when it runs. It
was rewritten to put the entry with `.environment['ABCDEFGHIJKLMNOP'] = 'one'` first (oracle and
crate IDENTICAL, `one`/`two`). Re-run with the rewritten first two rows only: green; with the third
added: red, `a live value`. So the red is the absent-name write.

**Fix.** `environment_directory` (`builtin/platform.rs`) pushes `old` as a temporary before the write.
Two more rows were added before the fix was run, both IDENTICAL on the oracle
(`fix1/value_env_heap.rex`): a `new` value built in the call (`copies('NEWVALUE', 3)`), and a
`.local~put` of a heap value later read as `.NAME`. All five rows green with the fix.

**Control (prediction first: red, `a live value`, the third and fourth rows being the absent-name
write).** The push removed from the fixed tree: red, `a live value`. Restored from the scratch copy,
`cmp` clean.

**Every caller of the directory write path, checked for an `ObjRef` held across it**
(`/bin/grep -rn -a -E 'set_directory_entry\(|directory_put\(|directory_put_method_value\(|put_interpreter_entry\('`):

| site | values held across the write | verdict |
|---|---|---|
| `builtin/platform.rs` `environment_directory` | `old`; `value` (a builtin argument) | `old` was the defect, now pushed; `value` survives (row 4) |
| `environment.rs` `build_environment`, the `ORACLE_ENVIRONMENT` loop and the `rest` loop | the `known` map's values: class objects, `true_value`, `false_value`, `rexx_info`, `end_of_line`, `environment`; the owed placeholders | classes are never collected (D59); the four texts pushed in the frame; the directories and placeholders `add_global`ed before any later allocation |
| `environment.rs` `build_environment`, `LOCAL` into the method half | `local` | `add_global` |
| `environment.rs` `build_environment`, the `ORACLE_LOCAL` loop | the owed placeholders | `add_global` |
| `environment.rs` `mint_local_entries`, `SYSCARGS` | `arguments`, the word texts inside it | both pushed in the mint's frame |
| `environment.rs` `mint_local_entries`, the streams | `built`, `stream_class` | `built` pushed by `new_instance`; a class is never collected |
| `environment.rs` `mint_local_entries`, the monitors | `built`, `target`, `monitor_class` | `built` pushed; `target` is held by `.local` itself |
| `environment.rs` `set_directory_entry` -> `hash::directory_put` | `value`, and the index text | index pushed inside `directory_put`; `value` is each caller's, above |

Sends that now reach the store for these two directories (`native_hash_put`, `native_hash_unknown`'s
write, `setEntry`) take their values from the send's own argument list, which is the path every
`.Directory~new` already took under the stress gate. The read path also gained allocations (a
`setMethod` or `UNKNOWN` entry now runs Rexx code on `.NAME`, `SAY`'s route, `LINEIN`'s route and the
trace route); its callers (`run.rs` `SAY`, `environment.rs` `route_trace_line`, `input.rs`
`linein_line`, `run.rs` `resolve_stream`, `datatype.rs` `literal_value`) hold bytes rather than
`ObjRef`s across the lookup, or nothing, by reading them. The expression evaluator's `.NAME` was run
rather than read: `fix1/stress/readpath.rex` (`.local~setMethod` entries for `ZZM` and `OUTPUT`, an
`UNKNOWN` method, and heap operands on both sides of `.zzm`, `.zznotthere`, `.rs` and `.line` in a
concatenation and an `Array~of`) is IDENTICAL to the oracle, and under collect-on-every-allocation
(the reviewer's harness, built from this tree in `fix1/harness-target`) prints the same at exit 0
with 32 collections.

Committed `81c92716053d05b5d55c57c3322ffbcb49704ce0`. At that commit, tree frozen until the status
file said `finished` (`surface-t1/fix1/gate-81c9.txt`): `cargo fmt --all --check` 0, `cargo clippy
-j 4 --workspace --all-targets -- -D warnings` 0 (before the commit), and `REXX_CORPUS_GATE=1 memcap
16G cargo test --release -j 4 -p rexx-exec --no-fail-fast` exit 101 with corpus **548 of 548**, the
new stress row `ok`, and the two failing binaries the BASE pair (`collect_stress`'s L0 subset,
`gate_table_c`).

## Item 2 -- `Package~findClass` of an owed `.local` name

Oracle (`fix1/w/fc.rex`): `.context~package~findClass('STDQUE')` and `findClass('stdque')` both
print `SESSION`, rc 0. **Prediction for the new `findClass('stdque')` row in
`a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud`, run at `81c92716` with no fix:**
red with `(0, "The NIL object\n", "")`, the other rows green.

**Fix.** `package_find_class` answers `Result`: `Owed` refuses with `Loud::environment_entry` (the
refusal every other item read of `STDQUE` gives), and the lookup's own errors propagate where
`if let Ok(...)` swallowed them. The new row is green. **A second silent answer the same line
held**: a `.local~setMethod('ZZRAISE', 'return 1 / 0')` then `findClass('zzraise')` answered `The NIL
object` at rc 0 on the pre-fix build (`surface-t1/bench/rexx-run.final`, from `6bf1401fd`), where
the oracle raises 42.3 at rc 214 (`fix1/w/fc_raise.rex`). It now raises 42.3 at rc 214; stderr
still differs in one word, `running RUN` against `running ZZRAISE`, the review's pre-existing
"method name of a stored method" divergence. Adjacent success `fix1/w/fc_ok.rex` (a `.local` entry
found, an absent name `.nil`) IDENTICAL.

## Item 3 -- removing `STDQUE` without reading it

Oracle (`fix1/w/local_stdque_removal.rex`): `setEntry('stdque')` with no value leaves `9 0 9`,
`.stdque` is `.STDQUE`, a put restores the entry in its place, `setMethod('stdque', ...)` answers
through the method with `items` 10, `unsetMethod` leaves 9. **Prediction at the current tree (item
2 applied, item 3 not): the crate refuses at the first line, rc 120, `directory entry "STDQUE" is not
implemented (Phase 10)`, stdout empty.**

Confirmed at the tree with item 2 applied: rc 120, stdout empty, that refusal.

**Fix.** `remove_at` reads the item without the owed check, and `take_merged` takes a `Removal`:
`remove` and `removeEntry` are `Answered` and check the entry before taking it; `setEntry` with no
value and `removeItem` (whose `pairs` walk already refused) are `Discarded`, and where a method table
is present a discarded contents hit skips `get` -- `DirectoryClass::get` runs a method only on a
contents miss, so nothing observable is skipped. `setMethod`'s own `take` discards. The witness is
now IDENTICAL. Extra routes, IDENTICAL (`fix1/rm/a.rex`): `setEntry('STDQUE')` with a `setMethod`
entry present, a `setEntry` of a method name running it, `.environment~setEntry('array')` and
`removeEntry('string')`, and a `.local` `UNKNOWN` method -- which also answers `.local` itself, so the
next `.local~setEntry` is 97.1 on `5` on both sides. `removeEntry('stdque')` still refuses and is a
new row in the route test.

Witness `corpus/lang/local_stdque_removal.rex`, filed in `phase-8.txt` with its companion (count 18,
body `diff`-clean). `corpus/refusal-sites.tsv` re-derived with `REXX_REFUSAL_SITES_REFRESH=1`:
`environment_entry`'s surface is now `body+send` (the new construction site is in `environment.rs`);
its witness column now names `.local['STDQUE'] or .context~package~findClass('STDQUE'), before STDQUE
is replaced`, which replaces the universal the review flagged. `dispatch.rs`'s route-test comment
now reads "Each read below ...".

**Controls, predictions first.**
* C2, the `Owed` arm of `package_find_class` answering `Ok(ObjRef::NIL)`: the route test red at the
  `findClass` row with `(0, "The NIL object\n", "")`; no corpus program red (none sends `findClass`
  for an owed name).
* C3, `remove_at` reading through `item_at` again: `local_stdque_removal.rex` red (rc 120 at its first
  send); no other corpus program red; the route test green.

**C2, run:** the route test red, `left: (0, "The NIL object\n", "")`; corpus 549 of 549 (exit 0). As
predicted. **C3, run:** corpus 548 of 549, `local_stdque_removal.rex` alone red (rc 120, `directory
entry "STDQUE"`); the route test green. As predicted. Both restored from scratch copies, `cmp`
clean.

## Item 7 -- the false doc comments

* `environment.rs` `local_route`: "it is owed only before `Interp::mint_local_directory` runs" is now
  "An owed entry answers `None`."
* `dispatch.rs` route test: "Every read that answers or compares `STDQUE`'s item refuses" is now
  "Each read below answers or compares `STDQUE`'s item" (item 2).
* Four more comments the first commit left describing the old representation, found by
  `/bin/grep -rn -a -E '\.environment` and `\.local|Body::Native.*\.local|NativeObject.*\.environment'`
  over `crates/rexx-exec/src`: `Primitive::Directory`'s doc named `.environment` and `.local` (now: "such
  as a condition object"); the
  `native_hash_collection_new` comment said the map is what the two are built on; `value.rs`'s
  `Body::Native` string-value comment said the two directories were given their names there;
  `environment.rs`'s trace-object comment contrasted a `StringTable` with "the `Body::Native` map
  `.local` uses". Each narrowed to what still holds. `lib.rs`'s field-audit comment gains the owed
  placeholders as globals.

Readings for items 2, 3 and 7 before the commit: `cargo fmt --all --check` 0; `cargo clippy -j 4
--workspace --all-targets -- -D warnings` 0; `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec
--lib` 0, 821 passed; `... --test corpus --test refusal_sites --test environment_seam` 0, corpus
**549 of 549**; `cargo test --release -p rexx-parse --test sourceline_oracle` 0.

Committed `24f8879e3b823835912f81057908b080e05def93`.

## Not done (recorded, as the round directs)

Probes in `surface-t1/fix1/notdone/`, oracle against `bench/rexx-run.24f8` (the release binary built
for the `24f8879e` gate run), three descriptors.

* **`VALUE(name, , '')` reads through `.NAME`, where the oracle reads `.environment` only**
  (`BuiltinFunctions.cpp:1856`, `TheEnvironment->entry`; `builtin/platform.rs`
  `environment_directory` calls `dot_variable`). Existing at BASE for a `.local` name; this task makes
  a new case reachable through a `.local` `UNKNOWN` method.
  ```
  value_unknown.rex   .local~setMethod('UNKNOWN', 'use arg n; return "U:" n'); say value('ZZNOPE', , '')
    oracle rc 0: .ZZNOPE
    crate  rc 0: U: ZZNOPE
  value_env_only.rex  .local~zzOnlyLocal = 'from .local'; say value('STDOUT', , ''); say value('ZZONLYLOCAL', , '')
    oracle rc 0: .STDOUT / .ZZONLYLOCAL
    crate  rc 0: STDOUT / from .local
  ```
  Silent wrong answers at rc 0. With `.environment` a store, the fix is `hash::directory_get` on
  `.environment` alone behind the seam, then the `.NAME` text on a miss.
* **`equivalent` refuses on both directories**, through the Phase 5-named operator refusal, where the
  oracle answers `1`; on a plain `.Directory~new` it is identical.
  ```
  equiv_env.rex    say .environment~equivalent(.environment)
    oracle rc 0: 1
    crate  rc 120: rexx-exec: the operator `\=` applied to one of the interpreter's own objects is not implemented (Phase 5)
  equiv_local.rex  .local['STDQUE'] = 'q'; say .local~equivalent(.local)
    oracle rc 0: 1
    crate  rc 120: rexx-exec: the operator `\=` applied to an array is not implemented (Phase 5)
  equiv_plain.rex  d = .Directory~new; d['a'] = 1; say d~equivalent(d)
    oracle rc 0: 1 / crate rc 0: 1
  ```
  `Collection~equivalent` compares items with `\=`; the refusal is the item's (`.RexxInfo`, a
  `Body::Native`, on `.environment`; `SYSCARGS`'s array on `.local`), not the directory's.
  Confirmed by running: `say (.rexxinfo \= .rexxinfo)` gives the same refusal (oracle `0`),
  `say (.local~syscargs \= .local~syscargs)` the array one (oracle `0`), and a copy of
  `.environment` with `REXXINFO` removed answers `equivalent` `1` on both sides (`ri.rex`, `sa.rex`,
  `equiv_env_no_ri.rex`). So the owner is the operator refusal on those two receivers, not this
  task's directories.

Gate at `24f8879e` (status `fix1/gate-24f8.txt`, tree frozen until `finished`):
`REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-parse --no-fail-fast`
exit 101, corpus **549 of 549**, the two BASE failures only (`collect_stress` L0 subset,
`gate_table_c`).

## Item 5 -- a witness that index reads run no `setMethod` method

`corpus/lang/directory_enumeration_runs_no_method.rex`: a `.Directory~new` with a `setMethod` entry
that `say`s when it runs, then `allIndexes`, `makeArray`, `DO OVER`, `items`/`hasIndex`, `allItems`,
`supplier` and `[]`, each labelled; the same on `.local` with its own method entry. IDENTICAL on
`bench/rexx-run.24f8`. On `bench/rexx-run.base` (BASE `d7eafeb54`) the directory half already
diverges at its first line (`ran TICK` printed under `allIndexes`, `makeArray` and `do over`), which
is the defect the first commit fixed and this file now pins; the `.local` half refuses there.

Filed in `phase-8.txt` with its companion (body `diff`-clean). **Control, prediction first:**
`native_hash_all_indexes` built from `pairs` again. Predicted red in the gated corpus:
`directory_enumeration_runs_no_method.rex` (stdout, `ran TICK` under `allIndexes`) and
`local_directory_surface.rex` (rc 120 at its first `allIndexes`, `STDQUE`'s refusal); nothing else.

**C5, run:** gated corpus 548 of 550, exactly `local_directory_surface.rex` (rc 120, `STDQUE`) and
`directory_enumeration_runs_no_method.rex` (stdout). As predicted. Restored, `cmp` clean.

Committed `7bfe5d42821e2c4c5a7550cd035245625ff9f276`. Gate at that commit (`fix1/gate-7bfe.txt`):
`REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-parse --no-fail-fast`
exit 101, corpus **550 of 550**, the two BASE failures only.

## Item 6 -- the enumeration test compares answers

**The degenerate, D:** in `Interp::invoke`'s native arm, once the library bootstrap is over, a send
of any name in `Directory`'s own table to `.environment` or `.local` answers `.nil` without running.
**Prediction for D at `7bfe5d42` (the test as the first commit wrote it):**
`every_directory_method_answers_on_both_directories` **green** (it asserts resolution, rc 0 and no
refusal text, and D refuses nothing -- though a message instruction whose send answers `.nil` is
fine, a later `r['ZZ']`-style clause is not in that test); `the_bootstrap_leaves_the_oracles_names_in_the_oracles_order`
red (its `ALLINDEXES` send answers `.nil`, which is not an array); corpus red on both surface
witnesses and on every other program that reads either directory through a send.

**D at `7bfe5d42`, run** (under `--profile mutation`). The first version of D found the two
directories through `dot_variable`, which itself sends through a security check and recursed until
the corpus binary overflowed its stack; the second compares against the model's own handles through
a temporary helper in `environment.rs`. With it: `every_directory_method_answers_on_both_directories`
**green**, `the_bootstrap_leaves_the_oracles_names_in_the_oracles_order` red, corpus 538 of 550. As
predicted: the test as first written cannot see D, which is the review's point.

**The new test** (`every_directory_method_answers_on_both_directories_as_on_a_copy`) keeps the
registry enumeration and the resolution check, and adds a comparison. For each directory, one program
sends every table name to the directory and a second sends the same to a `.Directory~new` built at
the directory's own size and filled in its own order (`LOCAL` as a `setMethod` entry for
`.environment`); `ZZ` is put before each send so every answer has something to answer; each answer
is printed (`the receiver` for the receiver itself, arrays and suppliers expanded); a final line gives
`items` and `allIndexes`. The two transcripts must be equal, both programs exit 0 with empty stderr,
and the copy program also prints a line if its order does not match the directory's.
**Prediction with D still applied:** red on `.environment` at the transcript comparison.

**With D, run.** Red twice before it was red for the predicted reason, and neither time at the
comparison, so the test was rebuilt each time: first the copy program read the directory with
`src~allIndexes`, which D also answers `.nil`, so it stopped at 98.913 (and the copy was circular:
it trusted the very sends under test); the copy is now filled from `ORACLE_ENVIRONMENT` or
`ORACLE_LOCAL` with each value read as `.NAME`. Then `show` compared `value == receiver`, and `==`
with an array on the left is this crate's Phase 5 operator refusal, rc 120 on the copy; it now
compares `receiver == value`. The third run: red at the transcript comparison, `.environment`,
left `ALLINDEXES The NIL object ...`, right `ALLINDEXES array 70 INPUTOUTPUTSTREAM|...`. As
predicted. With D removed (`dispatch.rs` restored from its scratch copy, the helper deleted, no `zz_`
left): green in release. A one-off print of the `.local` transcript (removed, file `cmp`-restored)
shows each send answering something to compare: `AT zz value`, `HASITEM 1`, `INDEX ZZ`, `REMOVE zz
value`, `SUPPLIER supplier SYSCARGS=an Array ...`, `EMPTY the receiver`, `final 0 array 0`.

Committed `3cd228b8bac5cb16b5a2bbf3e194b046f3a58e89`. Gate at that commit (`fix1/gate-3cd2.txt`):
`REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec --no-fail-fast` exit 101,
corpus **550 of 550**, the two BASE failures only.

## Item 4 -- the `.NAME` miss path

**Design.** The store stays the only holder of the entries; what is kept is a reading of each
directory's pool. `hash::StoreView` is both halves' `Store`s and the unknown method, stamped with a
`store_generation` counter on `Interp` that `install_store`, `set_free` and `set_unknown_method` --
every write to a store's pool entries -- bump. `EnvironmentModel` keeps one view per directory;
`directory_lookup` hands it to `hash::directory_get_viewed`, which reuses it when the generation
matches and re-reads the pool otherwise, and a `debug_assert_eq!` inside the reuse compares it with
a fresh reading. Entries themselves are always read from the arrays.

**Before, callgrind, `bench/miss.rex` (50,000 passes of `.rs`, `.context`, `.zzunresolvedname`)
against `bench/ctl.rex` (the same loop over constants)**, the review's programs: `rexx-run.base`
miss 406,222,635 / ctl 169,290,020; `rexx-run.3cd2` (this round's tree before item 4) miss 615,273,247
/ ctl 170,082,663. Loop-subtracted per pass: base 4,738.6, 3cd2 8,903.8 -- the review's figures.

**Iterations, measured on the miss program** (per pass, loop-subtracted): the view as first written
(`view1`: copy the view in, write it back after every lookup) 5,669.4; hashing the name once per
`.NAME` and writing the view back only when re-read (`view2`) 5,448.4; copying the view out rather
than `take` and re-store (`view3`, the committed shape) **5,147.9**, i.e. +136 Ir per `.NAME` that
misses both directories (1.086x per pass), from +1,388. What is left per lookup is the chain probe
itself (`text_slot`, a `u64` remainder and one array read, about 53 Ir) and `view_get`'s bookkeeping.

**The witness** `corpus/lang/environment_symbol_after_store_changes.rex` (IDENTICAL on the oracle and
`view3`; companion count 39): `.NAME` reads of `.local` and `.environment` entries between puts that
grow each store, a removal, a replacing put, a `setMethod` entry, an `UNKNOWN` method set and unset,
and `.local~empty`.

**Controls, predictions first.**
* CV1, `install_store` not bumping the generation (release, mutation profile): `set_free` also bumps,
  and every put that lands in an overflow slot calls it, so growth alone is unlikely to leave a stale
  view in use. `.local~empty` is `install_store` with no put: the witness's `emptied` line reads
  `.zzkey1` from the pre-empty arrays, so **`environment_symbol_after_store_changes.rex` red on the
  `emptied` line, and no other corpus program red.**
* CV2, `set_unknown_method` not bumping: **the same witness red on its `unknown` line** (`.zzabsent`
  answering `.ZZABSENT` from a view without the unknown method); `local_directory_surface.rex` does not
  set an `UNKNOWN` method, so nothing else red.
* CV3, `set_free` not bumping: release lookups never read `free`, so **release corpus green**; in a
  **debug** build the reuse assertion fires on the first lookup after a put that took an overflow
  slot, so the debug corpus run is red with `a StoreView was reused` in the failure text.

**CV1, run:** corpus 549 of 551. Red: `environment_symbol_after_store_changes.rex` on `emptied` (as
predicted: `local one again local 2` from the pre-empty arrays) **and on `method`** (`.ZZKEY2`), and
`local_directory_surface.rex` on its `setMethod` line (`.ZZVIAMETHOD`). **The extent was falsified:**
a directory's first `setMethod` installs the method half with `install_store` and then puts into an
empty bucket, which calls no `set_free`, so a view from before it stays in use. Lines read from a
mutant binary built the same way (`bench/rexx-run.cv1`) against the oracle. Restored, `cmp` clean.

**CV2, run:** corpus 550 of 551, only `environment_symbol_after_store_changes.rex`, on its `unknown`
line (`.ZZABSENT local one again` against `unknown ZZABSENT local one again`). As predicted. Restored,
`cmp` clean.

**CV3, run:** release (mutation profile) corpus 551 of 551, exit 0; debug corpus red,
`a StoreView was reused after its directory's pool changed without a bump_store_generation` in two
tests (first at `directory_at_and_put.rex` and `library_package_retried.rex`), the two views differing
in `free` alone (94 against 95). As predicted. Restored, `cmp` clean. The real code under debug:
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` 551 of 551, exit 0, so the assertion is
silent where the generation is kept; `cargo test -p rexx-exec --lib -- environment::tests
dispatch::tests::a_directory_entry` 6 passed.

Committed `96bf51e3e1f227d6f096da8b4fa1f982cc380aa2`. Gate at that commit (`fix1/gate-96bf.txt`):
`REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-parse --no-fail-fast`
exit 101, corpus **551 of 551**, the two BASE failures only. Before the commit: `cargo fmt --all
--check` 0, `cargo clippy -j 4 --workspace --all-targets -- -D warnings` 0, `--lib` 821 passed,
`--test corpus --test refusal_sites --test environment_seam` 0, `rexx-parse --test sourceline_oracle`
0, and the debug corpus run above.

The report's Step 1e sentence on who pays the lookup cost left out the miss path; it is corrected in
place and marked as corrected in this round.

**Interleaved, final** (`bench/interleaved2.txt`; base, `3cd2`, `view3` in turn for each program,
two rounds; `view3` is byte-identical to the release build of `96bf51e3`, sha256 prefix
`fbdcedbf42830686` on both):

| program | base r1 / r2 | before item 4 (`3cd2`) r1 / r2 | after (`view3`) r1 / r2 | after / base |
|---|---|---|---|---|
| `miss.rex` less `ctl.rex`, per pass | 4,738.3 / 4,753.2 | 8,903.8 / 8,903.8 | 5,147.8 / 5,147.9 | 1.086 |
| `dotname.rex` | 2,309,518,318 / 2,309,524,763 | 3,735,307,904 / 3,736,118,116 | 2,649,660,480 / 2,648,839,051 | 1.147 |
| `startup.rex` | 122,986,173 / 122,984,235 | 123,552,082 / 123,578,093 | 123,504,096 / 123,509,880 | 1.004 |
| `rexxcps.rex` | 21,198,031,758 / 21,196,618,090 | 21,202,723,604 / 21,203,796,603 | 21,209,520,239 / 21,206,432,165 | 1.0005 |

The base miss figure moves by 15 per pass between rounds (the seeded `HashMap` it used); the others
reproduce to four or more significant figures. Not taken all the way to base: what remains is the
bucket probe a store lookup is, against a `HashMap` lookup, and closing it would mean a second index
over the entries beside the store.

## Re-reading this round's own comments

Every comment line the round added (`git diff 6bf1401fd..HEAD -- rust/crates | /bin/grep -a -E
'^\+ *(//|/\*|\*)'`) was re-read against the code. Three stated a universal over in-repo code, and are
narrowed in a comment-only commit: `Primitive::Directory`'s list of native directories (checked by
`/bin/grep -rn -a 'native_instance('` and the class each call passes, but a list is still a list; now
"such as a condition object"), and two sentences saying every pool write bumps the generation
(`bump_store_generation`'s doc and `Interp::store_generation`'s); the reuse `debug_assert` in
`hash::directory_get` is what holds that property, and CV3 showed it live.

Committed `24d459c82e2b2bdf33cd0240aa4fd65f3cba4d79`. Before it: `cargo fmt --all --check` 0,
`cargo clippy -j 4 --workspace --all-targets -- -D warnings` 0, `REXX_CORPUS_GATE=1 cargo test
--release -p rexx-exec --lib --test refusal_sites --test environment_seam --test corpus` 0 (821
passed; corpus 551 of 551). At it (`fix1/gate-24d4.txt`, tree frozen until `finished`):
`REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-parse -p rexx-core
--no-fail-fast` exit 101, 70 test binaries `ok`, corpus **551 of 551**, the two BASE failures only.

## Fix round 1 -- status: DONE_WITH_CONCERNS

Commits: `81c927160` (item 1), `24f8879e3` (items 2, 3, 7), `7bfe5d428` (item 5), `3cd228b8b`
(item 6), `96bf51e3e` (item 4), `24d459c82` (comment narrowing).

Concerns:
1. A `.NAME` that misses both directories is still 1.086x base per pass in instructions (+136 per
   name), and a directory hit (`dotname.rex`) 1.147x; `rexxcps` 1.0005x, startup 1.004x.
2. Two predictions were falsified in extent and are recorded as such: item 1's second stress row
   (itself the failing shape as first written) and CV1 (the method half's first install also leaves a
   view stale without the `install_store` bump).
3. Not done, recorded above: `VALUE(name, , '')` reading through `.NAME`; `equivalent` on both
   directories refusing through the operator refusal on `.RexxInfo` and `SYSCARGS`'s array. Also the
   review's note that a stored method which raises reports `running RUN` where the oracle names it
   (seen again on `findClass`, item 2).
4. `collect_stress`'s L0 subset and `gate_table_c` still fail as at BASE.

---

# Fix round 2

Re-review: `task-1-rereview.md` (every original finding closed; needs another small round). BASE
for this round `24d459c82`. Appended as the work goes.

## Item 1 -- `directive_class` swallowing lookup errors, and the same shape elsewhere

**A new oracle crash, found while checking the other reads (not to be re-run).** `route_trace_line`
reads `.local`'s `TRACEOUTPUT` with `let Ok(Some(route)) = ... else { return; }`. Probing what the
oracle does when that lookup fails:
```
fix2/p/trace_exit.rex
  .local~setMethod('TRACEOUTPUT', 'say "lookup ran"; exit 7')
  trace r
  zz = 1
  trace off
  say 'after'
oracle: rc 139 (SIGSEGV), stdout "lookup ran" / "     3 *-* zz = 1" / "lookup ran" / "       >>>   \"1\"" /
        "lookup ran" / "     4 *-* trace off" / "after", stderr empty
crate:  rc 0, the same stdout, stderr empty
```
Run once under the standard wrapper with a 20 s deadline; it is the crash shape of
`corpus/oracle-crashes.txt` entry 11c (a trace route that cannot take a line), reached through a
`.local` method entry rather than `~destination(.nil)`. It is not added to that file here, because
the file is outside this round's scope; the controller decides. The raising variant
(`fix2/p/trace_raise.rex`, `return 1 / 0` instead of `exit 7`) is not a crash but is not a usable
answer either: oracle rc 207 with **both** stdout and stderr empty (not even `after`); crate rc 0,
the trace lines on stderr and `after` on stdout. Both fall under entry 11's licence (a route that
cannot take the line leaves it where the crate writes it, and the program ends normally), so
`route_trace_line` is left as it is.

**The defect, reproduced** (`fix2/dc/`, the reviewer's two files): oracle rc 214 `Error 42.3` running
`UNKNOWN`; crate at `24d459c82` rc 158 `Error 98.909: Class "ZZBASE" not found.`

**Fix.** `directive_class` answers `Result<Option<ObjRef>, Failure>` and propagates the lookup's error
with `?`; `resolve_class_target` blames the directive (the same helper its class-not-found arm uses)
and returns the error. After: rc 214, `Error 42.3`, the traceback lines equal; stderr differs only in
the recorded `running RUN` / `running UNKNOWN` word.

**Witness** `corpus/lang/directive_class_environment_raises.rex` with `directive_class_environment_raises.d/`
(`raises.cls`, `method.cls`, `answers.cls`, each one `::class ... public subclass zz...`). The program
sets an `.environment` `UNKNOWN` method, then `loadPackage`s each package under `SIGNAL ON SYNTAX` and
prints the condition's code and message: an `UNKNOWN` that divides by zero, a method entry that does,
and an `UNKNOWN` answering `.array` (the adjacent success, whose class then has superclass `Array`).
It is trapped so that stderr carries no `running RUN` line. IDENTICAL on the oracle and the fixed
build; on `bench/rexx-run.24f8` the two raising rows read `raised 98.909 Class "ZZBASE" not found.`
and `... "ZZMETHOD" not found.` Companion count 21, body `diff`-clean. Gated corpus 552 of 552.

Two things the witness avoids, both pre-existing (`bench/rexx-run.base` from `d7eafeb54` answers the
same), recorded under "Not done" below: a condition object's `POSITION` for an error raised inside a
method, and the external-call traceback line.

**Prediction for the control** (the `?` back to `if let Ok(hash::DirectoryEntry::Found(found)) =`):
gated corpus 551 of 552, only the new witness, on its `raises.cls` and `method.cls` lines.

**Control, run:** gated corpus (mutation profile) 551 of 552, only
`directive_class_environment_raises.rex`, stdout, on the `raises.cls` (`98.909 Class "ZZBASE" not
found.`) and `method.cls` (`98.909 Class "ZZMETHOD" not found.`) lines. As predicted. Restored, `cmp`
clean.

**The other reads of the two directories that could hide an error**, from `git diff d7eafeb54 HEAD --
rust/crates | /bin/grep -a -n -E '^\+.*(\.ok\(\)|if let Ok\(|let Ok\()'` plus every caller of
`directory_lookup`, `local_route`, `directory_get` and `owed_table_owner`
(`/bin/grep -rn -a -E 'directory_lookup\(|local_route\(|directory_get\(|owed_table_owner\('`):

| site | shape | what the `Err` can be | verdict |
|---|---|---|---|
| `environment.rs` `dot_variable`, `package_find_class`, `local_route`, `output_route_uncached`, `input.rs` `linein_line`, `directory_lookup` itself | `?` / `match ...?` | a raising method, `UNKNOWN` or security check | propagates |
| `environment.rs` `directive_class` | was `if let Ok(...)` | the same | **fixed here** |
| `environment.rs` `route_trace_line` | `let Ok(Some(route)) = self.local_route(b"TRACEOUTPUT") else { return; }` (text unchanged since `d7eafeb54`; the lookup became able to fail in Task 1) | a raising `.local` method entry or `UNKNOWN` | left: the function cannot fail and its callers are the trace emitters, and the oracle's answers on this route are a SIGSEGV and a silent rc 207 (above), both under `oracle-crashes.txt` entry 11's licence |
| `environment.rs` `mint_local_entries` | `let Ok(Some(target)) = self.local_route(over) else { return; }` | only a method entry or `UNKNOWN` on `.local`, which nothing can install before the mint: it runs at the end of `bootstrap_library`, before any program clause, and the embedded library sends `.local` only `objectName=` and `setEntry` (`CoreClasses.orx:990`-`:1007`, read) | left; a failure there would stop the mint and leave the entries owed, which refuse loudly |
| `hash.rs` `owed_table_owner` | `.ok()??` on `read_store`, `walk_in`, `slot_at` | only `array_slots` on a store's own arrays being something else, which runs no Rexx code | left |
| `hash.rs` `directory_get`'s reuse check | `read_view(...).ok()` inside `debug_assert_eq!` | the same | left |
| `hash.rs` `store_indexes`, `store_item` (before Task 1; reached for the two directories through `native_keys`/`native_entry` since it) | `.ok()` | the same: string-keyed probing hashes by bytes and sends nothing | left |

Committed `d9262d474cf413765b6bce1d934a8e3775c21a96`.

## Not done, added in this round

* **Minor 1's route loses a write, not only a read** (the re-review's out-of-scope note). The write
  lands in `.environment`; the read-back goes through `.NAME`, which finds `.local`'s method entry
  first. `fix2/nd/value_write_behind_method.rex`, run against `bench/rexx-run.view3` (the release
  build of `96bf51e3e`; the `VALUE` path is unchanged since):
  ```
  .local~setMethod('QRSTUVWXYZABCDEF', 'return copies("M", 40)')
  say value('QRSTUVWXYZABCDEF', copies('Y', 30), '')
  say value('QRSTUVWXYZABCDEF', , '')
  say .environment['QRSTUVWXYZABCDEF']
  oracle rc 0: .QRSTUVWXYZABCDEF / YYY... (30) / YYY... (30)
  crate  rc 0: MMM... (40)       / MMM... (40) / YYY... (30)
  ```
  The last line shows the write did land in `.environment`; both `VALUE` reads are wrong. Same fix as
  the recorded Minor 1: read `.environment` alone.
* **A condition object's `POSITION` for an error raised inside a method answers the caller's line.**
  `fix2/pos/a.rex`: a stored `UNKNOWN` method and a `::method` that each divide by zero, trapped in
  the caller. Oracle `stored 1` / `method 14` (the raising clause's own line); crate `stored 4` /
  `method 8` (the sending clause's). `bench/rexx-run.base` (`d7eafeb54`) answers the same, so it
  predates Task 1. The item 1 witness prints `code` and `message` rather than `position` for this
  reason.
* **An external call's traceback line is missing below a failed `::REQUIRES`.** `fix2/dcu2/`
  (`call 'inner.rex'`, whose `::requires 'dcu.cls'` names an unresolvable superclass): the oracle
  prints `2 *-* call 'inner.rex'` under the `::requires` line and names `dcu.cls` in the `Error 98`
  header; the crate omits the call line and names `main.rex`. Same at BASE; the header half is
  `phase-8-l2.md` section 5's recorded directive-attribution divergence, the missing call line is the
  shape the Step 5 walk saw at `worker.rex`.
* **A non-class `.environment` entry as a `::CLASS` target** (`fix2/nc/`, `.environment['ZZBASE'] =
  'not a class'` in a required file, then `::class foo subclass zzbase`): oracle rc 157 `Error
  99.949: "ZZBASE" is not a valid class.`; crate rc 158 `Error 98.909: Class "ZZBASE" not found.`
  Same at BASE. `directive_class`'s own comment says a non-class answer falls through to the native
  table and "a miss is the same 98.909 it was", which the oracle contradicts; the comment is left as
  it is because it describes the code, and the behaviour is recorded here for its owner.
* **The `TRACEOUTPUT` lookup that exits crashes the oracle** (item 1 above, `fix2/p/trace_exit.rex`,
  rc 139); proposed for `corpus/oracle-crashes.txt`, not added by this round.

## Item 2 -- the false comments (one commit)

**The rooting check first**, as directed, before the `views` comment: a debug harness linking
`rexx_exec::run_program_collect_every_alloc`, built from the tree at `d9262d474` in its own target
(`fix2/harness`, `fix2/harness-target-dbg`), over the three witnesses that read the directories most
(each from a fresh directory, compared with the oracle): `environment_symbol_after_store_changes.rex`
exit 0, 629 collections; `environment_directory_surface.rex` exit 0, 713; `local_directory_surface.rex`
exit 0, 273; stdout and stderr equal to the oracle's on all three, so neither the reuse assertion nor
a dead-handle panic fired.

| site | was | now |
|---|---|---|
| `lib.rs` `Interp::store_generation` | "a write to a mapped collection's pool entries moves it" (false: `duplicate_collection_stores` writes a copy's store without it) | "What a `StoreView` is valid against. `hash::bump_store_generation` moves it." |
| `lib.rs` collector field audit, `environment: _` | "`.environment`, `.local` and the owed placeholders are globals; the rest are classes." (false since `views`) | adds: a `StoreView`'s store arrays are held by its directory's own pool while its generation is current, and a view whose generation is not current is not read |
| `environment.rs` the method-table test | "`EMPTY` last, since it takes every entry out." (the oracle leaves method-table entries; the transcript ends `final 1 array 1 LOCAL`) | "... since it takes the contents out." |
| `hash.rs` `Removal` | "Whether a removal hands the item it takes back to the program." (`removeItem` passes `Discarded` and answers the item) | whether a removal refuses an owed entry before taking it: `Answered` hands the item back unread, `Discarded` drops it or its caller already read it through `item_at` |
| `dispatch.rs` the `STDQUE` route test | "... and each reaches the refusal through a different path." (`allItems~items` and `do over allItems` share one; so do `remove` and `removeEntry`) | the clause removed |
| `environment.rs` `EnvironmentModel::views` | "The last reading of ..." (an outer lookup can store an older reading than an inner one) | "A reading ... from an earlier lookup; one that is not current is read again." |

Before the commit: `cargo fmt --all --check` 0, `cargo clippy -j 4 --workspace --all-targets -- -D
warnings` 0, `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --lib` 821 passed, `--test corpus`
552 of 552.

Committed `3c38f62a45e3b95ecf69c930a58511f59aadd8b1`.

## Item 3 -- the test's `SETENTRY` row

`every_directory_method_answers_on_both_directories_as_on_a_copy` sent `setEntry('zz value', 'ZZ')`,
which writes an entry named `ZZ VALUE` holding `ZZ`. The row now sends `setEntry('ZZ', 'zz value')`,
on its own arm beside `PUT`/`[]=`. Checked with a one-off print of both transcripts (removed, file
`cmp`-restored): `ZZ VALUE` appears nowhere, and the `.local` `SUPPLIER` line lists `ZZ=zz value`.
The test passes. Before the commit: fmt 0, clippy 0, `--lib` 821 passed.

Committed `35dc46110311c6b23cd01bda923e98524d37dc58`.

## Item 4 -- corrections to this report's earlier sections

The earlier text is left as written; each statement below is what replaces it.

1. **Fix round 1, item 6**, the sentence "and the copy program also prints a line if its order does
   not match the directory's." **False.** The committed test's copy program is the fill loop and the
   `LOCAL` method only; an earlier draft had an order check and it was dropped when the copy began to
   be filled from the oracle lists. Order is compared through the transcript's `ALLINDEXES`,
   `MAKEARRAY`, `SUPPLIER` and `final` lines.
2. **Fix round 1, item 4, Design**, "`install_store`, `set_free` and `set_unknown_method` -- every write
   to a store's pool entries -- bump." **False as a universal.** Those three call
   `bump_store_generation`; `duplicate_collection_stores` (a collection's `copy`) also writes store
   pool entries and does not. No view is taken of a copy, so behaviour is unaffected.
3. **The same paragraph**, "`directory_lookup` hands it to `hash::directory_get_viewed`". **No such
   function**: it is `hash::directory_get`, which takes the view as an argument.
4. **Fix round 1, "Re-reading this round's own comments"**, which says `Interp::store_generation`'s
   sentence was narrowed in `24d459c82`. It was rephrased and kept the universal; item 2 of this round
   is the correction.
5. **Stale `dotname` figures without a pointer**: the first build log's "`dotname` stays **1.62x**
   base", the "Step 1e -- interleaved, final code" table's `1.614`, and the first status list's
   "costs 1.61x base". All three describe the first commit (`6bf1401fd`). The figure after fix round 1
   (`96bf51e3e`) is **1.147x** (`dotname.rex` 2,649,660,480 / 2,648,839,051 against base
   2,309,518,318 / 2,309,524,763, interleaved; Fix round 1, item 4); nothing in this round changes the
   lookup path.

## Fix round 2 -- gates and status

| commit | what | release run at it |
|---|---|---|
| `d9262d474` | item 1 | `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-parse --no-fail-fast` exit 101, corpus 552 of 552, the BASE pair only (`fix2/gate-d926.txt`) |
| `3c38f62a4` | item 2 | fmt 0, clippy 0, `--lib` 821 passed, `--test corpus` 552 of 552 (before the commit) |
| `35dc46110` | item 3 | `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec -p rexx-parse -p rexx-core --no-fail-fast` exit 101, 70 test binaries `ok`, corpus 552 of 552, the BASE pair only (`fix2/gate-35dc.txt`, tree frozen until `finished`) |

The BASE pair is `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation`
and `gate_table_c`'s `concept_and_class_gate_table`, both failing at `d7eafeb54`.

**Status: DONE_WITH_CONCERNS.**
1. A new oracle crash (`fix2/p/trace_exit.rex`, rc 139) is proposed for `corpus/oracle-crashes.txt`
   and not added.
2. Four divergences found on the way are recorded, not fixed, under "Not done, added in this round":
   `VALUE`'s lost write behind a `.local` method entry, a trapped method error's `POSITION`, the
   missing external-call traceback line, and a non-class `.environment` entry as a `::CLASS` target
   (99.949 against 98.909).
3. `route_trace_line` still ignores a failing `TRACEOUTPUT` lookup, deliberately, under entry 11's
   licence.
