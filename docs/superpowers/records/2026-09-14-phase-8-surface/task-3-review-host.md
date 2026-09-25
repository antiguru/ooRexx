# Phase 8 surface, Task 3 -- host-slice review

Reviewer: host slice (everything outside `rust/crates/rexx-api/` and `rust/crates/rexx-num/`).
BASE `5d84dd8cb`, HEAD `034c1c7d7`. Read-only. Scratch:
`scratchpad/t3-review-b/`.

Work log (appended as I go; the structured sections follow at the end).

## Log

### Setup (ran)

* Diff reviewed in passes: (1) `rexx-exec` outside `library.rs` (`lib.rs`, `dispatch.rs`, `error.rs`,
  `run.rs`, `datatype.rs`, `tests/refusal_sites.rs`); (2) `dispatch/library.rs`; (3) corpus
  witnesses, `phase-8.txt`, sourceline companions; (4) `refusal-sites.tsv`; (5)
  `phase-4-exclusions.txt`. `rexx-api`/`rexx-num` hunks read only where the host depends on them.
* Base `5d84dd8cb` and head `034c1c7d7` built from `git archive` copies (`rust`, `interpreter`,
  `api`) into `t3-review-b/target-base` and `target-head`, `cargo build --release --locked
  --offline -p rexx-exec --bin rexx-run`, both exit 0.
* Probe runner `t3-review-b/probe.sh`: each side from a fresh copy of `src/` at the same path,
  three descriptors to files, `cmp` against the oracle. `liborxmethod`, `liborxfunction`,
  `librxmath`, `librxregexp`: `readelf -d` NEEDED libc/libm/libstdc++/libgcc only.

### Risk 2, `found` rendering (ran, `p/found2`, `p/rb1`-`rb8`)

`RexxObject::stringValue()` is `sendMessage(OBJECTNAME)` (`interpreter/classes/ObjectClass.cpp:1157`),
and `RexxObject::objectName` sends `DEFAULTNAME` for a non-base class (`:1713`); message
substitution calls `stringValue()` on every insert (`concurrency/Activity.cpp:1300`). Values
through `t~int` (88.907), `t~nonneg` (88.904), `t~logical` (34.901), `t~array` (98.913),
`RxCalcSqrt` (88.921), `t~positive` (88.905), `t~uint64` (88.907), each trapped, printing code and
message:

* **Agree** (head = oracle): stem with a default (`found "stemdefault"`), stem without
  (`"Z."`), `.nil`, object with `STRING` only, object with `MAKESTRING` only (both `"a WS"`/`"a WM"`
  for every row but logical), `Directory`, `MutableBuffer` (`"buf"`), `objectName=` set (`"bob"`),
  `Array` (`"an Array"`, every row but logical), a class (`"The String class"`).
* **String subclass: unreachable in the crate.** `.SS~new('subtext')` (`::class SS subclass
  String`) is `rexx-exec: method "NEW" of class "SS" is not implemented (Phase 5)`, rc 120
  (`p/found1`). Oracle renders `found "subtext"` for every row.
* **Diverges, introduced by this task: `logical_t`'s `found` is the string conversion's result,
  not `stringValue()`.** Oracle: `.array~of(1,2)` gives `found "1<LF>2"`, the `STRING` object
  `found "viaString"`, the `MAKESTRING` object `found "viaMake"`; head gives `"an Array"`, `"a
  WS"`, `"a WM"`. `truthValue` converts first and reports against the converted string. rc 0 both
  (trapped); untrapped this is a stderr difference. The witness's `t~logical(.object~new)` cannot
  see it: for an object with neither method the conversion falls back to `stringValue()`.
* **Diverges, pre-existing for 88.921/88.905, extended by this task to every new row:** a class
  overriding `DEFAULTNAME` or `OBJECTNAME`. Oracle `found "myDefault"` / `"myObjectName"`, head `"a
  DN"` / `"an ON"` on every row. Side-effect witness `p/rb8`: the oracle runs `DEFAULTNAME` twice
  (the conversion, then the message), base and head once (stdout `in defaultname|in
  defaultname|trapped 88.921` against `in defaultname|trapped 88.921`). So the new comment at
  `dispatch/library.rs:246`, "`found` is the argument's `stringValue()`, which sends nothing", is
  false about the oracle.
