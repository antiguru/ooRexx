# Task 1 review -- `.environment` and `.local` answer the whole `Directory` surface

BASE `d7eafeb54`, HEAD `6bf1401fd`. Read-only review; probes ran from
`scratchpad/t1-review/` (fresh directory per side per run, three descriptors). The working notes
the verdicts below come from are at the end, each marked RAN or READ.

### Spec Compliance

- ✅ Step 1: each candidate (a)-(f) has a probe and a verdict in the report (`task-1-report.md:18`-`:129`),
  plus (g). I re-ran (a)'s order claim past growth and removal on both directories, and (d)'s
  security-manager claim, extended: both hold (RAN, notes).
- ✅ Step 2: the representation is written before the build log (`task-1-report.md:131`-`:172`) and
  is what the diff builds: `hash::new_directory` with `ENVIRONMENT_CAPACITY` 68 and default size for
  `.local`, oracle order, owed placeholders, `LOCAL` in the method half (`environment.rs`,
  `build_environment`).
- ✅ Step 3: `environment_directory_surface.rex` and `local_directory_surface.rex` send every name the
  in-crate test's own match enumerates (checked name by name against both files), with the
  whole-collection reads as ordered output, upper-casing, `setEntry` with no value, program-added
  entries, `remove('ARRAY')` then `.array`, and `.local` read before and after first `.output` use.
  Filed in `phase-8.txt`; the three `sourceline_oracle` companions have the right `count` and bodies
  byte-equal to the corpus files (RAN `cmp`).
- ✅ Step 4: controls A, A2 and B each have a prediction written before the run, and the falsified
  parts are reported as falsified.
- ✅ Step 5: recorded in the report and appended as section 8 of `docs/superpowers/plans/phase-8-l2.md`;
  the diff only appends to that file.
- ✅ Step 6: `closed_phases.rs` untouched, and what still names Phase 5 is tabled with owners.
- ❌ Step 3's "where `STDQUE` makes an answer Phase 10's, the refusal names Phase 10":
  `Package~findClass('STDQUE')` answers `.nil` at rc 0 (`environment.rs:1854`). See Important 1.
- ⚠️ "every `Directory` method" is read as `Directory`'s own table, which matches the brief's wording.
  Of the inherited methods, `union`, `intersection`, `difference`, `subset`, `disjoint` and `putAll`
  on `.environment` match the oracle; `equivalent` refuses on both directories through a Phase
  5-named operator refusal where the oracle answers `1`. That refusal is loud (RAN).
- ⚠️ Not re-run by me: the control runs (A2's 539 of 548, B's 546 of 548), the gate readings, and the
  `rexxcps`/`dotname` interleave.

### Strengths

- The representation removes a whole class of state: "owed" is now a fact about the store, which
  fixes the `unbuilt` snapshot defect (c) as a side effect rather than with a second mechanism.
- Order is right beyond what the witnesses cover. 150 program puts with interleaved removes on
  `.environment`, forcing growth, then re-putting a removed class name: all 197 names match the
  oracle's order. On `.local`, growth past `STDQUE`'s placeholder with `STDOUT` removed and re-put
  matches too (RAN).
- The owed check tests identity, so a `~copy` or a `difference` result that carries the placeholder
  still refuses rather than handing it out (RAN `q4`).
- The 88.909 change is correct down to the message text on every index-taking route I tried,
  including `setEntry`/`setMethod` and `.environment` (RAN). The `allIndexes`/`makeArray`/`DO OVER`
  change matches the oracle for a side-effecting `setMethod` (RAN).
- The D45 seam is still on every lookup path it was on (`dot_variable`, `local_route`,
  `set_directory_entry`, `package_find_class`, `resolve_class_target`). An auditing manager sees
  identical events on both sides, including the new store routes: UNKNOWN entry reads, `[]`,
  `entry`, and a program-added entry (RAN).
- Control A was reported honestly as falsified in its reach, and a narrower A2 followed. That is
  the right response.

### Issues

#### Critical

