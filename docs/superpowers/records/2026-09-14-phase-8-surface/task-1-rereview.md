# Task 1 fix round 1 re-review

Fix base `6bf1401fd`, head `24d459c82`. Read-only; probes from
`scratchpad/t1-rereview/`, a fresh directory per side per run, three descriptors. Release crate
binary `rust/target/release/rexx-run` built 07:13:52 after the 07:11:28 head commit, tree clean
(sha256 `5aeb7682...`). Working notes are appended at the end as the review goes, each marked RAN
or READ; the verdict sections above them are written last.

### Finding Verdicts

- ✅ **Critical 1** (`VALUE(name, new, '')` crash). `builtin/platform.rs:326` pushes `old` before
  the write. RAN: `probes/v1_value.rex` (absent name, heap `new`, a `.local` method entry answering
  a fresh string, a `.local` `UNKNOWN`, 30 fresh names in a loop) under collect-on-every-allocation
  in a debug harness built from `git archive 24d459c82`: exit 0, 332 collections, stdout equal to
  release. READ: the report's caller table checked row by row against `environment.rs`; it holds.
  Row `values_environment_write_roots_the_old_value_before_the_stores_allocation`
  (`tests/collect_stress.rs:408`) exists; its can-fail control was not re-run by me.
- ✅ **Important 1** (`findClass` of an owed name). `environment.rs:1880` refuses `Owed`, and
  errors propagate. RAN, `probes/fc/`: a raising stored method, a raising `.local` `UNKNOWN`, a
  raising `.environment` `UNKNOWN`, a raising security manager, a trapped raise, method entries on
  both directories -- rc and stdout match the oracle on all; stderr differs only in the recorded
  pre-existing `running RUN` word, and the manager case is IDENTICAL on all three descriptors.
  Nothing found that now raises where the oracle answers. **But** see Out-of-Scope: the same
  swallow survives in `directive_class` and gives a wrong rc.
- ✅ **Minor 2** (`STDQUE` removals over-refusing). `hash.rs:899` reads with `slot_at`;
  `take_merged` (`hash.rs:1099`) splits answered from discarded. READ the C++
  (`HashCollection.cpp:839`-`:867`, `DirectoryClass.cpp:308`-`:347`, `:450`-`:530`): `setEntry`
  and `removeEntry` share `DirectoryClass::remove` and differ only in the stub discarding the
  answer; a contents-hit `get` has no side effect; `setMethodRexx` never reads. The split is the
  oracle's observable one. RAN, `probes/rm/side.rex` (side-effect counts for every
  discarded/answered/method/`UNKNOWN` combination) IDENTICAL; `probes/rm/copy.rex` agrees except the
  expected loud `removeEntry` refusal.
- ✅ **Minor 3** (`.NAME` miss path 1.88x). Report corrected in place (the sentence's order checked
  against `dot_variable`, `environment.rs:493`-`:532`) and a cache added. The cache is reviewed as
  a new risk below. RAN, callgrind, a variant where every pass makes a `Directory` (so the counter
  moves every pass): head 6,404 / 6,407 Ir per pass against the pre-cache build's 8,775 / 8,784, so
  no regression on that shape.
- ✅ **Minor 4** (no witness for index reads running no method).
  `corpus/lang/directory_enumeration_runs_no_method.rex` added; its header matches the oracle's
  output line by line (RAN, IDENTICAL, and IDENTICAL under the debug stress harness); companion
  `count 42`, body `cmp`-equal. The report's C5 control not re-run.
- ✅ **Minor 5** (the method-table test passing a `.nil` degenerate). Renamed to
  `every_directory_method_answers_on_both_directories_as_on_a_copy` (`environment.rs:2363`) and
  compares transcripts against a filled `Directory`. RAN control C-D (my own `.nil` degenerate in
  `invoke`'s native arm), prediction written first: export `1 passed`, mutant `1 failed` at the
  transcript `assert_eq!` with `ALLINDEXES The NIL object` -- as predicted.
- ⚠️ **Minor 6** (over-claiming doc comments). `environment.rs:620` is now true ("An owed entry
  answers `None`"). `dispatch.rs:10491`-`:10494` lost the flagged universal, but the rewritten
  sentence keeps "each reaches the refusal through a different path", which is false for
  `allItems~items` against `do over allItems` and for `remove` against the round's own new
  `removeEntry` row (both refuse at `hash.rs:1109`).
- ✅ **Minor 1 and the `equivalent` refusal** recorded, not fixed, as directed. RAN three of the
  recorded shapes (`probes/nd/`): they still diverge exactly as the report records.

### New Issues In The Fix

#### Critical

None found.

#### Important

None introduced by the round's diff. (The Important-severity item found is pre-existing; see
Out-of-Scope.)

#### Minor

1. **`lib.rs:1937`-`:1938`, `Interp::store_generation`'s doc is still a false universal**: "a write
   to a mapped collection's pool entries moves it". `duplicate_collection_stores`
   (`dispatch.rs:5865`, a collection's `copy`) writes store pool entries through `set_pool_variable`
   without moving it. `24d459c82` says it narrowed this sentence; it rephrased it. READ (every write
   of the field enumerated by `/bin/grep -rn -a store_generation`). Harmless in behaviour.
2. **`lib.rs:5284`-`:5285`, the collector field audit's reason for `environment: _` is false since
   `96bf51e3e`**: `EnvironmentModel::views` (`environment.rs:256`) holds store arrays that are
   neither globals nor classes. READ. They are safe only through the generation check, which the
   audit does not say.
3. **`environment.rs:2378`, "`EMPTY` last, since it takes every entry out"**: RAN on the oracle,
   `empty` leaves method-table entries (`.environment~empty` leaves `1 LOCAL`); the test's own
   passing transcript ends `final 1 array 1 LOCAL`.
4. **`hash.rs:1087`, `Removal`'s doc** ("whether a removal hands the item it takes back to the
   program") is contradicted by `removeItem`, which passes `Discarded` (`hash.rs:1405`) and answers
   the item it removed. READ. It is safe because `pairs` checked first; the doc hides that.
5. **The report's Fix round 1 section states three things that are not so**: item 6's "the copy
   program also prints a line if its order does not match" (no such line in the test; order is
   compared only through the transcript), item 4's "every write to a store's pool entries" (see 1),
   and the function name `hash::directory_get_viewed` (zero hits; it is `directory_get`,
   `hash.rs:2242`). READ. Also stale without a pointer: `task-1-report.md:265` and `:384`-`:399`
   still give `dotname` 1.62x / 1.614 as the final figure.
6. Nits: the test's `SETENTRY` row (`environment.rs:2389`) passes `'zz value', 'ZZ'`, which
   `setEntry(name, value)` reads as an entry `ZZ VALUE` holding `ZZ` (visible as `ZZ VALUE=ZZ` in the
   `SUPPLIER` transcript), unlike `PUT`/`[]=` beside it; harmless to the comparison.
   `views`' doc "the last reading" is not the last in the nested case (an outer lookup writes back an
   older reading than an inner one), which only costs a miss.

**Risk 1, the cache, checked and clean.** Every mutation of an instance's pools goes through four
`Interp` methods; every `set_pool_variable` caller is tabled in the notes. The only unbumped
store-entry write targets a `copy`. EXPOSE/DROP/`VariableReference`/native `SetObjectVariable`
could alias the store only from a scope equal to the `Table` class, which no program can obtain
(`define` and `inherit` on it refuse 98.985 on both sides, RAN `probes/define_table.rex`; a stored
method runs at `.nil`; EXPOSE from `run` of a `Method~new` did not alias, RAN; no `::EXTENSION` in
this oracle). The heap does not move. RAN on oracle, release and debug:
`~objectName=`, `copy` then writes, `init`, EXPOSE of the store names from a stored method and from
`run`, nested growth inside `UNKNOWN` and method entries, `putAll`, bulk `setEntry` removal,
`removeItem`, `setMethod` over contents, on both directories -- all IDENTICAL, the debug assertion
silent, and the three new witnesses plus two probes byte-equal to the oracle under
collect-on-every-allocation in debug. Control C-A (the `set_unknown_method` bump deleted),
predictions written first: release reddens exactly the predicted lines of the witness and of
`c2b_nested.rex`; debug fires the reuse assertion (rc 101), with one prediction falsified -- stdout
is empty, not cut after the `method` line, because the panic loses the buffer.

