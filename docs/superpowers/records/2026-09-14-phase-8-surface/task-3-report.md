# Phase 8 surface, Task 3 -- the conversion table's remaining rows

BASE `5d84dd8cb`. Scratch: `scratchpad/surface-t3/` (probes under `p/<name>/`, run by
`surface-t3/probe.sh`: oracle and crate each from a fresh copy of `src/` at the same path, three
descriptors to files).

## Status

DONE_WITH_CONCERNS. Commits `e0b30d156` (the controller's `refusal-sites.tsv` re-derivation, before
the task), `edcf1ff40` (every row both ways, witnesses, table, exclusions), `c00d18052` (frame reuse
removing the per-call cost `edcf1ff40` added, and two tests), `034c1c7d7` (an exclusions record).
Concerns are at the end.

## Before Task 3: `refusal-sites.tsv` re-derived (controller's request)

The post-Task-2 gate run (`scratchpad/gate8t2/status.txt`: G1 0, G2 0, G3 101, G4 101, G5 0,
`ALLDONE`, rev and dirty-after `5d84dd8cb` and clean) failed
`the_table_holds_every_constructor_the_source_defines` (`g3.txt:2928`) beside the known
`the_l0_subset_passes_again_under_collect_on_every_allocation` (`g3.txt:1597`).

`REXX_REFUSAL_SITES_REFRESH=1 cargo test -j 4 --release -p rexx-exec --test refusal_sites`, exit 0.
A line-by-line comparison against a copy taken before (`surface-t3/refusal-sites.before.tsv`, a
python split on tabs): 307 lines both, 19 rows changed, every one in column 4 alone, 0 in any
other column. `cargo test -j 4 --release -p rexx-exec --test refusal_sites`: 5 passed. Committed
as `e0b30d156`.

## Oracle measurements taken before implementing

All three descriptors to files, from a fresh copy per side (`surface-t3/probe.sh`); the crate side
at `5d84dd8cb` refuses on the first unfilled row in every probe (rc 120), so only the oracle's
transcripts are the finding here.

### Which extension reaches which row

`testbinaries/orxmethod.cpp` and `orxfunction.cpp` (oracle `build/lib/liborxmethod.so` and
`liborxfunction.so`, `readelf -d` NEEDED `libc.so.6` alone) declare an echo method or routine for
every `REXX_VALUE_*` argument row, most returning the same type, so those reach both directions
with no interface call. The rows whose only declarations call an unfilled interface member on the
success path are `RexxMutableBufferObject` (`TestMutableBufferLength` and siblings) and
`RexxVariableReferenceObject` (`TestSetVariableReferenceValue`); their refusals (88.914) happen
before the call and are reachable. The special codes as return types are declared by nothing
shipped. Those rows are measured through a forged scratch library, `surface-t3/ext/libforge3.so`
built from `ext/forge3.cpp` with `g++ -shared -fPIC -std=gnu++11 -fpermissive -w -I api -I
api/platform/unix` (the header's own `oor__ret` cannot compile an `OSELF` result without
`-fpermissive`); `readelf -d` lists no NEEDED.

### Argument rows (`p/n2`, `p/m1`, `p/r1`)

* Every integer row refuses with **88.907** `Argument N must be in the range MIN to MAX; found
  "V".`, MIN and MAX the C type's (`int8` -128/127 ... `uint64` 0/18446744073709551615;
  `wholenumber_t` +-999999999999999999; `stringsize_t` 0/999999999999999999; `size_t` and
  `uintptr_t` 0/18446744073709551615; `ssize_t`, `intptr_t`, `int64_t`
  -9223372036854775808/9223372036854775807). `nonnegative_wholenumber_t` is **88.904** `The N
  argument must be zero or a positive whole number; found "V".`; `positive_wholenumber_t` 88.905
  as Task 2 measured. `found` is the argument's string value: `.object~new` renders `an Object`,
  `.nil` `The NIL object`.
* Accepted spellings, the same for every integer row: `1.0`, `1E2`, `1e+2`, ` 12 `, `0.0`, `-0`,
  `+5`, `10E-1`, `5.` convert (`0.0` and `-0` refuse only for `positive_wholenumber_t`); `1.5`,
  `2.50`, `1E-1`, `.5`, `abc`, `` refuse.
  `12345678901234567890` converts for `uint64_t`, `size_t` and `uintptr_t` and refuses elsewhere.
* `logical_t` refuses anything but exactly `0` or `1` with **34.901** `Logical value must be exactly
  "0" or "1"; found "V".` (`1.0`, `-0` and `.object~new` included; `.true` converts).
* `float` refuses as `double` does, 88.921.
* `RexxArrayObject`: a string converts through `makeArray` (`'abc'` a one-item array, `''` an
  empty one); `.object~new` and `.array~new(2,2)` refuse with **98.913** `Unable to convert object
  "V" to a single-dimensional array value.`
* `RexxClassObject` and `POINTER` refuse a non-instance with **88.914** `Argument N must be an
  instance of the Class class.` (`Pointer` for the pointer row, `MutableBuffer` and
  `VariableReference` for theirs, `p/f1`).
* `POINTERSTRING` refuses with **88.919** `Argument N is not in valid pointer format; found "V".`
  (`abc`, `0x`).
* `RexxStemObject` in a method refuses a name with **93.969** `Method argument N must have a stem
  object value; found "V".`; in a routine a name resolves the caller's stem, case-insensitively,
  with or without the trailing period (`y`, `Y.`), an unset name answering a stem named `ZZ.`;
  `a.b`, `1x`, ``, ` zz`, `zz..`, `.zz` and `.object~new` refuse with **40.919** `Argument N must
  have a stem object or stem name value; found "V".`
* Every refusal above is delivered as the existing ones are (`p/d-*`, a method declared in a
  required `pk.cls`): `Error NN running <pk.cls>:` with no line, under the method's
  `Compiled method` traceback line and the sender's line. rc 168 (88), 222 (34), 158 (98), 163 (93).

### Result rows (`p/n2`, `p/v1`, `p/v2`, `p/f1`)

* Integers render as their digits at every width, `int64_t` 1000000000000000000 and `uint64_t`
  18446744073709551615 included.
