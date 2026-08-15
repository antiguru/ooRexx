# Task 10: `builtin/state.rs` -- report

The eleven state-reading builtins are implemented, `builtin-status.txt` moves all eleven from `loud` to `implemented` with none `divergent`, and the two corpus witnesses the task owed are in place.
Two divergences that the brief's design did not anticipate were found by probing and fixed before they shipped, and three option letters answer an object this crate's value model has no representation for and fail loudly instead.

## What was built and where

All paths are absolute under `/home/moritz/dev/repos/ooRexx-rust-rewrite`.

| File | Change |
|---|---|
| `rust/crates/rexx-exec/src/builtin/state.rs` | New. The eleven builtins, `DEFAULT_ENVIRONMENT`, `option_letter`, `rendered`, and 21 unit tests. |
| `rust/crates/rexx-exec/src/builtin/mod.rs` | `mod state;` plus eleven `Builtin` rows with their `(min, max)` from `BuiltinFunctions.cpp`. |
| `rust/crates/rexx-exec/src/trace.rs` | `TraceMode` gains `letter: u8`; four new constants (`NORMAL`, `COMMANDS`, `ERRORS`, `FAILURES`); `mode_from_setting` maps all nine letters to nine distinct answers and an empty setting to `NORMAL`. |
| `rust/crates/rexx-exec/src/activation.rs` | `Trap` gains `delayed`; new `TrappedCondition`; `Activation` and `Inherited` gain `condition`; a fresh activation starts at `TraceMode::NORMAL`. |
| `rust/crates/rexx-exec/src/run.rs` | `trap_for` skips a delayed trap; `offer_to_trap` and `deliver_pending_trap` write the activation's `TrappedCondition`; `deliver_pending_trap` delays and undelays instead of removing and re-inserting; `exec_raise` keeps the `DESCRIPTION` value; `Trace::Default` is `NORMAL`; the callee inherits `condition`. |
| `rust/crates/rexx-exec/src/error.rs` | `Raised::description`; four raisers: `argument_not_positive` (40.14), `sourceline_out_of_range` (40.34), `argument_out_of_range` (40.903), `argument_not_in_list` (40.904). |
| `rust/crates/rexx-exec/src/lib.rs` | `Loud::builtin_option_object`; `PendingTrap::description`. |
| `rust/crates/rexx-exec/src/queue.rs` | `Queue::len`. |
| `rust/corpus/builtin-status.txt` | Eleven rows `loud` -> `implemented`, copied from the harness's own derived table. |
| `rust/corpus/lang/address_env.rex` | Extended with the `D0`, `D` and `E` blocks. |
| `rust/corpus/lang/state_builtins.rex` | New. |
| `rust/corpus/phase-4c.txt` | Registers `state_builtins.rex`; the ADDRESS paragraph corrected. |
| `rust/corpus/builtin-probes.txt` | Header corrected where it assigned the platform default to Phase 7. |
| `rust/corpus/keyword-exempt.txt` | 111 rows -> 16; two attributions changed; header corrected. |
| `rust/crates/rexx-exec/tests/builtin_status.rs` | `STATE_FAMILY` + `every_state_builtin_is_implemented`. |
| `rust/crates/rexx-exec/tests/coverage.rs` | `EXPECTED_SUBSET_4C` gains the new program. |
| `rust/crates/rexx-exec/tests/keyword_assertions.rs` | The attribution vocabulary admits the harness's own two derived tokens. |
| `rust/crates/rexx-parse/tests/sourceline_oracle/{address_env,state_builtins}.txt` | Regenerated / new. |
| `docs/superpowers/plans/phase-4-exclusions.txt` | Three rows corrected (below). |

## The `None`-rendering decision

`ADDRESS()` renders `current == None` as the platform default name, a `#[cfg]`-selected constant: `sh` on unix, `CMD` otherwise.

**The precedent it was weighed against.** `PARSE SOURCE`'s first two words are `LINUX COMMAND` on this host and sit in the corpus as measured constants with the host-dependence flagged.
`PARSE VERSION` carries the oracle's build date and is deliberately kept out of every corpus program, pinned instead by a gated differential test, because a build date moves under a rebuild and a platform name does not.

**Which of the two `sh` is, checked in the C++ rather than assumed. The citation below is fix round 1's; the one first shipped was wrong in three copies.** `InterpreterInstance.cpp:201` sets `defaultEnvironment = SystemInterpreter::getDefaultAddressName()`. **Both platforms' bodies are the identical one line** -- `platform/unix/SystemCommands.cpp:195` and `platform/windows/SystemCommands.cpp:207` each `return GlobalNames::INITIALADDRESS;` -- and `memory/GlobalNames.h:124` is `GLOBAL_NAME(INITIALADDRESS, SYSINITIALADDRESS)`. The platform split is therefore in `PlatformDefinitions.h`:

```
interpreter/platform/unix/PlatformDefinitions.h:66:    #define SYSINITIALADDRESS "sh"
interpreter/platform/windows/PlatformDefinitions.h:65:  #define SYSINITIALADDRESS "CMD"
```

The first report cited `platform/unix/SystemCommands.cpp:68`, which does carry a `#define SYSINITIALADDRESS "sh"` -- but it is local to that translation unit and is **not** the definition `GlobalNames.h` expands. Same value, wrong provenance: a citation that could change without changing the answer. `sh` and `CMD` were and remain correct.

So the value is fixed when the interpreter is built and read from no process, environment variable or file -- the same shape as `LINUX`, not the shape of a build date. It is therefore in the corpus, as `corpus/lang/address_env.rex`'s `D0` line.

**Why implementing it at all is a change to the exclusions file rather than a violation of it.** That file's `ADDRESS()` row said "The platform-supplied default is Phase 7's -- ... which comes from the layer D18 defers."
The reason is what fails: it conflates *naming* the default environment with *issuing a command to* it, and only the second needs the command layer.
Deferring the name also made the corpus witness the same row's own enumeration says `ADDRESS()` is owed impossible to write, because a callee's inherited alternate is the default in the simplest shape. The row is corrected and says what moved and why; issuing a command stays Phase 7's.

## Measurements

Every oracle run below was wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
from a fresh empty directory under the session scratchpad, with `</dev/null`, stdout, stderr and exit status read as three separate descriptors.

### ADDRESS: the default, the swap, and inheritance

Program `p1.rex`, `say '['address()']'` interleaved with `address envA`, `address envB` and four bare `ADDRESS`es. Output:

```
[sh]
[ENVA]
[ENVB]
[ENVA]
[ENVB]
[ENVA]
[ENVB]
```

Program `p2.rex`, `address outer` in the main body and an internal `sub`:

```
main1 [OUTER]
 sub1 [OUTER]
 sub2 [sh]
 sub3 [INNER]
main2 [OUTER]
main3 [sh]
```

### The simple readers

Program `p3.rex`, run with no argument:

```
digits 9        fuzz 0        form SCIENTIFIC
digits 12       fuzz 3        form ENGINEERING     (after NUMERIC)
gc 0
trace [N]
queued 0
sourceline 18
sl1 [say 'digits' digits()]
errortext5  [System resources exhausted.]
errortext0  []
errortext99 [Translation error.]
arg 0
```

### TRACE(): the letters

`p5.rex`, each setting followed by `say '...' trace('O')`:

```
default [N]   off [O]   all [A]   c [C]   e [E]   f [F]   i [I]   l [L]   n [N]   r [R]
setter returns [O] [L]
```

`p8.rex`: bare `trace` reports `N`, `trace value ''` reports `N`, and `trace r` then `trace value 'O'` reports `O`.

`p6.rex`: `trace 5` is `Error 24.901 Numeric TRACE requests are valid only from interactive debugging.` at rc 232.

`p7.rex` and `p9.rex`, the `?` prefix: `trace ?r` then `trace()` is `?R`; `trace l` then `trace ?` then `trace()` is `?L`, twice over (the second bare `?` does not toggle it back).

### ARG

`a1.rex`, run as `rexx a1.rex arghere`, with `call sub 'p1',,'p3'`:

```
top n 1        top 1 [arghere]   top 1E [1]   top 1O [0]   top 1N [arghere]
sub n 3
sub 1 [p1] 2 [] 3 [p3] 4 []
sub E 1 0 1 0
sub O 0 1 0 1
sub N [p1] [] [p3] []
sub e-lower 1 0
```

With no argument at all: `noargs n 0 [] 0 1`.
`arg(1,'A')` with one argument prints `zz` and `condition('A')~class` for it is `The Array class`.

### The `CONDITION()` option table, measured from inside a live handler

Every column below is one probe with the option read inside the handler, never at the top level.

| option | SYNTAX (`say 1/0`) | NOVALUE | USER via SIGNAL ON | USER via CALL ON | outside a handler |
|---|---|---|---|---|---|
| `CONDITION()` | `SIGNAL` | `SIGNAL` | `SIGNAL` | **`CALL`** | `''` |
| `A` | `<Array 0>` | `.NIL` | the `ADDITIONAL` value | same | `.NIL` |
| `C` | `SYNTAX` | `NOVALUE` | `USER UC` | same | `''` |
| `D` | `''` | `ZUNSETVAR` | the `DESCRIPTION` value | same | `''` |
| `E` | `3` | `''` | `''` | same | `''` |
| `I` | `SIGNAL` | `SIGNAL` | `SIGNAL` | **`CALL`** | `''` |
| `O` | `a Directory` (14 items) | `a Directory` (9) | `a Directory` (10) | same | `.NIL` |
| `S` | `OFF` | `OFF` | `OFF` | **`DELAY`** | `''` |
| `R` | `''`, and clears | same | same | same | `''` |

Additions to the brief's table, all measured:

* `E` is the part of the `CODE` item after the dot. `raise syntax 40.4` gives `4`; `raise syntax 40` and `raise syntax 5` both give `0`, so a missing sub is a real zero rather than an absence; `HALT`, `ERROR`, `FAILURE`, `NOVALUE` and `USER` all give `''`. `Activity::createExceptionObject` is the only place a `CODE` item is put in, and it is the SYNTAX path.
* `A` for `say substr('abc')` trapped is a two-element `Array` holding `SUBSTR` and `2` -- our `Raised::additional` -- and `say` renders an `Array` newline-joined. For `raise user uc array ('one','two')` it is a two-element `Array`; for `raise user uc` with nothing it is `.NIL`; for `raise user uc additional 'my add'` it is the plain string.
* After a `CALL ON` handler returns, `condition()` in the caller is `''`: the condition object was the handler activation's.
* `condition('R')` inside a subroutine called from the handler clears only that subroutine's copy -- the handler still reports `SYNTAX` when it resumes.
* A `::routine` called from a handler sees nothing at all (`C`, `I`, `S` all `''`), which matches the C++ constructing a fresh settings block for one. Not reachable from this crate today.
* `condition('')` and `condition('Z')` are both 40.904 at rc 216; `condition('cond')` behaves as `condition('C')`.
* `CALL ON SYNTAX` is a **translation-time** 25.1, so the whole program fails before running. One probe was lost to this and re-measured.

### Whether `I` and `S` can diverge -- run, not reasoned

They diverge. `S` is not stored beside `I`: `BUILTIN(CONDITION)`'s `'S'` arm calls `context->trapState(conditionobj->get(CONDITION))`, and `RexxActivation::trapState` looks the condition's own name up in the **live** trap table, returning `OFF` when absent and `TrapHandler::getState()` otherwise.

Measured, `c6.rex`, inside a `SIGNAL ON SYNTAX` handler:

```
before rearm I [SIGNAL] S [OFF]
after  rearm I [SIGNAL] S [ON]        (signal on syntax name sh1)
after  off   I [SIGNAL] S [OFF]       (signal off syntax)
R returns []
after  R     I [] S [] C [] A [The NIL object] O [The NIL object]
```

and the mirror, `c7.rex`, inside a `CALL ON USER UC` handler:

```
in CALL handler S [DELAY] I [CALL]
after re-arm   S [ON]    I [CALL]     (call on user uc name uh)
after off      S [OFF]   I [CALL]     (call off user uc)
after signalon S [ON]    I [CALL]     (signal on user uc name uh)
```

So both fields are carried, and they are carried *differently*: `TrappedCondition::call` is stored when the trap fires, and `S` is computed at every call from `Activation::traps`.
This also forced a change to how a `CALL ON` handler holds its own trap. The crate removed the trap for the handler's duration and re-inserted it afterwards; the oracle sets a `DELAYED` state (`TrapHandler::disable`/`enable`). Removal reports `OFF` where the oracle reports `DELAY`, so `Trap::delayed` replaces it and `Interp::trap_for` filters on the flag.

**The second benefit this paragraph originally claimed -- that removal "resurrects a trap the handler disabled" -- is false, and fix round 1 retracts it.** A handler runs as a nested activation with a *copy* of the trap table, so its `CALL OFF` never reaches the caller's table and there is nothing to resurrect. Measured two ways: on the oracle, a handler whose first act is `call off user uc` is entered again on a second raise (and this crate matches, byte for byte); and with the removal restored here, the same program prints identically except that `CONDITION('S')` reads `OFF` where it should read `DELAY`. `CONDITION('S')` is the whole of the observable difference. The corrected statement is in `Trap::delayed`'s doc, in `deliver_pending_trap`'s comment and in `corpus/lang/state_builtins.rex`'s F-block header; commit `f03d69f1`'s message carries the superseded claim and cannot be edited, which `Trap::delayed`'s doc says.

### Two divergences found by probing the *order* of checks

Both were in code that had already passed its own unit tests, and both were found by a differential sweep rather than by reasoning.

**`ARG`'s three checks.** Measured, rc 216 in all four rows:

```
arg('x','q')   40.12  ARG argument 1 must be a whole number; found "x".
arg(,'')       40.5   Missing argument in invocation of ARG; argument 1 is required.
arg(0,'')      40.14  ARG argument 1 must be positive; found "0".
arg(1,'')      40.904 ARG argument 2 must be one of AENO; found "".
```

The first implementation validated the option's emptiness while reading it, which gave 40.904 for `arg(0,'')` and `arg(,'')`. The rejection belongs to the option *switch*, which a bad position never reaches. Rows three and four are the pair that pins it: the same two arguments, one changed, opposite answers.

**`TRACE(setting)` against the `TRACE` instruction.** Measured, rc 232 for both:

```
trace value 5     24.901  Numeric TRACE requests are valid only from interactive debugging.
say trace('5')    24.1    TRACE request letter must be one of "ACEFILNOR"; found "5".
```

`RexxInstructionTrace::execute` tests for a whole number before parsing a setting; `BUILTIN(TRACE)` goes straight to `RexxActivation::setTrace(RexxString *)`, which does not. The first implementation reused the instruction's numeric check and gave 24.901 for both. Also measured: `trace('-3')` reports `found "-"`, the first byte and not the string.

### The differential sweep, with its alphabet

**RETRACTED AS A COVERAGE CLAIM BY FIX ROUND 1. The sixty programs and the three differences are both correct; what follows them is not.** Re-run after the round-1 fix, the same sixty still report 57 identical and the same 3 differing -- so the sweep was never wrong about what it ran. It was blind, and the alphabet published beside it concealed that: it names `1.0`, `1.5` and `99.0`, which reads as coverage of the numeric axis, while not one of those values reaches a message that substitutes anything. Twelve live divergences sat inside the swept surface. A count with its alphabet is still not a coverage claim when the alphabet lists values *supplied* rather than values that *reach each code path*. The sweep has been committed, with the missing crossing, as `crates/rexx-exec/tests/state_builtin_oracle.rs`.

60 single-clause programs run through both interpreters, comparing stdout, stderr and exit status as three separate descriptors: 57 identical, 3 differ and all three are the declared loud gaps below.

**Alphabet drawn from**, stated beside the count because a count without one is not a coverage claim: option letters in upper and lower case and as whole words (`'A'`, `'e'`, `'exists'`, `'cond'`, `'Fnord'`); the **null string**; a **byte >= 0x80** (`'80'x`, `'ff80'x`); **control bytes** (`'00'x`, `'09'x`); digit strings as options (`1`, `'5'`, `'-3'`, `'0'`); numeric arguments at and either side of every boundary (`-1`, `0`, `1`, `2`, `1.0`, `1.5`, `99.0`, `100`, `2000000000000`); arity at both ends of every row; and an environment name containing a blank.

The three that differ:

```
say arg(1,'A')      oracle rc 0, empty        rexx-exec rc 120, ARG option "A" answers an Array
say condition('A')  oracle rc 0, The NIL...   rexx-exec rc 120, CONDITION option "A" answers an Array or .NIL
say condition('O')  oracle rc 0, The NIL...   rexx-exec rc 120, CONDITION option "O" answers a Directory
```

## What is loud, and the correction to the brief that goes with it

The brief's Step 0 says "`A`, `C`, `D` and `O` come from the existing condition object and only `I`/`S` need new state." That is true of the *oracle's* condition Directory and false of this crate, which has no condition object at all -- `Raised` carries `condition`, `number`, `sub`, `additional` and `rc`, and nothing else. Three consequences, and the plan should say so for whoever revisits `CONDITION`:

* **`A` and `O` need objects that do not exist here.** `Body::Array` exists in `rexx-core` but nothing produces one and `Interp::to_text` reaches `unreachable!` on it; there is no `Directory` at all. Both fail loudly through `Loud::builtin_option_object`. Returning `''` would be right for exactly one of the four shapes an `A` answer takes (the empty `Array`) and wrong for the other three.
* **`D` needed new state and got it.** `exec_raise` evaluated the `DESCRIPTION` expression, traced it and dropped it. `Raised::description` and `PendingTrap::description` now carry it, so `D` is correct for a `RAISE ... DESCRIPTION`, for a `RAISE` without one, for an interpreter-raised `SYNTAX` and outside a handler.
* **`D` for `NOVALUE` is the one remaining pair, and it is loud.** The oracle answers the variable's derived name (`ZUNSETVAR`, measured) and nothing on the read path passes a name as far as `novalue_check`. `only_novalues_description_is_loud` pins it, together with its adjacent success -- the same option, the same handler shape, a `SYNTAX` condition, answering `''`.

A fourth correction, to the brief's own transcript: it renders `CONDITION('A')` for a SYNTAX condition as `<Array 0>`. Measured here, `say condition('A')` after `say 1/0` prints an empty line, because an empty `Array` renders as nothing; after `say substr('abc')` it prints two lines. The brief's notation was describing the object, not the bytes.

## The `TRACE()` decision, and the exclusions row it changes

`TraceMode`'s four booleans are lossy in exactly the direction `TRACE()` needs: `C`, `E`, `F`, `N` and `O` all leave every one of them false while the oracle reports five different letters, and the pre-`TRACE` initial state reports `N` where `TraceMode::OFF` would say `O`.
So `TraceMode` gained a `letter: u8` and five silent constants replaced one.

The `?` prefix is **not** carried, and that is a decision rather than an oversight. `phase-4-exclusions.txt` already has a `TRACE ?` row that records the prefix as "silently ignored", with its own owner, and that row's own argument against a partial fix applies here: reproducing the prefix in the *rendering* alone would make `trace()` report a mode nothing else honours, which is the "a fix that hides the remaining half" trade it describes for the stdin drain. Two measured consequences are now recorded in that row:

```
trace ?r ; say trace()            oracle ?R    rexx-exec R
trace l ; trace ? ; say trace()   oracle ?L    rexx-exec O
```

The second is the wider half: a bare `?` is the oracle's debug *toggle*, which keeps the current setting, where `mode_from_setting` has no current mode to keep and resets. Closing either half needs the interactive flag as a field and the current mode as an input to the classifier -- that row's work, not this one's.

That row also said "stdout and the exit code match; only stderr differs", which `TRACE()` makes false; and the deviations section said "TRACE() IS READABLE BY THE PROGRAM ... IS NOT A PROPERTY THIS CRATE HAS", which is true again. Both are corrected in place.

## The `ADDRESS` corpus witness, and what it now asserts

`corpus/lang/address_env.rex` keeps blocks A, B and C unchanged and gains `D0`, `D` and `E`. Oracle output for the whole program, matched byte for byte on all three descriptors:

```
D0 default sh
A done
B done
D1 ENVD2   D2 ENVD1   D3 ENVD2   D4 ENVD1   D5 ENVD2
E1 main ENVE2
E2 sub ENVE2
E3 sub ENVE1
E3b sub ENVE3
E4 main ENVE2
E5 main ENVE1
C1 accepted 250
C2 trapped rc 29
```

* `D0` is the platform default before any `ADDRESS` instruction.
* `D` is the swap **four toggles deep**. Two is the fewest that can tell a swap from a pop and it is not enough: a stack popped twice and a pair swapped twice agree at that depth.
* `E` is inheritance into a callee **in both halves**, plus the return direction. **Both environments are named and neither is the default**, deliberately: with an unset alternate, "the callee inherited the caller's pair" and "the callee started fresh" print the same bytes, and the block would prove nothing.

`state_builtins.rex` is the second new program: the `NUMERIC` trio across a call, `TRACE()`'s five silent settings, `ARG` over a call with an interior omission, `CONDITION` from inside a `SIGNAL ON SYNTAX` handler including the `I`/`S` divergence, `CONDITION` from inside a `CALL ON USER` handler including `DELAY`, and `ERRORTEXT`/`SOURCELINE`/`GC`/`QUEUED`.

## "Can fail" is not "adds coverage": eight mutations, `--no-fail-fast`

Every run below is `REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast`, and every one reported **72 processes**, the documented baseline, so none was truncated.

| mutation | caught by |
|---|---|
| M1 `TraceMode::NORMAL` renders `O` | 2 unit tests + `corpus_differential` (47 of 48) + `keyword_assertions` |
| M2 bare `CONDITION()` defaults to `'C'` | `condition_answers_a_syntax_handler_from_inside_it` + corpus (47 of 48) |
| M3 `CONDITION('S')` reports the stored `call` bit instead of the live table | `the_instruction_is_stored_and_the_state_is_looked_up_live` + corpus (47 of 48) |
| M4 the callee inherits only the *current* half of the `ADDRESS` pair | **`corpus_differential` alone** (47 of 48) |
| M5 `ADDRESS()` renders the default as `''` | `address_reports_the_default_and_then_follows_the_swap` + corpus |
| M6 the `CALL ON` handler's trap is removed rather than delayed (before the `F` block) | **`a_call_handler_reports_call_and_delay_and_leaves_nothing_behind` alone**; corpus stayed 48 of 48 |
| M6b the same, after the `F` block was added | that unit test **and** `corpus_differential` (47 of 48) |
| M7 the callee does not inherit the condition | 2 unit tests + corpus |
| M8 the `CALL ON` handler's condition is not restored on return | 1 unit test + corpus |

Two of these answer the "adds coverage" question directly rather than merely going red.

**M4 is the debt Step 0a names.** It is caught by nothing but `lang/address_env.rex`, and the report names the exact line: `E3 sub sh` against the oracle's `E3 sub ENVE1`. No in-crate test can catch it, because `resolve_and_run_call` pops the callee unconditionally.

**M6 measured the gap it then closed.** Run before `state_builtins.rex` had a `CALL ON` block, the delayed-vs-removed mutation was caught by one unit test and the corpus stayed green at 48 of 48. The `F` block was added for exactly that, and M6b confirms the corpus now catches it too.

## Inferred rather than measured, flagged

* ~~**`code_sub` is `None` for every condition a `CALL ON` trap can carry.**~~ **Promoted to a measurement by fix round 1.** The flag was on the missing premise, not on the conclusion: `CALL ON SYNTAX` is a translation-time 25.1, but `CALL ON ANY` is legal and could have smuggled a `SYNTAX` condition in. Run rather than reasoned about -- `call on any name uh` with `say 1/0` is the ordinary fatal 42.3 at rc 214 on **both** interpreters, so the trap never fires and no condition reaching `deliver_pending_trap` has a `CODE`. `TrapHandler::canHandle` is the C++ side of the same rule. The premise now stands in the comment where the flag was.
* ~~**`DEFAULT_ENVIRONMENT` on non-unix is `CMD`.**~~ **Promoted to a measurement by fix round 1**, once the citation was corrected: it is `#define SYSINITIALADDRESS "CMD"` at `platform/windows/PlatformDefinitions.h:65`, read directly. Still not *run*, because there is no Windows build here, but it is no longer an inference from an adjacent file.
* **The C++ keys a `USER` trap by the two-word name.** Inferred from `condition('S')` reporting `ON` after `call on user uc name uh` re-arms inside its own handler, which requires `trapState("USER UC")` to find it. It matches this crate's existing key and nothing was changed on the strength of it.

## Also corrected while here

* **`corpus/keyword-exempt.txt`: 111 rows to 16.** 95 bodies now pass -- every `ADDRESS::test_environment_*`, most of `NUMERIC`, `DO::test_DO_standardTest5-*`, the three `TRACE::` rows, `PARSE::Test_638`, `CALL::test_7`. The differential report moves from 785 to **880 of 896 bodies, 1710 of 1773 `assertSame` calls**.
* Two attributions moved. `DO::test_DO_standardTest2A` reads `defect:compound-do-control-variable`, joining five siblings. `NUMERIC::test_42` reads `RAISED`, and the cause was chased rather than accepted: `::method "test_42"` is the only method in its neighbourhood with no `return` before its trailing `dig: Return digits()`, so the extracted program falls into that subroutine and its `RETURN` value becomes the exit code. **Measured, the C++ oracle exits 3 on the same program**, so this is the extraction and not a divergence. The exempt file's header already recorded the fall-through and now records that it is what the row reads.
* `every_exempt_attribution_is_a_known_phase_or_a_declared_defect` admits the harness's own two derived tokens (`RAISED`, `NO-ASSERTION-EXECUTED`) alongside the phases and `defect:` tags. They are string constants in that file, not hand-written attributions, so a typo still cannot become a category; requiring a phase for a row the harness cannot attribute would mean inventing one.
* Stale prose corrected where `ADDRESS()`/`TRACE()` became answerable: `AddressState::current`'s doc, the `ADDRESS` unit-test section header in `run.rs`, `a_callees_own_environment_does_not_survive_the_return`'s doc (which said the other direction could not be asserted anywhere), `corpus/builtin-probes.txt`'s header, and `corpus/phase-4c.txt`'s ADDRESS paragraph.

## Verification

All from `rust/`, exit statuses read unpiped.

```
$ cargo test --offline --workspace --no-fail-fast
exit=0
73 `test result: ok` lines, 1181 tests passed, 0 failed
72 Running/Doc-tests process headers   (the documented baseline)

$ cargo fmt --all --check
exit=0

$ CARGO_TARGET_DIR=<empty dir> cargo clippy --offline --workspace --all-targets -- -D warnings
exit=0                                (from a clean target directory)

$ REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus
48 of 48 matching                     (was 47 of 47; state_builtins.rex is the new one)
exit=0

$ cargo test --offline -p rexx-exec --test builtin_status
13 passed, 0 failed
```

`builtin-status.txt` after the change: **60 `implemented`**, **6 `loud`** (`DATATYPE`, `DATE`, `SYMBOL`, `TIME`, `VALUE`, `VAR` -- Tasks 11 and 12), **15 `excluded`**, **0 `divergent`**. 60 + 6 + 15 = 81, and the eleven that flipped are exactly `STATE_FAMILY`.

## Commits

```
f03d69f183abec262d80b8381b7303df54e8aee1  Answer the eleven builtins that read interpreter state
```

Read back with `git rev-parse HEAD` after committing, not written from memory.

---

# Fix round 1