### Out-of-Scope Observations

- **Important, pre-existing since `6bf1401fd`, same class as Important 1:** `directive_class`
  (`environment.rs:557`) still swallows `directory_lookup`'s errors with `if let Ok(...)`. RAN,
  `probes/dc/main.rex` (a required file's prolog gives `.environment` an `UNKNOWN` that divides by
  zero for `ZZBASE`; main declares `::class foo subclass zzbase`): oracle rc 214, `Error 42.3`
  running `UNKNOWN`; head rc **158**, `Error 98.909: Class "ZZBASE" not found.`, stdout equal. The
  fix base binary (`rexx-run.final`) answers the same as head; the task base refuses `setMethod`.
  The swallow predates Task 1 (`d7eafeb54` reads `if let Ok(Some(found))`); Task 1 rewrote the
  line and made a raising entry reachable. Item 2's change (propagate) is the fix.
- `VALUE(name, new, '')` followed by `VALUE(name, , '')` loses the write behind a `.local` method
  entry of the same name (oracle `YYY...`, crate `MMM...`); this is Minor 1's recorded route, noted
  because the recorded probes show only reads.
- The stored-method name divergence (`running RUN` for `running <name>`) now also appears on every
  raising `findClass` route.

### Assessment

**Fix round:** Needs another round

Every finding is fixed or recorded as directed, and the cache survived an adversarial enumeration,
two controls and a debug stress run. The next round is small: apply item 2's propagation to
`directive_class` (a wrong rc reachable today through a public, oracle-answering path in the same
defect class), and correct the round's false statements -- the `store_generation` doc it claimed
to narrow, the field-audit reason `views` falsified, the `EMPTY` comment, `Removal`'s doc, the
route test's "different path" clause, and the three report sentences.

## Working notes

### Risk 1, the pool reading cache (`96bf51e3e`)

* READ, the enumeration. `ScopePools` is mutated only by `set` and `clear`
  (`rexx-core/src/body.rs:436`, `:453`), and those are called from four `Interp` methods only:
  `set_exposed_variable` (EXPOSE), `clear_exposed_variable` (DROP of an exposed name),
  `set_pool_variable` and `clear_pool_variable` (`lib.rs:5047`-`:5120`). No whole-body replacement
  of an instance exists (`/bin/grep -a '\.body = '` finds only the weak-reference clear in
  `heap.rs:149`), and the collector is non-moving mark-sweep (`heap.rs:95`), so a handle in a view
  cannot be relocated. `store_generation` is written only by `bump_store_generation`
  (`hash.rs:410`), called from `install_store`, `set_free` and `set_unknown_method`.
  `set_pool_variable`'s callers, and whether each can write `.local`'s or `.environment`'s pool
  under the store's scope (`hash_scope` = the `Table` class object, `hash.rs:88`):
  | caller | reaches the two directories' store entries? | counter |
  |---|---|---|
  | `hash.rs` `install_store` (store of `new`, `empty`, growth, first `setMethod`) | yes | moves |
  | `hash.rs` `set_free` (overflow put, overflow removal, chain close) | yes | moves |
  | `hash.rs` `set_unknown_method` (`setMethod`/`unsetMethod` of `UNKNOWN`) | yes | moves |
  | `dispatch.rs:5880` `duplicate_collection_stores` (`copy`) | writes the COPY's store entries only | does not move |
  | `dispatch.rs:2268` attribute setter | scope is the attribute's defining class; `Table` is Rexx-defined and refuses `define` (`dispatch.rs:3885`, oracle `ClassClass.cpp:823`) | n/a |
  | `dispatch.rs:5363` `VariableReference~value=` | only a reference made under an EXPOSE in scope `Table` | n/a |
  | `dispatch/library.rs:214` native `SetObjectVariable` | the native frame's scope, which a program cannot make `Table` | n/a |
  | `collection.rs` queue/supplier/list entries, `stream.rs:92`, `dispatch.rs:8636` | other names on other classes | n/a |
  EXPOSE and DROP write under the running activation's scope. A stored `setMethod` entry runs at
  scope `.nil` (`hash.rs:975`); `run` runs a `Method~new` at its own scope; no Rexx code in
  `CoreClasses.orx` is scoped to `Table`; `::EXTENSION` does not exist in this oracle's parser
  (`/bin/grep -i extension interpreter/parser/*.cpp` empty).
  `~objectName=`, `init`, `uninit` and `enhanced` write nothing in the store's scope (`init` on a
  hash collection is `native_capacity_init`, which validates and answers nothing,
  `dispatch.rs:6382`; `enhanced` makes a new instance).
