# Task 11 report: the Array and the Directory 5a's own mechanisms send to

Commits: `f4b21eadb` (the implementation), `6f3434e88` (two performance amendments
found by the sitting), `f92e00b86` (the two sittings' rows). Fix round 1, near the
end of this file: `071835c9c`, `91935800d`, `fefe33597`. Fix round 2, at the end:
`3983105e1`, `0ef493ae7`.

**This file is not in git history.** `.gitignore:30` ignores `.superpowers/`, and
no Phase 5a task's report is committed; the committed copies under
`docs/superpowers/records/` stop at `2026-08-15-phase-5-object-model`. Copying
this plan's workspace there is a phase-level call and was not made here.

Status: **DONE_WITH_CONCERNS**. Everything the brief asked for is built, measured
against the oracle on both engines, and green on all five gate commands. The
concerns are two open divergence families this task found and did not close, both
pre-existing, both recorded below with reproducers.

---

## 1. What was built, by method name

**Array**, the four rows this task added to `.Array`'s own dictionary
(`dispatch.rs`'s `NATIVE_METHODS`, which has `MAKESTRING` and `TOSTRING` for
that class besides):

| name | arity row | function |
|---|---|---|
| `[]` | `A_COUNT` | `native_array_at` |
| `AT` | `A_COUNT` | `native_array_at` |
| `SIZE` | 0 | `native_array_size` |
| `ITEMS` | 0 | `native_array_items` |

**Directory**, three rows, donated into `.Directory`'s own dictionary by
`InheritInstanceMethods(StringTable)`:

| name | arity row | function |
|---|---|---|
| `[]` | 1 | `native_directory_at` |
| `AT` | 1 | `native_directory_at` |
| `PUT` | 2 | `native_directory_put` |

**`ExprKind::List`** builds a real `.Array` from `f4b21eadb` (`eval.rs`'s
`eval_list`), and **`DO OVER`** iterates one (`run.rs`'s `Interp::over_items`
and `LoopState::OverItems`).

**No other name moved.** `MAKESTRING`/`TOSTRING` were already there (Task 9) and
were re-pointed at `Interp::array_string` so the join and the string value cannot
disagree about an empty slot. Every other documented Array and Directory method
stays refused: measured, `(1,2)~put(1)` is
`rexx-exec: method "PUT" of class "Array" is not implemented (Phase 5)` at rc 120,
and `.environment~items` is the same shape for `"ITEMS" of class "Directory"`.
That is 5c's method half and its rows stay red.

### The value kind: no new `Body` variant, and one payload widened

**No `Body` variant was added**, so `body.rs:292`'s
`const _: () = assert!(size_of::<Body>() <= 80);` and Q4's boxing rule are not
exercised.

`Body::Array`'s payload did change, from `Vec<ObjRef>` to `Vec<Option<ObjRef>>`,
and that is forced rather than chosen. An empty slot and a slot holding `.nil`
are different values on the oracle -- measured, three descriptors:

```text
a=(1,,3);   say a~items    2      say a~size   3
a=(1,.nil,3); say a~items  3      say a~size   3
```

and both answer `.nil` from `~at`, so the distinction cannot be reconstructed
from what a read answers.

**`size_of::<Body>()` is unchanged, and it is the type rather than the build
that says so.** A `Vec<T>` is three words for every sized `T`, so replacing
`Vec<ObjRef>` with `Vec<Option<ObjRef>>` cannot change the variant's width;
`body.rs:292`'s assertion is an inequality, so a successful build establishes
only that the size is at most 80. The cost is 8 bytes per element -- `ObjRef` is
`pub struct ObjRef(u64)` (`rexx-core/src/handle.rs:108`) with no niche, so
`Option<ObjRef>` is 16 bytes -- on a value kind a program allocates one of at a
time.

---

## 2. The differential results

Method: `orx`/`rrx` wrappers running the oracle
(`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10
.../build/bin/rexx FILE )`) and `target/release/rexx-run` from **the same fixed
empty directory path**, so the program path an error report quotes is identical on
both sides, with stdout, stderr and exit status captured to three separate files.
Never `2>&1`. Both crate engines run via `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker`.

### The rows the brief names

All three descriptors, both engines, all matching:

```text
say (1,)~size                     rc 0  `2`
say (1,2,3)~items                 rc 0  `3`
do e over a  (a = (10,20,30))     rc 0  three items in order
do name over "nl","cr","tab", -   rc 0  four names in order
 "null"
d~put('vv','KK'); say d['KK']     rc 0  `vv`
d~at('KK')                        rc 0  `vv`
```

The starting state, re-measured before any edit: `(1,)~size` was rc 120
`rexx-exec: a parenthesised list is not implemented (Phase 5)` on both engines,
and the Directory sends were rc 120
`rexx-exec: a message send to one of the interpreter's own objects is not
implemented (Phase 5)`.

### The sweep

82 probe programs across three batches, each run on the oracle and on both crate
engines and compared on all three descriptors. Batches: the constructs the brief
names and their neighbours (15), the refusal ladders for both receivers (39), and
the positions a value can reach outside a send -- operators, loop headers,
conditions, `SELECT CASE`, `RAISE`, `SIGNAL VALUE`, `NUMERIC`, `INTERPRET`,
`ADDRESS VALUE`, builtin arguments (28).

**72 of 82 match on all three descriptors on both engines**, re-run against the
committed build. (This read 71 of 82 before fix round 1, whose `RAISE` change
closed `raise syntax 93.900 additional (1,2)`; the Fix round 1 section has the
account.) The 10 that do not are each one of:

* **a refusal this task deliberately leaves** (`(1,2)~put(1)`,
  `.environment~items`) -- an unimplemented method resolving and failing loudly;
* **a refusal that predates this task** (`(1,2) + 1`, `(1,2) = '1'`) -- R12's
  `Loud::operator_operand`, which already refused `.Array~superClasses` in those
  positions;
* **a refusal this task adds on purpose** (`.local['STDOUT']`,
  `.environment['ALARM']`, and `.environment~nosuch`, which appears in two of the
  batches) -- section 5;
* **an open divergence** (`numeric digits (1,2)` and
  `interpret ("say 1","say 2")`) -- section 6.

`substr('abcdef',(1,2))` was one more and is fixed; it is the witness section 6
names for the builtin half of that family.

### The oracle behaviours this task had to read rather than assume

Each measured on the oracle, three descriptors:

* **`~size` counts slots and `~items` counts filled ones.** `(1,)~size` is 2 and
  `(1,)~items` is 1, because `RexxExpressionList::evaluate` builds
  `new_array(expressionCount)` (`expression/ExpressionList.cpp:93`) and
  `parseFullSubExpression` returns `total` where `parseArgList` returns
  `realcount` (`parser/LanguageParser.cpp:3145`).
* **`~at` past the end is `.nil`, not an error.** `ArrayClass::getRexx` validates
  under `IndexAccess`, which is `RaiseBoundsTooMany` alone
  (`classes/ArrayClass.hpp:62`), so `(1,2)~at(100000000000000001)` answers
  `The NIL object` even though that is past `MaxFixedArraySize`.
* **A subscript converts at `Numerics::ARGUMENT_DIGITS`, not at the digits in
  force.** `numeric digits 3; say (1,2)~at(1000000)` is `The NIL object`;
  `(1,2)~at(999999999999999999)` answers and `(1,2)~at(1000000000000000000)` is
  93.907.
* **The three subscript errors are not interchangeable, and `A_COUNT` is why.**
  `(1,2)~at()` is 93.901, `(1,2)~at(1,2)` is 93.926 (`Too many subscripts for
  array; 1 expected.`), `(1,2)~at('x')` is 93.907. `.environment~at(1,2)`, whose
  row carries a count, is 93.902 instead. A single `arity: usize` cannot express
  that, which is what `enum Arity { Fixed(usize), Counted }` is for.
* **A lone array argument is the subscript list.** `ArrayClass::validateIndex`
  takes its **item count** with its **slot array**
  (`classes/ArrayClass.cpp:1219`-`:1226`), so `(1,2)~at((1,))` answers `1`,
  `(1,2)~at((1,2))` is 93.926, `(1,2)~at((1,,3))` is 93.926 and `(1,2)~at((,))`
  is 93.901.
* **A Directory index is stored and matched verbatim.** `d~put('v','kk')` leaves
  `d['kk']` `v` and `d['KK']` `The NIL object`; `.environment['array']` is
  `The NIL object` where `.environment['ARRAY']` is `The Array class`.
* **`~put` answers no value.** `.environment~put('v','q')` is rc 0 as a whole
  clause and 91.999 at rc 165 under `say`, exactly as a `::METHOD` body ending in
  a bare `return` is. That is why `NativeMethod` now returns
  `Result<Option<ObjRef>, Failure>`.
* **`~put` checks its item before its index.** `.environment~put()` reports
  `argument item is required` and `.environment~put('a')` reports
  `argument index is required`.
* **`DO OVER` on an array iterates the non-empty slots.** `OverLoop::setup`
  tests `isArray` and takes `makeArray()` with no dispatch at all
  (`instructions/DoBlockComponents.cpp:233`-`:236`); `DoBlock::checkOver` walks
  it to `lastIndex()`. Measured, `do e over (1,,3)` yields `1` and `3` where
  `do e over (1,.nil,3)` yields `1`, `The NIL object` and `3`.
* **`FOR` is consulted after the item is bound.**
  `RexxInstructionDoOverFor::iterate` is
  `doblock->checkOver(context, stack) && doblock->checkFor()`
  (`instructions/DoOverInstruction.cpp:279`). Measured:
  `a = (10,20,30,40); do e over a for 2` prints 10 and 20 and leaves `e` at
  **30**; `do e over 'abc' for 0` never enters the body and leaves `e` at `abc`.
* **Every trace value line and every error-message substitution of an object
  renders through `stringValue()`, not through the string value a string context
  asks for.** Measured, both halves:

  ```text
  trace i / a = (1,,3)      >>>   "an Array"   and   >=>   A <= "an Array"
  say '<'||(1,,3)||'>'      <1  /  3>          (two lines, empty slot skipped)
  say '<'||((1,2),3)||'>'   <an Array  /  3>   (a nested array is its own name)
  say 1 + (1,2)             41.1 Nonnumeric value ("an Array")
  say 2 ** (1,2)            26.8 ... found "an Array"
  do (1,2)                  26.2 ... found "an Array"
  do i=1 to 3 for (1,2)     26.3 ... found "an Array"
  substr('abcdef',(1,2))    40.12 ... found "an Array"
  ```

  and the contrast that bounds it -- an instruction that **converts** first and
  traces or quotes the converted string keeps the joined text:
  `translate('abc','x','y',(1,2))` is 40.23 quoting `"1` / `2"` on two lines,
  because `padArgument` converts with `stringArgument` and then quotes the
  string. `numeric digits (3,)` sets DIGITS to 3, `substr('abcdef',(2,))` answers
  `bcdef` and `do a` for `a = (3,)` runs three times, so the value path is
  `requestString` in every one of those and only the message is `stringValue`.

---

## 3. The `owners.rs` items edited

`ExprKind::List` moved from `Owner::Phase("Phase 5")` to `Owner::InScope`, and
all five pinned items moved with it in `f4b21eadb`:

1. **`EXPECTED_OUT_OF_SCOPE`** (`owners.rs`): the
   `("ExprKind", "List", "Phase 5")` row deleted.
2. **`coverage.rs`'s subset list**: `EXPECTED_SUBSET` is `phase-4a.txt`'s and
   needed no change; the L0 subset this task widens is `phase-5a.txt`, whose
   pinned literal is `EXPECTED_SUBSET_5A`, and six lines were added to it
   alongside the same six in `corpus/phase-5a.txt`.
   `every_in_scope_variant_is_witnessed_by_the_phase_subsets` was red on
   `ExprKind: 1 in-scope variant(s) unwitnessed by the phase subsets: List`
   until those corpus programs landed, which is that control firing as designed.
3. **`owners.rs`'s `variant_counts_match_the_audited_split`**: `EXPR_TAGS`'
   `InScope` count 12 to 13 and its `Phase(_)` count 3 to 2. **`loud.rs`'s copy
   of the `InScope` figure** (`in_scope_counts_match_the_audited_split`) moved
   from 12 to 13 with it.
4. **`loud.rs`'s `EXPR_WITNESSES`**: the `List` witness **deleted**, not left
   stale, and `assert_witness_set_is_complete`'s `expected_exprs.len()` 3 to 2.
5. **`src/lib.rs`'s `expr_owner`**: `ExprKind::List` joined the `None` arm.
   Its neighbouring comment in `instruction_owner`, which cited `ExprKind::List`'s
   `Phase 5` owner as the reason `RAISE ... ADDITIONAL (a, b)` failed loudly, was
   corrected rather than left standing; so was the same claim in `owners.rs`'s
   own `Raise` comment.

`LoopKind::Over` was already `Owner::InScope` and was not touched. Verified
before the edit: `do i over 'abc'` / `say i` / `end` was rc 0 `abc` on the oracle
and on both crate engines.

**One item outside that list moved too**, and it had to: `spike.rs`'s two loud
witnesses were `say (1, 2)`, and both went red the moment `ExprKind::List`
evaluated. They now use `ExprKind::ClassResolver` (`say ns:Bar`,
`rexx-exec: a namespace-qualified class lookup is not implemented (Phase 5)`),
which is the fifth witness that file has had; its own doc records the sequence
and the property rather than predicting which form lasts.

---

## 4. The performance sitting

Recorded in `rust/bench-baselines/phase-5a-arms.tsv`, appended by `rexx-arms`,
`task` = `11`, two sittings -- one at `f4b21eadb` and one at `6f3434e88`.
The second is the one that describes the tree as it stands.

**Staleness test, run before trusting the pin**, from the repository root
(the pathspec is `rust/crates`, so it has to be run from there and not from
`rust/` -- run from `rust/` it silently lists nothing):

```
git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml
```

listed 41 commits at the task's base, and every one of them is named in
`.superpowers/sdd/2026-08-17-phase-5a/progress.md` -- checked by grepping the
ledger for each SHA, no misses. It lists 43 now; the two added are this task's
own, which is what the plan's own correction to that test says a phase's tasks
do. `git merge-base --is-ancestor 15a1ffa98 HEAD`
holds. `sha256sum bench-baselines/pinned/rexx-run-15a1ffa98` is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching
`PINNED.md`. The pin is live.

### The budget axis is flat

`strings`/`ir`, `instructions:u`, per pass, median of 5 rounds, from the sitting
at `6f3434e88`:

| build | per pass |
|---|---|
| `pinned` (`15a1ffa98`) | 5369.518296 |
| head (`6f3434e88`) | 5421.517987 |

That is +51.999691 against a ceiling of 53.695178, leaving **1.695487**. The
figure the brief carried was 1.695485, so this task spends **none** of the
standing budget on the axis the guard is written against; the two-millionth
difference is the pinned build's own reading moving between sittings.

### Every axis, and no figure at or above 1%

`across_builds`, `pinned>head`, `instructions:u`, at `6f3434e88`
(the `small` and `large` sizes differ by at most a thousandth of a percent on
any row -- `alloc4c`/`tw` is the widest, +0.457% against +0.445% -- so one
column carries both and the TSV has each):

| axis | `tw` | `ir` |
|---|---|---|
| `alloc4c` | +0.457% | +0.271% |
| `arith` | +0.151% | +0.012% |
| `compound` | +0.440% | +0.262% |
| `emptyloop` | 0.000% | 0.000% |
| `strings` | +0.940% | +0.969% |
| `varlookup` | +0.681% | 0.000% |

Nothing reaches the plan's 1% threshold, so by its own rule none of this is a
finding. `cycles:u` figures are in the TSV and are not read as results here, per
the plan's own rule about them.

### The first sitting was above the threshold, and what the control said

The sitting at `f4b21eadb` read `strings`/tw at +1.371% and `alloc4c`/tw at
+1.132%, both above the threshold, so the interleaved control the plan asks for
was run: the same committed source built twice, differing by **one added comment
line** in `eval.rs`, interleaved over `alloc4c`, `strings`, `varlookup` and
`compound`. Every `head>control` `across_builds` ratio came back **exactly
1.000000** on `instructions:u`, on every axis and both arms.

**That is all it establishes, and it is less than it looks.** A rebuild
differing by one comment produces identical machine code, so the ratio had to
come back 1.000000 whether or not layout can move that counter: the control
cannot fail on this instrument. What licenses reading a non-zero
`instructions:u` move as executed work is the counter itself -- an instruction
count is a function of the path executed, not of where the code sits -- and it
was the bisection below, not the control, that found the causes. (Task 4c's 7.8%
figure was `cycles:u`, which layout does move.)

With layout excluded, the move was bisected by building variants and reading
`varlookup` and `strings` per-pass `instructions:u` at 3 rounds each. Baseline
re-measured at `HEAD~1` on the same machine and reproducing exactly:

| variant | `varlookup` tw | `strings` tw |
|---|---|---|
| `HEAD~1` (baseline) | 1761.0000 | 9096.5208 |
| `f4b21eadb` as committed | 1777.0001 | 9168.5207 |
| minus `eval_node`'s own `List` arm | 1773.0001 | 9129.5213 |
| minus the trace gates' `string_value_text` | 1766.0000 | 9139.5208 |
| both outlined (`6f3434e88`'s shape) | 1773.0001 | 9129.5210 |

Two causes, both fixed in `6f3434e88`, both on programs containing no list and
tracing nothing:

* **An arm of its own for `ExprKind::List` in `eval_node`'s match** cost
  `strings` 43 and `varlookup` 4 instructions per pass. That match is the
  tree-walker's entire expression dispatch, so every node evaluated pays for its
  shape. Answering the form from behind the existing catch-all
  (`Interp::eval_cold`) costs both nothing.
* **`Interp::string_value_text` inlined into `intermediate_text` and
  `result_text`**, which are both `#[inline(always)]`, cost `strings` 29 and
  `varlookup` 7. `#[inline(never)]` on it is the same gate-then-outline shape
  `Interp::echo_symbol_read` already has.

**What is left after those two, and it is not attributed.** The committed build
reads `varlookup` tw 1772.999988 against the baseline's 1761.000069 and `strings`
tw 9129.520919 against 9044.521120 for the pin -- 12 and 33 instructions per pass
this bisection did not place. Reverting all of `eval.rs`'s remaining
`string_value_text` calls changed neither figure. A variant with `run.rs` also at
`HEAD~1` read 1771 and 9121.52, which is lower, but it was built on a base that
did not carry the outlining, so it cannot be subtracted from the row above and no
share is claimed for `run.rs` here. `dispatch.rs` cannot be reverted in isolation
at all -- it does not compile against the new `Body::Array` payload -- so the
bisection stopped. The residual is inside the plan's threshold on every axis, but
a reader should not read "layout excluded" as "cause identified".

---

## 5. What this task refuses on purpose, and the instrument for each

Three refusals were added. Each replaces something that would otherwise be a
wrong answer a program could trap, and the corpus gate cannot see any of them --
a refusal the oracle does not share is not expressible as a differential row --
so each names its own in-crate instrument.

1. **A directory index the oracle's own directory holds and this crate builds
   nothing for.** Measured, `.local['STDOUT']` is `STDOUT` on the oracle and
   `.environment['STDOUT']` is `The NIL object`, so the refusal has to be **per
   directory** and not over the union of the two name lists;
   `Unbuilt::scope` carries that, and `environment.rs`'s
   `the_two_oracle_directories_share_no_name` asserts the premise that one row
   per name is enough. Instrument:
   `dispatch.rs`'s `a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud`,
   which also asserts the three `.nil` answers and the `~put`-then-read row that
   a refuse-by-name-alone build would fail.
2. **A name a receiver's behaviour does not answer, when that behaviour answers
   `UNKNOWN`.** Measured, `.environment~nosuch` is `The NIL object` at rc 0,
   because `Directory`'s donated `UNKNOWN` reads it as an entry, so 97.1 there is
   a wrong answer a program can trap -- the same argument
   `Loud::receiver_class` makes for a stem. Asked of the behaviour rather than
   listed per class, so a `::METHOD UNKNOWN` or an `~inherit` is covered.
   Instrument: `an_unresolved_name_on_a_receiver_that_answers_unknown_is_loud`,
   whose paired row is `'abc'~nosuch` still raising 97.1.
3. **An array subscript that is an empty slot**, where the oracle segfaults.
   Instrument: `an_expanded_index_of_one_empty_slot_is_loud`, plus
   `corpus/oracle-crashes.txt` entry 6.

### A new oracle crasher, recorded

`a = (1,2); say a~at((,2))` is **SIGSEGV, rc 139**, measured 3 runs of 3, and so
is `a~at((,,3))`. `ArrayClass::validateIndex` spreads a lone array argument by
item count with slot array, so an array whose leading slot is empty and whose
item count is one hands `validateSingleDimensionIndex` a null `index[0]` and
`:1264` dereferences it. The neighbours are all clean and bound the shape:
`a~at((1,))` answers `1`, `a~at((1,,3))` is 93.926 (two subscripts counted before
either is read), `a~at((,))` is 93.901. Written up as entry 6 of
`rust/corpus/oracle-crashes.txt`; not filed upstream, which is Moritz's call.

---

## 6. Divergences found and not closed

Both are the same defect and both predate this task: `.Array~superClasses` has
returned an array since Task 9, so every position below was already reachable.
This task fixed the sites it could reach cheaply and measured the rest.

**The rule**, established by measurement rather than inferred: an error message
substituting an object renders it with `stringValue()`; an instruction or a
builtin that **converts** the value first and then quotes the converted string
keeps the string value.

1. **`NUMERIC DIGITS`/`FUZZ`/`FORM VALUE`.** `numeric digits (1,2)` is
   `26.5 DIGITS value must be a positive whole number; found "an Array".` on the
   oracle and `found "1` / `2"` here. Not closed because
   `Settings::set_digits_str` takes the *text* and raises from it, so the parse
   input and the message substitution are one value, and `numeric digits (3,)`
   setting DIGITS to 3 proves they must part. Splitting them is a change to
   `activation.rs`'s settings API. The `>K>` line the same clause traces is
   affected identically.
2. **`INTERPRET` of a value with a newline in it.** `interpret ("say 1","say 2")`
   runs the first clause and then reports 13.1 on the oracle; here the clause
   never runs and 13.1's substitutions print as the literal `&1`/`&2`. That is
   `lib.rs`'s own documented parse-error gap ("Parse errors remain deliberately
   not reproduced byte for byte"), reached by a new route rather than a new
   defect.

Two members of the same family **were** closed, because their value and message
paths were already separate: `builtin::whole_number` (which covers every builtin
whole-number argument -- `substr('abcdef',(1,2))` was the witness) and
`eval.rs`'s 41.1 / 26.8 plus `run.rs`'s 26.2 / 26.3.

**A pre-existing `DO OVER ... FOR` defect was closed as a side effect**:
`do e over 'abc' for 0` left the control variable unset here where the oracle
leaves it `abc`, on a target that is not an array at all. The old
`LoopState::OverOnce` consulted `FOR` before binding.

**A test did cover the shape, and asserted the half that was right.**
`run/tests.rs`'s `do_over_for_0_skips_the_single_non_stem_iteration` runs
`do x over 'hello' for 0` and asserts the body does not run, which was and is
true; nothing asserted the control variable's value afterwards, and that test's
own doc named the rule "a judgement call" rather than a measurement. Fix round 2
corrects that doc and adds the missing assertion -- see that section.

---

## 7. Negative controls

Each mutation applied to the committed tree, measured, then reverted; the tree
was rebuilt and re-checked green after every one.

| mutation | what reddened |
|---|---|
| `~items` counts every slot instead of the filled ones | corpus 148 of 149 |
| `DO OVER` binds the array once instead of iterating it | corpus 147 of 149 |
| a traced array renders as its items (`string_value_text` returns `to_text`) | corpus 146 of 149 |
| `FOR` consulted before the item is bound | corpus 147 of 149 |
| `[]`/`AT` declare a count instead of `A_COUNT` | corpus 148 of 149 |
| an unbuilt directory entry answers `.nil` | `a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud` |
| an unresolved name on a directory raises 97.1 | `an_unresolved_name_on_a_receiver_that_answers_unknown_is_loud` |
| the empty-slot subscript raises 93.901 instead of refusing | `an_expanded_index_of_one_empty_slot_is_loud` |

The two `owners.rs`-adjacent controls fired without being asked for, which is
worth recording as evidence the pinning works: `every_in_scope_variant_is_
witnessed_by_the_phase_subsets` was red on the unwitnessed `List` variant until
the corpus programs landed, and `environment_seam.rs`'s item count was red on
`env_seam::which` until the count and its doc were updated to admit a function
that takes no clearance and hands back no handle.

---

## 8. The gate commands and their output

**This section describes the tree at `6f3434e88` and fix round 1 supersedes its
corpus counts** -- that section has its own gate block and its own count.

Run from `rust/`, in this order. All five were run again after the amendment, so
every status below describes `6f3434e88` and not the intermediate tree:

```
cargo fmt --all --check                                        exit 0
cargo clippy --workspace --all-targets -- -D warnings          exit 0
cargo test --release --workspace                               exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace            exit 0, 149 of 149 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
                                                               exit 0, 149 of 149 matching
```

Every status was read from an unpiped exit code (`echo $?` after the command, or
`${PIPESTATUS[0]}` where the output was filtered), and the two long runs were
detached to a log with `echo "EXIT $?"` appended, so a killed run cannot read as
a passing one.

`memcap` was present (`/home/moritz/.local/bin/memcap`), checked with
`command -v` rather than assumed. The corpus was **143 of 143** before this task -- measured, not
inferred, by checking `f4b21eadb~1`'s `rust/crates` and `rust/corpus` out and
running the gate -- and is **149 of 149** after it. The union is `phase-4a.txt` +
`phase-4b.txt` + `phase-4c.txt` + `phase-5a.txt`.

The six corpus programs added, all in `corpus/phase-5a.txt` and
`EXPECTED_SUBSET_5A`:

```
lang/array_list_expression.rex     the list expression and the Array method set
lang/array_do_over.rex             both DO OVER shapes, FOR, LEAVE/ITERATE, nesting
lang/directory_at_and_put.rex      ~put/[]/~at over both directories, and .NAME after a put
lang/array_index_refusals.rex      the Array subscript ladder (A_COUNT: 93.901/93.926/93.907)
lang/directory_index_refusals.rex  the Directory ladder (counts: 88.901/88.909/93.902)
lang/array_trace.rex               the trace transcript, in RAW_STDERR_COMPARISON
```

`array_trace.rex` is in `corpus.rs`'s `RAW_STDERR_COMPARISON` because the indents
are what it exists to witness -- the `>A>` per written list position against the
`>>>` for the list itself, and a `DO OVER`'s `>K>` against each pass's `>=>` two
columns further in. Verified byte-identical raw before adding it.

Each new program also got its `crates/rexx-parse/tests/sourceline_oracle/*.txt`,
generated with the driver in that test's own module comment (the sanctioned
`.Package~new` exception), and `sourceline_matches_the_interpreter_for_every_
corpus_program` passes.

---

## 9. What a check here could not see

* **The corpus gate cannot see any of section 5's refusals**, and says so above.
  Each is covered by a named in-crate test, and each of those tests was reddened
  by a mutation.
* **`identityHash` on an array** is deviation 4's licence, unchanged by this
  task: no differential row can compare handle-derived identity with an
  address-derived one.
* **The oracle crasher has no differential row at all**, by construction.
* **The `Vec<Option<ObjRef>>` payload widening is not measured as a memory
  figure.** The sitting instruments `instructions:u` and `cycles:u`, not resident
  memory, and no bench axis allocates an array. What is asserted is that
  `size_of::<Body>()` did not move, which is the assertion the plan cares about
  and is a compile-time one.
* **The residual per-pass move in section 4 is bounded but not attributed.** The
  bisection localised two causes, fixed both, and stopped at `dispatch.rs`, which
  cannot be reverted in isolation. How many causes are left in the residual is
  not known. A reader should not read "layout excluded" as "cause identified".

---

## Fix round 1

Commits `071835c9c` (B2 plus observations O5 and O7), `91935800d` (B1's rows and
comments), `fefe33597` (one figure in `91935800d` corrected). All five gates
green on `fefe33597`, corpus **152 of 152**, up from the 149 of 149 section 8
records.

```
cargo fmt --all --check                                        exit 0
cargo clippy --workspace --all-targets -- -D warnings          exit 0
cargo test --release --workspace                               exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace            exit 0, 152 of 152 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
                                                               exit 0, 152 of 152 matching
```

### B1: one subtraction, stated, and every build it is over committed

**What the two comments said.** `eval.rs`'s `eval_cold` doc said an arm of its
own for `ExprKind::List` "cost the `strings` axis **43** instructions per pass on
the tree-walker arm and the `varlookup` axis **4** ... where answering it from
behind the existing catch-all **costs both nothing**". `value.rs`'s
`string_value_text` doc said inlining it "cost the `strings` axis **29** ... and
the `varlookup` axis **7**".

Three of those four figures came from three different subtractions over a table
whose variants did not share a base -- some carried the outlining and some did
not -- and "costs both nothing" asserted a zero the same table read as nonzero.

**What was done.** The bisection was rebuilt as six binaries, each rooted at the
same commit and each carrying exactly one change, and measured in **one
interleaved sitting** appended to `bench-baselines/phase-5a-arms.tsv` under
`task` = `11-bisection`. The `build` column names each variant:

| `build` | what it is | `varlookup` tw | `strings` tw |
|---|---|---|---|
| `a-base-21cde29af` | the commit before this task | 1761.000061 | 9096.519093 |
| `b-committed-f4b21eadb` | this task as first committed | 1776.999966 | 9168.518788 |
| `c-no-list-arm` | `b` minus `eval_node`'s own `List` arm | 1773.000081 | 9129.518712 |
| `d-no-trace-gates` | `b` with the two trace gates back on `to_text` | 1766.000074 | 9139.518738 |
| `e-outlined-6f3434e88` | the amended shape | 1773.000054 | 9129.518361 |
| `f-comment-control` | `e` plus one comment line | 1773.000048 | 9129.518465 |

`instructions:u`, per pass, median of 5 rounds. The subtractions a reader can now
perform, and the only ones the comments claim:

| quantity | `varlookup` | `strings` |
|---|---|---|
| the arm, marginal (`b` - `c`) | +3.999885 | +39.000076 |
| the gates, marginal (`b` - `d`) | +10.999892 | +29.000050 |
| the residual (`e` - `a`) | +11.999993 | +32.999268 |
| the whole move (`b` - `a`) | +15.999905 | +71.999695 |
| the layout control (`f` - `e`) | -0.000006 | +0.000104 |

Both comments now carry the marginal reading -- 39 and 4 for the arm, 29 and 11
for the gates -- say in so many words that the two are **not additive** (they sum
to 67 and 15 against a whole move of 72 and 16), and point at the rows. "Costs
both nothing" is gone; `eval_cold`'s doc states the residual instead.

**One correction inside this fix round.** `91935800d` put **38** in `eval_cold`'s
doc for the `strings` axis. `b` - `c` is 39.000076, so 39 is what that
subtraction yields; 38 was a misreading of the two rows rather than a second
measurement, and `fefe33597` corrects it. Its commit message says so.

**What the control establishes, restated.** A rebuild differing by one comment
produces identical machine code, so `f` - `e` had to come back nil: the control
cannot fail on `instructions:u`. What licenses reading a non-zero
`instructions:u` move as executed work is the counter, not the control, and the
causes came from the table above. Section 4's paragraph on this was rewritten to
say that rather than "on this instrument layout is not a term at all"
(observation O1).

### B2: `RAISE ADDITIONAL` given an array, and the sentences that claimed it away

**What the two comments said.** `lib.rs`'s `instruction_owner`: "`RAISE` is
likewise whole: `ADDITIONAL (a, b)` is a parenthesised list, which is an
*expression* and is implemented, and `ARRAY (a, b)` reaches the identical oracle
bytes". `owners.rs`: "... and `ExprKind::List` is in scope. So there is no
`RAISE` shape whose gap belongs to `RAISE`."

Reproduced independently before fixing anything, three descriptors, both
engines, fresh empty directory at a fixed path: `raise syntax 93.900 additional
(1,2)` was oracle rc 163 `Error 93.900:  1.` against crate rc 120
`rexx-exec: an array as a RAISE ADDITIONAL value is not implemented (Phase 5)`,
while `... array (1,2)` was byte-identical to the oracle. So the sentence
asserted a state a run contradicted, and the two spellings the oracle makes
identical were not identical here.

**The behaviour fix landed, so no arm-grained row was needed.** The change is
contained to the arm that already existed: `RexxObject::requestArray` answers an
array **unchanged** rather than compacting it, so an array's own slots are the
substitution list -- exactly what the `ARRAY` arm builds, empty slot holding its
place included. No new mechanism, no new value kind, one `Option`-returning
accessor on `Interp`. Measured after, both engines, three descriptors:

```text
raise syntax 93.900 additional (1,2)      = ... array (1,2)      = oracle
raise syntax 40.4 additional (1,,3)       = ... array (1,,3)     = oracle
raise syntax 40.4 additional ('R',,'X')   = ... array ('R',,'X') = oracle
raise syntax 40.4 additional 'JUSTONE'    unchanged, still oracle
raise syntax 40.4 additional (1)          oracle
```

**Three keyword value lines were wrong for the same reason section 2 records for
every other trace line**, found while checking the fix's neighbourhood rather
than reported by the review: `traceKeywordResult` is handed the *object* at the
condition keyword, at `DESCRIPTION` and at `ADDITIONAL`, and each keyword
separately converts the value for its own data. Measured, `trace i`:

```text
raise syntax (1,2)                                >K>   "SYNTAX" => "an Array"
raise syntax 93.900 description (1,2) ...         >K>   "DESCRIPTION" => "an Array"
raise syntax 93.900 additional (1,2)              >K>   "ADDITIONAL" => "an Array"
```

All three now render through `Interp::string_value_text` on the line and keep
`to_text` for the data.

**What still has no code, and the corrected sentences say it.** An `ADDITIONAL`
value under a `SYNTAX` condition that is a class object or one of the
interpreter's own is refused through `Loud::object_position` -- measured,
`additional (.array)` is rc 120 here against oracle rc 158 `98.939`, and
`additional (.environment)` is rc 120 against oracle rc 216 substituting the
directory's own first entry. That refusal predates this task (the review
confirmed it independently at `additional (.Array~superClasses)`) and is
unchanged. `Loud::object_position` is not reached through `Loud::instruction`, so
no owner string is read for it, which is the same shape `lib.rs` already
documents for `Expose`'s two sub-cases; both comments now say that instead of
claiming the variant whole.