* Found in passing, **pre-existing and not native**: `p/rb7`, a `procedure` with `signal on syntax
  name refused` whose `r = v~string` reaches a `DEFAULTNAME` that does `raise syntax 40.1`: oracle
  traps (`trapped 40.1|back`, rc 0), base and head do not (rc 216, `Error 40.1`). The same at top
  level (`p/rb1`) and through a native call at top level (`p/rb2`, `p/rb4`) agree.

### Risk 1, frame reuse (`c00d18052`)

Read: `push_native_frame`/`pop_native_frame` (`dispatch/library.rs:194`-`237`), the roots walk
(`lib.rs:5634`-`5639`). No `?` or early return between push and pop in either caller
(`library.rs:85`-`96`, `:148`-`159`). Pop takes `raised`, clears `name`, `arguments`,
`argument_list`, `locals`; `owner`, `scope`, `receiver`, `method` stay stale in the spare and are
overwritten at the next push. Spares are not walked as roots, so a stale `ObjRef` there keeps
nothing alive; it is never read, since every reader goes through `native_handles.last()`.

Ran, `p/reuse3`, oracle against head, every line identical up to the last (which is
`MethodContextInterface.GetArgument is not implemented (Phase 8)`, rc 120, an unfilled slot and
expected):
* a refusal then a success, method (`t~int('x')` 88.907, then `t~int(5)` 5) and routine
  (`RxCalcSqrt('x')` 88.921, then `RxCalcSqrt(4)` 2);
* `OPTIONAL_int` given then omitted (`t~optint(5) t~optint()` is `5 0`; a leaked value or flag
  would make the extension raise "Conversion error"), and again after an omitted
  `OPTIONAL_RexxStemObject` call;
* the argument array not shared: `a = t~arglist(1); b = t~arglist(1)`, `a~append` then
  `IdentityTable~hasIndex(b)` `0`, items `2 1`; sizes `3 0 4 0` across `(1,2,3)`, `()`,
  `(,,,4)`, `()`; routine `3 0`;
* `NAME` across methods of different lengths and a routine (`NAME OTHER NAME NAMELONG TESTNAMEARG
  NAME`); `SCOPE`/`SUPER` alternating between a `T` method and a `U subclass T` method
  (`The T class The U class The T class The T class`, `The BASE class The T class The BASE
  class`); `OSELF`;
* stems by name alternating (`why dub why`), a 93.969 stem refusal then a stem object call;
* **nested native calls** (a Rexx `MAKESTRING` run by the outer call's `int` conversion calls
  `TestArglistArg`, `TestNameArg` and a method): the inner answers are the inner call's and the
  outer converts to the `MAKESTRING` answer; an inner refusal trapped inside `MAKESTRING`; an
  inner refusal **untrapped** there, which the outer call re-raises as 88.907 `found "bad"`, then
  a clean call; and the outer `ARGLIST` built after an inner `TestArglistArg(5,6,7,8)` ran during
  its `size_t` conversion (the inner line `in NI 4 NAME` agrees; the outer answer needs
  `GetArgument`, unfilled).
* **Not observable here:** a refused slot then a clean call (a slot refusal is `Loud`, rc 120, and
  ends the run); a handle from call 1 used in call 2 (no shipped Linux extension holds one:
  `RequestGlobalReference`/`static Rexx*Object` appear only under
  `extensions/platform/windows/oodialog/`). By construction a `Table` handle is the `ObjRef`'s bits
  (`rexx-api/src/handles.rs:27`-`46`) and `clear` empties the map, as the dropped frame did before.

RSS, ran, head binary directly under `/usr/bin/time -v` (`rss2/`): a loop of an `ARGLIST` method
with a 10,000-byte string and an array, a `NAME` method, an `ARGLIST` routine, and a trapped
88.907 refusal in a procedure. Max RSS 32,884 kB at 10,000 iterations, 32,960 at 100,000, 32,436
at 300,000, rc 0. No growth.

### Risk 3, stem names and the specials (ran)

