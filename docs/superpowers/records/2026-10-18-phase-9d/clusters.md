# Phase 9d clusters

`clusters.tsv` assigns each test of `rust/corpus/suite-9d-set.txt` to one cluster. The `cause`
column is the first divergence the crate's run printed: the failing assertion's expected and actual
values, or the SYNTAX event the framework reported. The raw outputs are under `causes/<group>/`, one
`oracle-N` and one `crate-N` triple (`.stdout`, `.stderr`, `.rc`) per run, and none was trimmed.
Probes that name a mechanism are under `probes/<name>/`, each with its program and both sides'
three descriptors.

## How the runs were made

Each group of the set ran once per side, one group at a time, through a throwaway copy of
`rust/crates/rexx-exec/tests/suite.rs` whose `record` also wrote each run's stdout, stderr and status
to a directory. The copy was not committed. It ran `l3_core_suite` with `REXX_SUITE_GATE=1` and
`REXX_SUITE_GROUPS=<group>`, so the run directory, the renamed-out rxapi tests, `ooTest.frm` without
its `rxfuncquery` lines, `rxregexp.cls` beside the driver and the oracle's `build/lib` on the crate
side are the harness's own. The crate binary was built at `328fc0e28` with `--release`.

The failing set the runs printed equals the set file: the sorted `group test` pairs from every
`crate-N.stdout` failure and error block, diffed against the set file's rows, give no difference.

## Clusters

Each count is `awk -F'\t' '$3=="<cluster>"' clusters.tsv | wc -l`. "Probed" means a probe under
`probes/` shows the mechanism on both sides. Otherwise the mechanism is read from the test source
and the failure text. "Files" are where a fix will likely land, not a measured blast radius.

### requires-from-dynamic-code (25)

A `::REQUIRES` in code built by `.Routine~new(name, source, parentPackage)` inside a package that was
loaded from another directory is not found: the crate searches from the main program, the oracle
from the routine's package. Probed (`requires-from-loaded-package`, with a file that exists before
the run, so the file being written at run time is not the cause). This is the parent's "`::REQUIRES`
of a file written at run time".
* Files: `rust/crates/rexx-exec/src/require.rs`, `src/paths.rs`, `src/dispatch/executable.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### rexxinteger-min-representation (19)

The group's `isInteger` routine detects the oracle's internal `RexxInteger` class through
`RexxInteger::Min`, which answers its receiver before the `NUMERIC DIGITS` check
(`interpreter/classes/IntegerClass.cpp:1634`, whose comment says the order is deliberate so tests can
tell a `RexxInteger` from a `NumberString`). The crate has no such observable representation.
* Files: `rust/crates/rexx-exec/src/builtin/numeric.rs` (`MIN`), if the ruling is to match.
* Deviation: **yes**. The oracle makes its representation observable on purpose, which the crate's
  rule that representation is never observable forbids.
* Review Focus 3 or 4: no.

### lostdigits-not-raised (13)

`LOSTDIGITS` is never raised for an operand wider than `NUMERIC DIGITS`. Probed (`lostdigits`):
`1 / 9999999999`, `1111111111 + 1` and a variable operand all trap on the oracle and none does here.
`TESTINTEGERDIVIDE` and `TESTREMAINDER` show SYNTAX 26.11 and 26.12 in its place.
`SIGNAL TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS` also needs `raise-propagate-from-method`.
* Files: `rust/crates/rexx-num/src/`, `rust/crates/rexx-exec/src/eval.rs`, `src/run/condition.rs`.
* Deviation: no.
* Review Focus 4: yes, `LOSTDIGITS.testGroup` evaluates every expression through `INTERPRET`
  (`58 *-* interpret "x="expr` in its tracebacks).

### stream-notready-names (12)

Stream operations on a directory, an invalid name or a missing file. `CHARS` and `LINES` on `/` raise
no NOTREADY and `chars('/')` answers 340, NOTREADY on `*` and a missing file has no `ADDITIONAL`,
`lineout('/')` answers 0, and the `open` command on a directory answers no result (91.999) where the
oracle answers `ERROR:2`. Probed (`stream-open-directory`). This is the parent's "streams" cluster.
* Files: `rust/crates/rexx-exec/src/builtin/stream.rs`, `src/dispatch/stream.rs`.
* Deviation: no. Review Focus 3: yes, these are the directory and invalid-name paths.

### label-in-if-select-branch (9)

A label as the instruction after `THEN`, `ELSE` or `OTHERWISE` should be error 47.3 or 47.4 at
translation. The crate either accepts it, or reads `lbl: nop` as a namespace-qualified name (98.987)
or loses the `DO` (10.1).
* Files: `rust/crates/rexx-parse/src/block.rs`, `src/clause.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### stream-binary-records (6)