**Three corpus programs, and the first two I wrote were worthless.** The first
attempt was a trapped ladder printing `rc'.'condition('E')` per row. Measured
against a mutation that closes the empty slot up and against one that renders
the traced object as its string value: **both left the corpus fully green**, so
the program witnessed neither. The reason is that a trap exposes only the number
and sub -- the substituted message exists only in the untrapped report, and
`condition('A')` would answer the array itself, which this crate refuses
(`rexx-exec: CONDITION option "A" answers an Array or .NIL, which is not
implemented`, measured). The rewrite is three untrapped, traced programs, all in
`corpus.rs`'s `RAW_STDERR_COMPARISON`:

```
lang/raise_additional_array.rex       trace i + additional ('R',,'X'), rc 216
lang/raise_array_spelling.rex         trace i + array ('R',,'X'), the pair
lang/raise_keyword_object_traces.rex  trace i + raise syntax (1,2), rc 223
```

Negative controls, each applied to the fixed tree and then reverted, with the
tree rebuilt and re-checked green after every one:

| mutation | corpus |
|---|---|
| an empty slot closes up instead of holding its place | 151 of 152 |
| `ADDITIONAL`'s own line renders the string value | 151 of 152 |
| `DESCRIPTION`'s own line renders the string value | 150 of 152 |
| the condition keyword's own line renders the string value | 151 of 152 |
| the refusal fires for an array instead of for an object | 151 of 152 |