`p/stem1`, oracle against head, **identical on all three descriptors**, rc 0: `TestStemArg` by name
from the top level (`y`, `Y.`); an internal routine without `PROCEDURE` (writes the caller's);
`PROCEDURE` (a fresh local stem, `symbol('Y.')` `VAR` inside, caller unchanged, `Z.` `LIT` after);
`PROCEDURE EXPOSE y.`; a `::ROUTINE` (its own); a method with `EXPOSE y.` (the object's), twice;
`INTERPRET`; `CALL TestStemArg 'y'` with `RESULT`; `.Routine~loadExternalRoutine(...)~call('y')`
and `~callWith` from a procedure (the procedure's local); a name resolved **inside a `MAKESTRING`
run by another `TestStemArg`'s conversion** (the method's local, then the outer resolves the
caller's); refusals for `.nil`, an object with no string method (`found "an OB"`), `5`, `'y '`,
`'y.1'`; `'!x'` accepted; an object whose `STRING` answers `y` accepted.

`p/cself1`: CSELF through `rxregexp` alternating two receivers (one a subclass) with an `orxmethod`
`NAME` call between, twice, identical (`1 0 0 1 NAME 1 3`).

Not reachable with shipped extensions, so ⚠️ unverifiable here: `OSELF`/`SCOPE`/`SUPER`/`CSELF`
in a routine (`orxfunction` declares only `NAME` and `ARGLIST` among the specials,
`testbinaries/orxfunction.cpp:404`-`428`); `orxmethod`'s CSELF buffer methods (`TestBufferInit`
needs `NewBuffer`, which `rexx-api` has no fill for). The implementer's forged-library transcript
`p/f1` is the only evidence for the routine side, outside the corpus as the brief requires.
`OSELF`/`SCOPE`/`SUPER`/`NAME`/`ARGLIST` method side agree in `p/reuse3` above.

### Risk 4, trapped refusal's `POSITION`/`PROGRAM`/`TRACEBACK` (ran against base, `p/pos1`)

Methods declared in a required `pk.cls`, trapped, printing code, `position`, `program` basename and
each `traceback` line. **Pre-existing, not introduced and not changed by this task.** For the rows
that existed at base (88.905 `t~positive(-1)`, 88.921 `t~double('x')`, 88.901 `t~positive()`)
base and head print byte-identical lines: `88.905 13 main.rex` and no `Compiled method` line, where
the oracle prints `88.905 The NIL object pk.cls` with `*-* Compiled method "POSITIVE" with scope
"T".` first. Head extends the same shape to the new rows (88.907, 34.901); base stops at rc 120 on
the unfilled `int` row. A **routine** refusal (`RxCalcSqrt('x')`, 88.921) diverges too, at base and
head: `POSITION`/`PROGRAM` agree (`13 main.rex`) but the oracle's first traceback line `*-* Compiled
routine "RXCALCSQRT".` is missing. The report's concern names only the method case.

### Risk 5, the SYNTAX message panic (ran)

Harness: `examples/stress.rs` added to the head archive copy only (calls the `#[doc(hidden)] pub`
`rexx_exec::run_program_collect_every_alloc`). Reductions under it (`st/`):
* `signal on syntax` / `y = 1/0` / `syntax: say condition('O')~message`, **three lines**, panics
  `dispatch.rs:1510:9` "a live value" (`Interp::not_in_arena`), backtrace through `try_text`,
  `render`, `concat_values`. So do `~errortext`, `c['MESSAGE']` and **`~program`**; `~code` runs
  (rc 0, 31 collections); a `raise syntax 93.900 array('boom')` condition's `~message` runs.
* Cause, read: `Interp::build_condition_object` (`rexx-exec/src/condition.rs:70`-`146`) gathers
  each entry's `ObjRef` in a plain `Vec<(&[u8], ObjRef)>` and roots none of them before the `PUT`
  loop; the allocations after each (`text_built`, `additional_array`, the index strings, the `PUT`
  sends) can collect every one that is not inline. `PROGRAM`, `ERRORTEXT` and `MESSAGE` are long
  enough to take a heap slot; `CODE`, `INSTRUCTION`, `PROPAGATED` are inline, which is why `~code`
  survives.
* **Reachable in an ordinary run, so Critical though pre-existing.** `p/ord1`, the ordinary head
  binary, no stress mode:

  ```
  a = .array~new
  do i = 1 to 20000
    a[i] = copies('x', 40) || i
    call probe
  end
  say 'done' i
  exit
  probe: procedure
    signal on syntax
    y = 1/0
    return
  syntax:
    c = condition('O')
    m = c~message c~errortext
    return
  ```

  Oracle rc 0, stdout `done 20001`. Head **rc 101**, stderr `thread 'rexx-interp' panicked at
  crates/rexx-exec/src/dispatch.rs:1510:9: a live value`, stdout empty. **Base the same** (rc 101,
  same panic). Deterministic: bisected, a loop bound of 1764 panics and 1763 prints `done 1764` at
  rc 0 (`bis3/`). The live array is what moves the collection's phase: without it (`p/ord2`) and
  with `call gc 'F'` or 200,000 string allocations at a clause boundary before `~message`
  (`p/gc2`, `p/gc3`) all runs agree, because nothing collects inside the window.
* Prediction for the fix mutant, written before running: rooting each entry in
  `build_condition_object` (a `push_temp` beside every `entries.push` of a named value) makes the
  three-line program and `p/ord1` answer as the oracle does, and makes
  `the_l0_subset_passes_again_under_collect_on_every_allocation` pass, **if** this is the only
  cause of that test's panic; unmodified head fails that test at `dispatch.rs:1510:9`.

### Risk 1, cost claim re-measured (ran, `perf/`)

Callgrind `Ir` totals, interleaved base then head per program per round, a fresh run directory each,
binaries copied from `target-base`/`target-head` (sha256 `b7503217a261…` / `69655e5ab705…`).
`method.rex` 20,000 `RegExp_Match` sends, `routine.rex` 20,000 `RxCalcSqrt(4)`, `zero.rex` the same
loop requiring `rxmath` with no native call (a control for startup and library load), `cps` the
tree's `bench-rexxcps/rexxcps.rex`. All rc 0, same stdout per pair.

| program | round | base | head | head - base | per iteration |
|---|---|---|---|---|---|
| method | 1 | 290,692,315 | 283,091,526 | -2.615% | -380 |
| method | 2 | 290,771,123 | 283,087,582 | -2.643% | -384 |
| routine | 1 | 301,263,210 | 293,222,310 | -2.669% | -402 |
| routine | 2 | 301,176,189 | 293,216,711 | -2.643% | -398 |
| zero | 1 | 134,447,349 | 134,448,591 | +0.001% | 0 |
| zero | 2 | 134,407,939 | 134,443,646 | +0.027% | +2 |
| rexxcps | 1 | 21,143,149,142 | 21,143,719,272 | +0.003% | |
| rexxcps | 2 | 21,142,751,612 | 21,149,995,602 | +0.034% | |

**Confirmed**: about 2.6-2.7% fewer instructions per native call than base on both loops, and the
control shows the difference is per call, not startup. `rexxcps` moves +0.003% and +0.034%; head's
own two runs differ by 0.030%, so it is inside run-to-run spread, as the report says.
* **Mutant run** (`src-fix/`, own `target-fix/`, head plus `self.roots.push_temp(v)` before each
  `entries.push((key::NAME, v))` in `condition.rs`, fifteen insertions, nothing else):
  * **Confirmed**: the three-line program, `~program`, `c['MESSAGE']` all rc 0 under stress; `p/ord1`
    ordinary run rc 0 `done 20001`, the oracle's stdout; `lang/condition_object_syntax.rex` (the
    program the exclusions entry "L0 TEST STOPS BEFORE ANY phase-8.txt PROGRAM" names) panics on head
    under stress and runs on the mutant; the four programs the report says panic
    (`library_native_integer_arguments`, `_object_arguments`, `_special_arguments`, Task 2's
    `library_routine_argument_errors`) panic on head and on the mutant match their own plain run on
    stdout and stderr (2227, 1632, 141, 364 collections).
  * **Falsified, the "only cause" half**: `memcap 16G cargo test --release -p rexx-exec --test
    collect_stress the_l0_subset_passes_again_under_collect_on_every_allocation` on the mutant exits
    101, no panic, **six mismatches** over the whole subset, each stress run rc 120 `rexx-exec: a
    message send to a value whose object is no longer live is not implemented (Phase 5)`:
    `lang/condition_object.rex`, `address_with_stream.rex`, `executable_context.rex`,
    `sys_file_functions.rex`, `security_manager.rex`, `call_miss_not_cached.rex`. Two of them run
    on **unmodified head** under the harness give the same rc 120 (`call_miss_not_cached` 38
    collections, `condition_object` 28), so they are pre-existing and were hidden by the first
    panic, not made by the mutant.