Every finding addressed. The Critical was a real silent wrong answer with twelve witnesses, not eight; the Important "false justification" was reproduced and retracted in five places, not the four the review named; and the sweep contradiction resolved into a precise statement about what my published alphabet claimed and did not cover.

## Critical: three range messages substituted the rendered value

**Reproduced, and wider than reported.** 21 programs run through both interpreters at the shipped commit: **12 differ**, across all three families. The four the review named, plus eight more of the same shape.

```
say arg(0.0)                                oracle found "0"        ours "0.0"
say arg('+0')                               oracle found "0"        ours "+0"
say arg(-1.0)                               oracle found "-1"       ours "-1.0"
say errortext(1e2)                          oracle found "100"      ours "1E2"
say errortext(' 100 ')                      oracle found "100"      ours " 100 "
say sourceline(1e1)                         oracle ("10")           ours ("1E1")
say sourceline(0.0)                         oracle found "0"        ours "0.0"
say sourceline('0000')                      oracle found "0"        ours "0000"
numeric digits 3; say errortext(999999+1)   oracle found "1000000"  ours "1.00E+6"
numeric digits 3; say sourceline(999999+1)  oracle ("1000000")      ours ("1.00E+6")
numeric digits 3; say arg(-(999999+1))      oracle found "-1000000" ours "-1.00E+6"
numeric digits 3; z = 999999.5+0; numeric digits 9; say errortext(z)
                                            oracle found "1000000"  ours "1.00E+6"
```

**The boundary, probed rather than reasoned about, because the review is explicit that the neighbouring precedent proves nothing.** The split is *where the conversion succeeded*, and both sides of it were measured:

* **Conversion failed -> the value's own rendering.** 40.12 is raised *because* there is no integer. `numeric digits 3; z = 1/3; numeric digits 9; errortext(z)` reports `found "0.333"` -- the rendering captured at creation, so D15-correct -- and `errortext(99999999999999999999)` reports all twenty digits. `Raised::argument_not_whole` was already right.
* **Conversion succeeded, then a range check failed -> the converted integer.** 40.14, 40.34 and 40.903 were wrong; the neighbouring 93.9xx family was already right and is now measured to be, not assumed: `numeric digits 3; word('a b', -(999999+1))` is 93.924 `found "-1000000"`, and `left`/`copies` under the same crossing give 93.923 and 93.906 with the same substitution. Those three go through `builtin/mod.rs`'s `length_of`/`position_of`/`count_of`, which pass `value.to_string()` -- the converted integer -- and match.

**The fix.** `rendered()` is gone; `converted(i64) -> String` replaces it, and its doc carries the six-row transcript, the D15 explanation and the explicit contrast with `argument_not_whole`. All 21 cases now agree.

**The causal chain, which is the part that is not the fix.** Three shared-block requirements were missed, and the third is why the first went wrong.

* *"Measure whether a 40.12 or 40.23 message substitutes the rendered value or the source spelling"* was taken and **answered incorrectly** -- and the answer's own justification was the tell. `rendered()`'s doc said the choice "is the same choice `argument_not_whole` makes". That is true and worthless: in every case `argument_not_whole` can reach, the rendered form *is* the converted form, so it is evidence for nothing. **A precedent only transfers where the two candidate answers differ**, and the sentence should not have been written without finding a case where they do.
* The *per-numeric-builtin D15 probe* was not done. Three of these eleven builtins take a numeric argument; none got the probe the shared block asks for by name.
* *"Cross the axes"* was not done. The axes here are **what the argument looks like** x **what `NUMERIC DIGITS` is when it is read**, and holding either at its safe value -- an integer literal, or the default `DIGITS` -- hides all twelve witnesses. This is the same shape as the `verify('abcde','','00'x)` case the shared block already carries: neither axis was missing from my probe set, and each was varied with the other held safe.

**Coverage, and it is provable rather than argued.** The mutation is "restore the shipped behaviour at those four call sites", which *is* commit `f03d69f1`, whose suite was green at 1181 passed / 0 failed. So nothing pre-existing caught it. Applied to the current tree with `--no-fail-fast`, 73 process headers:

| catcher | result |
|---|---|
| `a_range_message_substitutes_the_converted_integer` | FAILED |
| `a_range_message_ignores_the_digits_the_value_was_rendered_under` | FAILED |
| `corpus_differential` | 48 of 49, on `lang/builtin_argument_range.rex` |
| `every_state_builtin_case_matches_the_oracle_except_the_declared_gaps` | FAILED |

`lang/builtin_argument_range.rex` is the new corpus program: the same message twice, once as an integer literal (the control, where every engine agrees) and once at the crossing. Its header says why a trapped handler cannot stand in -- the message is on stderr and fatal, and `condition('o')` cannot read a substitution back.

## Important: the false justification for a correct design

**Reproduced both ways the review describes.** A `CALL ON` handler's `call off user uc` does **not** reach the caller's trap table, because the handler is a nested activation with a copy of it. On the oracle, a handler whose first act is `call off user uc` is entered again on a second raise; this crate matches byte for byte. With the removal mutation restored, the same program prints identically except that `CONDITION('S')` reads `OFF` where it should read `DELAY`.

So `Trap::delayed` is right and `CONDITION('S')` is the whole of what it buys. The claim that removal "resurrects a trap the handler turned off" is retracted.

**Five copies, not four**, found by sweeping for every statement of the fact before fixing any of it -- the corpus program's own header was the fifth and was not in the review's list:

1. `Trap::delayed`'s doc (`activation.rs`) -- corrected, and it is where the note about the commit message lives.
2. `deliver_pending_trap`'s comment (`run.rs`) -- corrected.
3. `corpus/lang/state_builtins.rex`'s F-block header -- corrected.
4. This report -- corrected in place, above.
5. Commit `f03d69f1`'s message -- **cannot be edited**, so `Trap::delayed`'s doc names the commit and says its message carries the superseded claim.