1. **`VALUE(name, new, '')` holds an unrooted value across the allocation this change added**.
   Sites: `rust/crates/rexx-exec/src/builtin/platform.rs:323`-`:325` and
   `environment.rs:1047` (`set_directory_entry`), which now calls `hash::directory_put`
   (`dispatch/hash.rs:2135`, `interp.text(name)`).
   - **What.** `old = interp.dot_variable(&dotted)?` is a fresh heap string (`.NAME`) when the
     name is absent. Before this diff, `set_directory_entry` wrote into `NativeObject`'s Rust map and
     allocated nothing, so `old` was safe. Now it allocates the index text, and it can also
     `expand`, while `old` is reachable from nowhere.
   - **Ran it.** `say value('ABCDEFGHIJKLMNOP', 'new', '')` under collect-on-every-allocation, with a
     harness linking `run_program_collect_every_alloc`. At HEAD it panics `a live value`
     (`dispatch.rs:1510`, via `to_text <- say_evaluated`). The same harness on a `git archive
     d7eafeb54` build prints `.ABCDEFGHIJKLMNOP` / `new`, exit 0, collections=21. On HEAD the
     read-only form, a present name and `say 1` all pass. The oracle and the ordinary release
     binary agree with each other, so only a collection landing on that allocation exposes it. That
     is the crash class `values_compound_write_roots_the_old_value_before_the_stems_first_allocation`
     exists for.
   - **Fix.** Push `old` as a temp before `set_directory_entry` in `environment_directory`. Add a
     `collect_stress.rs` row for the long-name `VALUE(name, new, '')` shape, modelled on the
     compound row, and show that it reddens without the push.

#### Important

1. **`Package~findClass` of an owed `.local` name answers `.nil` silently** (`environment.rs:1854`,
   reached from `dispatch/package.rs:687`).
   - **What.** The rewritten line matches only `DirectoryEntry::Found`, so `Owed` falls through to
     `ObjRef::NIL`. `say .context~package~findClass('STDQUE')`: the oracle prints `SESSION` (the
     queue) and the crate prints `The NIL object`, both at rc 0 (RAN).
   - **Why it matters.** The same answer comes from the base binary the report recorded
     (`surface-t1/bench/rexx-run.base`, sha256 `b8106b3f...`), so the wrong answer predates this
     diff. But the diff rewrote this exact line with the refusal one variant away. It also makes
     two claims false: the task's "every value read refuses naming Phase 10", and the committed
     prose at `dispatch.rs:10493` ("Every read that answers or compares `STDQUE`'s item refuses")
     and `corpus/refusal-sites.tsv:122` ("any read of .local's items").
   - **Fix.** Let `package_find_class` return `Result` and turn `Owed(owner)` into
     `Loud::environment_symbol` or `environment_entry`. Add a `findClass('STDQUE')` row to the
     `STDQUE` route test.

#### Minor

1. **A new instance of an existing `VALUE(name, , '')` divergence** (`platform.rs:323`).
   - **What.** The oracle reads `TheEnvironment->entry` only (`BuiltinFunctions.cpp:1856`); the crate
     goes through `dot_variable`. At BASE this already diverged: `value('STDOUT', , '')` is oracle
     `.STDOUT` against crate `STDOUT`, and `value('ZZONLYLOCAL', , '')` answers `.local`'s entry
     (RAN on the base binary too). This diff adds a reachable case: with
     `.local~setMethod('UNKNOWN', ...)`, `value('ZZNOPE', , '')` is oracle `.ZZNOPE` against crate
     `U: ZZNOPE`.
   - **Fix.** Out of scope. Record it for the handover; `.environment` is now a real store, so the
     fix is a `directory_get` on it alone.
2. **Over-refusals on `STDQUE` removals that do not answer the item** (`hash.rs:891`, `remove_at`
   reading through `item_at`).
   - **What.** `.local~setEntry('STDQUE')` with no value (oracle removes it and prints `9 0`) and
     `.local~setMethod('STDQUE', ...)` (oracle `1 10`) both refuse naming Phase 10 (RAN).
   - **Impact.** Loud, so not a wrong answer. It is wider than "reads that answer or compare".
   - **Fix.** Accept it and say so in the refusal row, or skip the owed check where the removed item
     is discarded.
3. **The miss path of `.NAME` is 1.88x base, and the report's list of who pays leaves it out**.
   - **What.** Measured with callgrind, base then HEAD, two rounds that reproduced to five
     significant figures. 50,000 passes of `.rs`, `.context` and an unresolved `.zz...`, minus a
     constant-only control loop: 4,738 Ir per pass at base against 8,904 at HEAD, so about +1,390 Ir
     per `.NAME` that misses both directories. Reflection names (`.rs`, `.line`, `.context`,
     `.methods`) search both directories before `rexx_variable`, as the oracle's order does.
   - **Where it goes.** No allocation. `read_parts` walks each directory's pool (`hash.rs:329`) and
     `text_slot` walks the chain (`hash.rs:2198`) on every lookup. Class references and
     `.nil`/`.true`/`.false` do not pay: the first by the security-manager probe showing no seam
     event for `.array`, the second by `eval.rs:354`.
   - **Fix.** Correct the report's claim. Optionally cache the two directories' `Found` handles,
     invalidated where `install_store` runs.