* `double` and `float` render at **9 digits whatever the caller's `NUMERIC DIGITS`**: under
  `numeric digits 5` and `20` alike, `TestDoubleArg(2/3)` gives `0.66667` and `0.666666667`
  (the argument is formed at the caller's digits, the result at 9), `TestFloatArg(1.1)`
  `1.10000002`, `TestDoubleArg(123456789)` `123456789`; the same through a routine. A float past
  its range is `+infinity`, `1e-50` is `0`.
* `logical_t` answers `1` for any non-zero value (`ForgeLogicalOf(5)`, `(-1)`).
* `CSTRING`: a null result answers no value (44.1 `No data returned from function` in a routine
  call), and bytes stop at the first NUL (`"ab\0cd"` has length 2).
* `POINTERSTRING` null renders `0x0`; `TestPointerStringArg(TestPointerStringValue())` is `1`.
* The special codes as result types, every one of `ARGLIST`, `NAME`, `SCOPE`, `CSELF`, `OSELF`,
  `SUPER`: **40.918** in a routine and **93.968** in a method, after the call.

### Special argument rows (`p/v2`, `p/f1`)

* `OSELF` the receiver; `SCOPE` the method's scope (`The T class`); `SUPER` the scope's
  superclass (`The BASE class`, `The Object class`); in a routine each is 40.918.
* `ARGLIST` an array of the call's arguments with omitted positions empty (`(1, , 3)`: items 2,
  size 3); two `ARGLIST` parameters in one signature answer the same array. With an `ARGLIST`
  anywhere in the signature, extra arguments are not refused, in a method and in a routine.
* `NAME` the message name as sent (`NAME`, `OTHER` for two methods bound to one procedure), and in
  a routine the name as called, uppercased (`TESTNAMEARG` for either spelling).

### Integer and pointer-string edges (`p/q1`, `p/q2`)

`NumberString::int64Value`/`unsignedInt64Value` (`NumberStringClass.cpp:1024`, `:1132`) at 20
digits, not `numberValue`'s 18, and `createUnsignedInt64Value`'s overflow test (`:863`) is
`newNumber < intNumber`, which a multiplication by ten can wrap past undetected. Measured:
`uint64_t('30000000000000000000')` is `11553255926290448384`; `'21000000000000000000'` is
`2553255926290448384` for `int64_t`, `ssize_t`, `intptr_t` and `size_t`, refused for `int`.
Rounding past 20 digits carries: `uint64_t('1234567890123456789.95')` is `1234567890123456790`,
`.45` refuses; `int('1.99999999999999999995')` (21 digits) is `2`, `'1.9999999999999999995'` (20)
refuses. A value all of whose digits are decimals rounding up is `1` **with no sign**:
`int64_t('-0.999999999999999999999')` and `int(...)` are `1`. `int64_t('-9223372036854775808')`
converts and `'-9223372036854775808.0'` refuses; `'9223372036854775807.0'` converts.

`POINTERSTRING` is `sscanf(s, "0x%p")` over `stringValue()`, not the string conversion: an
instance whose class defines `string` is found as `a S`, `.array~of(v)` as `an Array`, a
`MutableBuffer` holding the address converts. Converting (`1`): the address as written, `0x0x`
prefix, `0x0X`, whitespace (blank, tab, newline, vertical tab) after `0x`, trailing junk, upper-case
digits, `+`, leading zeros, `-` with the two's complement, `-0x` with it. Refused, 88.919: `0X`,
a leading blank, `0xg`, `0x0xg`, `0x0x`, `0x-`, `0x `, `0x--1`, `0x+-1`, `0x0x ` then digits, a NUL
after `0x`, `12`, `.nil`. Converting to something else (`0`, no refusal): `0x0`, an overflow either
way.

## Design

* **The table stays the one switch.** Every row gets both conversions; `stub` and
  `Failure::Unfilled` go, which closes the doubled-suffix entry by removing the text that doubled.
  The special codes' result conversion is a row function answering `ResultSignature`.
* **Host primitives, not host conversions.** Each row function in `values.rs` does what its
  `processArguments` case does and asks the host only for what the interpreter owns:
  `signed_integer(object, min, max)` (`objectToSignedInteger`, which `int64Value`'s callers reduce
  to at `SIZE_DIGITS` = `DIGITS64` = 20) replacing `positive_whole_number`;
  `unsigned_integer(object, max)`; `logical(object)` (`truthValue`); `array_value(object)`
  (`requestArray` plus the dimension check); `is_stem(object)` and `context_stem(object)`
  (`getContextStem` over the string conversion); `instance_of(object, class)`;
  `pointer_value(object)`; `string_value_text(object)` (`stringValue`); `receiver`, `scope`,
  `super_scope`, `arguments` (one array per call, as `getArguments` caches it) and
  `message_name`; `unsigned_number(u64)` and `new_string(bytes)` for results. The integer
  semantics live in `rexx-num` as `Number::int64_value`/`unsigned_int64_value`, beside
  `whole_value` (`numberValue`), which answers differently past 18 digits.
* **`float` and `double` results at 9 digits** through the existing `double_object`.
* **A `CSTRING` result is copied where the union is read.** Only `load.rs` may dereference, so
  `call_stub` copies the bytes of a declared `CSTRING` result (never of a special code's word,
  which `valueToObject` does not read) and `invoke::run` interns them into the call's pool; the
  row reads them back through `CStringPool::bytes_at`.
* **`usedArglist`** is a third `Source`, read by `invoke::run`'s one too-many check (S2).

### Found before building: the refusal's `found` for an array (`p/a1`)

Task 2's refusals render `found` through `to_text`, which for an array is its items joined.
`RxCalcSqrt(.array~of(1, 2))`: oracle `88.921 ... found "an Array".`, crate at `e0b30d156`
`found "1<LF>2".` (both rc 168); the same for 88.905 through the precision. The substitution is
the object's `stringValue()`. Fixed here with the new rows, which render every `found` the same
way.

## What was built (in progress, uncommitted)

* `rexx-num`: `Number::int64_value`/`unsigned_int64_value` and `DIGITS64`; `checkIntegerDigits`
  extracted as `Number::integer_digits`, which `whole_value` now calls (its tests unchanged,
  green); `tests/int64.rs` holds the measured cases above.
* `rexx-api/src/values.rs`: every row has both conversions; `stub`, `Failure::Unfilled` gone;
  new failures `NotNonnegative`, `OutOfRange`, `NotLogical`, `NotArray`, `NotInstance`,
  `NotPointerString`, `NoStem`; `Source::Arguments`; `ResultRead`/`Written`; the host primitives
  of the design; `pointer_string` (the `sscanf` reading).
* `invoke.rs`: the too-many check lifts for a signature with `ARGLIST`; a `CSTRING` result is
  interned into the call's pool before `from_native`. `load.rs`: `call_stub` copies a `CSTRING`
  result's bytes.