### Observations: what was fixed and what was not

| | verdict |
|---|---|
| **O1** control's conclusion does not follow | fixed -- section 4's paragraph rewritten, and the Fix round 1 restatement above |
| **O2** "the sixth witness" is one too many | fixed -- report prose now says fifth; `spike.rs` itself never carried a count |
| **O3** `small`/`large` folded on an untrue claim | fixed -- the table now states the widest gap, `alloc4c`/`tw` at +0.457% against +0.445% |
| **O4** the `size_of::<Body>()` verification cannot establish it | fixed -- the type is the argument now, and the inequality is named as one; the reviewer's `Option<ObjRef>` reasoning is folded in |
| **O5** two comments name the size of a set | fixed -- `dispatch.rs`'s `Primitive::Directory` doc names the row set instead of counting it, and `lib.rs`'s `array_index_hole` doc names `Loud::instruction`'s `Do`/`Loop` carve-outs instead of counting "two" |
| **O6** `owners.rs`'s five-item block implies a bounded failure set | **skipped**, on the coordinator's instruction: the block's claim is a necessary condition and is still true, and the reviewer ruled this commit owes no amendment |
| **O7** `Array~at` clones the whole slot vector | fixed -- `Interp::array_slots` borrows, `~at`/`~size`/`~items` take the borrow, and only `makeString` and the subscript spread (both of which render and so need `interp` back) copy |
| **O8** "all four rows in `.Array`'s own dictionary" | fixed -- the sentence now says the four rows this task added and names the two that predate it |