## Important: two stale descriptions of the deleted mechanism, and a third

Swept rather than fixed one at a time, which found one the review had not named.

* `run.rs`'s `deliver_pending_trap` **doc** said the trap "is removed for the handler's duration and put back afterwards" -- contradicting the comment sixty lines below it in the same function. Now "held ... and released", with a note that the two agree on everything but `CONDITION('S')`.
* `run.rs`'s `a_call_trap_is_put_back_after_its_handler_returns` cited "deleting the re-insertion". Re-measured against the new spelling rather than translated: skipping `trap.delayed = false` fails that test exactly as before **and** drops the corpus to 48 of 49.
* **`corpus/lang/call_on_trap_rearms.rex:41` cited the `if let Some(trap) = removed` arm by name** -- a fourth-generation copy the review did not list. Its recorded transcript is still exactly right: the new mutation gives `S[H54]1:VK2:V`, the `[H60]` segment gone, byte for byte what the header says. Only the name of the mutated code changed. **The replacement is line-count-neutral on purpose**: that program's own output embeds `SIGL`, so a header one line longer would have shifted `[H54]`/`[H60]` and falsified four more sentences in the same comment.

## Important: `keyword-exempt.txt`'s header over-reached by one name

Confirmed from the run's own "first construct hit" breakdown and the `.testGroup` sources, not from the file being corrected:

| body | first blocker | what it really needs |
|---|---|---|
| `CALL::test_literal` | `routine ""` | a `::routine` the standalone program does not carry |
| `CALL::test_expression` | `routine "label"` | ditto |
| `CALL::test_on_name` | `routine "CHARIN"` | a **builtin** -- `call charin "/"`, its third clause |
| `CALL::test_9` | `routine "LINEIN"` | a builtin |
| `ASSIGNMENT::test_3` | `routine "DATATYPE"` | a builtin, Task 11's |

So `CALL::test_on_name` was in a sentence about missing `::routine`s and does not belong there. The header now says so in as many words, because a correction that over-reaches is still a false statement.

**And a second limit the correction exposed, now written down.** `Loud::unresolved_call` carries a fixed `"4c"`, so the derived owner is `4c` for *every* unresolved name -- including `CHARIN` and `LINEIN`, which are **whole** exclusions and therefore Phase 7's. Three of the five `4c` rows are really waiting on a builtin, and two of those on a phase the column cannot name.

## Important: the `DEFAULT_ENVIRONMENT` citation, in four copies

Corrected in `state.rs`, `phase-4-exclusions.txt`, `corpus/lang/address_env.rex`, `corpus/phase-4c.txt` and the report body above -- five sites, one more than the review's three, found by grepping for all four of `SYSINITIALADDRESS`, `SystemCommands.cpp`, `getDefaultAddressName` and `INITIALADDRESS` rather than for the phrase I remembered writing. The values `sh` and `CMD` were never wrong; the provenance was. Detail in the corrected body section above.

## Minor findings

* **`Loud::builtin_option_object`'s contract was false for its `CONDITION('D')` use.** It said the option "answers an object this crate's value model cannot make"; `D` answers an ordinary *string*, the `NOVALUE` variable's derived name, which this crate simply does not carry. The doc now names both reasons and says only the first is about an object. The constructor keeps its name.
* **`caller_trap_for` does not filter `Trap::delayed`, and the comment was wrong, not the code.** Which one is right was settled by running it: a `CALL ON` handler that calls a routine raising the same condition runs **once** on both interpreters, byte for byte. The C++ draws the same line -- `raiseCondition` queues a delayed handler without asking, `processTraps` is what skips it -- and here `caller_trap_for` matches while `deliver_pending_trap`'s own `trap_for` declines. `Trap::delayed`'s "every lookup that decides whether to trap" was one lookup too many; both that doc and `caller_trap_for`'s now say so.
* **`trace('?')` diverges too and is now in the `TRACE ?` row**, which lists all three spellings. Measured: `trace l` then `trace('?')` returns `L` on both -- the returned value is right -- and leaves the setting at `?L` on the oracle against `O` here. Same defect as the bare `trace ?`, reached through the builtin's setter form. `mode_from_setting`'s doc carries it as well.
* **The rotted `796-row` count, in both copies.** `rust/CLAUDE.md`'s rule against writing down mutable in-repo counts illustrated itself with one, and `builtin_status.rs`'s module doc repeated it; the table is now 16 rows. Both now state the contrast without the number, and the CLAUDE.md rule notes that its own illustration is what caught it.
* **`corpus/README.md`** gained rows for all three new programs, not only the one the review named.
* **`code_sub: None` is now stated as a conclusion with its premise**, not flagged. The premise the original comment omitted -- that `CALL ON ANY` is legal where `CALL ON SYNTAX` is a translation error -- was run rather than reasoned about: `call on any name uh` with `say 1/0` is the ordinary fatal 42.3 at rc 214 on both interpreters.

## The sweep contradiction, resolved

**My sweep was not wrong about what it ran, and re-running it says so: the same 60 programs still report 57 identical and 3 differing.** What was wrong is the sentence after the number.

The alphabet I published beside the count named `1.0`, `1.5` and `99.0` among its numeric values. That reads as coverage of the numeric axis. None of the three reaches a message that substitutes anything: `errortext(1.0)` and `errortext(99.0)` are in range and exit 0, `sourceline(1.0)` returns line 1, and `errortext(-1.5)` lands in 40.12, the one family that was already correct. Every out-of-range numeric in the set was an **integer literal**, where the rendered and converted forms coincide by construction. So twelve live divergences sat inside the surface the sweep declared clean.