4. **The side-effect observable of the `allIndexes`/`makeArray`/`DO OVER` change has no committed
   witness**.
   - **What.** Reverting `native_hash_all_indexes` to `pairs` reddens `local_directory_surface.rex`,
     but only through the `STDQUE` refusal. No corpus program enumerates a `Directory` whose
     `setMethod` method has a side effect: `directory_set_method.rex`'s `TICK` is never enumerated.
     The oracle-matching behaviour on a program's own directory is unpinned. I ran the shape and it
     is identical.
   - **Fix.** Add the report's `dir_method_side.rex` shape as a witness.
5. **`every_directory_method_answers_on_both_directories` asserts less than its name says**
   (`environment.rs:2336`).
   - **What.** It checks that each own-table name resolves to the same method as on a new
     `Directory`, and that a program sending each one exits 0 with no "is not implemented". An
     implementation answering `.nil` for every send passes it.
   - **Impact.** The committed set as a whole does catch that. Both corpus witnesses send every name
     in its match against the live oracle, and A2 shows a post-bootstrap refusal reddens nine corpus
     programs and all three in-crate tests.
   - **Fix.** Rename it, or let the doc say that the values are the corpus witnesses' job.
6. **Two doc comments state false universals**.
   - `environment.rs:602`: "it is owed only before `Interp::mint_local_directory` runs". `STDQUE`
     stays owed after the mint. The sentence holds only for the route names callers pass.
   - `dispatch.rs:10493`: "Every read ... refuses". See Important 1.
   - **Fix.** Narrow both.

Not this task's, seen on the way (RAN): an `UNKNOWN` method that raises prints `running RUN` where
the oracle prints `running UNKNOWN`, on a plain `.Directory~new` too.

### Assessment

**Task quality:** Needs fixes

Critical 1 is a crash this diff introduced, found by running the project's own stress instrument,
and the fix is one push plus a stress row. Important 1 is a silent wrong answer on a route the diff
rewrote. The representation, order, growth, 88.909 and the D45 seam all check out against the
oracle past what the witnesses cover.

## Working notes (appended as the review went)

* Probe harness: `scratchpad/t1-review/ab.sh` (oracle under the wrapper, crate
  `rust/target/release/rexx-run` built 04:35 after the 04:30 commit, fresh dirs, three
  descriptors). A collect-on-every-allocation harness `scratchpad/t1-review/harness` links
  `rexx_exec::run_program_collect_every_alloc` from the HEAD tree in its own target dir;
  `scratchpad/t1-review/harness-base` the same against a `git archive d7eafeb54` export.
* RAN, risk 2: `probes/grow_env.rex` (remove two class names, 150 program puts with a remove
  every 7th, forcing growth, re-put one removed name) -- IDENTICAL, 197 items, order equal.
  `probes/grow_local.rex` (remove STDOUT, 40 puts with removes, growth past STDQUE's placeholder,
  re-put STDOUT) -- every ordered line identical; only difference the expected `.STDQUE` Phase 10
  refusal at the end.
* RAN, risk 1: `probes/value_rooting.rex` = `say value('ABCDEFGHIJKLMNOP', 'new', '')` under
  collect-on-every-allocation. HEAD harness: **panic `a live value`** at `dispatch.rs:1510`,
  backtrace `to_text <- say_evaluated`. BASE harness (same `main.rs`, own target dir): exit 0,
  `.ABCDEFGHIJKLMNOP` / `new`, collections=21. Controls on HEAD: `say 1` ok; the read-only form
  `value('ABCDEFGHIJKLMNOP', , '')` ok (collections=2); a present name ok. Oracle and the normal
  release binary both print `.ABCDEFGHIJKLMNOP` / `new`, rc 0.
* RAN, risk 4b: `probes/method_side.rex` (a `.Directory~new` with a side-effecting `setMethod`,
  then allIndexes/makeArray/DO OVER/items/hasIndex/allItems/supplier/[]/remove) -- IDENTICAL.
  No committed corpus program observes the side effect: `directory_set_method.rex`'s only
  side-effecting method (`TICK`) is never enumerated.
* RAN, risk 4a: ten uncaught one-liners (`at`, `entry`, `setEntry`, `setMethod`, `put`, `[]=`,
  `remove` on a `.Directory~new`, `StringTable~hasIndex`, `.environment~at(.nil)`,
  `.local~hasEntry(.array~new)`) -- all IDENTICAL including the 88.909 message text.
  `probes/keyed_rooting.rex` (60 fresh `makeString` keys forcing growth) IDENTICAL, and survives
  the stress harness (collections=135) -- the converted index is not a rooting defect.