* `rexx-exec`: `NativeFrame` carries the receiver, the name, the arguments and the argument array
  once built (all rooted); the host methods; the refusal constructors
  (`native_argument_not_nonnegative`, `_outside_range`, `_not_logical`, `_not_an_array`,
  `_not_an_instance`, `_not_a_pointer`, `_not_a_stem`), each lineless as measured; `found` through
  `string_value_text` for every native refusal.

### Probes against the release binary of the uncommitted tree

`p/n2`, `m1`, `v1`, `v2`, `r1`, `q3`, `a1`, every `p/d-*`, `f1` (forged library), `x1` (rxmath's
`CSTRING` results, `orxmethod`'s `size_t` results, an unset stem name binding the caller's
variable, `~send`/`~sendWith` into `ARGLIST`, `NAME` through `loadExternalRoutine`, `CALL` and a
`::ROUTINE` alias, class, array-convertible and logical arguments, `found` for an array),
`x2`, `x5`: identical on all three descriptors. `q1` and `q2` identical but for the address
`TestPointerStringValue` answers, which differs between any two runs. `x4` differs: a library
method given to `setMethod` refuses naming Phase 5 (rc 120) where the oracle runs it (`SCOPE` of
such a method is `.nil`); recorded in Task 2's exclusions block, not this task's.

### Tests written

`rexx-num/tests/int64.rs` (7 tests, the measured cases). `rexx-api/tests/values.rs`: the host
stand-in answers every new primitive; per-row tests for every row this task fills, both
directions (`the_<type>_row_converts_its_range_both_ways` over each integer row through
`signed_row`/`unsigned_row`, the nonnegative, positive, logical, float, double, `CSTRING` result,
object, array, stem, class, mutable buffer, variable reference, `POINTER`, `POINTERSTRING` rows,
`OSELF`/`SCOPE`/`SUPER` through `a_method_only_object_row`, `ARGLIST`, `NAME`, `CSELF`'s result),
set assertions (`the_result_of_exactly_the_special_rows_is_refused`,
`only_the_argument_list_lifts_the_too_many_check`), the error numbers, and the `sscanf` reading
(`a_pointer_string_is_read_as_sscanf_reads_0x_p`). The removed tests were the ones pinning rows as
unfilled. `rexx-api/src/invoke.rs` (lib, so Miri runs them): a `CSTRING` result copied to its
first NUL and a null one, a special code as the result, `ARGLIST` lifting the too-many check in a
method and a routine, over new stubs in `ffi.rs`. `rexx-exec`: the frame's new fields rooted while
the frame lives (`a_native_activations_call_state_is_rooted_only_while_it_lives`);
`an_extension_reading_the_instance_reaches_its_table` now asserts `before`/`328448` at rc 0, the
oracle's answer, where it pinned the unfilled `size_t` refusal.

`cargo test -p rexx-api`: every binary ok. `cargo test -p rexx-num`: ok.
`cargo test --release -p rexx-exec --lib`: 829 passed. `refusal_sites` fails until re-derived
(new constructors), below.

### Found in passing, not this task's (`p/b1`, `p/b2`)

A trapped refusal from a native method declared in a required package carries the caller's
`POSITION`, `PROGRAM` and `TRACEBACK` here: oracle `88.901 The NIL object pk.cls` with a
`Compiled method "INT" with scope "T".` traceback line first, crate `88.901 8 main.rex` without
that line. The same for `88.907` and `88.904`, and for `t~int()` and `t~float()`, whose 88.901
comes from `MissingArgument`'s arm, which this task's diff does not touch; not run against a base
binary. Witnesses here print `code` and `message` only.

## Witnesses

Under "Surface Task 3" in `corpus/phase-8.txt`, each with an `.env` naming `{oraclelib}` and a
`sourceline_oracle` companion generated by the module comment's driver from a directory holding
only a copy (still only the copy afterwards; each companion's lines `cmp`-identical to the file):
`library_native_integer_arguments` (every integer row's ends and one past each, the spellings,
the rounding, the sign and wrap answers, a routine), `library_native_results` (float and double
at digits 5, 9 and 20, logical, `CSTRING` including rxmath's `MathLoadFuncs`/`MathDropFuncs` and
`CALL`, `size_t` through `TestInterpreterVersion`/`TestLanguageLevel`, pointer and pointer
string), `library_native_object_arguments` (object, array and what converts to one, class,
logical, pointer, the pointer-string forms, stems by object and by name, `found` for an array
through 88.921, 88.905, 88.907 and 40.919), `library_native_special_arguments` (`OSELF`,
`SCOPE`, `SUPER` instance and class side, `ARGLIST` through sends and routines, `NAME` through
two methods on one procedure, a send, a function call, `CALL`, a `::ROUTINE` alias and
`loadExternalRoutine`, 88.922 for a signature without the list), and one untrapped two-file
refusal per new constructor, `library_native_refusal_range`, `_nonnegative`, `_logical`,
`_array`, `_instance`, `_pointer_string`, `_stem`, `_stem_routine`. All identical on three
descriptors through `probe.sh` against the release binary before they were copied.
`cargo test --release -p rexx-parse --test sourceline_oracle`: 1 passed.

`refusal-sites.tsv` re-derived (`REXX_REFUSAL_SITES_REFRESH=1`), the seven new rows filled
`agrees`/`yes` with their witnesses; `SHARED_ANSWERS` and the header gain the groups the new rows
make (34.901, 88.907, 98.913, and a fourth member of 88.914), and the header's "Measured" holds for
them: transposing the answer and witness of each new pair, and of `native_argument_not_an_instance`
with `argument_not_a_class`, left `refusal_sites` at 5 passed; the table restored from a copy and
`cmp`-checked after each. `phase-4-exclusions.txt`: the `Unfilled` entry closed, the routine
half's "still refuses" list loses its conversion-row limb, and the condition-object divergence
above recorded.

## Negative control: prediction, written before running

Mutation: delete the `uint16_t` row from `values::TABLE`, a row this task added.

* `rexx-api` `tests/values.rs` red, exactly: `the_table_has_a_row_for_every_code_the_header_defines`,
  `the_uint16_row_converts_its_range_both_ways`,
  `a_result_is_read_as_its_rows_member_and_a_cstring_as_its_bytes`. Every other test in that file
  green, the set assertions included (they iterate `rows()`, which no longer names the code).
* Every other `rexx-api` test binary green, lib included.
* Corpus: `library_native_integer_arguments.rex` red (the `uint16` line and the routine line) and
  no other program.

## Negative control: what ran

**Confirmed.** `cargo test -j 4 -p rexx-api --no-fail-fast` (`surface-t3/ctl/api.txt`): red exactly
`a_result_is_read_as_its_rows_member_and_a_cstring_as_its_bytes`,
`the_uint16_row_converts_its_range_both_ways`, `the_table_has_a_row_for_every_code_the_header_defines`
(values: 80 passed, 3 failed); every other binary ok. `REXX_CORPUS_GATE=1 cargo test -j 4
--release -p rexx-exec --test corpus` (`ctl/corpus.txt`): `602 of 603 matching`, the one mismatch
`lang/library_native_integer_arguments.rex: stdout, stderr, exit code differ`. `values.rs` restored
from the copy and `cmp`-checked.

## Miri

`RUSTUP_HOME=scratchpad/final-fix/rustup-home CARGO_TARGET_DIR=scratchpad/surface-t3/miri-target
rustup run nightly cargo miri test -p rexx-api --lib --locked` over the worktree, Stacked Borrows
(`MIRIFLAGS` unset): 27 passed, 0 failed, 3 ignored (the child-process test and the two that open
the running image), exit 0 (`surface-t3/miri-sb.txt`). Among the passes are the three new
`invoke` tests, one of which reads a `CSTRING` result through `load.rs`'s new `CStr::from_ptr`.

## Commit

`edcf1ff40` "Fill every row of the native API conversion table in both directions" (read back with
`git log -1`). Before it, on the tree it commits: `cargo fmt --all --check` exit 0; `cargo clippy -j
4 --workspace --all-targets -- -D warnings` exit 0; `cargo test -j 4 -p rexx-api` every binary ok;
`cargo test -j 4 -p rexx-num` no failure; `cargo test --release -p rexx-parse --test
sourceline_oracle` 1 passed; `cargo test --release -p rexx-exec --test refusal_sites` 5 passed.
The crate test suites and the corpus gate run after the commit from
`surface-t3/gates/run.sh`, statuses in `surface-t3/gates/status.txt`; results below once read.

### Gates at `edcf1ff40`

`surface-t3/gates/status.txt`: first line `edcf1ff40`, `git status --untracked-files=no` empty
before and after, `rev-after` `edcf1ff40`.
* `memcap 16G cargo test -j 4 --release -p rexx-api -p rexx-num -p rexx-parse -p rexx-exec
  --no-fail-fast`: **exit 101**, 89 `test result: ok` lines and one FAILED binary, `collect_stress`
  31 passed / 1 failed: `the_l0_subset_passes_again_under_collect_on_every_allocation`, panicking at
  `crates/rexx-exec/src/dispatch.rs:1510:9`, the known failure the post-Task-2 gate run shows at the
  same line (`gate8t2/g3.txt:1603`).
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  `603 of 603 matching`.

## Cost, and a second commit

Callgrind `Ir` totals, interleaved base then head per program, a fresh run directory each
(`surface-t3/perf/run.sh`, `results.txt`, `results2.txt`). Base is `e0b30d156` built from a `git
archive` copy in its own target directory (`rexx-run-base`, sha256 `8605fb2975ded0b8…`); head is
`edcf1ff40`'s release binary copied out after its gates (`rexx-run-head`, `7eed8c762437d772…`).
`method.rex` sends `RegExp_Match` 20000 times; `routine.rex` calls `RxCalcSqrt(4)` 20000 times;
`cps` is Task 2's pinned REXXCPS.

| program | base r1 | head r1 | base r2 | head r2 |
|---|---|---|---|---|
| `method.rex` | 290,689,528 | 296,257,340 | 290,693,608 | 296,494,755 |
| `routine.rex` | 301,217,145 | 306,883,322 | 301,203,929 | 306,905,620 |
| `rexxcps` (one run each) | 21,142,952,666 | 21,142,188,653 | | |

About 280 instructions more per native call (+1.9%), which `callgrind_annotate` puts in `malloc`,
`free` and `memcpy`: `edcf1ff40` copied the name and the arguments into every frame. Fixed by
reusing frames: `Interp::native_spares` holds popped frames emptied, and the next call extends
their buffers, which also keeps the local-reference table's capacity. Re-measured against the same
base (`rexx-run-head2`):

| program | base r1 | reuse r1 | base r2 | reuse r2 |
|---|---|---|---|---|
| `method.rex` | 290,689,748 | 283,195,232 | 290,694,312 | 283,104,907 |
| `routine.rex` | 301,185,174 | 293,192,503 | 301,165,081 | 293,348,076 |
| `rexxcps` (one run) | 21,142,952,666 (above) | 21,147,784,825 | | |

Now about 375 instructions fewer per call than base (-2.6%, -2.7%). `rexxcps` +0.02%, one run, under
the layout noise this tree has measured; not evidence either way.

Tests with it: `a_reused_native_frame_holds_nothing_of_the_call_before` (a handle the last call
registered resolves to nothing, and name, arguments, argument array, condition and receiver are
the new call's), and
`the_conversion_rows_answer_the_same_under_a_collection_at_every_allocation` over an inline program
whose stdout is the oracle's (`surface-t3/p/st1`, identical on three descriptors). That program
reads refusal codes, not messages: `library_native_integer_arguments`, `_object_arguments` and
`_special_arguments` (not `_results`), and Task 2's
`library_routine_argument_errors.rex`, panic under collect-on-every-allocation at
`dispatch.rs:1510` (`a live value`), and so does the four-line `signal on syntax` / `y = 1/0` /
`exit` / `syntax: say 'caught' condition('O')~message` with no native call (`surface-t3/stress/s13`;
`~code` in its place runs, `s12`).

Controls, predictions in `surface-t3/ctl/c2-prediction.txt` and `c3-prediction.txt`, written before
running:
* **C2 confirmed**: deleting the `push_temp` in `unsigned_number` and in `new_string` leaves the
  collection test green; each answer reaches its caller with no allocation between, so those roots
  are not load-bearing for it.
* **C3 confirmed**: deleting any one of the four resets in `pop_native_frame` reddens the reuse test
  (four runs, each `0 passed; 1 failed`); restored from a copy and `cmp`-checked.

Before the second commit: `cargo fmt --all --check` exit 0, `cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0, `cargo test --release -p rexx-exec --lib` 831 passed,
`refusal_sites` 5 passed.

### Gates at `c00d18052`

`surface-t3/gates2/status.txt`: first line `c00d18052`, `git status --untracked-files=no` empty
before and after, `rev-after` `c00d18052`.
* `memcap 16G cargo test -j 4 --release -p rexx-api -p rexx-num -p rexx-parse -p rexx-exec
  --no-fail-fast`: **exit 101**, 89 `test result: ok` lines, one FAILED binary: `collect_stress` 31
  passed / 1 failed, `the_l0_subset_passes_again_under_collect_on_every_allocation` at
  `dispatch.rs:1510:9`, as at `edcf1ff40` and before the task.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  `603 of 603 matching`.

The base binary of the cost tables was checked to be the revision before the task: over
`TestInterpreterVersion` it refuses naming the unfilled `size_t` row at rc 120, where
`rexx-run-head2` prints `328448` at rc 0 (`surface-t3/perf/verify-base`).

### `034c1c7d7`

`phase-4-exclusions.txt` gains the four-line `condition('O')~message` panic under
collect-on-every-allocation as its own entry in this task's block. The tests that read the file,
each `cargo test -j 4 --release -p rexx-exec --test <name>`: `owners` 6, `licensed_divergences` 22,
`coverage` 21, `builtin_status` 26, `keyword_assertions` 7, `bif_assertions` 5 passed.

Scratch target directories deleted by path (`surface-t3/miri-target`, `surface-t3/perf/base-target`).

## Concerns

* **Two rows succeed only where a forged extension reaches them.** `RexxMutableBufferObject` and
  `RexxVariableReferenceObject` arguments converting (and coming back) are witnessed by unit tests
  and the scratch transcript `p/f1` over `ext/libforge3.so`; every shipped declaration calls an
  unwritten interface member on that path. Their 88.914 refusals are corpus-witnessed through
  `orxmethod`. The special codes as result types likewise: unit tests and `p/f1`.
* **A `POINTERSTRING` past the range** reads as `strtoul` answers (the largest value, whatever the
  sign). The oracle's answer is observable only as "not the address" through `orxmethod`, which is
  what the witness checks; the value itself is not measured.
* **`float`/`double` results at nine digits** were measured under `NUMERIC DIGITS 5` and `20`, not
  under `::OPTIONS DIGITS`; the row passes a constant.
* **`positive_wholenumber_t` now converts through `int64_value` at 20 digits** (the shared
  `signed_integer` primitive) where Task 2's host used `whole_value`. Its witnesses and the oracle's
  answers over `p/n2`'s spellings agree; read from the two functions' code, not measured, they can
  answer differently only for a value of more than 18 integer digits, which the row refuses either
  way.
* **Found in passing, recorded, not fixed:** a trapped native refusal's condition object carries the
  caller's `POSITION`/`PROGRAM`/`TRACEBACK`; `condition('O')~message` on a trapped SYNTAX panics under
  collect-on-every-allocation (three of this task's witnesses, and one of Task 2's, cannot run in
  that mode); a library method given to `setMethod` refuses naming Phase 5 (Task 2's entry).
* **The collection test does not see the new `push_temp`s** (C2): it witnesses that the rows run
  unchanged under a collection at every allocation, not that each root is needed.
* **The in-crate tests load the worktree's `build/lib`**, a second build of the same sources; the
  corpus loads the oracle checkout's.

# Fix round 1

BASE `034c1c7d7`. Both reviews read in full (`task-3-review-rows.md`, `task-3-review-host.md`).
Order as the controller set it: the condition object's roots, the six stress mismatches behind it,
`logical_t`'s `found`, `found` through `OBJECTNAME`/`DEFAULTNAME`, glibc's `(nil)`, the minors, and a
record of the host review's Minor 6. Scratch: `scratchpad/surface-t3/fix1/`.

## Item 1: a condition object's entries rooted while it is built

Reproduced (`fix1/p/ord1`, the reviewer's loop): oracle rc 0 `done 20001`; the crate at
`c00d18052` (`perf/rexx-run-head2`) rc 101, `panicked at crates/rexx-exec/src/dispatch.rs:1510:9: a
live value`, stdout empty. Fix: `condition.rs`'s `build_condition_object` roots each entry's value
as it is made (`push_temp` before every `entries.push`, inside the frame the function already
pops). `STACKFRAMES`/`TRACEBACK` were rooted by `new_list` already; they are rooted again beside
their push, which costs a temp slot and keeps the rule one line per entry.

After it: `ord1` identical on three descriptors (rc 0). Witnesses:
* corpus `lang/condition_object_entries_rooted.rex` (the loop at 3000 iterations; the
  `c00d18052` binary is rc 101 on it, the fixed one rc 0 and identical to the oracle), with its
  companion (`count 19`, lines `cmp`-identical, generated from a directory holding only a copy);
* `condition::tests::a_syntax_condition_objects_entries_survive_a_collection_at_every_allocation`,
  the reduction under `run_program_collect_every_alloc`, its stdout the oracle's line (`fix1/p/s13`,
  identical).

Control C4, prediction before running: deleting the `push_temp`s this item adds reddens that unit
test with the `a live value` panic, and the corpus gate goes red on
`condition_object_entries_rooted.rex` alone.

C4 ran: the unit test **confirmed** (`0 passed; 1 failed`, the `a live value` panic). The corpus gate
**confirmed as far as it can see**: `corpus_differential` fails with the interpreter thread
panicking while running `lang/condition_object_entries_rooted.rex` (`fix1/ctl/c4-corpus.txt`); a
panic there stops the harness, so "alone" is not observable. `condition.rs` restored from the copy
and `cmp`-checked.

## Item 2: the six stress mismatches

A scratch example binary (`crates/rexx-exec/examples/stress_probe.rs`, never committed, deleted
before each commit) runs one program plainly and under `run_program_collect_every_alloc`, each from
its own fresh directory (`fix1/st/run.sh`, `st/cmp.py` comparing rc, stdout and stderr exactly).
`collect_stress`'s L0 test with item 1's fix alone (`fix1/l0-a.txt`): exactly the six the reviewer
named, each stress run rc 120 `a message send to a value whose object is no longer live`.

**Cause A, the stream name** (five of the six). `call_miss_not_cached` reduced to one line,
`call lineout 'zappear.rex', 'return 1'` (`st/src/cm5.rex`, stress rc 120 after 4 collections). A
temporary backtrace at the refusal (removed) put the dead receiver inside `.Stream~new`'s `INIT`,
reached from `Interp::resolve_stream` (`run.rs`), which builds the name with `self.text(name)` and
sends `NEW` with it unrooted. Fix: `push_temp` on the name. With it, `address_with_stream`,
`executable_context`, `sys_file_functions`, `security_manager` (with its `.d/`) and
`call_miss_not_cached` match their plain runs under stress.

Control C5, prediction in `fix1/ctl/c5-prediction.txt` before running (with that `push_temp`
removed, those five DIFF rc 120 and `condition_object` stays DIFF): **confirmed**, all six DIFF rc
120; `run.rs` restored and `cmp`-checked.

**Reached in an ordinary run.** `fix1/p/str2`: `do i = 1 to 40000; s =
stream('a_stream_name_long_enough_for_the_heap.txt', 's'); end; say 'done' i s`. Oracle rc 0
`done 40001 UNKNOWN`; `c00d18052` rc 120 with the same refusal; fixed rc 0, identical. Bounds 2000
to 30000 agree on the old binary, so the first failing iteration is between 30001 and 40000. A
corpus witness of it was written and **withdrawn**: under collect-on-every-allocation, which the
stress harness applies to every phase file, it ran 46.8 s (120,003 collections), too slow for the
gate. The witness is the unit test
`run::tests::a_stream_builtins_name_survives_a_collection_at_every_allocation` (two `STREAM` queries
that keep nothing in the table, stdout `UNKNOWN`/`UNKNOWN:` the oracle's, `fix1/p/sn`). Control C7,
prediction in `ctl/c7-prediction.txt`: without the `push_temp` the test reddens. **Confirmed**
(`0 passed; 1 failed`); restored and `cmp`-checked.

**Cause B, a `CALL ON` handler's condition object** (`condition_object`). Reduced to
`st/src/co12.rex`: `call on error name onerror` / a failing command / `onerror: say '...'`, `call
report` / `report: o = condition('O')`, `say o~class~id`. Without the `say` before `call report`, or
with `condition('O')` read in the handler itself, it runs. The trap queue roots a pending trap's
object (`object_roots` destructures `PendingTrap`); delivery moves it into the handler activation's
`TrappedCondition`, and a callee inherits a copy. `Activation::object_roots` names that field, but
`Interp::object_roots` walks only each running and suspended activation's `context_object`, and the
full per-activation walk runs only for a parked `REPLY`. Fix: `Interp::object_roots` also walks each
running and suspended activation's `condition.object`. Local: the same shape as the `context_object`
walk beside it.

Witness: `condition::tests::a_call_on_handlers_condition_object_survives_a_collection_at_every_allocation`
over `co12`, stdout the oracle's (`fix1/p/co12`). An earlier form printing `o~at('CONDITION')` as
well passed on `034c1c7d7`'s probe build (`fix1/headtree`), so it could not witness the fix, and was
replaced. Control C6, prediction in `ctl/c6-prediction.txt`: without the walk the handler test
reddens and the SYNTAX test stays green. **Confirmed** (`1 passed; 1 failed`, the handler test);
`lib.rs` restored and `cmp`-checked.

**Not reached in an ordinary run**, two attempts on the `c00d18052` binary: 40,000 `NOTREADY` traps
(`linein` of a missing file under `call on notready`) whose handler calls a routine reading
`condition('O')`, with and without a varying count of allocations per iteration before the read and
a `say` in the handler (`fix1/p/cond1`); both rc 0 and identical to the oracle.

**The L0 test is green.** `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress`
with both causes fixed: first run (`fix1/l0-b.txt`) no mismatch, and the only failure the
zero-collection list assertion. The committed `NO_ALLOCATION_PROGRAMS` no longer matched: seventeen
programs performing no collection were missing from it, every one added to the corpus after the
list was last checked (sixteen Phase 8 `library_*` programs, including this task's
`library_native_refusal_stem_routine.rex`, and `trace_debug_skip.rex`, each dated by `git log
--diff-filter=A`), and `class_rexx_defined_library_no_mutation.rex` now collects: its `signal on
syntax` builds a condition object, and it collects 29 times on `034c1c7d7`'s probe build as well, so
it left the set before this round. The list is now the observed set, its doc stating what it is.
Second run (`fix1/l0-c.txt`): **exit 0, 32 passed**, the L0 test among them, for the first time
since before this phase. That run still carried the withdrawn stream witness; the gate at this
round's commit re-runs it without.

Control C8 (the exclusions entry "NO RUNNING GATE SEES THE IMPORTED LIBRARY ROUTINE OBJECT'S
ROOT" claimed the L0 test stops first), prediction in `ctl/c8-prediction.txt`: with
`library_routine_object`'s `add_global` deleted, the L0 test names
`library_routine_imported.rex`. **Confirmed**: exactly that program (plain exit 0, stress rc 120),
`fix1/ctl/c8.txt`; `environment.rs` restored and `cmp`-checked. That entry is now closed.

Exclusions: the "L0 TEST STOPS" entry closed naming the causes; the imported-routine-root entry
closed with C8; the entry `034c1c7d7` added is rewritten as the three causes, each with its
ordinary-run result and witness; the POSITION/PROGRAM entry says the review ran base (pre-existing)
and points at its routine half.

### Commit `2d155158c` and its gates

Before it: `cargo fmt --all --check` exit 0, `cargo clippy -j 4 --workspace --all-targets -- -D
warnings` exit 0 (the scratch example removed first). Gates from `fix1/gates-a/run.sh`,
`CARGO_TARGET_DIR=fix1/target-a`, status first line `2d155158c`, `git status --porcelain` empty
before and after:
* `memcap 16G cargo test -j 4 --release -p rexx-exec -p rexx-parse --no-fail-fast`: **exit 0**, 62
  `test result: ok` lines, `the_l0_subset_passes_again_under_collect_on_every_allocation ... ok`.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  `604 of 604 matching`.

## Item 3: `logical_t`'s `found` is the string the conversion answered

Oracle, measured (`fix1/p/add1`, `p/nostr`, `p/G`): `truthValue`
(`interpreter/classes/StringClass.cpp:1475-1492`) reports the string it tested, which is what
`requestString` answered, not the argument. `t~logical(.array~of(1, 2))` finds `"1\n2"`, an instance
whose `STRING` answers `viaString` finds `"viaString"`, one whose `MAKESTRING` answers `viaMake`
finds `"viaMake"`, and an instance whose `STRING` answers `.nil` finds `"The NIL object"`. Every
other row's `found` is the argument's own `stringValue()`: the same `.S~new(...)` under
`t~int`/`t~wholenumber`/`t~double`/`t~positive` finds `"a S"`.

Built: `Failure::NotLogical` carries `found: ObjRef`, `Host::logical` answers
`Result<Result<bool, ObjRef>, Raised>` -- the truth value, or the object that was tested -- and
`logical_to_native` passes that object through. `rexx-exec`'s impl answers `Err(text)` with the
string the conversion produced (and `Err(object)` for the small-integer short circuit, where no
conversion runs). The three stand-in hosts were updated; the `rexx-api` logical test now pushes a
separate `tested` object and asserts `Failure::NotLogical { found: tested }`.

Witnesses in `corpus/lang/library_native_object_arguments.rex`: the four oracle cases above, plus
`t~logical(.D~new)` and `t~logical(d)` for item 4. Its header said "what a refusal finds is the
argument's own string value"; corrected, and the companion regenerated with the sanctioned
`.Package~new` driver (`count 172`).

Controls, predictions written first (`fix1/ctl2/predictions.txt`):
* C1, `rexx-exec`'s `logical` answering the argument instead of the converted string: corpus half
  **confirmed** (exactly the four lines, found `"an Array"`, `"a S"`, `"a M"`, `"a S"`), unit half
  **falsified as written** -- C1 perturbs the host, and the `rexx-api` test drives a stand-in, so it
  cannot see it.
* C1b, the row reporting `argument` instead of the host's `found`: **confirmed** on both halves
  (`the_logical_row_converts_zero_and_one_and_any_non_zero_back_as_one` red, `found: ObjRef(8)`
  against `ObjRef(12)`; the same four corpus lines).

Not fixed, recorded instead: where the `STRING` method answers something that is not a string, the
oracle converts that answer again (`requestString`, `ObjectClass.cpp:1256-1288`) and this crate takes
the answer's own string value, so `t~logical(.S~new(.array~of(1,2)))` finds `"an Array"` here against
`"1\n2"` there. Pre-existing, reachable with no extension loaded (`say '[' || y || ']'` is
`[an Array]` here and `[1\n2]` on the oracle, identical at the Task 3 base `5d84dd8cb`), and the
string conversion protocol is not this phase's surface. Exclusions entry added with the transcript.

## Item 4: a native refusal's `found` runs the `stringValue` protocol

The shared helper was **not** changed. `Interp::string_value_text` is reached from paths that must
not send a message (the value model's own renderings), so the native refusal got its own call,
`Interp::native_found`, and every `found` insert in `Interp::refusal` goes through it. It sends
`OBJECTNAME` for a `Body::Instance`, which runs a class's own `DEFAULTNAME`, and falls back to the
default name where the send raises, as message substitution does
(`interpreter/concurrency/Activity.cpp:1300-1306`).

Measured regression during the round, and the reason for the guard: a `MutableBuffer` is an instance
and its `stringValue` is its contents, so sending made `t~int(.mutablebuffer~new('buf'))` report
`found "a MutableBuffer"` where the oracle reports `found "buf"` (`fix1/p/found2`). `native_found`
now sends only where `Interp::to_text` would answer a default name -- not for an instance carrying
buffer or pointer state, which have their own `stringValue`.

Witnesses in the same corpus program: `t~int(.D~new)` and `t~logical(.D~new)` over a class defining
`::method defaultname`, found `"a named thing"`; `t~int(d)` and `t~logical(d)` after
`d~objectName = 'named'`, found `"named"`; `t~int(.mutablebuffer~new('buf'))`, found `"buf"`.
The oracle agrees on all three descriptors.

Control C2, `native_found` never sending: `t~int(.D~new)` **confirmed** (found `"a D"`). The
predicted second half, `t~logical(.D~new)`, was **falsified**: a logical's `found` is the string the
conversion answered, and the conversion runs `DEFAULTNAME` itself, so that line never reaches
`native_found`. Nothing else moved.

The reviewer's `rb8` probe (a `DEFAULTNAME` counting its own runs) is identical on both sides after
this, so the count matches the oracle's.

## Item 5: glibc's `(nil)` pointer string

Measured with `sscanf(text, "0x%p", &p)` directly (`fix1/c/nil2.c`, glibc): `0x(nil)`, `0x (nil)`,
`0x\t(nil)`, `0x(NIL)`, `0x(Nil)`, `0x(nIL)`, `0x(nil)zz`, `0x(nil)(nil)`, `0x0x(nil)` and
`0x0X(nil)` read a null pointer; `0x-(nil)`, `0x+(nil)`, `0x-0x(nil)`, `0x(nil`, `0x(nill)`,
`0x(ni)`, `0x( nil)`, `0x(nil )` and `0x0x (nil)` convert nothing. The two forms the review asked to
check first: `0x0X(nil)` reads null, `0x0x (nil)` does not.

`values::pointer_string` accepts the spelling case-insensitively after the optional whitespace run,
only where no sign was consumed, and once more after an inner `0x`/`0X` prefix that no hex digit
follows. Unit cases added for each form above; corpus lines added through
`TestPointerStringArg`. Oracle and crate agree.

Control C3, both acceptance arms deleted: **confirmed** on both halves -- the `rexx-api` pointer
test red, and the six accepted corpus lines print `88.919` while the four refused ones do not move.

## Item 6: the minors

* `RESULT_DIGITS`'s comment cited `TestDoubleArg(2/3)`, whose argument is already rounded by the
  division. Measured and rewritten: `TestDoubleArg('0.6666666666666666')` is `0.666666667` under
  `NUMERIC DIGITS 5` and under `20` (`fix1/p/dig2`, identical on both sides).
* `int_from_native` was left building the tagged value directly rather than going through
  `Host::whole_number`. Routing it through the seam makes
  `a_collection_inside_the_call_leaves_the_cself_intact` see a second collection, because the
  stand-in's hook fires on every `whole_number` call; the row's value is a tagged small integer that
  needs no host allocation, so the direct construction stays, with the reason at the site.
* `corpus/phase-8.txt`'s "every conversion row" was false: it now says the rows a shipped extension
  declares, and names the unit tests as the witness for the rest.
* The `MutableBuffer` and `VariableReference` 88.914 rows are now corpus-witnessed
  (`t~bufferlength('abc')`, `t~refvalue('abc')`, `t~refvalue(.object~new)`). Their adjacent
  successes are **not** there: `TestMutableBufferLength` and `TestVariableReferenceValue` on a real
  instance reach `MutableBufferLength` and `VariableReferenceValue`, which are Phase 8 loud refusals
  (rc 120), so only the refusal half can run today.
* `refusal_sites.rs`'s `SHARED_ANSWERS` comment called `not_logical` "a method's own logical
  argument"; its send-surface witness in `refusal-sites.tsv` is a string method's receiver
  (`.String~new("abc")~"?"("y", "n")`), and the comment now says so.
* The three "emptied" comments (`lib.rs`'s `native_spares` field and its `object_roots`
  destructuring, `library.rs`'s `pop_native_frame`) said more than the code does: a pop clears the
  buffers that held objects and leaves the handle fields to be overwritten on the next push. Stated
  that way now.
* `NO_ALLOCATION_PROGRAMS`'s doc now says how to re-derive it: run the test binary, whose assertion
  prints the observed set beside the list whenever they differ.
* The exclusions POSITION/PROGRAM entry already said pre-existing per the reviewer's base run.

## Item 7: recorded, not fixed

The host review's Minor 6, with its transcript: a `PROCEDURE` with `signal on syntax` does not trap
a `raise syntax 40.1` from inside a `DEFAULTNAME` reached by `v~string`. Oracle rc 0, `trapped 40.1`
then `back`; this crate rc 216 with a traceback through `OBJECTNAME` and `STRING`, and nothing on
stdout. Measured at the Task 3 base `5d84dd8cb`, at `034c1c7d7` and at this round's build, with no
extension loaded (`fix1/p/rb7`). The same shape is why the reviewer's `found2` probe ends at rc 216
where the oracle prints `40.1 | External routine "&1" failed.` for each row whose `found` runs such a
`DEFAULTNAME`. Recorded as NO OWNER: it is a condition delivered out of a generated method's send.

## Checks before the fix round's second commit

Run on the tree as committed, each status read unpiped:
* `cargo fmt --all --check`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
* `cargo test --release -p rexx-api -p rexx-parse`: all binaries `ok`, no failures.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --no-fail-fast`: every binary `ok` except
  `gate_table_c`, which under that environment enforces closed-phase rows and reports `82 row(s) of
  gate table C owned by a closing or closed phase do not agree`: `File` instance 50, `Stream`
  instance 24 and `StreamSupplier` instance 8, all Phase 7 and all `unanswered`. `Alarm` and
  `Ticker` are `unanswered` too and are Phase 6, which that check does not gate. **Pre-existing and not
  this change**: a pristine `git archive HEAD` tree at `2d155158c` built in its own target dir
  reports the identical 82 rows (`fix1/head3-gtc.txt`). Without that environment the binary is in
  report mode and passes (`22 passed`), which is how the round's gate runs it.
  `collect_stress` in that run: **32 passed**, `the_l0_subset_passes_again_under_collect_on_every_allocation`
  among them. `corpus`: `29 passed`.
* Miri, `rexx-api` changed: `MIRIFLAGS=-Zmiri-strict-provenance rustup run nightly cargo miri test
  -p rexx-api --lib --locked`: exit 0, `27 passed`.

**The L0 stress test is green from `2d155158c`** -- the commit that rooted the condition object's
entries, the stream name and the handler's condition -- and stays green here.

### Commit `6c96144d8` and its gates

Items 3 to 7 committed as `6c96144d8`. Gates from `fix1/gates-b/run.sh`,
`CARGO_TARGET_DIR=fix1/target-b`, status file's first line `6c96144d8`, `git status --porcelain`
empty before and after, `git rev-parse` unchanged after:
* `memcap 16G cargo test -j 4 --release -p rexx-exec -p rexx-parse -p rexx-api --no-fail-fast`:
  **exit 0**, 71 `test result: ok` lines, no failures, with
  `the_l0_subset_passes_again_under_collect_on_every_allocation ... ok`.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: **exit 0**,
  `604 of 604 matching`, `29 passed`.

# Fix round 2

BASE `6c96144d8`, brief `task-3-fix2-brief.md`, re-review `task-3-fix1-rereview.md`. Nothing here
changes an observable, and the one code change was measured to prove it.

**Item 5, the duplicated rule (the only code change).** `NativeState::renders_its_own_string_value`
answers "does the object render its string value from the state itself" by a match over the
variants, so a state added to the enum is a compile error at the decision. `native_found`
(`dispatch/library.rs`) and `Interp::redirect_of` (`value.rs`) both call it, in place of the two
`buffer()`/`pointer()` spellings that agreed by inspection only. Witnesses before and after, built
in the same target directory: `corpus/lang/library_native_object_arguments.rex` through the probe
harness, **byte-identical on stdout, stderr and exit status** across the change, and identical to
the oracle on both sides of it; `cargo test -p rexx-api --test values`, `83 passed` before and
after.

**Item 1**, `dispatch/library.rs`'s refusal-match head comment now names `NotLogical` as the arm
where `found` is the string conversion, `"1\n2"` for the array the comment uses as its example.

**Item 2**, the exclusions name the corpus programs that run unchanged under
collect-on-every-allocation (`lang/library_native_integer_arguments.rex`,
`lang/library_native_object_arguments.rex`, `lang/library_native_special_arguments.rex`, and Task
2's `lang/library_routine_argument_errors.rex`) instead of counting them and calling them in-crate.

**Item 3**, `corpus/phase-8.txt` no longer splits the rows by who declares them. It says a row is
witnessed there as far as its entry point runs, and names `TestMutableBufferLength` and
`TestVariableReferenceValue` as the case where that is the 88.914 refusal alone.

**Item 4**, the condition walk's comment drops "and nothing else does".

**Item 6**, measured before writing. Control C9, prediction in `fix1/ctl3/prediction.txt`: with the
stream name's `push_temp` removed, the L0 test names exactly `address_with_stream`,
`executable_context`, `sys_file_functions`, `security_manager` and `call_miss_not_cached`, each
stress run rc 120 against a plain run that matches, and no other program. **Confirmed**, all four
parts (`fix1/ctl3/c9.txt`: `0 passed; 1 failed`, five `lang/` lines, each `stress exit=120`);
`run.rs` restored and `git diff` empty for it. The exclusions entry now records that witness beside
the unit test.

**Item 7**, `value.rs`'s superseded comment head ("A buffer's own body holds the text, named or
not.") deleted; the surviving one is the one the arm justifies. A scan of the round's other touched
files for a second head on one arm found only aligned tables, no other instance.

**Item 8**, the report's `gate_table_c` sentence corrected: `File` instance 50, `Stream` instance 24
and `StreamSupplier` instance 8, all Phase 7; `Alarm` and `Ticker` are Phase 6 and that check does
not gate them.

### Commit `a442e0b5b` and its gates

Before it: `cargo fmt --all --check` exit 0, `cargo clippy -j 4 --workspace --all-targets -- -D
warnings` exit 0. Gates from `fix1/gates-c/run.sh`, `CARGO_TARGET_DIR=fix1/target-c`, status file's
first line `a442e0b5b`, `git status --porcelain` empty before and after, `git rev-parse` unchanged:
* `memcap 16G cargo test -j 4 --release -p rexx-exec -p rexx-parse -p rexx-api -p rexx-core
  --no-fail-fast`: **exit 0**, 82 `test result: ok` lines, no failures, with
  `the_l0_subset_passes_again_under_collect_on_every_allocation ... ok`.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: **exit 0**,
  `604 of 604 matching`, `29 passed`.