**The lesson, and it sharpens the shared block's own rule.** An alphabet must list the values that *reach each code path*, not the values *supplied*. Mine listed the second and was read as the first, which is how a count-plus-alphabet still failed to be a coverage claim.

**Committed rather than described**, as `crates/rexx-exec/tests/state_builtin_oracle.rs`: 87 cases through `support::oracle`, the alphabet *is* the table, `DECLARED_GAPS` is a named set rather than a count, and every case at the intersection is named `*_crossed`. Its own module doc records why it exists.

Both directions of its central assertion were mutation-checked:

| mutation | result |
|---|---|
| drop `condition_object` from `DECLARED_GAPS` | FAILED -- "differ but are not declared gaps" |
| add a case that agrees to `DECLARED_GAPS` | FAILED -- "are declared gaps but now agree" |
| make `condition('O')` return `''` instead of failing loudly | FAILED -- `every_declared_gap_names_a_case_and_fails_loudly` |

**And the first attempt at the first of those was a no-op**, caught here rather than shipped: the `DECLARED_GAPS` line had been re-wrapped by `cargo fmt`, so a single-line string replacement matched nothing and the test passed for the wrong reason. Re-run with an assertion on the replacement count. A mutation that does not apply reads exactly like a test that cannot be broken -- the same shape as a verification command that never runs.

## Verification

All from `rust/`, exit statuses read unpiped.

```
$ cargo test --offline --workspace --no-fail-fast
exit=0
74 `test result: ok` lines, 1192 tests passed, 0 failed
73 Running/Doc-tests process headers

$ cargo fmt --all --check
exit=0

$ CARGO_TARGET_DIR=<empty dir> cargo clippy --offline --workspace --all-targets -- -D warnings
exit=0

$ REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast
49 of 49 matching
```

**The mutation baseline moved from 72 process headers to 73**, and it is the new differential test's own binary. Every mutation run in this round reported 73 and is recorded above; a run reporting 72 from here on is a truncated run, not a clean one.

`git status --porcelain` empty after committing.

## Commits

```
9b4ee92a289974f5a4160ad7553cbca8ef25b88c  Substitute the converted integer in a builtin's range message
```

Read back with `git rev-parse HEAD` after committing, not written from memory.
Round 0 was `f03d69f183abec262d80b8381b7303df54e8aee1`, whose message carries the retracted `CALL OFF` claim.

# Fix round 2

One Important finding from a scoped re-review: `argument_not_positive`'s
(40.14) doc at `error.rs:554-555` said its substitutions were "as
[`argument_not_whole`]'s" in full. That was true of the three-slot layout
(routine, position, found) but false of `found` itself: the fix in round 1
changed the call sites to pass the converted whole number, and the doc
still said `found` was the rendered value -- the exact opposite, at the
definition site of the error the fix was about.

## Edits

**`error.rs:554-555`, `argument_not_positive` (40.14).**

Before:

```
/// 40.14: a builtin's argument converted to a whole number but is not
/// strictly positive. Substitutions as [`argument_not_whole`]'s.
```

After:

```
/// 40.14: a builtin's argument converted to a whole number but is not
/// strictly positive. Substituted as the same three slots -- routine,
/// position, found -- as [`argument_not_whole`]'s.
///
/// **`found` here is the converted whole number, not the rendered
/// value -- the opposite of [`argument_not_whole`]'s.** This error fires
/// only after the argument has converted successfully, so an integer
/// exists and the oracle substitutes it, where 40.12 fires on a value
/// that never converted and has only its own rendering to give. Measured,
/// rc 216 both sides: `say arg(0.0)` reports `found "0"`, not `found
/// "0.0"`.
```

**`error.rs:579-585` (now further down after the insertion above), `sourceline_out_of_range` (40.34).**

Before: no mention of rendered-versus-converted.

After, appended after the existing measured example:

```
///
/// `requested` is the converted whole number, not the rendered value, for
/// the reason [`argument_not_positive`]'s doc gives.
///
/// [`argument_not_positive`]: Raised::argument_not_positive
```

**`error.rs:594-599`, `argument_out_of_range` (40.903).**

Before: no mention of rendered-versus-converted.

After, appended after the existing measured example:

```
///
/// `found` is the converted whole number, not the rendered value, for
/// the reason [`argument_not_positive`]'s doc gives.
///
/// [`argument_not_positive`]: Raised::argument_not_positive
```

**`task-10-report.md:458`** (this file, gitignored, not part of the commit): corrected
"82 cases" to "87 cases" for `state_builtin_oracle.rs`. Verified two ways from
`rust/crates/rexx-exec/tests/state_builtin_oracle.rs`: `grep -ac "Case {"`
gives 88 (one is the `struct Case {` declaration), and `grep -ac "source:"`
gives 89 (one is the field declaration, one is inside a format string) --
both leave 87.

## Verification

```
$ cargo fmt --all --check
exit=0

$ cargo clippy --workspace --all-targets -- -D warnings
exit=0
(Checking rexx-exec, Finished dev profile, no warnings)

$ cargo test --offline --workspace --no-fail-fast
exit=0
(all suites reported ok; grep -aE "FAILED|error\[" over the full output matched nothing)
```

## Commit

```
67382f88be40d5216140037ba20c17db8093b5d8  Fix 40.14's found doc, and record the family's rendered/converted split
```

Read back with `git log -1 --format="%H %s"` after committing, not written from memory.
Scope: exactly `rust/crates/rexx-exec/src/error.rs` (3 doc edits) and this report file (1 line). No code behaviour changed.
