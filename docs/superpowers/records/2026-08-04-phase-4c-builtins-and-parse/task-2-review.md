# Task 2 review: builtin dispatch, the arity table, and the 40.x error family

Reviewing `c10e40df` against parent `3a51a3b0`. HEAD is `fc23d6ca` (plan-document-only,
controller's own fold of this task's findings); ignored, as instructed.

**Spec compliance: PASS.**
**Quality: APPROVED.**

Everything the report asserts was re-measured independently rather than accepted. All
eleven oracle transcripts the report relies on reproduce exactly, in a directory created
empty for this review. The Step 4.3 falsification reproduces exactly. Three separate
mutations were used to make the important new guards fire.

---

## 1. Re-run verification (my numbers, each status read unpiped)

| command | exit | result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1039 passed, 0 failed, 4 ignored, across 71 `test result` lines; zero `FAILED` lines |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean (warm) |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | 9 passed, 1 ignored, **42 of 42 matching** |
| `cargo clippy` from a cold `CARGO_TARGET_DIR` | **0** | clean from a cold start; the report's own extra check confirmed |

1039 matches the report. The diff adds 8 tests (5 in `builtin/mod.rs`, 3 in `run.rs`) and
renames one in `eval.rs`, which is consistent with the report's `1031 -> 1039`.

Probe hygiene: every oracle invocation used the mandated
`( ulimit -v 1048576; LD_LIBRARY_PATH=... rexx ABSOLUTE_PATH )` wrapper, three separate
descriptors, from `.../scratchpad/rev2-probes/`, which I `mkdir`ed and confirmed held two
entries (`.` and `..`) before writing anything into it.

---

## 2. The restructuring of `resolve_and_run_call` (review point 1)

I extracted the function body from both revisions and diffed them in isolation. The whole
change is **three hunks**:

1. `let target = if search_labels` renamed to `let label = ...`.
2. The old two-line `let Some(target) = target else { return Err(Loud::unresolved_call(name)) };`
   replaced by the three-arm `match label` producing `Resolved::Label` / `Resolved::Builtin` /
   an immediate `return Err(Loud::unresolved_call(name))`.
3. The new `Resolved::Builtin` block inserted after the argument loop.

**Nothing else in the function changed -- not one line.** The argument loop, `set_sigl`, the
`MAX_ACTIVATION_DEPTH` guard, the activation push, the five pieces of saved level state and
both restore paths are byte-identical to the parent. So:

* **Argument evaluation order:** unchanged. The loop is the same loop, in the same place.
* **`>A>` trace lines:** unchanged; `trace_argument` is called from the same two places
  inside the same loop.
* **`SIGL`:** unchanged. `set_sigl` sits below the insertion point and is unreachable from
  the builtin arm.
* **Depth counter:** unchanged, same reasoning.
* **Temps-frame discipline:** the loop's `roots.push_temp(argument.value())` is untouched.
  4b's `step_in_temps_frame` chokepoint pushes a frame around the whole `step` call and pops
  it on every exit including the error ones, so the two *new* exits (the `return Ok(...)`
  in the builtin arm and the `result?`) are inside exactly the same frame as every
  pre-existing `?` in this function. **No `?` return moved.**

The loud return for an unresolvable name is still upstream of the argument loop, exactly
where the parent had it, so the 4b behaviour it guards is preserved bit-for-bit.

The hook is in the place the brief demanded -- after argument evaluation, not at the
label-lookup fallback. The `Resolved` enum having two variants rather than three, with the
"nothing" outcome returning at the point of decision, is the right shape: it makes it
impossible to carry a resolution result that has nothing to run. **No second resolution
path was added in `eval.rs`** -- the only `eval.rs` change is a test rename (section 6).

**A measured improvement that falls out of the restructure.** Because a builtin *name* now
resolves before the loud return, its arguments are evaluated even when the builtin is
unimplemented. Measured, `say substr(1/0)`:

* parent: loud, rc 120
* this commit: `Error 42.3: Arithmetic overflow; divisor must not be zero.`, rc **214**
* oracle: identical bytes, rc **214**

So the restructure closed a real divergence for all 65 unimplemented builtins as a side
effect. The remaining deviation is the one the report declares and hands to Task 13:
`say zorkolo(1/0)` is still loud here where the oracle gives 42.3 rc 214 (I confirmed the
oracle side). That is the declared gap, correctly scoped.

---

## 3. The three measured answers (review point 2)

All three re-measured on the oracle, and all three also re-run through `rexx-run` so the
implementation is checked against the measurement rather than against the report.

**3.1 `SIGL` is not set on the builtin path.** Oracle and Rust produce identical stdout:

```
sigl0= SIGL
sigl1= SIGL
sigl2= 4
```

`SIGL` is still the uninitialised derived name after `n = length('abc')`, and is the
`CALL`'s own line 4 after `call sub`. Confirmed both sides.

**3.2 `>A>` fires identically to a label.** Oracle and Rust stderr, byte-identical:

```
     2 *-* n = length('abc')
       >L>   "abc"
       >A>   "abc"
       >F>   LENGTH => "3"
       >>>   "3"
       >=>   N <= "3"
```

and the `CALL` form, also byte-identical on both sides:

```
     2 *-* call length 'abc'
       >L>   "abc"
       >A>   "abc"
       >>>   "3"
```

The label comparison (`n = sub('abc')`) shows the same `>L>`/`>A>` shape with the callee's
own clauses echoing two columns further in, exactly as the report states.

**3.3 The activation depth counter does not increment.** Both observables reproduce. The
builtin's `>F>`/`>>>` sit at the calling clause's indent (above). And the echo stack:
`say substr('abc')` at top level echoes **one** clause; inside `sub:` it echoes **two**
(the failing clause, then `call sub`) and none for the builtin itself.

All three are stated in `builtin/mod.rs`'s module doc with their probes, as the brief
required.

---

## 4. Quoted literal targets and case sensitivity (review point 3)

Oracle, re-measured:

| probe | rc | result |
|---|---|---|
| `say "LENGTH"('abc')` | 0 | `3` |
| `say "length"('abc')` | 213 | `Error 43.1:  Could not find routine "length".` |

Rust: `"LENGTH"('abc')` prints `3` rc 0; `"length"('abc')` is the declared gap, rc 120.
The report's claim is correct in both directions.

**What `dispatch` receives on each path**, read from the code rather than the report:
`eval_call` maps `CallTarget::Symbol(id)` to `code.symbols.name(*id).as_bytes()` (already
upcased -- confirmed behaviourally, `say Length('abcd')` gives `4`) and
`CallTarget::Literal(bytes)` to the verbatim bytes. `is_builtin` does
`str::from_utf8(name).is_ok_and(|n| in_scope().contains(n))` and `dispatch` does
`builtin.name == name` -- **no folding on either side**, which is exactly the oracle's rule.
Non-UTF-8 literal bytes match nothing, which the unit test pins with `&[0xff, 0xfe]`.

---

## 5. Arity and the error families (review point 4)

All three of the report's corrections to the brief are **confirmed**:

| probe | rc | secondary |
|---|---|---|
| `say substr('abc',,2)` | 216 | `40.5  Missing argument in invocation of SUBSTR; argument 2 is required.` |
| `say substr('abc',2,-1)` | **163** | `93.923  Invalid length argument specified; found "-1".` |
| `q(1,,)` `q(,1)` `q(,)` `q()` `q(1,,2,,)` into `q: return arg()` | 0 | `[1] [2] [0] [0] [3]` |

So 40.5 is genuinely missing from the brief's table; a negative length is genuinely 93.923
at rc 163 and not in the 40 family at all; and trailing omissions are genuinely dropped
before the count is taken. The brief was wrong twice and silent once, as reported.

Check order also confirmed: `say substr(,2,3,'p','q')` is **40.4**, not 40.5, so the maximum
is checked before which positions were supplied.

**`check_arity` implements what was measured, not what the brief said.** Verified line by
line: max first, then min, then required-position. I also probed a case the report did not:
`say substr(,,3)` and `say substr(,2,3)` both blame **argument 1**, i.e. the *first* missing
required position wins -- which is what `.position(Option::is_none)` returns. Correct.

Rust-vs-oracle on the boundary cases, byte-identical including full transcripts:

| probe | oracle | rust |
|---|---|---|
| `say length(,)` | 216, `40.3 ... minimum expected is 1.` | identical |
| `say length('abc',)` | 0, `3` | identical |
| `say length(,'x')` | 216, `40.4 ... maximum expected is 1.` | identical |
| `say length('abc','x')` | 216, 40.4 | identical |
| `say length()` | 216, 40.3 | identical |

### Finding I1 (Important): the required-argument model cannot express conditional requirements

`Builtin.min` doubles as "the number of leading positions that may not be omitted", and
`check_arity` enforces it as `args[..min].iter().position(Option::is_none)`. That assumes
the required positions are always a **prefix**. Measured, they are not:

```
say date()            rc 0    -> 5 Aug 2026            (so DATE's min is 0)
say date('S',,'S')    rc 216  -> 40.5  Missing argument in invocation of DATE; argument 2 is required.
```

Argument 2 of `DATE` is required only *because* argument 3 was supplied. With `min = 0`,
`args[..0]` is empty and `check_arity` can never raise 40.5 for `DATE` at all. `DATE` is an
in-scope `loud` row in `builtin-status.txt`, so a family task will implement it and hit
this.

Three other builtins I probed *are* expressible by the prefix model and would be answered
correctly (`translate(,'a','b')` -> argument 1; `copies(,2)` -> argument 1;
`overlay('a',,3)` -> argument 2), so this is not a common shape -- but it is a real one.

Nothing shipped is wrong: `LENGTH` is unaffected and no builtin with this shape exists yet.
The issue is that the report describes `check_arity` as "the 40.x incorrect-call checks
every builtin shares", and the module doc says an implementation "cannot be reached with an
argument list it did not ask for" -- both slightly overclaim. The `(min, max)` model came
from the brief, so this is not the implementer inventing a weak design; it is a gap that
needs to reach the family task that implements `DATE`. Building the conditional machinery
*now* would be the same uncalled-code problem that (correctly) deferred 40.12/40.23, so the
right fix is a plan note, not a code change in this task.

---

## 6. `LENGTH` itself (review point 5)

Eleven shapes probed on the oracle and re-run through `rexx-run`. **Every one is
byte-identical**, stdout, stderr and exit status:

| probe | both |
|---|---|
| `length('')` | `0` |
| `length(123)` | `3` |
| `length(1.50)` | `4` |
| `length(zork)` (novalue) | `4` -- the derived name `ZORK` |
| `length(x.1)` (unset tail) | `3` -- `X.1` |
| `length(1e3)` | `3` -- `1E3` |
| `length(2+3)` | `1` |
| `length(00012)` | `5` -- the literal's own bytes |
| `length(' ab ')` | `4` |
| `length(x.1)` with `x.1='hello'` | `5` |
| `length(s.99)` with `s.='dflt'` | `4` -- the stem default |

No argument, two arguments and the omission forms are in the table in section 5, also
byte-identical.

The D15 handling is right and I proved the test is not vacuous (section 8, M3). The result
is created through `Interp::text` -> `Interp::alloc_with`; **no `Heap::alloc` or
`alloc_with_uncollected` appears anywhere under `src/builtin/`**, and there is no `unsafe`
(the workspace sets `unsafe_code = "forbid"`).

### Finding I2 (Minor): the new allocation site has no GC-stress witness

`collect_stress.rs` reads its subset from `corpus/phase-4a.txt` + `corpus/phase-4b.txt`
(42 programs). **None of them calls a builtin** -- the single grep hit for "length" is a
comment in `lang/if_else_chain.rex`. So the shared block's rule ("a builtin's result must
be rooted before any subsequent allocation") is unwitnessed for the path this task created.

I settled whether that is a coverage gap or a live bug by writing a throwaway integration
test against `run_program_collect_every_alloc` (since deleted; tree verified clean after).
Eight shapes -- plain, as a concatenation operand, nested `length(length(..))`, as an
arithmetic operand, 25 in a loop, the `CALL` form, as a sub-expression of its own argument,
and stored-then-reread across later allocations -- **108 collections, zero divergence from
the plain run.** So the rooting is correct; it is only untested. Recommend the first family
task add one corpus program that calls a builtin to the stress subset.

---

## 7. The two files outside the brief's list (review point 6)

**`eval.rs` -- necessary and minimal.** 4b's `a_builtin_name_still_fails_loudly_naming_4c`
used `say length('abc')` as its witness; implementing `LENGTH` falsifies it directly, so the
test had to change. The change is a rename plus swapping the witness to `zorkolo`. This is
the right repair rather than picking another not-yet-implemented builtin: a real builtin
would encode where the boundary sits in a test body and go red the day that name landed,
which is what `builtin-status.txt` exists to carry instead. No coverage is lost -- the loud
property over all 65 unimplemented builtins is asserted by `builtin_status.rs`, and the
renamed test now covers the genuinely-unresolvable case, which nothing else did.

**`corpus/keyword-exempt.txt` -- correct, and correct in the way that matters.** The row was
**removed**, not re-attributed. The diff shows `-NUMERIC::test_26	4c` with no `+` line. That
is what the file's own harness demands: `keyword_assertions.rs:432` emits
`"{key} now PASSES but is still on the committed exempt list -- remove it"`. Counts verified
independently: 789 rows tagged `4c` + 6 `defect:` rows = 795 total data rows, and the header
now reads `4c  789 bodies`. Consistent in both directions. And the body genuinely passes for
the stated reason -- it is `Numeric Digits 1` with `If length(s)>30`, so it passes precisely
because `LENGTH`'s result is DIGITS-independent, an independent confirmation of the D15
handling.

That the whole workspace is green is itself the strong check here: `keyword_assertions.rs`
asserts membership in both directions, so any other body that started passing would have
gone red.

---

## 8. Making the guards fire (review point: "a guard nobody has watched fail is untested")

Three mutations, each restored from a `cp` backup and each verified with `git status`.

**M1 -- the 63-vs-66 trap.** Made `in_scope()` subtract `PARTIALLY_EXCLUDED` as well
(i.e. the `NAMES - EXCLUDED` = 63 mistake the brief warns about).
`the_partial_exclusions_are_builtin_names_and_the_whole_ones_are_not` went red with
`VALUE is excluded only in part, so its in-scope form must dispatch`. 4 passed, 1 failed.

**This is the interesting part:** under the same mutation, `builtin_status.rs` stayed
**fully green, 11 passed, 0 failed**. It cannot see the trap, because
`Loud::unresolved_call(name)` is the answer for *both* "not a builtin name" and "a builtin
name with no row" -- the two are indistinguishable from outside. So the unit test is the
**only** guard for a 3-name silent-wrong-routine hazard. That is a real addition, not a
"can fail" test.

**M2 -- the check order.** Moved the required-position check ahead of the maximum check.
`too_many_arguments_wins_over_a_missing_required_one` went red, and only that test.

**M3 -- D15.** Rewrote `string::length` to build its result through `Interp::number` under
the digits in force. `a_builtins_result_renders_independently_of_the_digits_in_force` went
red with exactly the predicted divergence -- `1E+1` where the oracle gives `10`:

```
  left: "16\n2E+1\n1E+1\n"
 right: "16\n2E+1\n10\n"
```

So the D15 test does discriminate, and it discriminates on the third line, which is the line
the report identified as the one that rules out `Interp::number`.

**"Can fail" vs "adds coverage"** for the rest: `dispatch_declines_a_name_that_is_not_a_builtin`
pins `None` against `Some(loud)`, which -- per M1's result -- nothing else can see, so it
adds coverage. `the_arity_checks_answer_the_oracles_own_sub_codes` is the only test that
reaches 40.5 at all. The three `run.rs` tests are the only end-to-end builtin coverage.
`every_implemented_row_names_an_in_scope_builtin` is the weakest of the eight: a typo'd row
would also flip `builtin-status.txt` and be caught there. Its `max >= min` assertion is new,
though, so it is not redundant. No test in the set is pure duplication.

---

## 9. Task 1's Step 4.3 -- reproduced (review point 7)

Backed up `builtin/mod.rs` (md5 `4b5c96bf...`), replaced the single `IMPLEMENTED` row with
`const IMPLEMENTED: &[Builtin] = &[];` -- `LENGTH`'s dispatch arm and nothing else -- and ran
`cargo test --offline -p rexx-exec --test builtin_status`.

**11 tests ran, 10 passed, 1 failed.** Exit 101. Exactly the report's numbers.

```
---- the_status_file_matches_a_live_differential_run stdout ----
rows whose measured status differs from .../corpus/builtin-status.txt:
  LENGTH: committed implemented, measured loud
        differing: [stdout, stderr, exit code]
        rust:   stdout="" stderr="rexx-exec: routine \"LENGTH\" is not implemented (4c)\n" exit=120
        oracle: stdout="6\n" stderr="" exit=0
```

**Exactly one row flipped and it was `LENGTH`'s** -- the panic lists no other row, so nothing
else moved in either direction. The build emitted one `dead_code` warning for
`string::length`, the expected shape of the mutation.
`every_loud_row_is_loud_about_its_own_builtin` stayed green, so the newly-loud row named
`LENGTH` and not something else.

This is the falsification Task 1 owed: the status file is derived from a live differential
run of the interpreter, not from a name table. A name-table classifier would have gone on
reporting `implemented`.

Restored with `cp` from the backup (**not** `git checkout --`). md5 matches the backup,
`git status --porcelain` is empty, and the harness is green again at 11 passed / 0 failed.

---

## 10. The deferred 40.12 / 40.23 raisers (review point 8)

**The deferral is right, and it costs nothing.** Two reasons, both checked:

1. The `dead_code` argument is real. `Raised::*` are `pub(crate)`; an uncalled one warns in
   the plain library build, and the verify block's
   `cargo clippy --all-targets -- -D warnings` is itself a binding requirement. A `cfg(test)`
   caller would not rescue it, because the lib target is built without `cfg(test)`. So adding
   the raisers now would have failed a binding gate to satisfy a descriptive sentence.
2. **The information is not lost, because the message text is generated, not hand-written.**
   `Raised::message` looks up `rexx_inventory::errors::lookup(major, sub)`, whose table
   `crates/rexx-inventory/build.rs` generates from `interpreter/messages/rexxmsg.xml`. I read
   both entries there directly:

   * 40.12 `Error_Incorrect_call_whole`:
     `<Sub 1 function_name/> argument <Sub 2 argument_number/> must be a whole number; found <q><Sub 3 value/></q>.`
   * 40.23 `Error_Incorrect_call_pad`:
     `<Sub 1 function_name/> argument <Sub 2 argument_number/> must be a single character; found <q><Sub 3 value/></q>.`

   So the substitution count and order are already in the tree. A family task writes
   `Raised::syntax(40, 12, vec![name, position, found])` and the report's transcripts confirm
   the rendered bytes. **The transcripts are sufficient.**

The one thing the report does not spell out is that third substitution's *identity* -- whether
`found` is the argument's rendered value or its source spelling. The 88.928 raiser directly
above the new 40.5 in `error.rs` documents having measured exactly that distinction for
itself, so the family task should not assume; but the probes are cheap and the brief already
tells each family task to measure. Not a blocker.

40.5 *was* added, correctly, because `check_arity` has a live caller for it in shared
machinery. Its doc comment carries the measurement and the trailing-omission rule.

---

## 11. Project-constraint compliance

* No `unsafe` anywhere in `src/builtin/`; workspace `unsafe_code = "forbid"` intact.
* Every allocation goes through `Interp::alloc_with` (via `Interp::text`); no
  `Heap::alloc*` under `src/builtin/`.
* Zero em-dashes in either new file; `--` used throughout.
* **The comment rule is respected.** `IMPLEMENTED`'s own doc explicitly refuses to state the
  boundary and points at `corpus/builtin-status.txt` instead; `string.rs`'s module doc is
  "The string builtins", not "LENGTH only". Comments state oracle behaviour and measurements
  and nothing about where the implemented/not-implemented line sits. The closest call is the
  test fixture `const SUBSTR`, whose doc calls it a "stand-in" -- but it describes an arity
  *shape*, stays true whatever lands later, and is a test-local fixture. Not a violation.
* The commit is a single commit, parent `3a51a3b0`, and `.superpowers/` is gitignored so the
  report is not in it.

---

## 12. Cannot verify from the diff

* **The report's own probe directory hygiene** (that `t2probe-clean` held two entries when
  the measurements were taken). Moot in practice: I re-measured every transcript the report
  relies on in my own freshly-created empty directory and all reproduce.
* **The "baseline 1,031" figure.** I did not build the parent to count its tests. The current
  1039 and the 8 new tests visible in the diff are consistent with it.
* **Whether the 93.923 rc 163 answer generalises** beyond `substr`'s length argument to other
  range violations. One probe was run (the brief's), not a family. The family tasks should
  not assume 93.923 is the universal answer for "converted fine, out of range".
* The report's claim that `cargo fmt` failed once mid-task and was fixed -- unobservable
  after the fact, and immaterial.

---

## 13. Findings

| # | Severity | Finding |
|---|---|---|
| I1 | Important | `check_arity`'s required-argument model assumes the required positions are a prefix of length `min`; measured, `DATE` has `min` 0 yet `date('S',,'S')` is 40.5 "argument 2 is required", so the shared machinery cannot express it. In-scope `loud` row, so a family task will hit it. Nothing shipped is wrong; needs to reach the plan, not this commit. |
| I2 | Minor | The new allocation site has no collect-on-every-allocation witness: the stress subset's 42 programs call no builtin. I verified by hand that the rooting is correct (108 collections, 8 shapes, zero divergence), so this is a coverage gap and not a bug. |
| I3 | Minor | `dispatch` is handed a freshly built `Vec<Option<ObjRef>>` on every builtin call, purely to strip `Argument` to `ObjRef`. Harmless now, but it is a per-call heap allocation on the path all 66 builtins will take. |
| I4 | Minor | `check_arity`'s doc ("an implementation cannot be reached with an argument list it did not ask for") and the report's "the 40.x checks every builtin shares" both overclaim slightly, given I1. |
| I5 | Minor | The report does not record whether 40.12/40.23's third substitution is the argument's rendered value or its source spelling -- the exact distinction `error.rs`'s neighbouring 88.928 raiser documents having measured. The family task should measure rather than assume. |

No Critical findings. No spec requirement unmet.

---

## 14. Verdicts

**Spec compliance: PASS.** All six brief steps are discharged, including Step 5's
falsification, which I reproduced. Two deviations are declared in the report and both are
sound: (a) 40.12/40.23 deferred because an uncalled raiser fails the binding
`-D warnings` gate, with the measurements recorded and the generated catalog making the
deferral information-free; (b) the loud return for an unresolvable name kept upstream of
argument evaluation per Step 3's own wording, with the oracle's contrary behaviour measured
and handed to Task 13, which owns that fallback.

**Quality: APPROVED.** The restructuring -- the risky part -- is minimal and provably so: the
label path is byte-identical to the parent apart from one rename, and no `?` return moved.
The measurements are real and reproduce. The tests are non-redundant and three of the most
important ones were watched to fail. I1 should be folded into the plan for the family task
that implements `DATE`, and I2 into whichever family task lands next, but neither is a
defect in this commit.

**Tree state after review:** `git status --porcelain` empty, HEAD still `fc23d6ca`, no
stash created, `builtin/mod.rs` md5 `4b5c96bf...` matching its pre-review backup.