* RAN, risk 5: `probes/secmgr2.rex` (the implementer's auditing manager plus UNKNOWN routes
  `.environment~array`, `.local~stdout`, `[]`, `entry`, a program-added entry, `.local~allIndexes`)
  -- IDENTICAL event lines; `.array` produces no event on either side.
* RAN, `probes/unknown_local.rex`: `.local~setMethod('UNKNOWN', ...)` then `.NAME`s -- `.foo`,
  `.rs`, `.line`, `.context` identical; **`value('ZZNOPE', , '')` differs**: oracle `.ZZNOPE`,
  crate `U: ZZNOPE`, rc equal. Oracle `BuiltinFunctions.cpp:1856` reads `TheEnvironment->entry`
  only; `platform.rs:323` reads through `dot_variable`. `probes/value_env_only.rex`: the same
  route already diverged at BASE (`value('STDOUT', , '')`, a `.local`-only name), HEAD and base
  binary print the same.
* RAN, risk 3 beyond the committed routes (`probes/sq/`): `.context~package~findClass('STDQUE')`
  -- oracle `SESSION` (the queue), crate **`The NIL object` at rc 0**; the implementer's copied
  BASE binary (`surface-t1/bench/rexx-run.base`, sha256 prefix `b8106b3f` as the report records)
  answers the same, so pre-existing, but the diff rewrote that line to match only
  `DirectoryEntry::Found`. Over-refusals, all loud Phase 10: `.local~setEntry('STDQUE')` with no
  value (oracle removes, `9 0`), `.local~setMethod('STDQUE', ...)` (oracle `1 10`). Loud and
  expected: `removeEntry`, a `~copy`'s `['STDQUE']`, `unsetMethod` then `[]`. `.local~empty`
  identical. `value('STDQUE', , '')` refuses where the oracle answers `.STDQUE` (VALUE's route,
  same at BASE). `probes/lq/`: `d~union(.local)` and `d~putAll(.local)` refuse Phase 10 (loud);
  `.local~intersection`, `~difference`, `~subset`/`~disjoint` identical; `.local~equivalent` and
  `.environment~equivalent` refuse through the Phase 5 operator refusal where the oracle answers 1,
  while `equivalent` on a plain `Directory` is identical.
* RAN, `probes/inherited2.rex`: `union`, `intersection`, `difference`, `subset`, `disjoint`,
  `putAll` and `copy` on `.environment` -- the answers are identical.
* RAN, `probes/unk_raise*.rex`: an UNKNOWN that raises -- `running RUN` here vs `running UNKNOWN`
  on the oracle; identical on a plain `.Directory~new`, so pre-existing and not this task's.
* RAN, `probes/req.rex`: the moved conversion row's comment (`14` items) and `.environment +
  1` = `The NIL object` confirmed on the oracle.
* RAN, risk 7: callgrind, `bench/miss.rex` (50,000 passes of `.rs`, `.context`,
  `.zzunresolvedname`) against `bench/ctl.rex` (same loop, constants), base binary then HEAD, two
  rounds, figures reproduced to 5 significant digits:
  base miss 406,207,370 / ctl 169,295,169; HEAD miss 615,267,079 / ctl 170,080,335. Loop-subtracted
  per pass: base 4,738, HEAD 8,904 Ir, so about +1,390 Ir per missing `.NAME` (1.88x). These
  names are not in the report's list of who pays.
* READ, risk 7: `.NIL`/`.TRUE`/`.FALSE` are parse-time constants (`eval.rs:354`), so they do not
  reach `dot_variable`. No allocation in `directory_get`'s hit or miss path; `read_parts` walks
  each directory's pool once per lookup, `text_slot` walks the bucket chain.
* READ, risk 1, every new ObjRef held across an allocation in the diff: owed placeholders
  (`add_global` before the next alloc), both directories (`new_instance` pushes a temp, then
  `add_global`), `true_value`/`false_value`/`rexx_info`/`end_of_line` (pushed), `directory_put`'s
  index (pushed), `directory_get`'s UNKNOWN index (pushed), the mint's word texts, `arguments`,
  streams (`new_instance`'s temp inside the mint frame) and monitors (pushed), `indexes()`'s
  indexes (pushed). The one unrooted value is VALUE's `old` in `builtin/platform.rs:323`-`:325`,
  newly exposed because `set_directory_entry` now allocates; see the stress run above.
  `validated_index`'s converted string survives `insert` growth and `take_merged`'s method run
  under stress (`probes/keyed_rooting.rex`, `probes/keyed_remove.rex`, collections 135 and 610).
* READ, risk 6: the in-crate enumeration test's program sends each name as a message clause and
  asserts only exit 0 and no "is not implemented", plus `lookup_at` equality; both corpus witnesses
  send every name in that test's match, compared against the live oracle.
