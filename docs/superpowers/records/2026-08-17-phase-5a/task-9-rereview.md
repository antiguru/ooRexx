# Task 9, fix round 1 -- rereview

Base `3e695182c`, head `40ea10305`. Diff:
`.superpowers/sdd/2026-08-17-phase-5a/review-3e695182c..40ea10305.diff`.

## Verdict: approved, one minor defect (prose only, not code)

All six brief items are addressed correctly, and the seventh (the path projection the
implementer found itself) is correct and genuinely closes a determinism hole no gate was
watching. Live oracle probing of the round's only code change (`TOSTRING`) on shapes the
committed corpus does not cover found no new divergence. Gates are genuinely green, run cold.
One new false statement was found, in the report's own prose, not in code, and it does not
change any conclusion the round or Task 10 depends on.

## 1. Citation sweep

Checked every citation the fix touches against the actual C++ source:

* `PackageClass::getProgramName` -- `classes/PackageClass.hpp:147` is
  `RexxString *getProgramName() { return programName; }`; `memory/Setup.cpp:1189` is
  `AddMethod("Name", PackageClass::getProgramName, 0);`. Both exact.
* `RexxObject::classObject` -- `classes/ObjectClass.cpp:1814` is
  `RexxClass *RexxObject::classObject() { return behaviour->getOwningClass(); }`. Exact, and the
  doc's quoted body matches verbatim.
* `runtime/MethodArguments.hpp:136` / `:161` -- line 136 is the `size_t position` overload, line
  161 is the `const char *name` overload whose `OREF_NULL` arm is
  `reportException(Error_Invalid_argument_noarg, name)`. Exact.
* `memory/Setup.cpp:733`-`:734` -- line 733 is `AddMethod("MakeString", ArrayClass::toString, 2);`,
  line 734 is `AddMethod("ToString", ArrayClass::toString, 2);`. Both bind the same function at
  arity 2, exactly as claimed.
* The one citation left deliberately bare, "`.Package` is not in `CoreClasses.orx` or
  `StreamClasses.orx`", is a whole-file negative claim with a stated pattern, not a method
  citation. Verified the pattern itself: `/bin/grep -ain '\.package\b'` over both files matches
  nothing; the same pattern with `\.array\b` matches 11 times across the two files. So the control
  the report cites (the pattern can match, and does for a different token) is real, and the claim
  is exactly as wide as its pattern.