* RAN, both routes I doubted, oracle vs release vs debug (`probes/c1_generic.rex`): a `.NAME` read
  of a `.local` entry after `~objectName=`, a `~copy` whose store is then written and emptied,
  `init` and `init(500)`, a stored method that EXPOSEs `hashitems hashindexes methodunknown` and
  reads them (`0 0 0` on all three sides), a stored method that assigns `hashitems`, a
  `self~run(.Method~new(...))` that assigns `hashitems` and `methodunknown`, then the same set on
  `.environment` with a copy given an `UNKNOWN` method. IDENTICAL on release; the debug binary's
  stdout, stderr and rc equal the oracle's too, so the reuse assertion stayed silent.
* RAN, nested writes (`probes/c2b_nested.rex`): an `UNKNOWN` method on `.local` that grows `.local`
  by 70 puts and then reads another `.NAME` inside the outer lookup (so the outer lookup writes back
  a view older than the inner one), `putAll` of 40 entries, 70 `setEntry` removals, `removeItem`,
  `setMethod`/`setMethod` with no source over a contents name, a stored method that grows `.local`
  and is read twice. IDENTICAL on release and on debug.
  (A first version, `c2_nested.rex`, used `.local~put` inside `.local`'s own `UNKNOWN` method;
  `.LOCAL` is itself looked up in `.local` first, so it recursed to rc 245 on the oracle. Probe
  error, discarded.)
* RAN, the three committed witnesses and both probes under collect-on-every-allocation in a
  **debug** harness built from a `git archive 24d459c82` export
  (`scratchpad/t1-rereview/harness`, `rexx_exec::run_program_collect_every_alloc`):
  `environment_symbol_after_store_changes.rex` 629 collections, `local_stdque_removal.rex` 33,
  `directory_enumeration_runs_no_method.rex` 61, `c1_generic.rex` 59, `c2b_nested.rex` 305; all
  exit 0 and every stdout and stderr byte-equal to the oracle's.
* Verdict: I found no reachable write to either directory's store entries that leaves the counter
  where it was. The one unbumped store-entry write, `copy`'s, targets an object no view is ever
  taken of.

### Risk 2, the VALUE fix and the caller table (`81c927160`)

* READ, `builtin/platform.rs:314`-`:331` at head: `old` is pushed before the write, unconditionally
  (the read-only form pushes too, harmlessly). The caller table in the report's item 1 checked row
  by row against `environment.rs`: `build_environment` (`:384`-`:447`, the four texts pushed, the
  placeholders and directories global), `mint_local_entries` (`:961`-`:1057`: word texts and
  `arguments` pushed; each monitor's `target` is `.local`'s own entry, put before the monitor is
  built; `built` pushed), `set_directory_entry` (`:1064`-`:1082`: an `Access::Direct` admission
  returns before any allocation, then `directory_put` pushes the index). Matches.
* READ, the round's own new write paths. Item 3's `take_merged` holds `old` from `merged_get`
  across `take`/`take_in` exactly as base did on the answered path; the discarded path drops it.
  `native_hash_remove_item` holds `item` from `pairs`, which roots both halves (`hash.rs:1211`-`:1212`,
  `:1070`). Item 2 writes nothing.