### What this round could not close

* The two divergence families in section 6 are unchanged: `NUMERIC
  DIGITS`/`FUZZ`/`FORM VALUE` still quote the string value where the oracle
  quotes the object, and `INTERPRET` of a value holding a newline still reaches
  the documented parse-error gap. Both remain out of this task's scope for the
  reasons section 6 gives.
* The residual in the bisection table is still unattributed. This round made it
  derivable, not explained.
* The `RAISE` fix's own object-valued refusal has no differential row, by the
  same argument section 5 makes for the other three refusals. It **does** have an
  in-crate test; the sentence that stood here said it had none, which was wrong
  -- fix round 2 has the correction and names the test.

---

## Fix round 2

Commits `3983105e1` (the rendering fix, its corpus rows, and the `DO OVER ... FOR`
assertion) and `0ef493ae7` (the sitting's rows). All five gates green, corpus
**154 of 154**, up from the 152 of 152 fix round 1 records.

```
cargo fmt --all --check                                        exit 0
cargo clippy --workspace --all-targets -- -D warnings          exit 0
cargo test --release --workspace                               exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace            exit 0, 154 of 154 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
                                                               exit 0, 154 of 154 matching
```

### 1. A nested substitution item, and whether this task owns it

**The shape is this task's, established by running it rather than argued.** Both
programs on the crate source at `f4b21eadb~1`, both engines, fresh empty
directory at the same fixed path the oracle wrapper uses:

```text
raise syntax 93.900 array ((1,2),3)         rc 120  a parenthesised list is not implemented (Phase 5)
raise syntax 93.900 additional ((1,2),3)    rc 120  a parenthesised list is not implemented (Phase 5)
```

A nested item can only be written as a list inside a list, so before
`ExprKind::List` evaluated the inner `(1,2)` could not be built and neither
program ran. This task made the shape reachable and owns it; it is a fix and not
a recorded divergence.

**One aside on the identification, because the two names are not the same
commit.** `f4b21eadb~1` is `aba8b555e`, and the binary used above was built from
`21cde29af`'s crates. `git diff --stat 21cde29af aba8b555e -- rust/crates
rust/Cargo.toml rust/Cargo.lock Cargo.toml` is empty and the two commits between
them touch only `docs/superpowers/plans/2026-08-17-phase-5a.md`, so that binary
*is* `f4b21eadb~1`'s crate source.

**What was wrong and what fixed it.** The `ARRAY` arm rendered each element with
`to_text`. Measured before, three descriptors, both engines:

```text
raise syntax 93.900 array ((1,2),3)
  oracle  rc 163  Error 93.900:  an Array.
  crate   rc 163  Error 93.900:  1        (and `2.` on the next line)

trace i over the same program
  oracle  >A>   "an Array"   twice for the inner list
  crate   >A>   "1           (and `2"` on the next line)   twice
```

The arm builds its rendering once and uses it for the substitution and both
`>A>` lines, so one substitution -- `to_text` to `Interp::string_value_text` --
fixes all three. That is the renderer the `ADDITIONAL` arm already reaches, so
no second renderer was added.

**The two arms share the renderer and not the slot accessor, and that is the
construct rather than a compromise.** An `ARRAY` list's elements come from the
parse and there is no array object to read slots off; `ADDITIONAL`'s one value
*is* the array and its slots are the whole list. `Interp::array_slots_of` is
therefore reached only from the `ADDITIONAL` arm, and the comment at the `ARRAY`
arm says so.

Measured after: 13 programs across the nested neighbourhood -- both spellings
with a nested item, with a nested item beside an empty slot, with a one-item
inner list, with an inner list whose leading slot is empty, plus the class-object
and directory elements that must stay unchanged, and three traced variants --
**all 13 byte-identical on all three descriptors on both engines.**

Two corpus rows, both untrapped, traced and in `RAW_STDERR_COMPARISON`:

```
lang/raise_array_nested.rex        trace i + array ((1,2),,'X'), rc 216
lang/raise_additional_nested.rex   trace i + additional ((1,2),,'X'), the pair
```

One per spelling for the reason fix round 1's pair is two programs: the arms
share a renderer *today*, and a program per spelling is what keeps the pair
pinned if they stop. Negative controls, each applied and then reverted with the
tree rebuilt and re-checked green:

| mutation | corpus |
|---|---|
| the `ARRAY` arm joins a nested item instead of naming it | 153 of 154 |
| the `ADDITIONAL` arm joins a nested slot instead of naming it | 153 of 154 |

### 2. The report's own coverage claims, audited

**What the sentence said.** Fix round 1's "what this round could not close" said
the object-valued `ADDITIONAL` refusal has "no differential row ... and no
in-crate test either -- it is asserted only by the corrected comments and by the
last negative control above".

**The in-crate test half was wrong.** `eval.rs`'s
`an_object_as_a_raise_syntax_substitution_is_loud` runs
`raise syntax 40.1 additional (.array)` and `... (.environment)` through
`both_engines`, asserts rc 120 with empty stdout, and asserts the message names
`a RAISE ADDITIONAL value`. It predates this task. The sentence is corrected in
place and names the test.

**Every other sentence of that class in this report, checked against the tree
rather than reasoned about:**

| claim | verdict |
|---|---|
| section 5: the corpus gate cannot see the three added refusals | **holds** -- each is rc 120 where the oracle answers, so no differential row expresses it, and each names a test that a mutation reddened |
| section 9: `identityHash` has no differential row | **holds** -- the oracle's answer derives from an address, this crate's from a handle |
| section 9: the oracle crasher has no differential row | **holds** by construction; it is `oracle-crashes.txt` entry 6 |
| section 6: "a pre-existing `DO OVER ... FOR` defect ... no test in the tree covered it" | **wrong, and corrected** -- see below |
| fix round 1: the object-valued `ADDITIONAL` refusal has no in-crate test | **wrong, and corrected** -- the test is named above |

**The `DO OVER ... FOR` sentence.** `run/tests.rs`'s
`do_over_for_0_skips_the_single_non_stem_iteration` did cover the shape: it runs
`do x over 'hello' for 0` and asserts the body does not run, which was true
before this task and is true after. What nothing asserted is the control
variable's value afterwards -- the half that was wrong -- and that test's own doc
called the rule "the direct, minimal extension of `FOR`'s own general rule" and
"a judgement call", which stopped being true when fix round 1 measured
`checkOver(...) && checkFor()`. Both are fixed in `3983105e1`: the doc now
carries the C++ site and the measurement, and a third assertion covers the
binding -- `x = 'pre'` before `do x over 'hello' for 0` leaves `x` holding
`hello`, which `corpus/lang/array_do_over.rex` already pins against the oracle
and this now pins in-crate.

**Corrected by the round-2 re-review: that class of sentence has been wrong
twice on this task**, about the `RAISE` refusal and about `DO OVER ... FOR`.
This paragraph said "three times on this task" and counted the send-path
instrument as the third, which is Task 10's episode and not this task's -- and
Task 10's own re-review recorded it ADDRESSED and independently reproduced,
so it was never a standing false claim there either. The shape does recur
across the plan; the count of it here was borrowed. In both real cases a test
existed and the sentence was written from memory of what was added rather than
from a search.
The check that works is grepping the test files for the construct before writing
"no test covers X", and it is what produced this table.

### The sitting

Run because the change alters what executes. Staleness test first, from the
repository root: the pin's commit is an ancestor of `HEAD`, the crate-source
commit list is 47 long, and every entry is in this ledger except `3983105e1`
itself, which this round adds. `sha256sum` of the pin still
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`.

Recorded under `task` = `11-fixround-2`, `commit` = `3983105e1`.
`across_builds`, `pinned>head`, `instructions:u`:

| axis | `tw` | `ir` |
|---|---|---|
| `alloc4c` | +0.457% | +0.270% |
| `arith` | +0.150% | +0.012% |
| `compound` | +0.440% | +0.262% |
| `emptyloop` | 0.000% | 0.000% |
| `strings` | +0.940% | +0.969% |
| `varlookup` | +0.681% | 0.000% |

No figure moves a conclusion and nothing reaches the plan's 1% threshold, so
this round moved nothing measurable on any axis. This paragraph said "every
figure is the round-1 sitting's to three decimal places", which the round-2
re-review checked and found false for two of the published percentages:
`alloc4c`/`ir` is +0.271% against round 1's +0.270%, and `arith`/`tw` is
+0.151% against +0.150%. Both are a few parts per million of ratio drift
landing either side of a rounding boundary, which is what the methodology
finding below is about. `strings`/`ir` is `head - pinned` = +52.0003 against the 53.695178
ceiling, leaving 1.6949.

**One methodology finding worth recording, because it bears on how this report's
own tables may be read.** The *pinned* build's own per-pass figure moved between
sittings on two axes -- `arith`/`ir` read 23764.5222 in the round-1 sitting and
24059.5738 here, `alloc4c`/`ir` 3591.3984 and 3594.6850 -- for the same binary
and the same program. The within-sitting difference did not: `head - pinned` for
`arith`/`ir` is +2.9981 and +3.0008 across the two, and for `alloc4c`/`ir`
+8.9985 and +9.0012. So a per-pass figure is comparable **within** a sitting and
not across sittings on those axes, which is what the plan's interleaving rule
already says and is now measured here. Every subtraction this report performs is
within one sitting -- the bisection table is a single six-build sitting, and each
budget figure is a `head - pinned` pair from one -- so no claim above depends on
the cross-sitting comparison this finding invalidates.