* So: this is the L0 failure's panic (the same assertion, the same program, removed by rooting these
  entries), it is reachable in an ordinary run, and the L0 test's single recorded failure masks
  further stress-mode defects the exclusions do not list.

### Risk 6, records (ran)

* **`e0b30d156` changes column 4 only**: a tab split of the table at `5d84dd8cb` and `e0b30d156`,
  307 lines each, 19 rows differ, every difference in column 4 (the definition location). ✅
* **New rows** (`e0b30d156` against `034c1c7d7`, identical at `edcf1ff40` and `c00d18052`): seven
  added, none removed; 62 other rows differ in column 4 only. Every added row has verdict
  `agrees`, reached `yes`, an answer (34.901, 88.904, 88.907, 88.914, 88.919, 93.969, 98.913) and a
  witness naming its corpus program. No empty measured column. ✅ Each named witness, run by me
  through `probe.sh` with its `.d/`: identical to the oracle on all three descriptors (rc 168, 168,
  222, 158, 168, 168, 163, 216), as are the four larger witnesses (rc 0).
* **`SHARED_ANSWERS`**: derived independently with the test's own rule (a `send` tag in `surface`,
  answers carried by more than one row) and compared with the `const`: equal. ✅ The new groups
  are 34.901 {`native_argument_not_logical`, `not_logical`}, 88.907
  {`native_argument_out_of_range_unsigned`, `native_argument_outside_range`}, 98.913
  {`native_argument_not_an_array`, `object_not_single_dimensional`}, and 88.914 gains
  `native_argument_not_an_instance`. Prose: the 34.901 comment ("a method's own logical argument",
  `tests/refusal_sites.rs:618`-`620`) is not what `not_logical`'s send-surface row is: its witness
  is `String~"?"`, whose logical value is the **receiver** (`dispatch/string.rs:1253`), and the
  constructor's doc is the prefix `\` operator's operand. The header's "Measured" transposition
  claim was not re-run here (⚠️).