* RAN, `probes/v1_value.rex` (absent name, a heap `new`, a `.local` `setMethod` entry answering a
  fresh string then written over, a `.local` `UNKNOWN` method, 30 writes of fresh names in a loop)
  under the debug stress harness: exit 0, 332 collections, stdout equal to the release binary's.
  Against the oracle it differs on four lines, every one the recorded Minor 1 (the read goes
  through `.local` first): oracle `.QRSTUVWXYZABCDEF` / `YYY...` / `.ZZNOTTHEREATALLQ` /
  `LLL... 30`, crate `MMM...` / `MMM...` / `UUU... ZZNOTTHEREATALLQ` / `UUU... LOOPNAME30...`.
  Note the second of them is a *write* made invisible to a later `VALUE` read by a `.local` method
  of the same name; still the same recorded route, not new in this round.

### Risk 3, `findClass` propagating errors (`24f8879e3`)

* READ, `package_find_class`'s only caller is `dispatch/package.rs:687` (`/bin/grep -rn -a
  package_find_class crates/`), so no internal resolution path changed. At base the `if let Ok(...)`
  swallowed every `Err` from `directory_lookup`: a stored method or `UNKNOWN` method that raises,
  a security manager check that raises, and any `Loud` inside. The oracle
  (`PackageClass.cpp:1081`-`:1158`) calls `getLocalEnvironment` and `TheEnvironment->entry`, both
  `DirectoryClass::get` (`DirectoryClass.cpp:332`), with nothing catching.
* RAN (`probes/fc/`, package captured into a variable before any `UNKNOWN` is set -- a first
  pair of probes read `.context` after setting `UNKNOWN` and so tested `.CONTEXT`'s resolution
  instead; discarded):
  * `raise_method.rex` (stored method `1 / 0`): rc 214 both, stdout equal, stderr differs only in
    `running RUN` against `running ZZRAISE` (the review's pre-existing stored-method name
    divergence).
  * `raise_unknown_local2.rex` (a `.local` `UNKNOWN` raising 93.900 for one name, answering for
    others): `findClass('array')~id` `Array`, `findClass('zzother')` `u ZZOTHER`, then rc 163 both;
    stderr differs only in `RUN`/`UNKNOWN`.
  * `raise_unknown_env2.rex` (the same on `.environment`, `1 / 0`): rc 214 both, same one-word
    stderr difference.
  * `secmgr.rex` (a manager whose `LOCAL` check answers for one name and raises for another):
    IDENTICAL on all three descriptors, rc 214.
  * `trapped.rex` (the raise under `SIGNAL ON SYNTAX`, then an absent name, `STDQUE` after
    replacement, `LOCAL`, an absent name): IDENTICAL.
  * `methods_env.rex` (method entries on both directories, a `.local` entry shadowing an
    `.environment` method, then removed by `setEntry`): IDENTICAL.
* Verdict: rc and stdout match on every raising route I tried; the stderr word is the pre-existing
  divergence. I found nothing that now raises where the oracle answers.

### Risk 4, removal without reading (item 3)

* READ, the oracle. `StringHashCollection::setEntry` with no value and `removeEntry` both call the
  virtual `remove(name->upper())` (`support/HashCollection.cpp:854`-`:861`, `:839`-`:843`);
  `HashCollection::removeRexx` calls `remove` (`:317`-`:324`); `DirectoryClass::remove`
  (`DirectoryClass.cpp:308`-`:322`) is `get` then `contents->remove` then `methodTable->remove`;
  `DirectoryClass::get` (`:332`-`:347`) runs a method only on a contents miss;
  `DirectoryClass::removeItem` (`:450`-`:460`) is `getIndex` then `remove(i)`;
  `setMethodRexx` (`:480`-`:530`) ends in a bare `contents->remove(entryname)` with no `get`.
  So in C++ `setEntry` and `removeEntry` take the *same* path, reading the item in both; the only
  difference is the Rexx stub discarding or answering it. A contents-hit `get` has no side effect,
  so skipping it where the answer is discarded is unobservable, and the split is between what the
  program sees, not between C++ paths. `setMethod` never reads, which the crate's bare `take`
  (`hash.rs:2425`) matches. That is what `take_merged` (`hash.rs:1099`-`:1126`) implements:
  discarded-and-contents-hit skips `get`, a contents miss still runs `merged_get` (so a method or
  `UNKNOWN` still runs, as in C++).
* READ, what `remove_at` losing its owed check exposes. Its callers: `take_in` (from `take`,
  `take_merged`, `insert`'s method-half removal, `setMethod`, `unsetMethod`,
  `native_relation_remove_all`) and `take_at` (`native_relation_remove_item`). The answered
  Directory routes check first (`take_merged`'s answered branch, or `merged_get`'s `item_at`);
  `removeItem` answers the item `pairs` already checked; the `Relation` routes cannot hold a
  placeholder, which only a `.local`-derived Directory (a `copy`, a `difference`) carries.
* RAN, `probes/rm/side.rex` (side-effecting `UNKNOWN` and method entries): `setEntry` of a
  contents hit runs no `UNKNOWN`; `setEntry` and `removeEntry` of a miss run `UNKNOWN` once;
  `setEntry` of a method entry runs it once; a plain Directory's `removeItem` of a method result
  runs the method twice, of a contents item runs nothing; `setEntry`/`remove` of a miss with
  `UNKNOWN` run it once. IDENTICAL.
  `probes/rm/copy.rex`: a `.local~copy`'s `setEntry('stdque')` and `setMethod('stdque', ...)`
  identical to the oracle; its `removeEntry('stdque')` refuses Phase 10 where the oracle answers the
  queue (loud, expected).
* Verdict: the split matches the oracle's observable behaviour.

### Controls (predictions written before any run)

**C-A, item 4's `set_unknown_method` bump removed** (`hash.rs:952` deleted, nothing else), built
as `rexx-run` in release and in debug from a copy of the export.
* Release prediction. `environment_symbol_after_store_changes.rex` differs from the oracle on its
  `unknown` line only: `unknown  .ZZABSENT local one again` against the oracle's
  `unknown  unknown ZZABSENT local one again`; the `unset` line agrees by accident (the stale view
  has no unknown method and `unsetMethod` does not bump either). `probes/c2b_nested.rex` differs on
  lines `c` and `d` (`c .ZZMISS`; `d .ZZNEST1 .ZZNEST70 seed .ZZMISS2`) and agrees from `e` on.
  `probes/c1_generic.rex` IDENTICAL (its only `UNKNOWN` is set on a copy).
* Debug prediction. The same witness panics at the first `.NAME` read after `setMethod('UNKNOWN')`:
  stdout stops after `method   via a method`, stderr holds `a StoreView was reused after its
  directory's pool changed without a bump_store_generation`, rc not 0.

**C-D, item 6's degenerate** (my own implementation of the report's D: in `Interp::invoke`, once
the library bootstrap is over, a native method `Directory`'s own table binds, sent to
`.environment` or `.local`, answers `.nil` without running), run as the single lib test
`every_directory_method_answers_on_both_directories_as_on_a_copy`.
* Prediction: red, and at the transcript comparison rather than at the exit-code or stderr
  assertions: the message `.environment answered differently from a Directory holding its entries`,
  the left transcript's `ALLINDEXES` line `ALLINDEXES The NIL object`. The same test on the
  unmutated export in the same profile: green.

**C-A, run** (mutant `scratchpad/t1-rereview/mutA`, one line deleted, `diff` against the export shows
only it; binaries `bin/rexx-run.mutA-rel`, `bin/rexx-run.mutA-dbg`).
* Release: the witness differs on stdout line 7 only (`unknown  .ZZABSENT local one again` against
  `unknown  unknown ZZABSENT local one again`), rc 0 both, stderr equal. `c2b_nested.rex` differs on
  lines 3-4 only, exactly the predicted `c .ZZMISS` / `d .ZZNEST1 .ZZNEST70 seed .ZZMISS2`.
  `c1_generic.rex` IDENTICAL. **As predicted.**
* Debug: rc 101, stderr the reuse assertion at the mutant's `hash.rs:2250`, the two views differing in
  `unknown: None` against `unknown: Some(...)`. **Falsified in one part:** stdout is empty, not cut
  after the `method` line -- the panic loses the buffered stdout. The assertion is live in a
  debug build of this export, so the silent debug runs under Risk 1 above are evidence.

**C-D, run** (mutant `scratchpad/t1-rereview/mutD`: the early `.nil` return in `invoke`'s `Native`
arm plus a `zz_is_interpreter_directory` helper beside `directory_scope`; `diff -r` shows only
those; own target dirs for export and mutant, `cargo test -p rexx-exec --lib -- --exact
environment::tests::every_directory_method_answers_on_both_directories_as_on_a_copy`, run count
checked).
* Export: `1 passed; 0 failed; ... 820 filtered out`, exit 0.
* Mutant: `0 passed; 1 failed`, exit 101, panic at the mutant's `environment.rs:2483` (the transcript
  `assert_eq!`), message `.environment answered differently from a Directory holding its entries`,
  left `ALLINDEXES The NIL object\nALLITEMS The NIL object\n...final The NIL object The NIL object`.
  **As predicted.**
* Seen in the right-hand transcript: the `SETENTRY` row's arguments are `'zz value', 'ZZ'`, which
  `setEntry(name, value)` reads as an entry named `ZZ VALUE` holding `ZZ` (the `SUPPLIER` line
  lists `ZZ VALUE=ZZ`). The two programs send the same thing, so the comparison is unaffected, but
  the row does not write `ZZ` as the table beside it (`PUT`, `[]=`) does.

### Risk 3 extended: the same swallow one function over

* READ: `directive_class` (`environment.rs:557`-`:562`), which resolves an unqualified `::CLASS`
  `SUBCLASS`/`INHERIT`/`METACLASS` target against `.environment`, still reads
  `if let Ok(hash::DirectoryEntry::Found(found)) = self.directory_lookup(...)`. Task 1 rewrote that
  line from `d7eafeb54`'s `if let Ok(Some(found))`; it is the only `if let Ok(` over a directory
  lookup among the lines `git diff d7eafeb54 24d459c82` adds.
* RAN, `probes/dc/main.rex` + `probes/dc/zzdcreq.rex` (the required file's prolog gives
  `.environment` an `UNKNOWN` method that says it ran and divides by zero for `ZZBASE`; the main
  file declares `::class foo subclass zzbase`):
  oracle rc 214, stdout `unknown ran for ZZBASE`, stderr `Error 42 running UNKNOWN line 1` /
  `Error 42.3`; head rc **158**, same stdout, stderr `Error 98 running RUN line 1:  Execution
  error.` / `Error 98.909:  Class "ZZBASE" not found.` under the `return 1 / 0` traceback line.
  The fix base's binary (`surface-t1/bench/rexx-run.final`, from `6bf1401fd`) answers the same as
  head; the task base's (`rexx-run.base`, `d7eafeb54`) refuses `setMethod` at rc 120. So the task
  made it reachable, the fix round did not introduce it, and item 2's fix is exactly the change it
  needs.

### Risk 6, the round's comments and its report

Every comment line the round added (`git diff 6bf1401fd 24d459c82 -- rust/crates | /bin/grep -a -n
-E '^\+ *(//|/\*|\*)'`), each read against the code; the false ones:

* `lib.rs:1937`-`:1938` (`store_generation`): "a write to a mapped collection's pool entries moves
  it". READ: `store_generation` is written only in `bump_store_generation` (`/bin/grep -rn -a
  store_generation crates/ --include=*.rs`), whose callers are `install_store`, `set_free` and
  `set_unknown_method`; `duplicate_collection_stores` (`dispatch.rs:5865`-`:5882`, every `copy`)
  writes a `Directory`'s or `Table`'s `HASHINDEXES`/`HASHITEMS`/`HASHNEXT`/`METHOD*` pool entries
  through `set_pool_variable` and does not move it. This is the sentence `24d459c82` says it
  narrowed; "Bumped by every write" became "a write ... moves it", the same universal. Harmless
  (no view is taken of a copy), false.
* `lib.rs:5284`-`:5285` (collector field audit, `environment: _`): "`.environment`, `.local` and
  the owed placeholders are globals; the rest are classes." Since `96bf51e3e`,
  `EnvironmentModel::views` holds the directories' store arrays (and the unknown method's
  handle), which are neither; they are safe unrooted only because a view is dereferenced while its
  generation says the directory's pool still holds them. The audit's stated reason no longer
  covers the field.
* `environment.rs:2378` (test): "`EMPTY` last, since it takes every entry out." RAN,
  `probes/empty_methods.rex`: on the oracle `empty` leaves the method table -- a plain Directory
  with a method and an `UNKNOWN` answers `1 M 2 3` after `empty`, and `.environment~empty` leaves
  `1 LOCAL`; the crate agrees, and the test's own passing transcript ends `final 1 array 1 LOCAL`.
* `dispatch.rs:10491`-`:10494` (route test): "... and each reaches the refusal through a
  different path." READ: `.local~allItems~items` and `do n over .local~allItems` both refuse
  inside the one `ALLITEMS` send; `remove('STDQUE')` and the new `removeEntry('stdque')` both
  refuse at the same `item_at` call in `take_merged`'s answered branch (`hash.rs:1106`-`:1110`).
  The clause predates the round, but the round rewrote this sentence and added the row that
  duplicates `remove`'s path.
* `hash.rs:1087` (`Removal`): "Whether a removal hands the item it takes back to the program."
  `native_hash_remove_item` passes `Removal::Discarded` (`hash.rs:1405`) and then answers the very
  item it removed (`return Ok(Some(item))`). It is safe because `pairs` refused the owed item
  first; the doc describes the opposite of that caller.
* `environment.rs:254`-`:255` (`views`): "The last reading of `.local`'s pool and of
  `.environment`'s". In the nested case `c2b_nested.rex` runs, the outer lookup writes back the
  reading it made before the inner lookup's newer one. Stale, so only a miss; a nit.

The report's Fix round 1 section:

* Item 6: "and the copy program also prints a line if its order does not match the directory's."
  READ: the committed test's copy program (`environment.rs`, the `copy` `format!`) is the fill
  loop and the `LOCAL` method only; no such line exists (`awk` over the test body, `/bin/grep -i
  order` finds only the Rust `ordered` vector). Order is still compared, through the `ALLINDEXES`,
  `MAKEARRAY`, `SUPPLIER` and `final` transcript lines.
* Item 4: "`install_store`, `set_free` and `set_unknown_method` -- every write to a store's pool
  entries -- bump" is the universal the comment commit claims to have removed from the code,
  still stated here; and "`hash::directory_get_viewed`" names a function that does not exist (it is
  `hash::directory_get`).
* "Re-reading this round's own comments": says `Interp::store_generation`'s sentence was narrowed;
  see the first bullet above.
* The item it "corrected in place" (`task-1-report.md:88`-`:94`): READ against `dot_variable`
  (`environment.rs:493`-`:532`), which asks `installed_class`, `imported_class`,
  `rexx_package_class` and `package_local_entry` before `directory_lookup` and `rexx_variable`
  after it -- the order the corrected sentence states is the code's. Untouched and now stale: `task-1-report.md:265`
  ("`dotname` stays **1.62x** base") and `:384`-`:399` ("interleaved, final code", `1.614`),
  which item 4's `1.147` supersedes without a pointer back.

### Minor 3's cost when the counter moves every pass (RAN)

The counter is one per `Interp`, and `install_store` runs for every new mapped collection, so a
loop that makes a collection per pass invalidates both views every pass. `bench/missw.rex` is the
review's `miss.rex` with `d = .Directory~new` added to each of 20,000 passes, `bench/ctlw.rex` the
same over constants; callgrind, `surface-t1/bench/rexx-run.3cd2` (the round before item 4) and the
head release binary alternated, two rounds (`bench/results.txt`):

| per pass, missw less ctlw | round 1 | round 2 |
|---|---|---|
| `3cd2` | 8,775.3 | 8,783.6 |
| head | 6,404.3 | 6,407.3 |

So head is not slower than before the cache on that shape; the first `.NAME` per pass re-reads and
the rest reuse. Not a finding.

### Late check (RAN)

`probes/define_table.rex`: `.Table~define('ZZ', 'expose hashitems; ...')` and
`.Table~inherit(.Comparable)` both 98.985 "User additions are not allowed to the REXX language
classes" -- IDENTICAL on all three descriptors, rc 158. The unreachability premise for a
`Table`-scoped EXPOSE holds by running, not only by reading `rexx_defined_lock`.
Tree after the review: `git status --short` empty.
