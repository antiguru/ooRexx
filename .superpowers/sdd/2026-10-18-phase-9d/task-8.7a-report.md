# Phase 9d Task 8.7a report: crash, builtins, collections

Status: DONE_WITH_CONCERNS. Base `81f4bf250` (dispatch commit `d6a9ad129` on top). Code `eb3bea770`,
`e1e4803c0`, `030af45a1`; records, closings and inventory rows `560352131`; this report in the commit
after. The closing lines and inventory rows name `030af45a1`, the last code commit. Records:
`docs/superpowers/records/2026-10-18-phase-9d/t8.7a/` (`probes/` per item, `witnesses/` each
witness's oracle `.o.*`, base `.b.*` and head `.h.*` outputs and `summary.tsv`, `red/` the
collect_stress red run, `suite/`, `checks-*.txt`, `callgrind-*.txt`, `perf-tries/`).

## What changed, per item

**2026-10-28-trace-reference-assignment-unrooted.md (closed).** Under `TRACE I` an intermediate
value's line is delivered through `.TraceOutput`, which runs Rexx clauses that allocate and can
collect, while the value was held only by the caller (`>O>` for `q = >y`: the fresh reference died
before `>>>` rendered it). `trace_intermediate_event` (`eval.rs`) now roots the value in a temps
frame for the duration of its line; the untraced path is unchanged. Witness
`trace_reference_value_rooted.rex` (the item's `tr2` shape, both reference spellings, a `CALL` with a
reference argument, and the loop shape). Red: `red/collect-stress-trace-reference.txt`, the
collect_stress test binary built with the witness and before the fix, panics `dispatch.rs:1831:9: a
live value` (the panic text does not name the program; the witness was the only program added at
that point). Green: `checks-collect-stress.txt`, 74 passed. The item's `tr4.rex` (300000 traced
iterations) panics at base (rc 101) and runs at head; at 30000 and 60000 iterations head takes
6.62 s and 13.63 s (linear; traced speed is no target).

**2026-10-10-beep-wrong-type-message.md (closed).** Rule: `sysBeep` is
`RexxRoutine2(CSTRING, sysBeep, wholenumber_t, Frequency, wholenumber_t, Duration)`
(`runtime/InternalPackage.cpp:174`); the native marshalling converts both arguments first
(`NativeActivation::signedIntegerValue`, `objectToSignedInteger`): an omitted one is 88.901 naming
its position, one that is not a whole number is 88.907 over the whole-number range naming its
position, both reported against the package with no line (`running REXX:`); the body's own range
checks then name `frequency`/`duration` against the program line, with the converted number as the
found value. `beep` now does exactly that (`whole_number`, `in_range` in `builtin/platform.rs`,
through `rexxutil::stems::whole_value`); `'4E1'` and `' 40.0 '` convert, as on the oracle.

