# Task 8 re-review -- fix rounds A and B

Reviewer: review-s8. Ranges `2b26b6970..7f220b983` (A) and `7f220b983..a9fd1a7ec` (B). Scratch
`$R` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s8/`.
Build: `git archive a9fd1a7ec` into `$R/h2`, `ootest/` copied in, every file touched, own
`CARGO_TARGET_DIR=$R/tgt2` (`Compiling rexx-exec` seen). Probes run by `$R/cmp.sh` (standard wrapper,
fresh `mktemp -d`, stdin `/dev/null`, three descriptors separately). Written as it goes.

## Verdict

**Changes needed.**

* **Closed:** every first-review finding is fixed or recorded as ruled (I1, I3, I4, I5, I6, I7, m1,
  m2, m3 fixed; m4 and m5 queued as ruled). I2 is fixed for Rexx levels.
* **Instrument:** it now compares outcome, assertion count and failure detail. The repeated M2
  mutation turned it red exactly as predicted.
* **New: one Critical finding (N5).** The round-B stream reader hangs on three ordinary ends of
  input where the oracle completes.
* **New: four Important findings (N1-N4).** Each is an unrecorded divergence in I2's subject or on
  the native API surface.

## Open findings

### Critical

* **N5. ADDRESS WITH INPUT USING an InputStream object loops forever** when `linein` ends the
  input with `raise notready` or `raise user ...` without RETURN, or a bare `return`
  (`$R/p/sq1`, `sq3`, `sq6`: oracle `rc 0 a|b` at rc 0, here killed at 20 s, rc 137). `raise halt`
  (`sq7`) ends it with Error 4.1 here. At `7f220b983` all four ended at rc 159, so round B's reader
  (`redirect.rs` `read_stream_lines`, `input_dispatcher_takes`) turned a loud error into a hang. No
  witness covers these ends; the S5 control that would have shown a looping reader was not run
  (section 5).

### Important

* **N1. A trapped condition still loses an external program's level**: frame and traceback line
  (`$R/q/d2`, `d4`). Not in round B's residual record, which describes only the frame's type.
* **N2. PROPAGATED is 0 for every SYNTAX that leaves a method or routine** (`$R/p/pr1`; oracle 1).
  The only record (`phase-4-exclusions.txt:4693`) confines it to a native re-raise and gives a
  false reason.
* **N3. A tail-less `RAISE SYNTAX` inside a callback keeps the RAISE's POSITION** where the oracle
  re-raises at the caller's line (`$R/p/nr1` a1: 9 against 40). The native re-raise rule misses a
  condition that came from RAISE.
* **N4. CallRoutine exposes a `Routine~callWith` level**: trapped frames `ROUTINE CALL`, `METHOD
  CALLWITH` and a `Compiled method "CALLWITH"` traceback line the oracle does not have
  (`$R/p/nr1` a4, `nr2`). Inside the called routine, `.context~stackframes` lacks the native frame
  (`nr4`). Unrecorded; Phase 8's API.

### Minor

* **N6.** A `.Stream` given as input is read twice (`$R/p/sr4`), pre-existing, unrecorded.
* **m6.** The report's "+9.4% for an error trapped one level up" holds for an internal call; one
  method up costs +21.1% against round A (`$R/cg2/tm.rex`), still +6.2% against the Task 8 BASE.
* The Phase 9 residuals are true as measured. Two points need the lead: three of them are I2's
  shape, which was ruled "fix", and the fourth (`~empty` under an exposed tail) is the new EXPOSE
  code's own silent case (section 6).

## 1. The first review's findings

Each re-run at `a9fd1a7ec` with the first review's probes (`$R/p/`, `$R/cmp.sh`):

| Finding | Status | Evidence |
|---|---|---|
| I1 `condition('O')` a copy | **fixed** | `co1`, `co2`, `co4` identical on three descriptors (at `2b26b6970` stdout differed) |
| I2 trapped SYNTAX names the raiser | **fixed for Rexx levels; N1-N4 remain** | `sf1`-`sf3`, `tb1`, `tb2`, `rs1`-`rs6` identical; section 2 |
| I3 shared Routine annotations | **fixed** | `gr1`, `gr1b`, `gr1c`, `gr1d`, `gr2`, `gr3` identical |
| I4 36.901 from the compile path | **fixed** as ruled | `rec5`, `rec6`: code and POSITION agree; the message keeps `&1`/`&2` (`position &1 on line &2`), recorded under row 3's licence |
| I5 trapped-condition cost | **fixed** | section 4: the loop is -12.5% against `391242b7e` |
| I6 owners | **as ruled** | `RECORDED` is `FUNCTION.TEST_REXXQUEUE` alone (`api_group_tests.rs:58`); roadmap row 8 names it with rxapi's queue; `rec1`-`rec4` now identical |
| I7 count-only instrument | **fixed** | section 3, M2' red as predicted |
| m1 one assert hides the other | **fixed** | M2' reports both parts in one panic |
| m2 substring record check | **fixed** | a record is a trimmed line equal to the test's name (`api_group_tests.rs:342-350`) |
| m3 set sizes | **fixed** | `eval.rs:1189` "`Object`'s and `Pointer`'s"; the doc comment's count gone |
| m4 Phase 5 in refusal texts | **as ruled** | the four member refusals are gone with their fixes; the generic `Loud::native_method` text is queued (`queued/2026-09-28-native-method-refusal-names-closed-phase.md`) |
| m5 `USE ARG o~a` | **queued as ruled** | `pm8` still rc 120 here; `queued/2026-09-28-use-arg-message-term.md` |


## 2. Mutation hunt over the new code

Probes in `$R/p/` (single file) and `$R/q/dN/` (a directory whose `main.rex` runs, by
`$R/cmpd.sh`, paths masked to `/D`). Baselines: `$R/tgt2/release/rexx-run` (`a9fd1a7ec`), and
fix-s8b's `rexx-run-BASE` (`7f220b983`) and surface-8's `base-target` (`391242b7e`) to tell new from
old.

### I2: the frames of the levels a condition leaves

* **Agree** (`q/d1`, `d5`, `d6`, `d7`, `p/sf1`-`sf3`, `tb1`, `tb2`, `rs1`-`rs6`): SYNTAX from nested
  INTERPRET inside an internal call inside a `::ROUTINE`; `RAISE SYNTAX` from an internal call of a
  method three sends deep; recursion through four internal calls into an INTERPRET; a `CALL ON`/
  `SIGNAL ON USER` from `RAISE ... RETURN` two levels down; `RAISE PROPAGATE`; a SIGNAL ON ANY
  handler re-raising; `FORWARD ... CONTINUE` into `Message~sendWith` into INTERPRET. Frames (type,
  name, line, argument count), traceback lines and POSITION identical.
* **N1 (Important): an external program's own level is still missing.** `q/d2`, `q/d4`: a SYNTAX in
  an external `.rex` called as a routine or function, trapped by the caller. Oracle frames
  `INTERNALCALL LVL 4`, `ROUTINE EXT 2`, `PROGRAM`, traceback `4 *-* x = k + 'bad'`, `2 *-* call lvl
  n`, `2 *-* x = ext(5)`; here `INTERNALCALL LVL 4`, `PROGRAM`, and the lines without `2 *-* call
  lvl n`. `q/d2`'s second case: `ROUTINE EXTFN` and its `1 *-* return .req~new~go(arg(1))` line
  both missing. Round B's residual record says only that such a level "is a PROGRAM frame here and
  a ROUTINE frame on the oracle", measured through `.context~stackframes` inside the callee; the
  trapped condition does not have the frame at all, which is the I2 shape itself and is recorded
  nowhere. (Untrapped, a different line goes, the caller's `call ext 5`: that is the pre-existing
  `phase-4-exclusions.txt:5137` entry, NO OWNER.)
* **N2 (Important): PROPAGATED is 0 for every SYNTAX that leaves a method or routine.** `p/pr1`:
  a plain error in a method, in a routine, and a `RAISE SYNTAX` in a method, each trapped by the
  caller: oracle `1`, here `0` (an internal call, an INTERPRET and the own activation: `0` on both).
  Same at `391242b7e`. The only record, `phase-4-exclusions.txt:4693`, narrows it to "A NATIVE SYNTAX
  CONDITION RE-RAISED INTO A CALLER", and its reason, "PROPAGATED in a re-raised condition is the
  native boundary's", is false: no native frame is involved in `pr1`. `q/d6`, `q/d7` show it too.
* **N3 (Important): a tail-less `RAISE SYNTAX` inside a callback keeps its own POSITION.** `p/nr1`,
  case a1: `TestSendMessage0` over a method whose body is `raise syntax 93.900 array(...)`,
  trapped by the caller: oracle POSITION 9 (the caller's line, the native re-raise), here 40 (the
  RAISE's line). Frames and traceback agree. An arithmetic error in the same place (case a2) gets
  the caller's POSITION on both, so the native re-raise rule round B added (`native_reraise`)
  misses a condition that came from RAISE. Untrapped (`nr3`) agrees.
* **N4 (Important): the API's CallRoutine shows its implementation.** `p/nr1` case a4,
  `TestCallRoutine` over `.routine~new('rr', 'return 1 / 0')`, trapped: here the frames
  `ROUTINE CALL 1`, `METHOD CALLWITH`, `METHOD CALLR` and a traceback line `Compiled method
  "CALLWITH" with scope "Routine"`; the oracle has `ROUTINE (no name) 1`, `METHOD CALLR` and no
  CALLWITH line. Untrapped (`nr2`) the CALLWITH line is there too, and inside the routine
  `.context~stackframes` has 2 frames against the oracle's 3 (`nr4`, the native CALLR frame
  missing). The untrapped line and `nr4` predate round B (`7f220b983` differs the same way); the
  trapped frames are round B's exposure of it. No record mentions CallRoutine. Phase 8's surface.

### EXPOSE of a compound tail

`p/ex1`-`ex4` agree: `drop` of an exposed tail and `symbol`/`var` after it, `value()` read and
write, `expose a.` replaced by a new stem in another method, an indirect `expose i a.i (lst)`,
`procedure expose a.k k` with `k` changed after (the binding stays), a drop through a PROCEDURE
EXPOSE, chains of PROCEDURE EXPOSE of tails, `expose s.1 s.2 s.1`, `items`/`allIndexes`/`hasIndex`/
`[]`/`at`/`put`/`remove` on the local stem, a stem copy, and a collection loop after the object's stem
was replaced (the exposed stem is traced, `body.rs`). `DO OVER` a stem refuses loudly here for any
stem (`p/do1`), pre-existing and not this change.

### Class~new

`p/cn1`-`cn6` agree: `defaultName`, `string`, `metaclass`, `superclasses`, `isSubclassOf` both ways,
`isMetaclass`, `queryMixinClass`, a subclass and its instance, `subclasses`, `hasMethod`,
`instanceMethods`, `mixinclass`, too many arguments (93.902 from INIT), `new` on the clone (97.1),
a Rexx metaclass's `new`, `inherit` of a non-mixin (98), `.object~class~new`, a `.nil` id (88).
`~enhanced` refuses loudly (`Table~supplier`, `cn4`), pre-existing.

### Package tables, Routine~new and Method~new over directives

`p/pk1`, `pk2`, `pk4` agree: `classes` answering a fresh StringTable per ask, a write to one not
seen by the next, public/private classes and routines, a supplier, a routine source with a helper
routine and a class, its package's tables, `findRoutine`, a `::requires` of a missing file, a source
whose first line is not a directive, `::options digits`. Two differences, neither new:

* `pk3` (a duplicate `::class`): 99.901 at POSITION 3 on the oracle, 2 here. Roadmap row 3's
  "a plausible line" licence.
* `pk5`-`pk7`: a class of the caller's package does not resolve inside `Routine~new`'s code, with
  or without directives (`.PC`, then 97.1), and `findClass` on its package answers `.nil`. That is
  `phase-4-exclusions.txt`'s "A CLASS RESOLVED THROUGH A PACKAGE PARENT ANSWERS THE UNRESOLVED
  SYMBOL", NO OWNER. Round B's directive path makes it reachable one more way; not a group member.

### The stream reader

* **N5 (Critical): the reader never ends for three ordinary ends of input.** `p/sq1`-`sq8`, an
  `InputStream` subclass whose `linein` answers two lines and then, at the end:
  * `raise notready` (no RETURN) (`sq1`), `raise user stop` (`sq3`), or a bare `return` (`sq6`):
    oracle `rc 0 a|b`, rc 0; **here the process runs until the 20 s kill, rc 137**.
  * `raise halt` (`sq7`): oracle `rc 0 a|b`; here Error 4.1, rc 252.
  * `raise notready return`, `raise user stop return`, `raise syntax ...`: agree (`sq2`, `sq4`,
    `sq5`).

  At `7f220b983` all of `sq1`, `sq3`, `sq6`, `sq7` ended at rc 159 (the 97.1 `STATE` refusal), so round
  B turned a loud error into a hang. The fix report's S5 row names the looping shape ("would leave
  the reader looping and was not run"). `p/sr1` is the same with NOTREADY and a `CALL ON NOTREADY`.
* **N6 (Minor, pre-existing): a `.Stream` as input is read twice.** `p/sr4`: a stream object over a
  two-line file: oracle `one|two`; here `one|two|one|two`, and after one `linein` the rest is
  `two|one|two`. Same at `7f220b983`. Unrecorded: `grep -a -n -i 'input using'` and `'twice'` over `phase-4-exclusions.txt` find nothing about it.


## 3. The instrument at -V 2

Unmutated at `a9fd1a7ec`: `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test
api_group_tests --test corpus`: ok, 51.10 s, and `644 of 644 matching` (`$R/r2-base.out`). The
harness runs `-V 2` (`api_group_tests.rs:61`), strips the two `rxfuncquery` tests from its copy with
a shape assertion (`:137-161`), and masks only the timing durations and the detail timestamps
(`:166-178`).

My first M2 subject (`stackframes[1]~name`) now agrees on both sides, which is I2's fix, so M2 is
repeated over a divergence still live, N2's PROPAGATED. Sanity runs alone at `-V 2` first
(`$R/mut2`, `$R/one.sh`): A (`assertSame(1, propagated)`) oracle pass / crate failure `Actual: 0`;
B (`assertSame('never', ...)`) failure on both, `Actual: 1` against `Actual: 0`; C (`if propagated
== 1 then assertTrue`) pass on both, 1 assertion against 0; D (the old `stackframes[1]` test) pass
on both, 1 assertion each.

Prediction, written before the instrument run: **M2' (B, C and D added to the archive's group
copy) is red in one panic carrying `newly failing: ["METHOD.TESTREVIEWC"]` (its assertion count),
`newly passing: []`, `detail newly differing: ["METHOD.TESTREVIEWB"]`, D in neither, and a
whole-group METHOD line (assertions differ by one).**

**Result: as predicted, every part in one panic** (`$R/m2p.out`): `newly failing:
["METHOD.TESTREVIEWC"]` (`Assertions: 1` against `0`), `newly passing: []`, `detail newly
differing: ["METHOD.TESTREVIEWB"]` (`Actual: 1` against `Actual: 0`), `detail no longer differing:
[]`, D in neither list, and `METHOD: ... assertions 1130, ... assertions 1129` for the whole-group
run. Group file restored, `cmp` equal to `ootest/`'s. I7 and m1 are closed.


## 4. Performance

Callgrind `Collected`, each program copied to a fresh directory and run by `$R/cg2/run.sh`
(programs and profiles in `$R/cg2/`). `b0` is `391242b7e` (surface-8's `base-target`), `fa` is round
A's `7f220b983` (fix-s8b's `rexx-run-BASE`), `hd` is my build of `a9fd1a7ec`:

| program | b0 | fa | hd | hd/fa | hd/b0 |
|---|---|---|---|---|---|
| `t.rex`, trapped in the raising activation, 5000 | 629,364,129 | 549,931,543 | 550,497,395 | +0.10% | -12.5% |
| `tu.rex` (fix-s8b's), one internal call up, 5000 | 1,232,651,879 | 1,076,227,937 | 1,176,832,181 | +9.35% | -4.5% |
| `tm.rex`, one **method** up, 5000 | 642,961,614 | 564,005,833 | 682,846,832 | **+21.1%** | +6.2% |
| `td.rex`, eleven internal calls up, 2000 | 5,364,233,244 | 4,996,054,421 | 5,446,879,828 | +9.0% | +1.5% |
| `calls.rex`, 100000 function calls and method sends, nothing raised | -- | 1,458,249,611 | 1,464,207,591 | +0.41% | -- |

* The report's `t.rex` (+0.14%) and `tu.rex` (+9.36%) figures reproduce to within 0.05 points.
* Against the Task 8 BASE, which the I5 ruling names, every trap loop is within 10%: -12.5%, -4.5%,
  +6.2% and +1.5%. I5 holds.
* The report's "+9.4% for an error trapped one level up" is the internal-call case only. A method
  level costs more: +21.1% against round A for one method left (`tm.rex`). Minor m6: the figure is
  true of what was measured and not of "one level up" in general.
* The call path with nothing raised moves +0.41%, inside the layout band this project measured
  (up to 4% from dead code alone), so no per-clause cost is shown; rexxcps was not re-run here.


## 5. The falsified controls

* **D5** (`run_loaded` restoring the caller's clause after `install_directives`): falsified because
  the restore was redundant. The code was removed, and D5b, the restore at the end of `run_loaded`,
  went red on `trapped_condition_raising_program.rex`. Nothing is left unwitnessed: the removed
  code does not exist.
* **E3** (the tail not made in the object's stem): the first witness could not see it (the same
  tree either way). The witness grew a defaulted stem whose tails a method exposes, and E3b went red
  on its `count` line. Witnessed now. The E3b prediction was written after the run, as the report
  says.
* **K3** (the receiver's package not copied): red as predicted, with the wrong mechanism (REXX
  rather than `.nil`). Witnessed.
* **S5** (the dispatcher's take on the RETURN path): the mutation that ran kept the take and only
  dropped the early return, and showed that the return keeps the issuing activation's `CALL ON
  NOTREADY` from running. **The described mutation, no take at all, was never run.** The report
  says it "would leave the reader looping". So no control shows that any witness notices when the
  reader fails to stop, and N5 is exactly such a case: three ordinary ends of input leave it
  looping, and no witness has any of them (`input_stream_object_reader.rex` ends each input with a
  RAISE ... RETURN or a native NOTREADY).


## 6. The Phase 9 residuals

Each residual's probe re-run from a fresh directory (`$R/q/r1`-`r4`):

* An INTERPRET fragment still running pushes no frame (`r1`): oracle INTERPRET, ROUTINE, PROGRAM;
  here ROUTINE, PROGRAM. True.
* An external program called as a function is a PROGRAM frame (`r2`): oracle ROUTINE, here PROGRAM.
  True, and incomplete: N1 (a trapped condition loses that level altogether) is not in it.
* A failure in an external program's directive install (`r3`): oracle POSITION 2 in `bad.rex`,
  frames `METHOD ::CONSTANT`, `ROUTINE bad.rex`, the caller's, traceback `2 *-* ::constant`,
  `1 *-* ::class K`, the call; here POSITION 2 in `main.rex`, the caller's frame alone, no `::class
  K` line. True (the record's POSITION 3 is for its own file's layout).
* `~empty` on an object's stem while a method's stem exposes one of its tails (`r4`): oracle `after
  empty x` / `object A.1 0`, here `after empty A.1` / `object y 1`. True.

Plausibility. None of the four is a member of the three groups (the instrument is green with
`RECORDED` the queue test alone), and Phase 9's exit is the full suite green, so Phase 9 is a
plausible owner under the I6 ruling. Two caveats for the lead:

* The first three are I2's own shape, which was ruled "fix", not "record".
* The fourth is new code's incomplete case: round B turned a loud EXPOSE refusal into this silent
  answer.

If the I2 ruling meant the whole shape, N1 and residuals 1-3 are Phase 8's.


## Housekeeping

`$R/tgt2` deleted after the runs. `$R/h2` (the archive, its `ootest/` copy restored and `cmp`-equal)
and every probe and profile stay in `$R`. Nothing in the repository was edited.