* **Exclusions**: the closed `Unfilled` entry keeps both sides (`oracle:` / `was:`) and names the
  closing task and the witness; `library_native_results.rex` does print `TestInterpreterVersion`
  and `MathLoadFuncs`/`MathDropFuncs` (read, and run identical). ✅ The two new entries carry both
  sides and an explicit `NO OWNER` with its reason. ✅ Both are now **stale against measurements**:
  the POSITION entry says "not run against an earlier binary" (it is pre-existing, `p/pos1`); the
  panic entry frames it as a collect-on-every-allocation panic and says whether it is the L0
  failure's cause "was not measured" (it is the L0 panic, it is reachable in an ordinary run, and it
  hides six further stress mismatches).
* **`phase-8.txt`**: the second block's comment holds for the eight refusal programs (each lineless
  against `pk.cls`, run). The first block's "**every conversion row**" is false: no corpus program
  declares a `RexxMutableBufferObject` or `RexxVariableReferenceObject` parameter (`/bin/grep -rn -a
  'TestMutableBuffer\|TestVariableReference\|TestSetVariableReferenceValue' rust/corpus` finds
  nothing), and the special codes as result types are reached only by the forged library. The
  report's concern "Their 88.914 refusals are corpus-witnessed through `orxmethod`" is false for
  the same reason. The refusals themselves agree with the oracle when run (`p/mbvr`:
  `TestMutableBufferLength('abc')`, `(.array~of(1,2))`, `TestVariableReferenceValue('abc')`,
  `(.mutablebuffer~new('x'))`, all 88.914 naming the class, identical).
* The in-crate stress test's asserted stdout (`dispatch/library.rs`,
  `the_conversion_rows_answer_the_same_under_a_collection_at_every_allocation`): its program run
  through the oracle and head (`p/cst`) prints exactly the asserted text, identical. ✅

### More probes (ran)

* `p/spec3`, identical: trailing omitted arguments into `ARGLIST` (`t~arglist(1, , )~size` 1,
  `(, )` 0, routine the same); `t~send('name')`, `t~send('Other')` (`NAME OTHER`); `FORWARD MESSAGE`
  into a `NAME` and an `ARGLIST` method; scope overrides `u~scope:.T`, `u~super:.T`, `u~name:.T`.
  (`.Message~new(...)~send` and `~start` refuse here naming Phase 5, rc 120, so they were dropped.)
* Sourceline companions for the twelve new programs: each `count N` equals the file's line count
  and the lines equal the file's, byte for byte (python over both). Whether they came from the
  oracle's driver is ⚠️ not re-run (it instantiates a repository file).

