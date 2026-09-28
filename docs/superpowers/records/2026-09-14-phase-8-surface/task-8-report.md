# Task 8 report -- METHOD, CONVERSION and FUNCTION on both sides

Implementer: surface-8. BASE `391242b7e`. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-8/`
(below, `$S`). Written first and filled as the work goes.

## Status

DONE_WITH_CONCERNS at `2b26b6970`, gates green.

## Harness design

`rust/crates/rexx-exec/tests/api_group_tests.rs`, gate-only (`REXX_CORPUS_GATE`), ruling S4.

* Groups: `CONVERSION`, `FUNCTION`, `METHOD`, the ones `api_group_partition.rs` derives as
  Phase 8's.
* Per group, the oracle lists the tests: `testOORexx.rex -f <copy>/ooRexx/API/oo/G.testGroup -U -V
  0 -S`, whose stderr carries `.. Executing ... test case NAME` per test, in run order.
* Per test, both sides run `testOORexx.rex -f G.testGroup -U -V 0 -t NAME`, each from a fresh copy
  of `ootest/framework`, `testOORexx.rex`, `worker.rex`, `ooTest.frm`, `ootest/ooRexx/API/oo` and
  `extensions/rxregexp/rxregexp.cls` laid at one path under `CARGO_TARGET_TMPDIR` (so paths are
  byte-identical between sides), removed at the end. `ootest/` itself is only read. The oracle
  goes through `support::oracle`'s `run_with` (the standard wrapper, from the copy's root); the
  crate runs in process through `watchdog::run_bounded` with the copy as its directory and the
  oracle's `build/lib` as `LD_LIBRARY_PATH` in the invocation's own environment.
* A test is a member of the failing set when stdout, stderr or exit status differ. The test
  asserts that set equals `RECORDED`, a list it holds, printing newly failing and newly passing
  names; and that each recorded name appears in `phase-4-exclusions.txt`.
* **Then a whole-group run per group**, both sides, `-f G.testGroup -U -V 0 -S`, from a copy in
  which each recorded test's `::method` is renamed `SKIPPED_...` (asserted: exactly the recorded
  tests of that group), and all three descriptors must agree. Added after the first version,
  because a lone run cannot see tests that depend on an earlier one: FUNCTION's `test` registers
  the `io` handler that `TEST_BUFFERED_INPUT`, `TEST_BUFFERING`, `TEST_FILE_INPUT`,
  `TEST_GLOBAL_SETTING`, `TEST_SIMPLE_WITH` and `TEST_WRITE_BUFFER` use, and alone each fails
  identically on both sides (`awk '$4!=0' pt2/verdicts.txt`). `-t TEST NAME` does not help: under
  `-t` the framework orders tests by `Class~methods` iteration (`ooTest.frm`
  `constructSuiteWithTestCases`), which differs between the sides (Deviation 8's territory); without
  `-t` the order is the sorted one, and the crate's order matched the oracle's for every test it
  reached (`$S/whole1`). Measured with the recorded tests renamed out: CONVERSION 97, FUNCTION 156,
  METHOD 308 tests, three descriptors identical (`$S/groupskip.py`). Control N12 shows what this
  part adds.
* One test per run rather than the whole group in one process: a loud refusal ends the process
  (rc 120), so a group run can only report the first. Per-test runs give every test its own
  result; the whole-group part covers order.
* `-U` and `-V 0`, on both sides alike. Measured at the start: the default run stops at the
  ticker (`ooTest.frm` `PhaseReport~tickTock` is a `REPLY` thread, `~stopTicking` a `GUARD ON
  WHEN`; this crate refuses naming Phase 6), and the full summary calls `RXFUNCQUERY` (refused
  naming Phase 10); on the oracle `rxfuncquery("SysWinVer")` reaches `RexxQueryFunction`, rxapi
  (read, `interpreter/package/PackageManager.cpp:618-635`). `-U` starts no ticker; `-V 0` prints
  the brief summary. The single-group form needs `-f` once a second option is present
  (`worker.rex:1060`: more than one token must all be options).
* Runtime: 52.47 s for the test function (both parts) (`cargo test --release -p rexx-exec --test
  api_group_tests`, gated).
* Scratch equivalent used to measure: `$S/pertest.sh OUT [BINARY]` (the same runs through the
  `rexx-run` binary; verdicts in `OUT/verdicts.txt`, differing tests' descriptors in `OUT/d/`).

## Initial failing set

**At BASE `391242b7e`** (`$S/pertest.sh pt-base $S/base-target/release/rexx-run`, a `git archive`
of BASE built in its own target dir): every test the oracle lists differs, and each crate run
stops before its first test with the same stderr, `rexx-exec: method "COPY" of class "Object" is
not implemented (Phase 5)` (`worker.rex:1074`, `cmdLine~copy`). Command:
`awk '{print $1, ($3=="same"?"same":"differs")}' pt-base/verdicts.txt | sort | uniq -c` gives
differs for all of CONVERSION, FUNCTION and METHOD, and `grep -h -v 'Executing|Searching'
pt-base/d/*/c.err | sort | uniq -c` gives that one line. The names, from the oracle's `-S` listing:

```
CONVERSION: TESTDOUBLE01 TESTDOUBLE03 TESTDOUBLE04 TESTDOUBLEL02 TESTDOUBLEL05 TESTFLOAT01 TESTFLOAT02 TESTINT01 TESTINT02 TESTINT03 TESTINT1601 TESTINT1602 TESTINT1603 TESTINT3201 TESTINT3202 TESTINT3203 TESTINT3204 TESTINT3205 TESTINT3206 TESTINT3207 TESTINT6401 TESTINT6402 TESTINT6403 TESTINT6404 TESTINT6405 TESTINT6406 TESTINT6407 TESTINT801 TESTINT802 TESTINT803 TESTINTPTR01 TESTINTPTR02 TESTINTPTR03 TESTINTPTR04 TESTINTPTR05 TESTINTPTR06 TESTINTPTR07 TESTISSTRING TESTLOGICAL01 TESTLOGICAL02 TESTLOGICAL03 TESTLOGICAL04 TESTLOGICAL05 TESTLOGICAL06 TESTLOGICAL07 TESTSIZE01 TESTSIZE02 TESTSIZE03 TESTSSIZE05 TESTSSIZE06 TESTSSIZE07 TESTSTEM01 TESTSTEM02 TESTSTEM03 TESTSTEM04 TESTSTRINGLENGTH TESTSTRINGSIZE01 TESTSTRINGSIZE02 TESTSTRINGSIZE03 TESTSTRINGSIZE04 TESTSTRINGSIZE05 TESTSTRINGSIZE06 TESTSTRINGSIZE07 TESTUINT1601 TESTUINT1602 TESTUINT1603 TESTUINT3201 TESTUINT3202 TESTUINT3203 TESTUINT3204 TESTUINT3205 TESTUINT3206 TESTUINT3207 TESTUINT6401 TESTUINT6402 TESTUINT6403 TESTUINT6404 TESTUINT6405 TESTUINT6406 TESTUINT6407 TESTUINT801 TESTUINT802 TESTUINT803 TESTUINTPTR01 TESTUINTPTR02 TESTUINTPTR03 TESTUINTPTR04 TESTUINTPTR05 TESTUINTPTR06 TESTUINTPTR07 TESTWHOLENUMBER01 TESTWHOLENUMBER02 TESTWHOLENUMBER03 TESTWHOLENUMBER04 TESTWHOLENUMBER05 TESTWHOLENUMBER06 TESTWHOLENUMBER07 
FUNCTION: TEST TEST000 TEST001 TEST002 TEST003 TEST004 TEST005 TEST006 TEST007 TEST008 TEST009 TEST010 TEST011 TEST012 TEST013 TEST014 TEST015 TEST016 TEST017 TEST018 TEST019 TEST020 TEST021 TEST022 TEST023 TEST023A TEST024 TEST024A TEST025 TEST026 TEST027 TEST028 TEST029 TEST030 TEST031 TEST045 TEST046 TESTARGLIST01 TESTDOUBLE01 TESTDOUBLE02 TESTDROP01 TESTFINDCONTEXTCLASS01 TESTFINDCONTEXTCLASS02 TESTFLOAT01 TESTFLOAT02 TESTGET02 TESTGET03 TESTGETALLVARIABLES1 TESTGETARGUMENT01 TESTGETARGUMENT02 TESTGETARGUMENT03 TESTGETARGUMENT04 TESTGETARGUMENTS01 TESTGETDIGITS01 TESTGETFORM01 TESTGETFUZZ01 TESTGETROUTINE01 TESTGETVARIABLE01 TESTINT01 TESTINT02 TESTINT03 TESTINT1601 TESTINT1602 TESTINT1603 TESTINT3201 TESTINT3202 TESTINT3203 TESTINT6401 TESTINT6402 TESTINT6403 TESTINT801 TESTINT802 TESTINT803 TESTLOGICAL01 TESTLOGICAL02 TESTLOGICAL03 TESTNAME01 TESTNAME02 TESTNESTEDACTIVITIES TESTNONNEGATIVEWHOLENUMBER01 TESTNONNEGATIVEWHOLENUMBER02 TESTNONNEGATIVEWHOLENUMBER04 TESTPOSITIVEWHOLENUMBER01 TESTPOSITIVEWHOLENUMBER02 TESTPOSITIVEWHOLENUMBER03 TESTPOSITIVEWHOLENUMBER04 TESTRAISEEXCEPTION001 TESTRAISEEXCEPTION01 TESTRAISEEXCEPTION101 TESTRAISEEXCEPTION201 TESTRESOLVESTEM01 TESTRESOLVESTEM02 TESTSETVARIABLE01 TESTSIZE01 TESTSIZE02 TESTSIZE03 TESTSSIZE01 TESTSSIZE02 TESTSSIZE03 TESTSTEM01 TESTSTEM02 TESTSTEM03 TESTSTEM04 TESTSTRINGSIZE01 TESTSTRINGSIZE03 TESTTHROWEXCEPTION001 TESTTHROWEXCEPTION01 TESTTHROWEXCEPTION101 TESTTHROWEXCEPTION201 TESTUINT1601 TESTUINT1602 TESTUINT1603 TESTUINT3201 TESTUINT3202 TESTUINT3203 TESTUINT6401 TESTUINT6402 TESTUINT6403 TESTUINT801 TESTUINT802 TESTUINT803 TESTWHOLENUMBER01 TESTWHOLENUMBER02 TESTWHOLENUMBER03 TEST_ADDCOMMANDENVIRONMENT_DIRECT TEST_ADDCOMMANDENVIRONMENT_REDIRECTING TEST_BUFFERED_INPUT TEST_BUFFERING TEST_CSTRING_MAKESTRING TEST_CSTRING_NIL TEST_CSTRING_NO_HEX00 TEST_CSTRING_NO_STRING_NO_MAKESTRING TEST_CSTRING_STRING_NO_MAKESTRING TEST_FILE_INPUT TEST_GLOBAL_SETTING TEST_INPUT_OUTPUT_STREAM TEST_OPTIONALCSTRING TEST_OPTIONALREXXSTRING TEST_REXXQUEUE TEST_REXXSTRING TEST_REXXSTRING_HEX00 TEST_REXXSTRING_MAKESTRING TEST_REXXSTRING_NIL TEST_REXXSTRING_NO_STRING_NO_MAKESTRING TEST_REXXSTRING_STRING_NO_MAKESTRING TEST_SIMPLE_WITH TEST_UINT64_BUG1369_FAIL_E20 TEST_UINT64_BUG1369_FAIL_E21 TEST_UINT64_BUG1369_FAIL_E22 TEST_UINT64_BUG1369_FAIL_E23 TEST_UINT64_BUG1369_FAIL_E24 TEST_UINT64_BUG1369_FAIL_E25 TEST_UINT64_BUG1369_FAIL_E26 TEST_UINT64_BUG1369_FAIL_E27 TEST_UINT64_BUG1369_FAIL_E28 TEST_UINT64_BUG1369_FAIL_E29 TEST_UINT64_BUG1369_PASS TEST_WRITE_BUFFER 
METHOD: TEST000 TEST001 TEST002 TEST003 TEST004 TEST005 TEST006 TEST007 TEST008 TEST009 TEST010 TEST011 TEST012 TEST013 TEST014 TEST015 TEST016 TEST017 TEST018 TEST019 TEST020 TEST021 TEST022 TEST023 TEST023A TEST024 TEST024A TEST025 TEST026 TEST027 TEST028 TEST029 TEST030 TEST031 TEST046 TESTARGLIST01 TESTARRAY01 TESTARRAY02 TESTARRAY03 TESTARRAY04 TESTARRAY05 TESTARRAYAPPEND01 TESTARRAYAPPENDSTRING01 TESTARRAYAT01 TESTARRAYAT02 TESTARRAYAT03 TESTARRAYAT04 TESTARRAYDIMENSION01 TESTARRAYITEMS01 TESTARRAYOFFOUR01 TESTARRAYOFONE01 TESTARRAYOFTHREE01 TESTARRAYOFTWO01 TESTARRAYPUT01 TESTARRAYSIZE01 TESTBUFFER01 TESTBUFFERNEW01 TESTCLASS01 TESTCLASS02 TESTCLASS03 TESTCLASS04 TESTCSTRING01 TESTCSTRING02 TESTCSTRINGTOOBJECT TESTDIRECTORY01 TESTDIRECTORY02 TESTDIRECTORY03 TESTDOUBLE01 TESTDOUBLE02 TESTDOUBLETOOBJECT01 TESTDROPOBJECTVARIABLE01 TESTDROPSTEMARRAYELEMENT01 TESTDROPSTEMELEMENT01 TESTFINDCONTEXTCLASS01 TESTFINDCONTEXTCLASS02 TESTFLOAT01 TESTFLOAT02 TESTGETALLSTEMELEMENTS01 TESTGETARGUMENT01 TESTGETARGUMENT02 TESTGETARGUMENT03 TESTGETARGUMENT04 TESTGETARGUMENTS01 TESTGETMETHOD01 TESTGETMETHODPACKAGE01 TESTGETOBJECTVARIABLE01 TESTGETOBJECTVARIABLE02 TESTGETOBJECTVARIABLE03 TESTGETPACKAGECLASSES01 TESTGETPACKAGEPUBLICCLASSES01 TESTGETPACKAGEPUBLICROUTINES01 TESTGETPACKAGEROUTINES01 TESTGETROUTINEPACKAGE01 TESTGETSTEMARRAYELEMENT01 TESTGETSTEMARRAYELEMENT02 TESTGETSTEMELEMENT01 TESTGETSTEMELEMENT02 TESTGETSTEMELEMENT03 TESTGETSTEMVALUE01 TESTHASMETHOD01 TESTINT01 TESTINT02 TESTINT03 TESTINT1601 TESTINT1602 TESTINT1603 TESTINT3201 TESTINT3202 TESTINT3203 TESTINT32TOOBJECT01 TESTINT6401 TESTINT6402 TESTINT6403 TESTINT64TOOBJECT01 TESTINT801 TESTINT802 TESTINT803 TESTINTPTR01 TESTINTPTR02 TESTINTPTR03 TESTINTPTRTOOBJECT01 TESTISARRAY01 TESTISINSTANCEOF01 TESTISMETHOD01 TESTISMUTABLEBUFFER TESTISROUTINE01 TESTISSTEM01 TESTISSTRING01 TESTLOGICAL01 TESTLOGICAL02 TESTLOGICAL03 TESTLOGICALTOOBJECT01 TESTMUTABLEBUFFERCAPACITY TESTMUTABLEBUFFERLENGTH TESTNAME01 TESTNAME02 TESTNEWARRAY01 TESTNEWMETHOD01 TESTNEWMETHOD02 TESTNEWMUTABLEBUFFER TESTNEWROUTINE01 TESTNEWROUTINE02 TESTNEWSTEM01 TESTNEWSTRING TESTNEWSTRINGFROMASCIIZ TESTNONNEGATIVEWHOLENUMBER01 TESTNONNEGATIVEWHOLENUMBER02 TESTNONNEGATIVEWHOLENUMBER03 TESTOBJECT01 TESTOBJECT02 TESTOBJECT03 TESTOBJECTTOCSTRING TESTOBJECTTODOUBLE01 TESTOBJECTTODOUBLE01A TESTOBJECTTODOUBLE02 TESTOBJECTTODOUBLE02A TESTOBJECTTOINT3201 TESTOBJECTTOINT3202 TESTOBJECTTOINT3202A TESTOBJECTTOINT3203 TESTOBJECTTOINT3203A TESTOBJECTTOINT6401 TESTOBJECTTOINT6402 TESTOBJECTTOINT6402A TESTOBJECTTOINT6403 TESTOBJECTTOINT6403A TESTOBJECTTOINTPTR01 TESTOBJECTTOINTPTR02 TESTOBJECTTOINTPTR02A TESTOBJECTTOINTPTR03 TESTOBJECTTOINTPTR03A TESTOBJECTTOLOGICAL01 TESTOBJECTTOLOGICAL02 TESTOBJECTTOLOGICAL02A TESTOBJECTTOLOGICAL03 TESTOBJECTTOLOGICAL03A TESTOBJECTTOSTRING01 TESTOBJECTTOSTRINGSIZE01 TESTOBJECTTOSTRINGSIZE02 TESTOBJECTTOSTRINGSIZE02A TESTOBJECTTOSTRINGSIZE03 TESTOBJECTTOSTRINGSIZE03A TESTOBJECTTOUINTPTR01 TESTOBJECTTOUINTPTR02 TESTOBJECTTOUINTPTR02A TESTOBJECTTOUINTPTR03 TESTOBJECTTOUINTPTR03A TESTOBJECTTOUNSIGNEDINT6401 TESTOBJECTTOUNSIGNEDINT6402 TESTOBJECTTOUNSIGNEDINT6402A TESTOBJECTTOUNSIGNEDINT6403 TESTOBJECTTOUNSIGNEDINT6403A TESTOBJECTTOWHOLENUMBER01 TESTOBJECTTOWHOLENUMBER02 TESTOBJECTTOWHOLENUMBER02A TESTOBJECTTOWHOLENUMBER03 TESTOBJECTTOWHOLENUMBER03A TESTOSELF01 TESTOSELF02 TESTPOSITIVEWHOLENUMBER01 TESTPOSITIVEWHOLENUMBER02 TESTPOSITIVEWHOLENUMBER03 TESTPOSITIVEWHOLENUMBER04 TESTRAISEEXCEPTION001 TESTRAISEEXCEPTION01 TESTRAISEEXCEPTION101 TESTRAISEEXCEPTION201 TESTSCOPE01 TESTSCOPE02 TESTSENDMESSAGE001 TESTSENDMESSAGE002 TESTSENDMESSAGE003 TESTSENDMESSAGE01 TESTSENDMESSAGE02 TESTSENDMESSAGE03 TESTSENDMESSAGE04 TESTSENDMESSAGE101 TESTSENDMESSAGE102 TESTSENDMESSAGE103 TESTSENDMESSAGE201 TESTSENDMESSAGE202 TESTSENDMESSAGE203 TESTSENDMESSAGE204 TESTSENDMESSAGESCOPED TESTSETGETMUTABLEBUFFER TESTSETMUTABLEBUFFERCAPACITY TESTSETMUTABLEBUFFERLENGTH TESTSETOBJECTVARIABLE01 TESTSETSTEMARRAYELEMENT01 TESTSETSTEMELEMENT01 TESTSIZE01 TESTSIZE02 TESTSIZE03 TESTSSIZE01 TESTSSIZE02 TESTSSIZE03 TESTSTEM01 TESTSTEM02 TESTSTEM03 TESTSTEM04 TESTSTEM05 TESTSTRING01 TESTSTRING02 TESTSTRING03 TESTSTRING04 TESTSTRINGDATA01 TESTSTRINGGET01 TESTSTRINGLENGTH01 TESTSTRINGLOWER01 TESTSTRINGSIZE01 TESTSTRINGSIZE02 TESTSTRINGSIZE03 TESTSTRINGSIZETOOBJECT01 TESTSTRINGTABLE01 TESTSTRINGTABLE02 TESTSTRINGTABLE03 TESTSTRINGUPPER01 TESTSUPER01 TESTSUPER02 TESTSUPPLIER01 TESTTESTUNSIGNEDINT32TOOBJECT01 TESTTESTUNSIGNEDINT32TOOBJECT02 TESTTESTUNSIGNEDINT32TOOBJECT02A TESTTESTUNSIGNEDINT32TOOBJECT03 TESTTESTUNSIGNEDINT32TOOBJECT03A TESTTHROWEXCEPTION001 TESTTHROWEXCEPTION01 TESTTHROWEXCEPTION101 TESTTHROWEXCEPTION201 TESTUINT1601 TESTUINT1602 TESTUINT1603 TESTUINT3201 TESTUINT3202 TESTUINT3203 TESTUINT6401 TESTUINT6402 TESTUINT6403 TESTUINT801 TESTUINT802 TESTUINT803 TESTUINTPTR01 TESTUINTPTR02 TESTUINTPTR03 TESTUINTPTRTOOBJECT01 TESTUNSIGNEDINT32TOOBJECT01 TESTUNSIGNEDINT64TOOBJECT01 TESTVERSION01 TESTWHOLENUMBER01 TESTWHOLENUMBER02 TESTWHOLENUMBER03 TESTWHOLENUMBERTOOBJECT01 TEST_BUG_1838 TEST_CSTRING_MAKESTRING TEST_CSTRING_NIL TEST_CSTRING_NO_HEX00 TEST_CSTRING_NO_STRING_NO_MAKESTRING TEST_CSTRING_STRING_NO_MAKESTRING TEST_GETOBJECTVARIABLEREFERENCE TEST_ISVARIABLEREFERENCE TEST_OBJECTMEMORY TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA TEST_REXXSTRING_HEX00 TEST_REXXSTRING_MAKESTRING TEST_REXXSTRING_NO_STRING_NO_MAKESTRING TEST_REXXSTRING_STRING_NO_MAKESTRING TEST_SETVARIABLEREFERENCEVALUE TEST_UINT64_BUG1369_FAIL_E20 TEST_UINT64_BUG1369_FAIL_E21 TEST_UINT64_BUG1369_FAIL_E22 TEST_UINT64_BUG1369_FAIL_E23 TEST_UINT64_BUG1369_FAIL_E24 TEST_UINT64_BUG1369_FAIL_E25 TEST_UINT64_BUG1369_FAIL_E26 TEST_UINT64_BUG1369_FAIL_E27 TEST_UINT64_BUG1369_FAIL_E28 TEST_UINT64_BUG1369_FAIL_E29 TEST_UINT64_BUG1369_PASS TEST_VARIABLEREFERENCENAME TEST_VARIABLEREFERENCEVALUE 
```

**The framework's stops**, walked one at a time with the default single-group form and then
`-f G -U` (each fixed below, witness named): `'abc'~copy`; `String~makeArray(' ')` (93.902, arity
0); the ticker's `GUARD ... WHEN` (Phase 6, avoided by `-U`); `Message~send` (`.message~new(self,
methodName)~send`, OOREXXUNIT.CLS:1585); `==` on an Array and on a native object (Phase 5
refusals); `SIGNAL ON ANY` in `doTheTest` not trapping a `RAISE SYNTAX` from a method it called;
the condition object's `ADDITIONAL[2]` a string rather than the `AssertFailure`; the condition
object refusing `HASENTRY`; `parse version self~rexxVersion` ("a message send is not
implemented"); `RXFUNCQUERY` in the summary (Phase 10, avoided by `-V 0`).

**After those** (`$S/pertest.sh pt1`, the binary with every framework fix): the failing set was

* FUNCTION: `TESTGETROUTINE01`, `TEST_INPUT_OUTPUT_STREAM`, `TEST_REXXQUEUE`
* METHOD: `TESTCLASS01`, `TESTDROPOBJECTVARIABLE01`, `TESTGETOBJECTVARIABLE01`,
  `TESTGETOBJECTVARIABLE02`, `TESTGETOBJECTVARIABLE03`, `TESTGETPACKAGECLASSES01`,
  `TESTGETPACKAGEPUBLICCLASSES01`, `TESTGETPACKAGEPUBLICROUTINES01`, `TESTGETPACKAGEROUTINES01`,
  `TESTNEWMETHOD02`, `TESTNEWROUTINE02`, `TESTSETOBJECTVARIABLE01`
* CONVERSION: none

(`grep -v ' same ' pt1/verdicts.txt`.)

## Per member

Every test of the BASE set was stopped by the framework, so the members are the framework's stops
first (each ended every test), then the tests the framework reached.

**Framework stops, fixed at `9dae74fac`** (general runtime, each local and witnessed by a STRICT
corpus program in `rust/corpus/lang/`, listed in `phase-8.txt`, with its sourceline file):

| Stop | Cause | Fix | Witness |
|---|---|---|---|
| `worker.rex:1074` `cmdLine~copy` | `native_copy` refused a String receiver | a heap string's copy is a new String; a handle-carried value is its own copy (identity of those is the licensed `identityHash` divergence) | `string_copy.rex` |
| `ooTest.frm:85` `~makearray(" ")` | `String~makeArray` had arity 0 | separator argument, `StringUtil::makearray`'s split | `string_makearray_separator.rex` |
| `OOREXXUNIT.CLS:1585` `.message~new(self, name)~send` | `Message~new` kept nothing, `SEND` unbuilt | `Message~new` keeps target, name, scope, arguments ('A'/'I'); `SEND`/`SENDWITH` make the send and record it for `RESULT`, `COMPLETED`, `HASERROR` | `message_send.rex`, `message_new_arguments.rex` |
| `==` on an Array (METHOD) and on a native object (FUNCTION) | `operator_operand_gap` refused both | both are operator receivers (`operator_message_receiver`); `Setup.cpp` gives them only `Object`'s operators (and `Pointer`'s, which have rows) | `operator_native_receivers.rex` |
| a failing assertion escaping `doTheTest`'s `SIGNAL ON ANY` | a tail-less `RAISE SYNTAX` was offered to the outermost activation only | offered at the nearest activation that is not an internal call, then outward (`RexxActivation::raiseExit`, `RexxActivation.cpp:1741`); measured three-level internal, `::ROUTINE` and method chains | `raise_syntax_nearest_top_level.rex` |
| `OOREXXUNIT.CLS:1612` `data~className = ...` on `"an AssertFailure"` | `RAISE ... ARRAY` put renderings in `ADDITIONAL` | the array of the items themselves goes to `pending_additional` | `raise_array_additional_objects.rex` |
| `cObj~hasentry("CODE")` | condition objects were `NativeObject` Directories | built by `Directory~new`, store-backed | `condition_object_directory.rex` |
| `ooTest.frm:598` `parse version self~rexxVersion` | `assign_expr_target` had no message-term arm | `NAME=` sent with the piece first, args traced as results, a setter's answer traced `>=>` (`RexxExpressionMessage::assign`) | `parse_message_targets.rex` |
| the ticker (`REPLY`, `GUARD ON WHEN`) | Phase 6's refusal | not fixed: `-U` on both sides | -- |
| `RXFUNCQUERY` in the full summary | Phase 10's refusal; rxapi on the oracle | not fixed: `-V 0` on both sides | -- |

Also in `9dae74fac`, because the answers changed what they asserted: `dispatch/tests.rs` (a
condition object's `request('ARRAY')` answers 14, `Message~send` answers, the loud-name pair
rebuilt on a `Method` object since `COPY` was `String`'s last loud name),
`eval/object_operand_tests.rs` (the old gap rows now assert the oracle's answers, each re-measured
three descriptors), `run/tests/conditions.rs` (a comment), `collect_stress.rs`
(`library_method_traceback`, `library_method_traceback_nested`, `raise_array_substitution` now
collect: the condition object and the ARRAY allocate), `method-bodies.txt` (refreshed: `Message
send`/`sendWith` answer), `refusal-sites.tsv` (re-derived; `diff` of both with column 4 blanked
is empty).

The queued item `.superpowers/sdd/queued/2026-09-21-parse-message-target-unimplemented.md` asked
for the target forms first: `parse_variable_or_message_term` (`rexx-parse/src/expr.rs:253`)
admits a message term or a symbol `need_variable` admits (Variable, Stem, Compound), and
`assign_expr_target` now has all four. Closing note appended there.

**Members the framework reached** (`pt1`):

* `FUNCTION.TESTGETROUTINE01` -- **Phase 8's, fixed at `8fb784e2d`**. `GetRoutine` answered
  `library_routine_object(code)` and the package's routine table a different object per
  `::ROUTINE`; the oracle has one object per library procedure, shared by every directive naming
  it (`LibraryPackage::resolveRoutine`). The table now holds the procedure's object, which carries
  the annotations of the first directive bound to it (measured: annotations answer `v1` on both
  sides whether `GetRoutine` or the table reaches the object first). Witness
  `library_routine_shared_object.rex` (with `.env`, the oracle's `build/lib`).
* `METHOD.TESTCLASS01` -- recorded, Phase 9: `Class~new` builds no class; the oracle's class
  answers no `NEW` and belongs to the REXX package, so it is not `~subclass`.
* `METHOD.TESTGETOBJECTVARIABLE01`, `...02`, `...03`, `METHOD.TESTSETOBJECTVARIABLE01`,
  `METHOD.TESTDROPOBJECTVARIABLE01` -- recorded, Phase 9: `EXPOSE` of a single compound tail
  (`VariableTester~init` exposes `stem2.1`).
* `METHOD.TESTGETPACKAGECLASSES01`, `METHOD.TESTGETPACKAGEPUBLICCLASSES01`,
  `METHOD.TESTGETPACKAGEPUBLICROUTINES01`, `METHOD.TESTGETPACKAGEROUTINES01` -- recorded, Phase 9:
  `ITEMS` on a package table, then `Routine~new` over a source with directives.
* `METHOD.TESTNEWMETHOD02`, `METHOD.TESTNEWROUTINE02` -- recorded, Phase 9: the API's
  `NewMethod`/`NewRoutine` compile through `Method~new`'s path, which refuses a source that does
  not parse; the oracle's 36.901 carries substitutions, a POSITION inside the source and a
  traceback line of it, none of which rexx-parse keeps.
* `FUNCTION.TEST_INPUT_OUTPUT_STREAM` -- recorded, Phase 9 (Task 6's entry, which had no owner):
  the stream-object input reader sends `STATE`; the oracle reads `LINEIN` under a
  condition-trapping dispatcher. Inside the suite the oracle fails this test too (RC(30),
  Failures 1) while here it is an error.
* `FUNCTION.TEST_REXXQUEUE` -- recorded, Phase 10: `rexx_create_queue`.

Records: `docs/superpowers/plans/phase-4-exclusions.txt`, block "TESTS OF THE METHOD, CONVERSION
AND FUNCTION GROUPS THAT DIFFER" (each test name on its own line; each probe was run from a fresh
directory before it was written there, `$S/rec/r1.rex`-`r5.rex`), and the Task 6 entry's owner.
Queued: `.superpowers/sdd/queued/2026-09-28-{class-new,expose-compound-tail,package-tables-and-directive-sources,compiled-source-parse-errors,input-stream-object-reader,rexxqueue-in-function-group}.md`.
No record names Phase 8.

## Final failing set

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test api_group_tests` at `2b26b6970`:
ok, 52.47 s, so the per-test differing set equals `RECORDED` and each whole-group run without it
is identical:

* FUNCTION: `TEST_INPUT_OUTPUT_STREAM`, `TEST_REXXQUEUE`
* METHOD: `TESTCLASS01`, `TESTDROPOBJECTVARIABLE01`, `TESTGETOBJECTVARIABLE01`,
  `TESTGETOBJECTVARIABLE02`, `TESTGETOBJECTVARIABLE03`, `TESTGETPACKAGECLASSES01`,
  `TESTGETPACKAGEPUBLICCLASSES01`, `TESTGETPACKAGEPUBLICROUTINES01`, `TESTGETPACKAGEROUTINES01`,
  `TESTNEWMETHOD02`, `TESTNEWROUTINE02`, `TESTSETOBJECTVARIABLE01`
* CONVERSION: none

Owners: Phase 9 for all but `TEST_REXXQUEUE`, Phase 10.

## Negative controls

Corpus runs: `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (`$S/ctl/corpus.sh`),
630 of 630 unmutated. Every witness but N6's went red alone, so each adds coverage the rest of the
corpus did not have.

Predictions written 2026-09-28 before any control ran. Each mutated file is copied first and
restored from the copy, checked with `cmp`.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| N1 | `api_group_tests.rs`: `METHOD.TESTCLASS01` removed from `RECORDED` | red; newly failing `["METHOD.TESTCLASS01"]`, newly passing `[]` | as predicted: `newly failing: ["METHOD.TESTCLASS01"]`, `newly passing: []`; restored, `cmp` clean |
| N2 | `api_group_tests.rs`: `METHOD.TESTCLASS02` added to `RECORDED` (a real test that passes; the string occurs nowhere in `phase-4-exclusions.txt`, `grep -c` 0) | red at the record check, `METHOD.TESTCLASS02 has no record in phase-4-exclusions.txt`, before any run | as predicted: `METHOD.TESTCLASS02 has no record in phase-4-exclusions.txt`, 0.00 s; restored |
| N3 | `environment.rs` `package_string_table`: the shared-object branch never taken (`routine` read as `false` there) | STRICT corpus red on `library_routine_shared_object.rex` alone; instrument red, newly failing `["FUNCTION.TESTGETROUTINE01"]` alone | as predicted: corpus 629 of 630, `lang/library_routine_shared_object.rex` (stdout); instrument `newly failing: ["FUNCTION.TESTGETROUTINE01"]`, `newly passing: []`; restored |
| N4 | `run/condition.rs` `offer_to_trap`: `Search::Top` back to "outermost only" (`activation_depth() > 1`) | STRICT corpus red on `raise_syntax_nearest_top_level.rex` alone | as predicted: 629 of 630, `lang/raise_syntax_nearest_top_level.rex` (all three descriptors); restored |
| N5 | `object_protocol.rs` `native_message_send`: a new receiver argument ignored | STRICT corpus red on `message_send.rex` alone | as predicted: 629 of 630, `lang/message_send.rex` (`o 10 20` for `x 10 20`); restored |
| N6 | `eval.rs` `operator_message_receiver`: the `Body::Array`/`Body::Native` arm removed | STRICT corpus red on `operator_native_receivers.rex`, and on nothing else | **falsified in part**: 628 of 630, `operator_native_receivers.rex` and `library_routine_shared_object.rex`, which compares Routine objects with `==` (I did not predict that); restored |
| N7 | `run.rs` `assign_expr_target`: the `ExprKind::Message` arm removed (falls to the loud arm) | STRICT corpus red on `parse_message_targets.rex` alone | as predicted: 629 of 630, `lang/parse_message_targets.rex` (rc 120, `a message send is not implemented`); restored |
| N8 | `condition.rs` `build_condition_object_from`: `native_instance` again instead of `NEW` | STRICT corpus red on `condition_object_directory.rex`; `raise_array_additional_objects.rex` too only if it reads the object past `AT` (it reads `~additional`, `~message`, `~code`: predicted green) | as predicted: 629 of 630, `lang/condition_object_directory.rex` (`method "ITEMS" of class "Directory"`); `raise_array_additional_objects.rex` green; restored |
| N9 | `run/condition.rs` `exec_raise`: `pending_additional` not set for the `ARRAY` form | STRICT corpus red on `raise_array_additional_objects.rex` alone | as predicted: 629 of 630, `lang/raise_array_additional_objects.rex`; restored |
| N10 | `string.rs` `native_string_makearray`: separator ignored | STRICT corpus red on `string_makearray_separator.rex` alone | as predicted: 629 of 630, `lang/string_makearray_separator.rex` (stdout); restored |
| N11 | `object_protocol.rs` `native_copy`: a heap string answers itself instead of a copy | STRICT corpus red on `string_copy.rex` alone (the IdentityTable line, `0 1` becoming `1 1`) | as predicted: 629 of 630, `lang/string_copy.rex` (stdout); restored |
| N12 | `rexx-api/src/redirect.rs` `Sink::write_buffer`: a buffer with no line end no longer extends a pending tail (`pending.extend_from_slice(data)` removed) | instrument: the per-test part green (`test_write_buffer` fails identically on both sides alone, no `io` handler), the whole-group part red for FUNCTION alone (`Failures: 1` against `0`) | as predicted: the newly-failing/passing assertion held and the whole-group assertion failed on `FUNCTION` alone, oracle `Failures: 0` rc 0, ours `Failures: 1` rc 1; restored |

## Commits

* `9dae74fac` Answer what the ooTest framework reaches in a single-group run
* `8fb784e2d` Give a library procedure one Routine object, the one GetRoutine answers
* `dc862d296` Run the Phase 8 API groups test by test against the oracle
* `2b26b6970` Run each Phase 8 API group whole as well, without its recorded tests

This report and the queued files are under `.superpowers/`, which `.gitignore` excludes.

## Performance-sensitive paths

Touched: `run.rs` (`assign_expr_target`, a new match arm reached only for a target no plan slot
binds), `run/condition.rs` (`offer_to_trap`'s `Search::Top` arm and `exec_raise`'s ARRAY branch,
both on the raise path) and `eval.rs` (`operator_message_receiver`, one more arm in the heap-body
match an operand on a heap handle already reaches). No per-clause work. Callgrind, BASE binary
against `dc862d296`'s, `bench-programs/`, `Collected` instructions: `strings` 19,010,017,092 /
19,010,042,588; `dispatch` 22,813,976,998 / 22,839,004,256 (+0.11%, inside the layout band);
`parse` 1,646,215,259 / 1,646,215,842.

## Gates

`$S/gates.sh` (surface-6's with `S` changed), run in the background at `2b26b6970`, tree untouched
until `finished`; `$S/gates/status.txt`:

```
2b26b69705ce7ec848f09d9fee7fce6ea0231181
started 2026-09-28T09:31:24+02:00
load at start 3.27 6.60 7.33 7/2419 1545385
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 12.01 18.63 12.89 3/2425 1549123 2026-09-28T09:36:09+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 6.30 11.09 11.43 5/2427 1676929
G5 debug build (test --no-run) exit 0
load G6 7.77 11.07 11.42 3/2420 1681952 2026-09-28T09:42:33+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 10.40 14.58 13.36 8/2425 1809735
2b26b69705ce7ec848f09d9fee7fce6ea0231181
finished 2026-09-28T09:49:16+02:00
```

`git status --short` empty at the end (nothing between the second sha and `finished`). Summed
`test result` lines: G4 2752 passed / 0 failed / 4 ignored, G6 2753 / 0 / 4; corpus `630 of 630
matching` in both; `every_test_of_the_phase_8_groups_matches_the_oracle_but_the_recorded ... ok`
in both. No re-run was needed.

## Concerns

* **The harness passes `-U` and `-V 0`.** The single-group form with default options cannot run on
  the crate (Phase 6's ticker, Phase 10's `RXFUNCQUERY`), and its oracle side reaches rxapi. If the
  gate must run the default form, it waits for Phases 6 and 10.
* **Tests whose isolated runs fail identically on both sides** (the FUNCTION io tests, and
  METHOD's `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, where `rexxc` is not on PATH on either
  side, RC 127) are covered only by the whole-group part, and only in aggregate: a divergence there
  shows as a count, not a name.
* **`TEST_INPUT_OUTPUT_STREAM`'s record rests on a standalone reproduction** (`$S/p/iostream.rex`,
  the group's own classes with the handler registered: oracle `2 2`, crate 97.1 `STATE`), because
  inside the per-test run the oracle fails it too (no handler) and inside a whole-group run the
  crate's result is not attributable by name.
* **Rows owned by Phase 8 remain in `phase-4-exclusions.txt`** from Task 5 (the native frame and
  `PROPAGATED`, `POSITION` on a trapped `RaiseCondition`, a blocking member on a busy outer context,
  Directory-subclass `AT`/`PUT`; `grep -n -i "owner: phase 8"`). None is a member of these groups'
  failing set; progress.md gives them to Task 9. No record this task wrote names Phase 8.
* **Scope of the runtime fixes.** Each is general runtime the framework needed, fixed because it
  was local; the ruling allowed it. The operator change widens what is sent rather than refused
  (every `Body::Native` class): a native class with an operator this crate has no row for would
  now resolve through lookup, which refuses loudly for a name without a native body (D37), so no
  silent answer was found, but it was not enumerated per class.
* **Pre-existing, found on the way, not changed:** a `loadExternalRoutine` object that no
  directive binds still has no annotation table (`~annotation` refuses loudly).
* The gate run started at `dc862d296` was stopped after G3 (fmt, clippy, release build exit 0,
  `$S/gates-r0-aborted/`) when the whole-group part was added; the gates below are the rerun.

