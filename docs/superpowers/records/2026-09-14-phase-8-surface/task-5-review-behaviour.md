# Task 5 review: behaviour against the oracle, and the tests

Reviewer: review-s5a (behaviour half). HEAD fb52b794e, built into my own target dir (release
`rexx-run`). Scratch `$S` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s5a/`.
Comparisons use `$S/cmp.sh` (the standard oracle wrapper from a fresh `mktemp -d`, and stdout, stderr and rc
compared separately); probes are in `$S/p/`, outputs in `$S/out/`, the base binary in `$S/bin/rexx-run-base`.

## Verdict

**Not clean: four Important findings and several Minor ones.** All eight witnesses and every forge probe the
report calls identical reproduce as identical. Two behaviours the METHOD group checks are wrong, and
one of them (a stem as an object variable) is also wrong through Task 5's own `GetObjectVariable`. The
Throw* members do not "refuse" the way a Rexx program sees a refusal. They **abort the process**, so each one
takes the whole ooTest run down with it. Control N6a's "unobservable" conclusion is false. None of the six
recorded divergences has an entry in `phase-4-exclusions.txt`.

## 1. Corpus witnesses

`$S/cmp.sh rust/corpus/lang/library_callback_*.rex`: all 8 are identical on stdout, stderr and rc (rc 0).
Each one calls the members it names through orxmethod/orxfunction (I read each file against the
header). The forge probes (`task-5-forge/probes/*.rex`, forge rebuilt with its `build.sh`, NEEDED
libc only) give the report's result: all identical except `cond2` (the `StackFrame~EXECUTABLE` refusal), `cond3`, `rcond`
and `refs` (oracle rc 139).

**Minor.** `cond3.rex` never measures the divergence it is cited for. On the crate it stops at the
`EXECUTABLE` refusal (rc 120) before it prints any key. `$S/p/c3b.rex` (the same program without `~executable`)
shows the actual divergence: the native `*-* Compiled routine` traceback line and the ROUTINE stack frame are missing, and
`PROPAGATED = 0` where the oracle has 1.

## 2. Sampled members vs the oracle (failure paths)

Identical: `s2.rex` (buffer length and capacity at 0 and past capacity, `ObjectToCString(.nil)`,
empty strings), `mem2.rex` (alloc 0, realloc smaller, realloc from another object), `n2.rex`
except for one line, `n6.rex`, `unt.rex`/`untf.rex` (an untrapped native raise, method and routine).

Divergences (probe, oracle -> crate):
- **Important: `ArrayDimension` of an empty array.** `co2.rex`: `.array~new` gives 1 -> 0. The METHOD group asserts 1
  (`METHOD.testGroup:1471`, TestArrayDimension01). This is the cost of "collection operations are
  message sends": Rexx `~dimension` answers 0 on both sides, but the C++ API answers 1.
- **Important: a stem as an object variable.** `v4.rex`: with `st.` set by `expose st.; st. = 'd'`,
  `GetObjectVariable('st.')` gives a Stem -> NULL (.nil). `SetObjectVariable('ST.', aStem)` does
  not replace the exposed stem (`st.q` stays `qq`; the oracle gives `ZQ`). The METHOD group's
  test_GetObjectVariableReference (`METHOD.testGroup:3039-3044`) fails because of this: the reference value is `[STEM.]`
  where a Stem is expected. The get side is Task 5's (`surface.rs:370`). The set side
  (`library.rs:539`) predates Task 5, but the Task 5 member reaches it.
- **Minor: class search from a method context.** `fc.rex`: a non-public class in the method's own
  package. The oracle finds `The HIDDEN class`; the crate raises 88.900. A plain Rexx program with
  orxmethod can see this (it is the report's "caller activation" divergence).
- **Minor: collection members on subclasses.** `ov.rex`: `DirectoryAt`/`DirectoryPut` on a Directory subclass that overrides
  `AT`/`PUT` gives `v` / `.nil 2` -> `overridden dat` / `w 2`. Array subclasses agree.
- **Minor: RaiseException with an unknown error number.** `cd2.rex`: 99999, 0 and 40 give 98.941 "Unknown
  error number specified on RAISE SYNTAX" -> 99.999, 0.0 and 0.40 with `<no message ... in the catalogue>`.
- **Minor: a SYNTAX condition that passes through SendMessage.** `m2.rex`: `PROPAGATED` 1 -> 0 (the `cond3`
  class). `c3m.rex` shows that the METHOD group's own `forward ... continue` shape is also missing
  the native METHOD frame.
- Minor, Task 2's member: `DoubleToObjectWithPrecision(2/3, 0)` gives `0` -> `6.7E-1` (`n2.rex`).
- Not Task 5: `pk2.rex`. Loading a package or creating a routine from source that does not parse ends the
  crate with a Phase 5 Loud refusal (rc 120), where the oracle raises 35.929 and continues.

## 3. Recorded divergences: observability and ownership

`git log e67b2f703..fb52b794e -- docs/superpowers/plans/phase-4-exclusions.txt` is empty. Searching that file for
PROPAGATED, ObjectToValue, "side area", RaiseCondition, FindContextClass, ThrowException and
GetObjectVariable finds nothing. **Important: none of the six recorded divergences, and not the Throw*
ruling, has an entry with an owner.**
- cond3 native frame and PROPAGATED: a Rexx program can see it (`condition('o')~propagated`, `~traceback`,
  `~stackframes`). No METHOD, FUNCTION or CONVERSION source reads those three.
- rcond POSITION: visible to a Rexx program. The groups never read it.
- class search: visible to a Rexx program (`fc.rex`). The groups' TestFindContextClass tests pass.
- ObjectToValue partial value: orxmethod raises and answers NULL on failure, so only a custom extension can see it.
- message sends: visible to a Rexx program, **and the METHOD group catches it** (ArrayDimension, section 2).
- buffer side area: I found no way for a Rexx program to observe it.

**Measured group reach.** I ran every test method of the three groups as its own process on both
sides. The harness is `$S/ot/run.sh`: a stub ooTestCase that prints every assertion, prepended to the group's class
text. Results: CONVERSION 97/97 identical, METHOD 296 identical / 20 differ / 4 Throw skipped, FUNCTION
148 / 6 / 4. Task 5's own differences: TestArrayDimension01 and test_GetObjectVariableReference.
The rest are refusals outside Task 5, and each one ends the process with rc 120:
- `==` on an Array or on the interpreter's own objects: testObject01, testArray01, testGetMethod01,
  TestGet{Routine,Method}Package01, FUNCTION TestGetRoutine01
- Class~new: testClass01
- `EXPOSE` of a compound tail: test{Get,Set,Drop}ObjectVariable01-03
- StringTable~items: TestGetPackage*01
- parse-error reporting: TestNew{Routine,Method}02
- AddCommandEnvironment (Task 6)
- two io tests that depend on test order (artefacts of running each test alone), and a rexxc test that is environmental

## 4. Controls N1-N8 and N6a

I built one mutant from a `git archive` copy with two mutations and wrote the predictions (below) before the run.
- N8 (reallocate without the copy): `library_callback_memory` stdout `0 hello` -> `0 `, the only
  difference. **Reproduced.**
- N6a (no upper-casing in `send_message`, `callbacks.rs:1309`): `library_callback_messages` stayed
  identical, which reproduces the report's green result. **The report's conclusion is false (Important).**
  `n6.rex` under the mutant gives UNKNOWN's name argument `foo`/`fOo` for `FOO`,
  `.context~name` gives `whoami` for `WHOAMI`, and 97.1's message names `"nosuchmethod"` for
  `"NOSUCHMETHOD"`. A program can see the name's case in four ways. The messages witness is blind: it
  prints 97.1's code, never its message. The report's line "no Rexx program can send a lower-case
  name, so the mutant is unobservable" needs correcting, and the witness needs a case that sees the name.

### Predictions written before the mutant run (one binary, two mutations in an archive copy)
- M-N8: drop `data[..bytes.len()].copy_from_slice(&bytes);` in surface.rs reallocate_object_memory.
  Predict: library_callback_memory differs on stdout at the `0 hello` line only.
- M-N6a: `let name = name.to_ascii_uppercase();` -> `name.to_vec()` in callbacks.rs send_message.
  Predict: library_callback_messages identical (report's result); my n6.rex differs
  (UNKNOWN's name argument `foo` for `FOO`, and 97.1's message naming `nosuchmethod`).


## 5. Oracle crashes (concerns 3, 4)

I ran each of these once, in a scratch directory:
- Concern 3: `task-5-forge/probes/refs.rex` gives oracle rc 139 with the same stdout as the crate (crate rc 0).
  **Reproduced.**
- Concern 4: `$S/p/crash4.rex` (orxmethod `TestRaiseCondition('USER BAR', 'a description', ...)`, then a
  `CALL ON` handler that reads `condition('d')`) gives oracle rc 139. **Reproduced.** The crate prints `handler ` with an empty
  description and exits with rc 0.
- New: `$S/p/crashsyn.rex`, which is `t~rc('SYNTAX')` through orxmethod's TestRaiseCondition with no
  other argument, gives oracle rc 139 after `before`. The crate reports `Error 0 ... <no message 0.0 in the catalogue>`.

**Important: all three belong in `rust/corpus/oracle-crashes.txt`.** That file exists so nobody reruns a crasher. `refs.rex`
is a committed probe that `compare.sh` runs by glob. The other two are reachable from the shipped orxmethod,
the extension the corpus witnesses already load.

## 6. Throw* members: which ooTest tests reach them

- METHOD: TestThrowException001, 101, 201 and 01 (`METHOD.testGroup:2018-2090`, through
  `METHODPackage.cls:327-357`).
- FUNCTION: the same four (`FUNCTION.testGroup:1803-1860`, through `ThrowException*X`, `:1928-1980`).
- No group calls ThrowCondition.

**Important: the cost of the ruling is larger than those 8 tests.** The slots are `aborts`
(`layout.rs:801-805, 829-833`). `$S/p/thr.rex` and `thrf.rex` panic "cannot unwind" and exit with **rc 134 (SIGABRT)**. ooTest loads every
group into one process (`ooTest.frm:2104`, `call (file)`), so the first Throw test kills the METHOD
or FUNCTION run and discards the results of every group already run. The report describes this as "left refusing"
and never says "aborts".