Streams opened `BINARY` with and without `RECLENGTH`. On the oracle `LINEIN` reads a record rather
than a line, `CHARS` counts per record, `LINEOUT` pads a short record and rejects a long one
(error 93), and `ARRAYIN` splits by record. Each test's first assertion on that behaviour fails here.
* Files: `rust/crates/rexx-exec/src/dispatch/stream.rs`, `src/builtin/stream.rs`.
* Deviation: possibly. `LINEOUT TEST_BINARY` fails first at line 220, which the oracle passes. Its
  later assertion at line 233 expects output the test's own comment marks `@@wrong`, so passing it
  means reproducing that oracle defect.
* Review Focus 3 or 4: no.

### stream-open-write-only (6)

`open write append` on a file with mode `a+w,a-r` fails with `ERROR:13`, and through `STREAM` the
command answers no result. The oracle answers `READY:`.
* Files: `rust/crates/rexx-exec/src/dispatch/stream.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### stream-command-position (6)

Stream commands on positions and handles: `query handle` after an implicit open,
`query position write` after a read open, `seek` on a closed or never-opened stream, `seek` past the
last line, and the `seek` parser's rejection of malformed offsets. Not probed one by one, so this
cluster may split.
* Files: `rust/crates/rexx-exec/src/dispatch/stream.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### identity-collection-equality (6)

`IdentityTable` finds a different string object equal to an index (`"2"||"2"`, `substr`), and
`hasItem` compares items with `=`. The oracle compares both by identity.
* Files: `rust/crates/rexx-exec/src/dispatch/hash.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### variable-reference-list (5)

A parenthesised variable list in `DROP`, `EXPOSE` and `PROCEDURE EXPOSE` reads the list variable
without the ordinary fetch: no NOVALUE when it is unset (98.986 under `::OPTIONS ALL SYNTAX`) and no
`>V>` trace line.
* Files: `rust/crates/rexx-exec/src/run.rs`, `src/variables.rs`, `src/trace.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### use-arg-reference-conflict (5)

`USE ARG >name` into an exposed, auto-exposed or already-bound variable should raise 98.995.
* Files: `rust/crates/rexx-exec/src/variables.rs`, `src/dispatch/method_arguments.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### traceobject-probe-option (5)

`.TraceObject~option = 'P'` with a collector set collects trace objects without writing them. The
crate writes the lines to the trace output and collects nothing. Probed (`traceobject-probe`).
* Files: `rust/crates/rexx-exec/src/trace.rs`.
* Deviation: no.
* Review Focus 4: yes, `TEST_CALLER_STACK_FRAME_REPLY_START` runs on a REPLY continuation.

### raise-propagate-from-method (5)

`RAISE PROPAGATE` in a method's trap handler does not reach the caller's `SIGNAL ON ANY`, through a
plain call or through `Message~send`. From an internal routine it does. Probed
(`raise-propagate-from-method`, `signal-any-in-method`). This is the parent's "`SIGNAL ON ANY`".
* Files: `rust/crates/rexx-exec/src/run/raised.rs`, `src/run/condition.rs`, `src/condition.rs`.
* Deviation: no.
* Review Focus 4: yes, the propagated path.

### collection-set-operations (5)

`difference` and `xor` on `Queue` and `CircularQueue` answer collections that are not `equal` to the
expected ones. Not probed.
* Files: `rust/crates/rexx-exec/src/dispatch/collection/queue.rs`, `src/dispatch/collection.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### dynamic-package-name (3)

Code built by `.Routine~new(name, source)` or `.Package~new` reports the creating package's file as
its package name in trace lines. The oracle reports `name`. Probed (`floating-method-trace`).
* Files: `rust/crates/rexx-exec/src/dispatch/executable.rs`, `src/dispatch/package.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### collection-argument-error-numbers (3)

`Array~section(1, .nil)` should raise 93.923 and `Queue~insert("a", 99)` 93.966. The crate raises
93.906 and 93.918.
* Files: `rust/crates/rexx-exec/src/dispatch/array/surface.rs`,
  `src/dispatch/collection/queue.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### datatype-whole-wide-numbers (3)

`DATATYPE(n, 'W')` answers 0 for a 19-digit whole number at `NUMERIC DIGITS 19` or 20. The failing
`.Validate` methods are the oracle's `CoreClasses.orx` code, which calls `DATATYPE`. Probed
(`datatype-whole-19-digits`).
* Files: `rust/crates/rexx-exec/src/builtin/datatype.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### security-manager-method-event (3)

A protected method (`Directory~setMethod`) sends no `METHOD` event to the security manager. Each
test fails the same way in its Routine, Package and Method subclass.
* Files: `rust/crates/rexx-exec/src/security.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### raise-result-entry (3)

`RAISE ... EXIT expr` and `RETURN expr` leave the condition object's `RESULT` unset. Probed
(`raise-exit-result`).
* Files: `rust/crates/rexx-exec/src/run/raised.rs`, `src/condition.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### call-on-trap-argument (3)

A `CALL ON` trap target gets the condition object as its only argument. The crate passes none, so a
builtin or external target raises 40.3.
* Files: `rust/crates/rexx-exec/src/run/condition.rs`, `src/run/call.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### array-array-index (3)

`Array~insert` and `Array~section` refuse an Array as the index argument (93.907). This is the
parent's "`Array~insert` with an Array index".
* Files: `rust/crates/rexx-exec/src/dispatch/array/surface.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### stream-eof-state (2)

`CHARIN` past the end leaves the stream `ERROR` (`ERROR:0`) where the oracle reports `NOTREADY`
(`NOTREADY:EOF`).
* Files: `rust/crates/rexx-exec/src/dispatch/stream.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### stream-argument-error-numbers (2)

`LINEOUT` with a nonnumeric line raises the Stream method's 93.907 where the BIF raises 40.12, and
`Stream~lineIn(.nil)` raises 93.907 where the oracle raises 88.907.
* Files: `rust/crates/rexx-exec/src/builtin/stream.rs`, `src/dispatch/stream.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### redirected-command-holds-baton (2)

`samples/scclient` and `samples/sfclient`. The test starts the server with `Object~start`, and the
server method runs `rexx scserver` under `ADDRESS ... WITH INPUT USING ... OUTPUT STEM`. A redirected
command waits for its child on the baton (`Interp::run_command`, `rust/crates/rexx-exec/src/command.rs`
`:728`, parks only when `io.is_none()`), so no other activity runs until the server exits. The clients
then start against a closed port: `Connect failed: ECONNREFUSED` and `SockConnect failed:
ECONNREFUSED`. The oracle releases the kernel lock around a command handler
(`NativeActivation::run`, `interpreter/execution/NativeActivation.cpp:1691`). Probed
(`command-redirected` against `command-plain`).
* Files: `rust/crates/rexx-exec/src/command.rs`, `src/redirect.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### method-object-identity (2)

A Method object answered through a subclass (`~methods`, `~instanceMethods(scope)`) is not the object
the defining class answers for the same method. Probed (`method-identity`).
* Files: `rust/crates/rexx-exec/src/dispatch/introspection.rs`, `rust/crates/rexx-classes/src/method_dict.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### interactive-trace-setting (2)

`Package~trace` answers `C` for `::OPTIONS TRACE ?C`, and a bare `TRACE` while interactive tracing
is on resets the setting to `N` where the oracle ignores it.
* Files: `rust/crates/rexx-exec/src/trace.rs`, `src/options.rs`.
* Deviation: no. Review Focus 3 or 4: no.

### Single-test clusters

Each count is 1. All are Deviation "no" and Review Focus "no" unless stated.
* `attribute-body-error-number`: `::attribute 'foo' class` with a body raises 99.937, the oracle
  99.905. `rust/crates/rexx-parse/src/directive.rs`.
* `bag-hasitem-nil`: `.Bag~of(1,1,1)~hasItem(1, .nil)` answers `.nil` on the oracle and 0 here.
  Probed (`bag-hasitem-queue-removeitem`). `BagClass::hasItemRexx`
  (`interpreter/classes/BagClass.cpp:173`) returns `TheNilObject` against its own doc comment
  ("@return .true if this was found, .false otherwise"). **Deviation: yes.**
  `rust/crates/rexx-exec/src/dispatch/hash.rs`.
* `class-activate-order`: class `activate` methods run in an order that puts `class1` before
  `class2`. `rust/crates/rexx-exec/src/install.rs`.
* `constant-expression-novalue-option`: `::OPTIONS NOVALUE ERROR` after a `::CONSTANT` expression is
  not applied to it (41.1 instead of 98.986). `rust/crates/rexx-exec/src/install.rs`, `src/options.rs`.
* `context-rs`: `.context~rs` stays `.nil` after a command. Probed (`context-rs`).
  `rust/crates/rexx-exec/src/dispatch/context.rs`.
* `hostemu-halt`: `SIGNAL TEST_SIGNAL_ANY_CONDITION_HALT` runs `address "hostemu" "HI"` after
  `::requires "hostemu" LIBRARY`. Here the environment is not registered and the command raises
  FAILURE with rc 30, measured with a probe outside `probes/`. `hostemu` is a C extension, which is
  Phase 10's. Needs an owner ruling, not a Deviation.
* `method-package-prolog`: `.Method~new(...)~package~prolog` answers a Routine where the oracle
  answers `.nil`. `rust/crates/rexx-exec/src/dispatch/package.rs`.
* `native-stack-frames`: the `SORTWITH` frame's `arguments` is not the comparator array. The
  parent's `2026-10-02-frame-executable-invocation-and-native-levels`.
  `rust/crates/rexx-exec/src/dispatch/context.rs`.
* `nil-comparison`: an ordered comparison of a string with `.nil` raises NOSTRING here and answers
  false on the oracle (`CompareOperator`, `interpreter/classes/StringClass.cpp:1248`). Probed
  (`nil-comparison`). `rust/crates/rexx-exec/src/eval.rs`.
* `prolog-trace-lines`: `.Routine~new` under `::OPTIONS TRACE` traces the new package's prolog entry
  and exit on the oracle and not here. The parent's
  `2026-10-10-prologue-frames-missing-loading-levels`. Probed (`floating-method-trace`).
  `rust/crates/rexx-exec/src/dispatch/executable.rs`, `src/trace.rs`.
* `queue-removeitem`: `Queue~removeItem` leaves a hole. Probed (`bag-hasitem-queue-removeitem`).
  `rust/crates/rexx-exec/src/dispatch/collection/queue.rs`.
* `reply-thread-error-report-line-ends`: a SYNTAX error raised on a REPLY thread with an insert
  holding a line end is written without the insert's second line. `rust/crates/rexx-exec/src/error.rs`.
  **Review Focus 4: yes** (REPLY).
* `security-manager-environment-result`: the manager's `RESULT` for an `ENVIRONMENT` event does not
  replace the looked-up value. `rust/crates/rexx-exec/src/security.rs`.
* `setmethod-instancemethod-kind`: after `setMethod` with a string source, `instanceMethod` answers
  an object that is not a Method. `rust/crates/rexx-exec/src/dispatch/object_protocol.rs`.
* `signal-trap-missing-label`: SYNTAX 16.1 from a `SIGNAL ON NOVALUE` whose label is missing is not
  trapped by `SIGNAL ON SYNTAX` (upstream bug 1877). Probed (`signal-missing-label`).
  `rust/crates/rexx-exec/src/run/condition.rs`.
* `superclass-not-a-class`: `::class test subclass input` should raise 99.949 because `.INPUT` is not
  a class. The crate raises 98.909 at run time. `rust/crates/rexx-exec/src/install.rs`.
* `trace-label-before-procedure`: under `TRACE I`, `PROCEDURE` after a traced label raises 17.1.
  `rust/crates/rexx-exec/src/run.rs`.
* `trace-reply-continuation`: `::OPTIONS TRACE LABELS` traces a REPLY continuation's method entry
  and exit on the oracle and not here. `rust/crates/rexx-exec/src/trace.rs`, `src/scheduler.rs`.
  **Review Focus 4: yes** (REPLY).
* `traceoutput-invalid-destination`: `.traceOutput~destination("")` then `TRACE R` should raise 97.1
  for the missing `LINEOUT`. `rust/crates/rexx-exec/src/trace.rs`.
* `uninit-define-on-superclass`: an `UNINIT` defined on a superclass after a subclass exists does not
  make the subclass's new instances run it. `rust/crates/rexx-exec/src/dispatch/class_protocol.rs`.

## Parent clusters

The parent spec (2026-10-10, section 3) names its own clusters. Its "conditions not raised" spans
`raise-propagate-from-method`, `call-on-trap-argument`, `variable-reference-list`,
`label-in-if-select-branch`, `lostdigits-not-raised`, the 98.974 rows of `stream-notready-names`
and `traceoutput-invalid-destination`. Its traceback family maps onto `native-stack-frames`,
`prolog-trace-lines`, `trace-reply-continuation` and `reply-thread-error-report-line-ends`. No test
of the set fails on an INTERPRET traceback line.