Swept `dispatch.rs`, `error.rs` and `value.rs` for every remaining `PascalCase::camelCase`-shaped
citation (the project's C++-method style) and confirmed each carries a `.cpp:`/`.hpp:` line. Three
citations turned up with no adjacent line number on a first pass --
`RexxObject::messageSend` (dispatch.rs:106), `RexxBehaviour::methodLookup` (dispatch.rs:532), and
`NativeActivation::run` (dispatch.rs:608) -- but `git blame` puts all three at `afe6985ebf`
(2026-08-16, Task 5), well before task 9's base commit `4729e5d3a` (2026-08-21) and untouched by
any commit in `4729e5d3a..40ea10305`. Out of scope for this task's sweep, which is `git diff
4729e5d3a..HEAD` by the report's own description. No bare citation remains inside task 9's own diff.

## 2. `TOSTRING`

Verified "same C++ function and arity" at the source: `memory/Setup.cpp:733`-`:734` bind
`ArrayClass::toString` at arity 2 under both names, and `ArrayClass.hpp:161` declares the one
function both rows in `NATIVE_METHODS` point at (`native_array_make_string`, unchanged). The
function itself does not special-case either name; the `Compiled method "..."` frame reads the
name from the `NATIVE_METHODS` tuple used to look the entry up, not from the implementation, so
`~toString`'s frame genuinely reads `TOSTRING` and not `MAKESTRING`.

Built `rexx-run` at head (`40ea10305`) and probed live against the oracle, `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker`, on shapes the committed corpus does not cover:

| probe | oracle | crate (both engines) |
|---|---|---|
| `.T~superClasses~toString` (default/`L`/`C`), `T` a `::CLASS`-declared class with a mixin | matches `~makeString` | matches, byte-identical to oracle |
| `.Array~toString` (the class object itself, i.e. a class-side send) | rc 159, 97.1 `does not understand message "TOSTRING"` | rc 159, identical 97.1 |
| `.Array~superClasses~toStrings` (name that does not exist) | rc 159, 97.1 `"TOSTRINGS"` | identical |
| `t~toString('L', ' ', 'z')` (wrong arity, 3 args) | rc 163, 93.902 `2 expected`, frame `Compiled method "TOSTRING" with scope "Array".` | identical, same frame |

One disagreement surfaced: `.Array~superClasses~toString` (a primitive's own superclasses) prints
only `The Object class` here against the oracle's `The Object class` / `The OrderedCollection
class`. Checked whether this is new: `.Array~superClasses~makeString` shows the identical gap
(same missing row, same both engines). This is the pre-existing `superclasses` divergence the
round's own plan-doc amendment names ("Array and List from OrderedCollection" among the eleven
classes whose wiring row needs `CoreClasses.orx`'s `~inherit` targets, owned by Task 13), surfacing
through a second method name rather than a new one. Not a new divergence; `TOSTRING` reproduces
`MAKESTRING` byte for byte, known gaps included.

## 3. Instruction accounting

Checked every number against `rust/bench-baselines/phase-5a-arms.tsv`:

* `strings` axis, `per_pass` `instructions:u`, commit `5fd2002cf`: pinned `ir` 5369.518137, head
  `ir` 5421.518032. Delta 51.999895, rounds to the claimed "+52 on both engines as `18626fdb1`" --
  confirmed identical to `18626fdb1`'s own rows.
* 1% of the pinned baseline: `5369.518137 * 0.01 = 53.69518137`, rounds to the claimed "53.70".
  `53.70 - 52.00 = 1.70` -- the claimed "1.7 instructions per pass of headroom" holds.
  `1.010000 - 1.009685 = 0.000315`, i.e. 0.0315 percentage points, rounds to the claimed "0.03
  points of ratio".
* The `try_text`-arm-removed row (9048.52 tw / 5373.52 ir, delta +4 from pinned) is not itself a
  TSV row in this fix round's data (it is an ablation build from the task's own prior sitting,
  reported already-approved), so `try_text`'s own arm costing 48 (52 - 4) and `to_text`/`text_len`
  costing 4 is arithmetic on numbers already in the approved report, not new to this round; it
  checks out.

Headroom conclusion for Task 10 holds: the `ir` arm has 1.7 instructions per pass left before the
1% guard on `strings`, and it is `try_text`'s array arm that is expensive, not `to_text`/`text_len`.

**One new false statement, low severity.** The round's own "reproduces the earlier build to six
decimal places" paragraph lists `alloc4c`'s four `pinned>head` ratios as
"1.000795/1.001204/1.000774/1.001157". The TSV row for `ir small` (`9-fixround-1 5fd2002cf alloc4c
pinned>head across_builds ir small instructions:u`) has `value_median 1.001203`, `value_min
1.001202`, `value_max 1.001204` -- the report quotes the max, not the median, and calls it a
six-decimal reproduction. The true median (1.001203) does match the prior sitting's `18626fdb1`
row exactly, so the *reproduction claim itself* is true against the data; only the transcribed
digit in the report is off by one in the sixth place. Does not touch the `strings`-axis headroom
figure Task 10 depends on, which is correct as checked above.

## 4. Path projection

`corpus/README.md`'s "one rule: determinism" (lines 8-9) states: "No `DATE()`, no `TIME()`, no
process IDs, no **file system state**..."; further down (lines 67-69) it says `PARSE SOURCE` is
admissible but its third word is not, "that word is the program's own absolute path, so a program
printing it would put the filesystem in its output and break the determinism rule above." This is
exactly what the report cites.

Ran `class_package.rex` as committed at head against the oracle and both engines from a fresh
directory: no row prints the path on any side. The three replacement assertions
(`k-package-name-is-the-source`, `k-package-name-is-rexx`, `array-package-name-is-the-source`) read
`1 0 0` identically on oracle, `ir` and `tree-walker`.

**"No gate could catch it" verified true.** `crates/rexx-exec/tests/corpus.rs`'s own module doc
confirms the differential runs both interpreters against the *same file on the same run*, then
compares their outputs byte for byte -- it never compares against a golden/fixed transcript. Since
both interpreters are handed the identical absolute path in the same invocation, both emit the
identical (if path-dependent) bytes, and the comparison sees no divergence regardless of the
determinism violation. No other gate (`sourceline_oracle`'s line-count/verbatim check, the
self-test the README describes) touches program *output* either. The claim holds as stated.

## 5. Comment-prose pass

Ran the collapsed-comment-block diff (base `3e695182c` vs head `40ea10305`) over `dispatch.rs`,
`error.rs`, `value.rs`: every changed line matches one of the six brief items already accounted
for above -- no line turned up that isn't already covered by the citation fixes or the enumeration
deletions. Read the full diff for the `.rex` corpus files, `phase-5a.txt` and the plan-doc
amendment by hand; nothing else introduces a set-cardinality, enumeration, or historical-framing
violation beyond what's already flagged in item 3 above (the six-decimal transcription slip).

One borderline case considered and not flagged: `class_method_own_dictionary.rex`'s new comment
and the plan-doc amendment both say the method-identity gap "belongs to... 5c, with the method
rows." `rust/CLAUDE.md`'s own rule against phase-status framing uses "owned by 4c" as its
counter-example, which this superficially resembles. Not flagging it: the fixround-1 brief
explicitly directed "say which task the identity half belongs to" (item 2), and the plan already
carries the same style of forward attribution before this round ("Task 21 owns it now", "Task 13"
in the pre-existing text this round only corrected). This is the SDD handoff convention the
project's own method memory endorses (write a cross-task finding into the receiving task's own
text), not a mutable-aggregate status count.

## 6. The sitting

Confirmed all `9-fixround-1 5fd2002cf` rows exist in `bench-baselines/phase-5a-arms.tsv` across
all six axes (`alloc4c`, `arith`, `compound`, `emptyloop`, `strings`, `varlookup`) with
`arm_ratio`/`absolute`/`per_pass`/`per_pass_gap`/`across_builds` scopes present for both `tw` and
`ir`. Compared every `pinned>head across_builds instructions:u` row against the prior sitting's
`9 18626fdb1` rows: identical to six decimal places on every one, matching the report's claim
(`compound`/`emptyloop`/`varlookup` all `1.000000`; `arith` and `strings` match digit for digit;
`alloc4c` matches at every value except the one transcription slip in item 3's table, which is a
report-prose error and not a data discrepancy).

The predicate ("run a sitting only if this round's code touches `to_text`/`text_len`/`try_text`'s
path") reads correctly against the diff: the only `src/` change is a new `NATIVE_METHODS` row
pointing at the existing `native_array_make_string`, plus doc comments. Nothing on the value-model
match arms moved. The round's own reasoning for running the sitting anyway (caution, plus stating
the headroom consequence) is sound and does not contradict the predicate.

## Gates, run myself, unpiped

* `cargo fmt --all --check` from `rust/`: exit 0.
* Touched `dispatch.rs`, `error.rs`, `value.rs` to force a cold recheck, then
  `cargo clippy --workspace --all-targets -- -D warnings`: recompiled `rexx-exec` (2.42s), exit 0.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`: exit 0. Every `test
  result:` line in the full log reads `ok`, `0 failed`; no `FAILED`, no panic text, anywhere in the
  output.

## Summary of what to carry forward

* No code-level regression or new divergence found. `TOSTRING` is a correct, byte-identical second
  name for `MAKESTRING`, confirmed against the live oracle on shapes beyond the committed corpus.
* The path-projection fix is correct and the "no gate could catch it" claim is verified true by
  reading the differential harness's own comparison model.
* One cosmetic defect: a transcribed digit in the round's own "reproduces to six decimals" table
  (`alloc4c`/`ir`/small, reported 1.001204, actual median 1.001203). Does not affect the `strings`-
  axis headroom figure Task 10 depends on (independently verified correct), and does not affect any
  code or test. Worth a one-line correction, not a re-open.