**2026-10-10-native-routine-arguments-not-strings.md and 2026-10-14-filespec-string-arg.md (closed).**
Rule: `sysFilespec(CSTRING option, CSTRING name)` and `sysDirectory(OPTIONAL_CSTRING dir)`; a
`CSTRING` argument is `requiredString` (88.909 for a value without a string value, no `STRING`
fallback: measured, an object whose `STRING` answers `'a K'` is 88.909), read up to its first NUL,
and every argument is marshalled before the body runs. `FILESPEC` and `DIRECTORY` now read their
arguments through `rexxutil::required_text`/`required_c_string`/`optional_text` (made `pub(crate)`
with `marshalled` and `required`); `FILESPEC` converts both before its option check, and its 40.904
names the option's first character only (`optionChar`, measured `found "z"` for `'zz'`).
`SysFileExists` and `SysFileTree` already agreed. A plain string argument is now read without the
conversion protocol (`optional_text`'s first arm), which the perf section needed. Witness
`native_routine_argument_marshalling.rex`.

**2026-10-14-class-side-defaultname.md (closed).** Rule: `RexxObject::objectName`
(`classes/ObjectClass.cpp:1696`) answers a set name, else `defaultName()` for an object whose
behaviour is primitive and a `DEFAULTNAME` send otherwise; a class with class methods, or of a
user metaclass, is not primitive. Here a class a program declared (`declared_class`,
`dispatch/object_protocol.rs`: not the language's own, which 98.985 keeps unmodified) now sends
`OBJECTNAME` for its string value and `DEFAULTNAME` for an unset object name, as instances did;
`rexx_classes::ClassRegistry::set_name` answers whether `~objectName=` stored one. A directive
installing `DEFAULTNAME` arms the required-string latch, as `MAKESTRING` does. Witness
`class_side_defaultname.rex` (class method, inheriting subclass, metaclass, plain class, set name).
Residual in Concerns 1.

**2026-10-10-string-translate-empty-tables.md (closed).** Rule: rexxref String `translate`: an
omitted `tablei` defaults to every byte; a supplied one is the input table, so an empty one matches
no byte and the string is unchanged. `RexxString::translate` reads the table only where it was not
supplied (`classes/StringClassMisc.cpp:730`). The String method now passes a supplied empty table
through (`dispatch/string.rs`); the MutableBuffer method keeps reading an empty table as none, which
is its oracle's own test (`MutableBufferClass.cpp:1418`, `getLength() != 0`) and where both sides
already agreed. Witness `string_translate_empty_input_table.rex` (literal tables only; Concerns 3).

**2026-10-10-array-sort-after-remove.md (closed).** Rule: `ArrayClass::stableSortRexx`/
`stableSortWithRexx` (`classes/ArrayClass.cpp:2838`, `:2873`): `count = items()`; `sort` answers at
once for no items and `sortWith` for at most one; the first `count` slots are checked in order (98.975
at the first empty one) and sorted; slots past them are left. `sort.rs`'s `leading_items` and
`sort_by`'s `unsorted` do that. Witness `array_sort_leading_items.rex` (every sort method over
trailing, interior and leading holes, items past `count`, empty arrays).

**2026-10-19-sortwith-on-foreign-class.md (closed).** Three bullets. The empty-array `sortWith`
answers (the rule above). `.OrderedCollection~method('SORTWITH')~source~items`: the oracle's REXX
package methods are restored from the image with no source: no method of the classes measured
(`probes/src/s.rex`: `.OrderedCollection`, `.Collection`, `.MapCollection`, `.Orderable`,
`.Comparable`, `.TraceObject`, `.File`, `.Monitor`, `.Alarm`, `.Ticker`, `.DateTime`, `.TimeSpan`,
`.StreamSupplier`, `.InputStream`, `.Stream`, `.RexxQueue`, `.CaselessComparator`, `.Properties`)
answers a line. `Method~source` of a method from a library program now answers an empty array
(`dispatch/executable.rs`). Witness
`method_source_rexx_package.rex`. The foreign-class `SORTWITH` shape segfaults the oracle: added to
`corpus/oracle-crashes.txt` as entry 62 from the triage's record, not re-run; the crate's loud
refusal is unchanged (Concerns 2).

**2026-10-19-index-argument-string-conversion.md (closed).** Rule: a whole-number or index argument
is `numberValue`/`unsignedNumberValue` of the object, which for a value that is not a string or a
number is `requestString()` first (`classes/ObjectClass.cpp:1047`, `:1074`): an object whose
`STRING` answers `2` is 2, and an Array is its lines (`.array~of(2)` is 2, two items or none are not
numbers). `Interp::requested_number` (`value.rs`, cold) does that after `to_number` fails;
`whole_method_argument`, `whole_index`, `unsigned_index` (now `Result`) and `array_size_argument`
use it. List `section`'s count is `optionalLengthArgument` (93.923, `ListClass.cpp:409`). Witness
`index_argument_string_value.rex` (Queue, Array and List indexes, nested Array subscripts, the
String and MutableBuffer whole-number arguments, `section`, `insert`, `remove`, `.array~new`).

**2026-10-27-string-equals-makestring-argument.md (closed).** Rule: rexxref String `equals`;
`RexxString::primitiveIsEqual` (`classes/StringClass.cpp:674`): `.nil` equals no string, any other
argument is its `requestString()`. `equals_argument` (`dispatch/buffer.rs`) now does that (it
rendered the object, so `'The NIL object'~equals(.nil)` was 1). A reference argument follows its
referenced value, the reading the Deviation 72 "Rule everywhere" ruling gives String method
arguments (`'kmake'~equals(>k)` is 1 here, 0 on the oracle). Witness
`string_equals_string_value.rex`. `lang/string_caseless.rex` now collects under the stress mode
(`'abc'~equals(.array)` requests the class's string), so it left `NO_ALLOCATION_PROGRAMS`
(`e1e4803c0`).

**2026-10-13-error-position-ampersand.md (closed: probe agrees).** Its `queued-probes/imeth/imeth.rex`
answers `0 Method` / `Method` on the oracle, base and head; the 35.1 `"~"` and label inserts agree
(`probes/amp/`).

**gate:phase-9a-gate.md:480 (closed: probe agrees).** `.D~new~instanceMethod('M') == .C~method('M')`:
`queued-probes/dimeth/dimeth.rex` answers 1 on the oracle, base and head. Row in `t8.7a/closed.tsv`.

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file generated by the
test's driver from a copy outside the repository: `trace_reference_value_rooted`,
`native_routine_argument_marshalling`, `class_side_defaultname`, `string_translate_empty_input_table`,
`array_sort_leading_items`, `method_source_rexx_package`, `index_argument_string_value`,
`string_equals_string_value`. `witnesses/summary.tsv`: at base each differs from the oracle (stdout,
or all three descriptors for the marshalling and sort witnesses) except
`trace_reference_value_rooted`, which differs only under the stress mode (`red/`); at head
`030af45a1` each is identical on all three. I read each oracle stdout for the paths it claims: every
BEEP, FILESPEC and DIRECTORY case prints its own code and message (the BEEP successes print the bell
byte and nothing else); the defaultname lines print `DN`, `METADN`, `idn`, `named` and the default
names; translate `[12]` for each supplied empty table and `[  ]` for the buffer; every sort shape
prints its 98.975 position or its items; the source counts `0` for the library and `1` for the
program's own methods; every index case prints its item or its error; equals prints `1 1 0 1`,
`0 0`, `1 1`, `1 0 1`, `1 0 1` and the 93.903 line. The debug binary agrees with the oracle on every
witness, `string_caseless.rex` and two latch-clear probes (`probes/dn/clsdn.rex`,
`probes/idx/i2.rex`).

## Perf

`callgrind-final.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7a/final2 base=.../rexx-base-81f4bf250
head=.../rexx-head-030af45a1`, every program, 3 rounds, exit 0, no OVER or SPREAD flag. Libc
subtracted, every program is at most +0.0017% (startup); heapshape -0.7867%, intcall -3.1204%, churn
-0.0948%; pingguard +0.0001%, pingsem +0.0002%, rexxcps +0.0000%. Whole process at most +0.1064%
(alloc, 0.0000% libc subtracted). The base was built from a `git archive` of `81f4bf250` in its own
target dir.

The first full run, at `eb3bea770` (`callgrind-eb3bea770.txt`, exit 1), was OVER on intcall
(+12.76% libc subtracted) and over budget on heapshape (+0.31%): FILESPEC's arguments went through
the conversion protocol, and `whole_index`'s `Result` return cost seven instructions per Array
subscript. `030af45a1` reads a plain string argument directly and keeps the small-integer index on
the infallible path (`perf-tries/`: try2 the index path, try3 to try5 the argument reads; try4's
single forward scan for the NUL and the last `/` was slower than `rposition` plus `contains`).

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87a-target`, at
`030af45a1` unless named; records in `t8.7a/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-classes --all-targets -- -D warnings` | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage` (`checks-touched.txt`) | 0; lib 1200 passed |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed. At `eb3bea770` 101: the zero-collection list drift for `string_caseless.rex` only, every program matching (fixed by `e1e4803c0`) |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: the one mismatch the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` (`checks-unsafe.txt`, at `eb3bea770`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.7a/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `t8.6/r1/suite/` (also at `e1e4803c0`) |
| `memcap 16G cargo test -j 4 --release --no-fail-fast --workspace`, `rust/target/release/rexx` built first, default target dir (`close-workspace-tests.txt`, status in `close-workspace-status.txt`) | 101 at `560352131`: one failure, `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake |

## Concerns

1. **Internal renderings do not send DEFAULTNAME, for instances or classes.** A trace value line, an
   error message's object insert and an Array's `makeString`/`toString` join print the default name
   here where the oracle's `stringValue` sends `DEFAULTNAME` for a non-primitive object
   (`probes/dn/dn4.rex` for a class, `dn5.rex` for an instance: `>>>   "DN"`, `Object "DN" does not
   understand`, `DN, The PLAIN class` on the oracle). The instance half predates this task. Those
   renderings go through `string_value_text`, which does not send. Proposed: a queued item for the
   object-protocol area.
2. **The foreign-class `SORTWITH`.** The oracle segfaults (entry 62, a new oracle defect for
   `defects.json`); this crate refuses loudly, rc 120, with a "not implemented" text. No rule gives
   an answer to model, so the refusal stays; if a raise is wanted instead, that is a ruling.
3. **TRANSLATE's null-string identity now reaches the String method.** An empty table that a
   builtin produced (`left('',0)`, `substr` past the end, `copies(...,0)`, `delstr` of all of it) is
   the oracle's `NULLSTRING` singleton and reads as omitted there; here every supplied empty table is
   one (`probes/tre/t1.rex`: five String-method lines `[  ]` on the oracle, `[12]` here). This is the
   `phase-4-exclusions.txt` KNOWN GAP "TRANSLATE -- A NULL STRING'S IDENTITY ...", whose text names
   the builtin only; the controller may extend it to the method. MutableBuffer's `translate` reads an
   empty supplied table as none on both sides, which departs from rexxref's rule as the String
   method's oracle answer would; unchanged.
4. **`Method~source` of every REXX package method is now empty**, matching the oracle. A program or
   tool that read this crate's library sources through `~source` no longer can.
5. `equals` with a reference argument answers the referenced value's comparison (Deviation 72's
   reach, as ruled for String method arguments), so `'kmake'~equals(>k)` is 1 here and 0 there.

## New oracle defects

* `SORTWITH` taken from `.Array` and defined on a class that is not an Array, sent to an instance:
  SIGSEGV, rc 139 (`oracle-crashes.txt` entry 62; the Task 8 triage's
  `t8-triage-2/q09/e.rex`).

## Fix round 1

Code `cc1e48c0c`. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7a/r1/`.

**Important 1.** `Interp::message_term` (`dispatch.rs`), the tree path every traced send takes,
rendered the result for its `>M>` line while nothing held it, so a fresh reference a method returned
was collected by that rendering's own allocation. It now calls `trace_sent` (`ir/drive.rs`, made
`pub(crate)`), which roots the value before rendering, as the IR's `deliver_sent` already did; the
untraced path is unchanged. Witness: `trace_reference_value_rooted.rex` gains the review's `c4o`
(`w = o~r(>v)`), `c4f` (`say o~r(>v)~class`), `c4n2` (the same with `v = 1`) and an INTERPRET of a
`[]` method returning a reference; its `sourceline_oracle` file regenerated by the test's driver from
a copy outside the repository. Red: `red-collect-stress-030af45a1.txt`, collect_stress built from a
`git archive` of `030af45a1` with the new witness, `73 passed; 1 failed`, panic
`dispatch.rs:1831:9` (the witness is the only change from the review's 74-passed run). Green:
`checks-collect-stress.txt`, 74 passed.

Sibling hunt (`probes/`, `probes/summary.txt`; a stress runner calling
`run_program_collect_every_alloc` on one file, `probes/stress-runner-main.rs`, against the oracle):
function and `CALL` results of a routine returning a reference (`s01`-`s03`, `s23`), an internal
label (`s25`), a `.routine~call` (`s15`), INTERPRET of an assignment and of a SAY (`s04`, `s05`), a
message instruction and `RESULT` (`s06`), `[]` (`s07`), an operator method (`s08`), parenthesised,
cascaded, chained and repeated sends (`s09`, `s10`, `s13*`, `s14`), routines returning a fresh Array
(`s11`, `s12`), `.message~send` (`s16`), IF, DO, PARSE VALUE, compound and RETURN targets (`s17`-`s24`),
and a send inside a called label (`s26`). At `030af45a1` every red one is a send reaching
`message_term` (`c4*`, `s04`-`s07`, `s09`, `s13`, `s13f`, `s13q`, `s14`, `s15`, `s17`-`s22`, `s24`,
`s26`); at `cc1e48c0c` all agree with the oracle on stdout and rc, and on stderr except the
oracle's absolute path in the `s13*` error lines and `s25`. Routine results, operator results and
`>>>`/`>=>` lines were green at `030af45a1` already. `s25` is a trace divergence unrelated to rooting
and present at `81f4bf250` in the normal binary: for `h: use arg >x; return >x` called with `>v`,
the oracle traces `>O>   ">" => "V"`, this crate `"X"`. Queue candidate.

**Minor 1.** `operand_number`'s doc is back on its function in `value.rs`.

**Minor 2.** `tests::a_references_equals_argument_is_its_referenced_values_string` pins
`'kmake'~equals(>k) 'a K'~equals(>k) 'KMAKE'~caselessEquals(>k)` at `1 0 1` and the trapped
`signal on nostring` form at `1` (the oracle: `0 1 0`, `NOSTRING a K`; review `eqr/r1.rex`).

**Oracle crash.** `oracle-crashes.txt` entry 63, `q = >y; trace i; q~value = 3`: rc 139 after
`>A>   "3"`, 3 of 3 runs under the standard wrapper from fresh directories; this crate prints `3`,
rc 0. Not added to `defects.json` (controller's).

**Perf.** `callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7a/r1 base=.../rexx-base-81f4bf250
head=.../rexx-head-cc1e48c0c`, every program, exit 0, no OVER or SPREAD flag; against the base at
most +0.0016% libc subtracted (startup). Own delta against `030af45a1`'s `callgrind-final.txt`
(`callgrind-own-delta.tsv`): at most +0.0022% libc subtracted (define); pingguard -0.0000%, pingsem
-0.0001%, pingmsg +0.0002%.

| command (from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87a-target`, at `cc1e48c0c`) | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage` (`checks-touched.txt`) | 0; lib 1201 passed |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: the one mismatch the known `rexxutil_file_date_time.rex` |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.7a/r1/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `t8.7a/suite/` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast --workspace`, `rust/target/release/rexx` built first, default target dir, at `98f716863` (`close-workspace-tests.txt`, status in `close-workspace-status.txt`) | 101: one failure, `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake |
