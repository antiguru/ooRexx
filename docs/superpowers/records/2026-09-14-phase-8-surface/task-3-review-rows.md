# Task 3 review, rows slice (`rexx-api` values/invoke/load/ffi and `rexx-num` int64)

Reviewer scope: `rust/crates/rexx-api/src/{values,invoke,load,ffi}.rs`, their tests, and
`rexx-num`'s `Number::int64_value`/`unsigned_int64_value`. Diff `5d84dd8cb..034c1c7d7`, read in three
passes: (1) `values.rs` whole file at head against `NativeActivation::processArguments` (`:219-682`)
and `valueToObject` (`:718-861`) plus the helpers they call (`:1901-2120`, `:2745-2860`), (2) the
`invoke.rs`/`load.rs`/`ffi.rs` hunks and the three test files, (3) `rexx-num/src/lib.rs`'s hunk
against `NumberStringClass.cpp:863-918`, `:937-1000`, `:1024-1198`, and `tests/int64.rs`. The
report's claims were treated as unverified; each row below says what was run.

Scratch: `scratchpad/t3-review-a/` (probes under `p/<name>/`, run by `probe.sh`: oracle and crate
from a fresh copy of `src/` at the same path, three descriptors to files; forged library
`ext/libforge4.so` from `ext/forge4.cpp`, `readelf -d` NEEDED count 0; C check `c/nil.c`; control
outputs `ctl-M*-{values,lib}.txt`, `miri-clean.txt`, `miri-M0.txt`, `stable-M0.txt`). Target
directories and the two `git archive` copies were deleted by path at the end.

## Measured, in the order run

### Header coverage (read, then the test's own derivation)

`api/oorexxapi.h:55-98` defines the `REXX_VALUE_` codes 2-7 and 11-40 (aliases excluded).
`values::TABLE` (`values.rs:905-1159`) has one row per code and none twice;
`tests/values.rs:352` derives the same set from the header and compares as a map. Every row's
`to_native`/`from_native` pair was read against its C++ case; the table under Spec Compliance
records the result per row. Every `NativeActivation.cpp` line the diff cites was printed and lands
on its subject (`awk` over the cited numbers), as do the `Numerics`, `NumberStringClass`,
`StringClass`, `ObjectClass`, `MethodArguments` and header citations.

### Probe L (`p/L`): `logical_t` on NumberString-valued and STRING-protocol arguments. DIFF, two lines.

Agree: `0.5 - 0.5` (a NumberString `0.0`, `isZero`) answers 0 where the literal `'0.0'` is 34.901;
`1.5 - 0.5` (`1.0`) is 34.901; `.S~new('1')` (class with `string`, no `makestring`) converts for
`logical_t`, `int`, `uint64`, `double`, `positive`, `nonneg` and a routine's stem name, and is
88.909 for `CSTRING`/`RexxStringObject`, 98.913 for an array, 88.919 for `POINTERSTRING`; `.nil`,
`.true`, `.false`, `'1 '`, `' 0'`, `'01'`, `''`, `.array~of(1)`, `.list~of(1)`.

**Divergence:** `t~logical(.S~new('2'))` and `(.S~new('x'))`: oracle `34.901 ... found "2".` /
`found "x".`; crate `found "a S".` (rc 0 under the trap; stdout differs).
`RexxString::truthValue` (`StringClass.cpp:1475-1497`) reports the string `requestString()`
produced, not the argument; `Failure::NotLogical { argument }` (`values.rs:126`, built at `:1566`)
carries the argument, which the host renders.

### Probe N (`p/N`): NUMERIC DIGITS 3/5/12/40 and FORM ENGINEERING around the integer, double and float rows. SAME, rc 0.

A 20-digit value built under `digits 40` converting for `uint64` and refused for `int64` naming
it; `123456 + 0` at digits 3 converting as `123000` (the arithmetic rounded it, both sides);
`double`/`float` results at 9 digits and scientific form under `FORM ENGINEERING`; denormals
(`-1e-320` is `-9.99988867E-321`), `-1e-400` is `0`, `1.7976931348623159E308` is `+infinity`,
`1e9` is `1.00000000E+9`, `123456789.5` is `123456790`.

### Probes O and D (`p/O`, `p/D`): `::OPTIONS DIGITS`, routine against method. SAME, rc 0.

`::OPTIONS DIGITS 20` in the caller, `::OPTIONS DIGITS 5` in a required package whose method makes
the call, `NUMERIC DIGITS 1`, `3`, `5`, `20`, an aliased `::ROUTINE`, `CALL`, `loadExternalRoutine`:
`TestDoubleArg('0.6666666666666666')` is `0.666666667` on every path, routine and method alike,
`1e31` stays `1E+31` under `FORM ENGINEERING`, and `RxCalcSqrt('2')` at digits 1 is `1`. The
constant `RESULT_DIGITS = 9` (`values.rs:227`) holds on every path reachable here. Read, not the
row's evidence: `Activity::updateFrameMarkers` (`Activity.cpp:1568-1590`) takes the top frame's
settings and `NativeActivation::getNumericSettings` (`NativeActivation.cpp:2253`) answers the
defaults unless `activation` is set.

### Probe E (`p/E`): integer edges. SAME, rc 0.

Digit counts to 41; `1E25`, `1E-25`, `1230E-1`, `12.30`, `6.5535E4`, `184467440737095516.15E2`
(converts to `u64::MAX`), `-9.223372036854775808E18`; the rounding at the twentieth digit (`-0.` +
19/20/21 nines: refused, refused, `1`; `1.` + 40 zeros + `5` is `1`; `0.` + 30 zeros + `1` refused);
`9223372036854775807.5` refused; `18446744073709551615.4` converting and `.5` refused; `127.5`,
`-128.4`, `-128.5` for `int8` all refused; `0x10`; a NUL inside the string (rendered in `found` as
the byte, both sides); tab inside and before; `'+ 5'` and `'- 5'` converting; `'--12'` refused;
`'-'`, `'+'`, `'1E'`, `'E1'` refused; the 88.907 text for every width and for
`size_t`/`ssize_t`/`stringsize_t`/`wholenumber_t`; 88.905 and 88.904 for `-0`, `0.0`, `0.5`.

### Probe F (`p/F`, forged `libforge4.so`, `LIB` naming the oracle's lib and `ext/`). SAME, rc 0.

* **Risk 4 answered.** `F4EchoPS` (`POINTERSTRING` in and out) makes the value observable: `0x1` +
  16 zeros, `0x1` + 17 zeros, 17 and 40 `f`s all answer `0xffffffffffffffff`; `0x-1` and `0x-1` + 16
  zeros answer `0xffffffffffffffff`; `0x-0` is `0x0`; `0x-8000000000000000` is
  `0x8000000000000000`; `0x` + 17 `f`s + `g` is `0xffffffffffffffff`; `0x 1` is `0x1`; `' 0x1'` is
  88.919. `F4PSAsUint` reads them back as `18446744073709551615`. The `strtoul` reading in
  `values::pointer_string` (`values.rs:1779-1782`) is the oracle's, measured.
* `MutableBuffer` and `VariableReference` echoed through a routine (`F4EchoBuffer`,
  `F4EchoRef(>v)`): the same object back, `r~name r~value` `V 7`. Class echo. A `CSTRING` with an
  embedded NUL (`ab`, length 2); null `CSTRING`, `RexxObjectPtr` and `RexxStringObject` results
  leave `RESULT` unset.
* Every width's extreme as a result: `size_t`/`uintptr_t`/`stringsize_t` `18446744073709551615`,
  `ssize_t`/`int64_t` `-9223372036854775808`, `wholenumber_t` `9223372036854775807` and its
  negative (past `MAX_WHOLENUMBER`, rendered as digits), `int` `-2147483648`; arithmetic on them
  (`1.84467441E+19`); `NUMERIC DIGITS 3` leaves them as digits.
* `logical_t` results 5, -1 and `(logical_t)-1` answer `1`. `double` results: `-0.0` is `0`, NaN is
  `nan`, `+infinity`/`-infinity`; a `float` `-0.0` is `0`, `(float)16777217` is `16777216`.
* Positions beside special parameters: `int, NAME, int` refuses argument 2 for the second int;
  `CSELF, int, uint8_t` refuses argument 2; `ARGLIST, int` refuses argument 1; a missing second is
  `88.901 argument 2`; `ARGLIST` lifts 88.922 in a method and a routine; without it `1 expected`.

### Probe G (`p/G`): eighteen against twenty digits; glibc's `(nil)`; `logical_t` `found`. DIFF.

* Values where `numberValue` at 18 digits and `int64Value` at 20 would answer differently
  (`12345678901234567.995`, `.005`, `99999999999999999.995`, `123456789012345678.95`,
  `12345678901234567.895`, `1.995`) through `positive`, `nonneg`, `wholenumber`, `int64`, `uint64`,
  `stringsize`, `ssize`: every one refused on both sides, identical text.
* **Divergence:** glibc's `%p` accepts `(nil)` (checked in `c/nil.c`, glibc 2.43: `0x(nil)`,
  `0x (nil)`, `0x(nil)zz`, `0x(NIL)`, `0x0x(nil)` read as NULL with one conversion; `0x-(nil)`,
  `0x+(nil)`, `0x(nil` do not). Oracle through `orxmethod`'s `TestPointerStringArg`: those five
  answer `0` (converted, not the address); crate: `88.919` for each. `values::pointer_string`
  (`values.rs:1740`) has no `(nil)` arm.
* **Divergence, wider than L showed:** `t~logical(.M~new('2'))` with `M` defining `makestring`:
  oracle `found "2"`, crate `found "a M"`; `.M~new('')` `found ""` against `"a M"`; `.S~new(.nil)`
  and a class whose `string` answers `.nil`: oracle `found "The NIL object"`, crate `"a S"` /
  `"a N"`. Any object reaching `logical_t` through `MAKESTRING` or `STRING` and answering a
  non-logical string prints a different `found` here.

### Probe G2 (`p/G2`): concern 6 against the base binary.

The base host (`git show 5d84dd8cb:rust/crates/rexx-exec/src/dispatch/library.rs:390`) converted
`positive_wholenumber_t` through `whole_value(SIZE_DIGITS)`, twenty digits, not eighteen. So the
question is `numberValue` against `int64Value` at the same precision, filtered to
`1..=MAX_WHOLENUMBER`. Run against `surface-t3/perf/rexx-run-base` (the revision before the task):
the five values above through `positive` are refused identically to the oracle (the run then
stops at base's unfilled result row, rc 120); head is identical to the oracle on all of G2.
Read: a wrap that `wrapping_unsigned_value` misses answers at least `2^64 / 9`, past
`MAX_WHOLENUMBER`, and the carry-to-one quirk is in both functions, so the filter leaves no
observable difference. Concern 6's conclusion holds; its premise ("18 digits") does not.

### Miri, Stacked Borrows (`MIRIFLAGS` unset), `git archive 034c1c7d7 rust interpreter api` copy

`RUSTUP_HOME=scratchpad/final-fix/rustup-home rustup run nightly cargo miri test -p rexx-api --lib
--locked`: `27 passed; 0 failed; 3 ignored` (the process spawner and the two that open the running
image), exit 0. `invoke::tests::a_cstring_result_is_copied_up_to_its_first_nul` among the passes.

## Controls: predictions, then what ran

**M0 (Miri), `load.rs` `call_stub`:** `CStr::from_ptr(pointer).to_bytes().to_vec()` replaced by
`std::slice::from_raw_parts(pointer.cast::<u8>(), 16).to_vec()`; `STUB_TEXT` is 15 bytes.
Prediction: Miri fails `a_cstring_result_is_copied_up_to_its_first_nul` with an out-of-bounds
access on `STUB_TEXT`'s allocation at that `load.rs` line, every other lib test unchanged; stable
keeps the test green (adjacent static memory, and the split at the first NUL).
**Confirmed:** Miri `Undefined Behavior: constructing invalid value of type &[u8]: encountered a
dangling reference (going beyond the bounds of its allocation)` at `load.rs:356:38`, backtrace
through `invoke::run` (`:162`) into that test, exit 1; stable `1 passed; 0 failed`, exit 0.

**M1, delete the `int8_t` row.** Prediction: `tests/values.rs` red exactly
`the_table_has_a_row_for_every_code_the_header_defines` and
`the_int8_row_converts_its_range_both_ways`; lib green. **Confirmed:** `81 passed; 2 failed`, those
two; lib `30 passed`.

**M2, delete the `SUPER` row.** Prediction: red exactly `the_table_has_a_row...`,
`the_special_rows_are_the_ones_that_consume_no_argument`,
`the_result_of_exactly_the_special_rows_is_refused`,
`the_super_row_hands_a_method_the_scope_above_its_own`; lib green. **Confirmed:** `79 passed;
4 failed`, those four; lib `30 passed`.

**M3 (concern 1, wrong class), `mutable_buffer_to_native` asks for `Class::VariableReference`.**
Prediction: red exactly `the_mutable_buffer_row_takes_only_a_mutable_buffer`. **Confirmed:**
`82 passed; 1 failed`; lib `30 passed`.

**M4 (concern 1, wrong object), `instance` registers `ObjRef::NIL`.** Prediction: red exactly the
three instance-row tests. **Confirmed:** `80 passed; 3 failed`, `the_class_row_takes_only_a_class`,
`the_mutable_buffer_row_takes_only_a_mutable_buffer`,
`the_variable_reference_row_takes_only_a_variable_reference`; lib `30 passed`.

So: no row's deletion goes unnoticed, because `tests/values.rs:352` derives the row set from the
header rather than from the table; and the `MutableBuffer`/`VariableReference` unit tests redden
for a wrong class and for a wrong object, not only for a missing row. Their success paths are
now also oracle-measured (Probe F).

`cargo test -p rexx-num --test int64` from the archive copy: `7 passed`.

## Spec Compliance

| Item | Verdict | Where / what ran |
|---|---|---|
| Every header code has one row, none double | ✅ | `values.rs:905-1159`; `tests/values.rs:352` header-derived; M1, M2 |
| `OSELF`/`SCOPE`/`SUPER`/`CSELF` method-only, 40.918 in a routine | ✅ | `values.rs:1300-1350` vs `:251-304`; corpus `library_native_special_arguments`; F |
| `ARGLIST` cached array, lifts the too-many check (S2, routines too) | ✅ | `values.rs:1280`, `:1201`, `invoke.rs:120-142` vs `:306-312`, `:680`; F (`F4IntThenArglist`, `F4ArglistFirst`, `F4MArglistInt`) |
| `NAME` as sent / as called | ✅ | `values.rs:1290` vs `:317`; corpus; F (`F4NameMid`) |
| `int`, `int8/16/32`, `ssize_t`, `intptr_t`, `wholenumber_t` ranges and 88.907 text | ✅ | `values.rs:1361-1469` vs `:341-380`, `:427`, `:1901-1912`; E |
| `int64_t` via `objectToInt64` (no range check past `int64Value`) | ✅ | `:1442-1450` vs `:366`, `Numerics.cpp:420-440`; E, N |
| `uint8/16/32`, `size_t`, `uintptr_t`, `stringsize_t`, `uint64_t` | ✅ | `:1471-1527` vs `:384-416`, `:448`, `:1966`; E, N |
| `positive`/`nonnegative_wholenumber_t`, 88.905/88.904 | ✅ | `:1529-1555` vs `:434-445`, `:1922-1955`; E, G, G2 |
| `logical_t` via `truthValue`, 34.901 | ✅ value, ❌ `found` | `:1557-1567` vs `:420`; L, G |
| `double`/`float` argument, 88.921; `(float)` cast | ✅ | `:1569-1596` vs `:454-465`; N, D |
| `CSTRING`/`RexxStringObject` via `requiredString`, 88.909 | ✅ | `:1598-1629` vs `:467-486`; L |
| `RexxArrayObject` via `requestArray` + dimension check, 98.913 | ✅ | `:1631-1642` vs `:488-500`, `MethodArguments.hpp:675`; corpus, L |
| `RexxStemObject`: stem anywhere, name in a call, 93.969/40.919 | ✅ | `:1644-1661` vs `:502-547`, `:2835`; corpus, L |
| `RexxClassObject`, `MutableBuffer`, `VariableReference`, 88.914 naming the class | ✅ | `:1663-1701` vs `:549-598`; corpus, F |
| `POINTER` unwrap, 88.914 `Pointer` | ✅ | `:1703-1716` vs `:560-569`, `:2113`; corpus |
| `POINTERSTRING` as `sscanf("0x%p")` over `stringValue()` | ✅ digits/sign/ws/overflow, ❌ `(nil)` | `:1718-1784` vs `:2049-2062`; corpus, F, G, `c/nil.c` |
| Absent optional: zero for the C++'s list, signature for `VariableReference` and unknown codes; required absent 88.901 first | ✅ | `:1210-1251`, `:1147-1158` vs `:600-663`; `tests/values.rs:769-807` |
| Unknown code with an argument: signature error | ✅ | `:1240` vs `:600-604`; `tests/values.rs:518` |
| Results: object codes (`OREF_NULL` for null), all integer widths, `logical_t` `!= 0`, `POINTER`, `POINTERSTRING` `0x0`, `CSTRING` null/NUL-cut | ✅ | `:1786-1967` vs `:722-846`; F, corpus `library_native_results` |
| Results: `double`/`float` at nine digits whatever `NUMERIC`/`::OPTIONS DIGITS` | ✅ | `:227`, `:1933-1950` vs `:815-823`, `Numerics.hpp:202`; N, O, D |
| Special codes as result: `ResultSignature` (93.968 / 40.918), after the call | ✅ | `:848-854`, `:1273-1276` vs `:855-858`; `Failure::error_number` `:186-188`; `tests/values.rs:1288`; lib test `a_special_code_as_the_result_is_refused` |
| Result word read unstripped; type 0 is no object | ✅ | `:1182`, `:1267` vs `:720`, `:848`; `tests/values.rs:549`, `:578` |
| `Failure::Unfilled`/`stub` gone, no doubled suffix from this crate | ✅ | grep of `values.rs`; only `UnfilledSlot` remains |
| `CSTRING` result copied where the union is read, interned into the pool | ✅ | `load.rs:339-360`, `invoke.rs:156-161`, `values.rs:1955-1967`; lib test; Miri; M0 |
| `unsafe` only in `ffi.rs`/`load.rs`, each block with a `SAFETY:` naming the invariant | ✅ | grep; `unsafe_sites.rs:89-91` unchanged; `load.rs:341-355`, `ffi.rs:721`, `:753`, `:790` read |
| Miri Stacked Borrows on the lib tests | ✅ | 27/0/3, exit 0; M0 |
| `int64_value`/`unsigned_int64_value` copy `int64Value`/`unsignedInt64Value` at `DIGITS64` including both quirks | ✅ | `rexx-num/src/lib.rs:228-268`, `:796-861` vs `NumberStringClass.cpp:863-918`, `:1024-1198`; E (`-0.`+21 nines is `1` both, `21000000000000000000` is `2553255926290448384` both), `tests/int64.rs` |
| No third behaviour differs: DIGITS, exponent forms, rounding | ✅ | N (digits 3/5/40), E |
| `positive_wholenumber_t` moved to `int64_value`: difference measured | ⚠️ premise wrong, conclusion holds | G, G2: base used `whole_value(20)`, not 18; no observable difference either way |

## Strengths

* The table is the one switch, and the header-derived coverage test plus the per-row tests make
  every row load-bearing in both directions (M1-M4 all caught exactly as predicted).
* The host primitives are the oracle's own seams (`objectToSignedInteger`, `truthValue`,
  `requestArray`, `getContextStem`, `stringValue`), so the rows read one-to-one against the C++
  cases and every citation I printed lands.
* `rexx-num`'s `int64_value`/`unsigned_int64_value` reproduce `int64Value` step for step, the two
  quirks included, with the C++'s own overflow test rather than a corrected one, and
  `tests/int64.rs` pins both quirks and the controls that show the detected wrap.
* The `CSTRING` result read is exactly where the boundary says it may be, copied before anything
  else runs, and Miri sees the block (M0).
* The `sscanf` reading is measured to an unusual depth (whitespace set, sign, inner prefix, upper
  `X`, overflow both signs), and the forged echo confirms the overflow value itself.

## Issues

### Important

1. **`logical_t` refusal names the argument, the oracle names the string it tested.**
   `values.rs:126` (`NotLogical { argument }`), `:1558-1567`; `Host::logical` `:534-539`.
   `RexxString::truthValue` (`StringClass.cpp:1475-1497`) reports `testString`/`this`, the string
   `requestString()` produced through `MAKESTRING` or `STRING`. Measured (L, G): `.M~new('2')`
   with a `makestring` answering `'2'` is `found "2"` on the oracle and `found "a M"` here; `''`,
   `.nil`-answering `string`, and `.S~new('x')` likewise. Reachable through `orxmethod`'s
   `TestLogicalArg`. Fix: `Host::logical` answers the string object it tested on the refusing path
   (`Result<Result<bool, ObjRef>, Raised>` or a `(Option<bool>, ObjRef)` pair), `NotLogical` carries
   that `found` object, and the host renders `found` from it; extend `tests/values.rs:1740` so the
   stand-in's `logical` answers a different object than the argument and the refusal names it.
   Ran.
2. **`pointer_string` lacks glibc's `(nil)` form.** `values.rs:1740-1784`. glibc's `%p` reads
   `(nil)` as NULL (checked, `c/nil.c`, glibc 2.43): `0x(nil)`, `0x (nil)`, `0x(nil)zz`, `0x(NIL)`,
   `0x0x(nil)` convert; `0x-(nil)`, `0x+(nil)`, `0x(nil` refuse. Oracle through
   `TestPointerStringArg`: `0` for the converting five; crate `88.919` for each (G). Fix: at the
   point where digits are read, with no sign consumed, before and after the inner `0x`, accept
   `(nil)` case-insensitively as `Some(0)`; add the C-checked forms to
   `tests/values.rs:2048` and a corpus line, and re-check `0x0X(nil)` and `0x0x (nil)` against
   `c/nil.c` first (not measured). Ran.

### Minor

3. **`RESULT_DIGITS`'s comment cites a measurement that does not discriminate.**
   `values.rs:224-226`: `TestDoubleArg(2/3)` under `NUMERIC DIGITS 5` is `0.66667` whether the
   result is rendered at 9 or at 5, because the argument was already 5 digits. The discriminating
   figure is `TestDoubleArg('0.6666666666666666')` at digits 5 answering `0.666666667` (D); use
   that one. Inferred from the comment, measured in D.
4. **Report concern 6 misstates the base.** `task-3-report.md` "Task 2's host used `whole_value`"
   with "18 digits": base used `whole_value(SIZE_DIGITS)`, twenty. The conclusion (no observable
   difference) holds for the reason under G2; the record should say so rather than "past 18
   digits". Ran (G2).
5. **`int_from_native` bypasses the host.** `values.rs:1828-1835` builds through
   `ObjRef::small_int(..).expect` where every sibling row goes through `Host::whole_number`;
   harmless (`the_int_row_converts_a_returned_value` pins both ends of `c_int`), one fewer seam if
   it matched. Inferred.

## Assessment

**Task quality (rows slice): Needs fixes.** The table, both directions, the integer semantics, the
result rendering, the special codes and the `CSTRING` copy are right and well instrumented, and
every named risk but one is closed by measurement. Two reachable divergences remain in the new
rows, both through shipped `orxmethod` declarations and both narrow: the `found` text of a
`logical_t` refusal for an object converted through `MAKESTRING`/`STRING`, and the `(nil)`
spelling glibc's `%p` accepts. Each is a small change in `values.rs` plus its host seam, with the
measurements above as the witnesses.