### Controls on the reuse test and on `p/reuse3` (predictions written before running)

Mutants in `src-mut/` (head archive, own `target-mut/`), restored from a copy between them:
* **A**, delete `frame.argument_list = None;` in `pop_native_frame`. Predicted:
  `a_reused_native_frame_holds_nothing_of_the_call_before` fails (its `argument_list` assertion);
  `p/reuse3` departs from the oracle at `arglist identity` (`1 2 2`: every later `ARGLIST` call gets
  the first call's array) and the size lines after it.
* **B**, delete `frame.locals.clear();`. Predicted: the reuse test fails (its `resolve(handle)`
  assertion); `p/reuse3` stays identical to the oracle, because no shipped extension can observe a
  stale handle.

Results:
* **A confirmed.** `cargo test --release -p rexx-exec --lib
  a_reused_native_frame_holds_nothing_of_the_call_before`: 0 passed, 1 failed, panic at
  `library.rs:1136` (the `argument_list` assertion). `p/reuse3` on the mutant: `arglist identity 1 2
  2`, `arglist sizes 2 2 2 2`, `routine arglist sizes 2 2` against the oracle's `0 2 1`, `3 0 4 0`,
  `3 0`, and the nested `in NI 3 NAME` against `4`. So the probe is live for that reset.
* **B confirmed.** The same test: 0 passed, 1 failed, panic at `library.rs:1133` (the
  `resolve(handle)` assertion). `p/reuse3` on the mutant: stdout and stderr identical to head's. The
  unit test is the only instrument for the handle-table reset. `library.rs` restored from the copy
  and `cmp`-checked after each.

---

## Spec Compliance (host slice)

* ✅ Host primitives for every row the table asks the host for (`dispatch/library.rs:441`-`601`):
  the twelve new witnesses run by me are identical to the oracle on all three descriptors; so are
  `p/reuse3` (up to the unfilled `GetArgument`), `p/stem1`, `p/cself1`, `p/spec3`, `p/mbvr`.
* ❌ `found` for a `logical_t` refusal (`dispatch/library.rs:269`-`272`): rendered from
  `stringValue()` where the oracle reports the string conversion's answer (Important 1).
* ✅ Stem names resolve in a routine's caller for every caller kind probed (`p/stem1`); a method
  refuses a name with 93.969 (`p/reuse3`, witness).
* ✅ `ARGLIST` one array per call, not shared across calls (`p/reuse3`), trailing omitted arguments
  (`p/spec3`); `NAME` through sends, `FORWARD`, a `::ROUTINE` alias, `loadExternalRoutine` (witness,
  `p/spec3`); `OSELF`/`SCOPE`/`SUPER` instance and class side and under a scope override (witness,
  `p/spec3`).
* ⚠️ `OSELF`/`SCOPE`/`SUPER`/`CSELF` in a routine, the special codes as result types, and a
  `MutableBuffer`/`VariableReference` argument that converts: no shipped extension reaches them
  (forged library only, outside the corpus, as the brief requires). `Refused::ResultSignature`
  routing (`library.rs:290`-`295`) is unit-tested only.
* ✅ `Refused::Unfilled` and its doubled suffix are gone (`library.rs:302`); the exclusions entry is
  closed with both sides kept.
* ✅ The `CSTRING` from_native row through `rxmath`'s `MathLoadFuncs`/`MathDropFuncs`, and `size_t`
  through `TestInterpreterVersion`, witnessed and identical (`library_native_results.rex`).
* ✅ Only `liborxmethod`, `liborxfunction`, `librxmath`, `librxregexp` are loaded by the witnesses
  (NEEDED libc/libm/libstdc++/libgcc); no forged library in the tree.
* ✅ `refusal-sites.tsv`: `e0b30d156` column 4 only; seven new rows fully measured;
  `SHARED_ANSWERS` equals an independent derivation.
* ✅ Frame reuse (`c00d18052`): no state from one call reaches the next in any probe; RSS flat to
  300,000 iterations; cost claim re-measured and confirmed (-2.6% to -2.7% per call, `rexxcps`
  inside run-to-run spread).
* ❌ `phase-8.txt:205` "every conversion row" is not what the corpus holds (Minor 1).

## Strengths

* Frame reuse is small and correct: one push site, one pop site, no early return between them in
  either caller, spares never walked as roots. It survived every leak shape I could build with
  shipped extensions, including nested native calls run from a Rexx `MAKESTRING` during an outer
  call's conversion, and both of its resets are pinned by a unit test that fails when either is
  deleted (controls A and B above).
* The cost claim is honest and reproduces to within 30 instructions per call.
* Stem resolution through `classify` and `read_stem` matches the oracle on every caller kind
  probed, including `PROCEDURE`, `EXPOSE`, `INTERPRET`, `loadExternalRoutine~call` and a nested
  resolution.
* The records are derived, not written: the table and `SHARED_ANSWERS` agree with an independent
  derivation, and the witnesses named in the rows are the programs that reach them.
* The report's own finding of the stress-mode panic was precise enough (the four-line program, the
  assertion site) to take straight to a cause.

## Issues

### Critical

1. **A condition object's entries are unrooted while it is built, and an ordinary run panics.**
   `rust/crates/rexx-exec/src/condition.rs:70`-`146`, `Interp::build_condition_object`. **Ran.**
   * **What.** Each entry's `ObjRef` goes into a plain `Vec` and nothing roots it before the `PUT`
     loop. Any collection in between (from `text_built`, `additional_array`, the index strings, or
     the sends) frees the non-inline ones: `PROGRAM`, `ERRORTEXT`, `MESSAGE`. Reading one then
     panics at `dispatch.rs:1510:9` "a live value".
   * **Why Critical.** It is reachable **without** stress mode. `p/ord1` (the loop in the log above:
     a growing array, and a procedure trapping `1/0` and reading `c~message c~errortext`) is oracle
     rc 0 `done 20001` and head rc 101 with the panic. Deterministic, first failing bound 1764.
     Pre-existing: base panics identically.
   * **The stress-mode reduction** is three lines: `signal on syntax` / `y = 1/0` / `syntax: say
     condition('O')~message`. `~program` and `c['MESSAGE']` panic too.
   * **It is the L0 test's panic.** `lang/condition_object_syntax.rex` panics on head and runs on the
     mutant. So do the three Task 3 witnesses and Task 2's `library_routine_argument_errors.rex`.
   * **Fix.** `self.roots.push_temp(value)` beside each `entries.push` (the frame at `:65` already
     bounds them). Verified by mutant: the ordinary-run program, the reduction, those five programs,
     and each witness's stress run equal to its plain run.
   * **Witness to add.** A unit test that builds a SYNTAX condition object under
     `enable_stress_collect` and reads `MESSAGE` and `PROGRAM`. The ordinary-run loop depends on
     allocation layout and is too fragile to pin.
   * **Also.** With the panic gone, the L0 test reports six further stress-mode mismatches (rc 120,
     "a message send to a value whose object is no longer live"): `condition_object.rex`,
     `address_with_stream.rex`, `executable_context.rex`, `sys_file_functions.rex`,
     `security_manager.rex`, `call_miss_not_cached.rex`. The two I ran on unmodified head fail the
     same way there, so the first panic was hiding them. Whether any is reachable in an ordinary run
     was not measured. The exclusions list none of them.
   * **Not in this task's diff.** Whether it is fixed here or gets an owner is the controller's
     call. Either way the exclusions entry `034c1c7d7` added (`phase-4-exclusions.txt:5068`-`5082`)
     must stop describing it as a collect-on-every-allocation panic of unmeasured cause.

### Important

1. **`logical_t`'s `found` is the wrong object.** `rust/crates/rexx-exec/src/dispatch/library.rs:269`-`272`.
   Introduced by this task. **Ran,** `p/found2`.
   * **What.** The oracle's `truthValue` converts first, then reports against the converted string.
     Oracle: `found "1<LF>2"` for `.array~of(1,2)`, `"viaString"` for an object whose `STRING`
     answers that, `"viaMake"` for `MAKESTRING`. Head: `"an Array"`, `"a WS"`, `"a WM"`. Untrapped,
     that is a stderr difference.
   * **Why missed.** The witness's `t~logical(.object~new)` cannot see it: with neither method, the
     conversion falls back to `stringValue()`.
   * **Fix.** Carry the conversion's answer in `Refused::NotLogical` (the row is in `values.rs`, the
     sibling's slice) and render that. Add the array and `STRING`-object lines to a witness.
   * **Commit message.** `edcf1ff40`'s "Every native refusal's found insert is now the argument's
     stringValue" is false for this row.
2. **`found` never sends `OBJECTNAME`/`DEFAULTNAME`, and the new comment says the oracle does not
   either.** `rust/crates/rexx-exec/src/dispatch/library.rs:246`-`248`; `value.rs:380`. **Ran,**
   `p/found2`, `p/rb8`.
   * **What.** `RexxObject::stringValue()` is `sendMessage(OBJECTNAME)`
     (`interpreter/classes/ObjectClass.cpp:1157`), which sends `DEFAULTNAME` (`:1713`), and every
     message insert goes through it (`concurrency/Activity.cpp:1300`). A class overriding
     `defaultName` or `objectName` renders `"myDefault"`/`"myObjectName"` on the oracle and `"a
     DN"`/`"an ON"` here, on every row. `p/rb8` shows the send itself: the oracle runs `DEFAULTNAME`
     twice, head once.
   * **Scope.** Pre-existing for 88.921/88.905 (base the same). This task routes seven more
     constructors through it and writes "`stringValue()`, which sends nothing" at the decision point.
   * **Fix.** At least correct the comment and record the divergence with an owner. The behavioural
     fix changes `string_value_text`'s other callers (trace value lines) and needs the oracle's
     fallback to `defaultName()` when the send raises, so it is an owner decision.

### Minor

1. `rust/corpus/phase-8.txt:205`: "every conversion row" is false.
   * No corpus program declares a `RexxMutableBufferObject` or `RexxVariableReferenceObject`
     parameter, and the special result codes are forged-library only. The report's "Their 88.914
     refusals are corpus-witnessed through `orxmethod`" is false for the same reason.
   * The refusals agree when run (`p/mbvr`). Narrow the comment, or add a refusal witness through
     `TestMutableBufferLength('abc')` and `TestVariableReferenceValue('abc')`.
2. `docs/superpowers/plans/phase-4-exclusions.txt:5050`-`5066`: the POSITION/PROGRAM/TRACEBACK
   entry says "not run against an earlier binary".
   * Measured now (`p/pos1`): base and head print byte-identical condition objects for 88.905,
     88.921 and 88.901, so it is pre-existing, not recorded by this task's own change.
   * The routine half (`Compiled routine` missing) is already the entry at `:4856`.
3. `rust/crates/rexx-exec/tests/refusal_sites.rs:618`-`620`: the 34.901 group comment calls
   `not_logical` "a method's own logical argument".
   * Its send-surface witness is `String~"?"`, whose tested value is the receiver
     (`dispatch/string.rs:1253`), and its doc is the `\` operator's operand. Say "a string method's
     receiver".
4. `rust/crates/rexx-exec/src/lib.rs:1732`, `:5548` and `dispatch/library.rs:224`: "emptied" frames.
   * `pop_native_frame` leaves `owner`, `scope`, `receiver` and `method` stale. Harmless (never
     read, never rooted, overwritten at push), but "emptied" is the stated reason the spares are not
     roots.
   * Either reset them to `NIL` or say "their buffers cleared".
5. `rust/corpus/lang/library_native_object_arguments.rex:3`-`4`: "What a refusal finds is the
   argument's own string value, an array's included" is false for the logical row (Important 1).
   Correct it with that fix, regenerating the sourceline companion.
6. Found in passing, pre-existing, not native (`p/rb7`): a raise inside a `DEFAULTNAME` is not
   trapped by the calling procedure.
   * The program: a `PROCEDURE` with `signal on syntax name refused`, then `r = v~string` where
     `v`'s `DEFAULTNAME` does `raise syntax 40.1`.
   * The oracle traps it (rc 0); base and head end rc 216 untrapped.
   * Unowned. Recorded here so it is not lost.

## Assessment

**Task quality (host slice): Needs fixes.**
* **Important 1** is a wrong answer this task introduced.
* **Important 2's** comment asserts the opposite of the oracle at the decision point.
* **Critical 1** is pre-existing, but it is an ordinary-run crash with a verified fifteen-line fix.
  The exclusions entry this task wrote misdescribes it.
* **Frame reuse, stems, the specials, the records and the cost claim all hold.**

Cleanup: `t3-review-b/target-base`, `target-head`, `target-fix`, `target-mut` deleted by path;
worktree `git status --short` empty at `034c1c7d7`.
