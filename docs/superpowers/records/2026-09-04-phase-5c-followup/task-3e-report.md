# Task 3e report -- the conversions, and the commit that flips `say buf`

BASE `4ac617074395a47dbd66f1a49bc29b4ea5913e2e`. The fifth and last Task 3
commit of `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`.

Scratch is `…/scratchpad/task3e/` (`$S` below); the BASE build is
`/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3e/` (`$B`).
Every claim below is marked **measured** (with the command or file that
produced it) or **inferred**.

---

## 1. What landed

* `crates/rexx-exec/src/builtin/string.rs` -- two cores over `&[u8]`:
  `line_slices` (`StringUtil::makearray`'s default separator,
  `classes/support/StringUtil.cpp:545`) and `split_slices` (the explicit
  separator, `:552`-`:640`). No existing core changed.
* `crates/rexx-exec/src/dispatch.rs` -- four `NATIVE_METHODS` rows in
  `MutableBuffer`'s own alphabetical block, at the counts `Setup.cpp` declares:
  `MAKEARRAY` (1, `:1445`), `MAKESTRING` (0, `:1473`), `SETTEXT` (1, `:1442`),
  `SUBWORDS` (2, `:1461`); their four bodies; the shared `array_of_texts`; and
  one argument helper, `optional_string_or_none_argument`.
  `native_string_makearray` now calls `line_slices` instead of carrying the
  same loop inline -- **its own array construction is untouched**, for the
  reason in open question 1.
* `crates/rexx-exec/src/dispatch.rs`'s
  `a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state`
  -- the `say .MutableBuffer~new('abc')` case asserted rc 120 with the
  `MAKESTRING` refusal; it now asserts `abc` at rc 0, and a second assertion
  beside it pins the four comparisons of §2.
* `corpus/lang/mutablebuffer_conversion.rex` (60 lines, 36 lines of output),
  filed in `corpus/phase-5c.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5C`,
  with its `crates/rexx-parse/tests/sourceline_oracle/mutablebuffer_conversion.txt`
  companion.
* `corpus/method-bodies.txt` -- refreshed; the eleven `loud` rows of §5 move to
  `answers`.

`corpus/refusal-sites.tsv` is **not** in the diff: no `Raised` constructor was
added or moved, and `--test refusal_sites` reads 5 passed without
re-derivation (§7).

### 1.1 The plan gap: `SETTEXT` is in no family list

**Measured** by reading the plan. Task 3's paragraph names four commits and
lists their methods: readers (`substr [] pos lastPos countStr verify subWord
word wordIndex wordLength words wordPos contains containsWord startsWith match
matchChar subChar`), mutators (`insert overlay replaceAt []= changeStr upper
lower translate space delWord delete`), caseless, and conversion (`makeString
string makeArray subWords`). `setText` appears in none of them, and no task
brief before this one claimed it. It is a mutator -- it replaces the contents
and answers the receiver -- so it belonged in Task 3c's list; Task 3c's report
§1 does not name it either.

It is here because it was the last unbound `MutableBuffer` instance row, and
because leaving it would have left the class one method short with no task
owning it. **The plan's Task 3 list is the thing to correct**, not this
report: a later reader deriving the family split from the plan will otherwise
count eleven mutators where there are twelve.

---

## 2. The three measurements the brief carried, re-measured here

All three were taken on the oracle before this task's dispatch; all three are
re-measured against the crate as committed (`$S/oracle/`, `$S/probes/`).

1. **`subWords` answers an Array.** Measured, oracle and both engines:
   `.MutableBuffer~new('a b  c')~subWords(2)~class` is `The Array class`, and
   the witness's `sw0` carries it. A body written as "the word range as text"
   would have been wrong; `word_slices` (`builtin/word.rs:167`) is what the
   body is built on.
2. **`makeArray` splits on line ends.** Measured: `~new('a b  c')~makeArray`
   answers one element that is the whole contents (`ma1`), and
   `~new('one'||'0a'x||'two')~makeArray` two (`ma2`). §3 says what the count-1
   argument does.
3. **The receiver-side comparison stays an identity compare.** Measured on the
   oracle and on both engines here: `buf == 'a b  c'` and `buf = 'a b  c'` are
   both `0`, `'a b  c' == buf` and `'a b  c' = buf` are both `1`, and
   `length(buf)` is `6`. The witness's `cmp` and `mtcmp` lines carry all four,
   in both operand orders.

**Why binding `MAKESTRING` cannot move the receiver side, read out of the
crate rather than assumed**: `Interp::operator_message_receiver`
(`eval.rs:1734`) answers `Some(value)` for a `Body::Instance`, so the operator
is **sent to the buffer as a message** before any operand test runs, and
`("Object", "==")`/`("Object", "=")` are `native_object_identical`, a handle
comparison. The required-string protocol is only reached for the *other*
operand order, through `classify_string_conversion`'s
`make_string_or_none`. Mutation M5 (§6.2) is the control that this reasoning
is load-bearing rather than decorative: defeating that arm for a buffer
reddens the witness.

---

## 3. What the arguments do

**Measured**, oracle and both engines (`$S/oracle/e/`, `$S/oracle/e2/`,
`$S/oracle/r/`).

### `makeArray`'s count-1 argument

Omitted, the separator is `LF` with `checkCR` on: a `CR` immediately before an
`LF` is dropped with it, a trailing `LF` terminates rather than separating, and
the empty buffer has no lines at all (`ma1`, `ma2`, `ma3`, `mtarr`).

Given, the argument is `stringArgument`-converted and `checkCR` is **cleared**,
so the `CR` rule stops applying (`ma4`: the same buffer under an explicit `LF`
separator keeps the `CR`, first piece 2 bytes against `ma3`'s 1). The null
string is the special case `StringUtil::makearray` handles first: one piece per
byte (`ma6`). A separator that does not occur leaves one piece (`ma7`), a
separator longer than the text likewise (`ma10`), a text that **ends** with the
separator adds no empty tail (`ma9`), and a numeric argument is used as its
string value (`ma8`, `2` splitting `a2b2c` into three).

### `subWords`' count-2 arguments

`optionalPositionArgument(position, 1, ARG_ONE)` then
`optionalLengthArgument(plength, MAX_WHOLENUMBER, ARG_TWO)`, **both converted
before the contents are looked at**. So the position defaults to 1 (`sw6`), the
count to "all the rest" (`sw2`), a count of `0` answers an empty array
(`sw5` field 1), a position past the last word answers an empty array (`sw4`
field 3), and a count past the end stops at the last word (`sw5` field 2).
The conversion order is observable: `subWords(9, .nil)` is **93.923** at rc
163, not the empty array a start past the last word would answer (§4).

---

## 4. Refusals measured before they were written

Every probe below was run on the oracle first, one two-line program each
(`buf = .MutableBuffer~new('a b  c')` then the probe line), from a fresh empty
directory made by `mktemp -d` (`$S/oracle/r/r_*.rex`, runner
`$S/run_oracle.sh`), then on this build under both engines
(`$S/run_crate.sh`, the same directory so both sides embed the same absolute
path in the traceback).

**Measured**: over `$S/oracle/{r,e,e2,mb,cmp,w,reach,dim}`, 94 probe-engine
pairs, **88 identical on all three descriptors** -- `cmp` on stdout, `cmp` on
stderr, and the rc compared -- and 6 mismatching, which are three exploration
programs and are accounted for in §8. **Every one of the 23 `r_*` refusal
probes agrees, on both engines.**

| probe | oracle | which raiser here |
|---|---|---|
| `buf~makeString(1)` | 93.902 `Too many arguments in invocation of method; 0 expected.`, rc 163 | `Arity::Fixed(0)`, the table |
| `buf~makeArray('a','b')` | 93.902 `1 expected` | `Arity::Fixed(1)` |
| `buf~setText('a','b')` | 93.902 `1 expected` | `Arity::Fixed(1)` |
| `buf~subWords(1,2,3)` | 93.902 `2 expected` | `Arity::Fixed(2)` |
| `buf~makeArray(.nil)` | 88.909 `Argument 1 must have a string value.`, rc 168 | `optional_string_or_none_argument` → `required_string_argument(.., 1)` |
| `buf~setText(.nil)` | 88.909 `Argument 1 must have a string value.`, rc 168 | `string_method_argument` → `required_string_argument(.., 1)` |
| `buf~setText`, `buf~setText(,)` | 93.903 `Missing argument in method; argument 1 is required.`, rc 163 | `string_method_argument` → `Raised::missing_method_argument(1)` |
| `buf~subWords(0)`, `('x')`, `('1.5')`, `(.nil)`, `(0,-1)` | 93.924 `Invalid position argument specified; found "0"` / `"x"` / `"1.5"` / `"The NIL object"`, rc 163 | `optional_position_argument` -- and `(0,-1)` proves argument 1 is converted first |
| `buf~subWords(1,-1)`, `(1,'y')`, `(1,.nil)`, `(1,99999999999999999999)`, `(9,.nil)` | 93.923 `Invalid length argument specified; found …`, rc 163 | `optional_length_argument` -- and `(9,.nil)` proves argument 2 is converted before the contents are read |
| `buf~makeArray`, `buf~makeArray()`, `buf~makeArray(,)` | rc 0, `1` item | the omitted argument, the default separator |
| `buf~subWords(,2)`, `buf~subWords(1,)` | rc 0, `2` and `3` items | the omitted position, the omitted count |
| `.MutableBuffer~makeString` | 97.1 `Object "The MutableBuffer class" does not understand message "MAKESTRING".`, rc 159 | the four rows are instance rows; the class arm is unbound and stays so |

**One argument helper was added, and only one**:
`optional_string_or_none_argument`. Every other refusal above is an existing
helper. It mirrors `StringUtil::makearray`'s own `separator != OREF_NULL` test
(`classes/support/StringUtil.cpp:552`), which is the one shape none of the
existing helpers covers -- `optional_string_method_argument` answers the null
string for an omitted argument, and `makeArray`'s omitted argument and its
explicit `''` are **different answers** (`mtarr` field 1 against field 3, and
`ma6`).

---

## 5. `corpus/method-bodies.txt` row moves

**Predicted before the refresh ran**, in
`$S/ctrl/method-bodies-prediction.txt`, and before any method-bodies probe was
run for this task. The oracle side was then measured separately
(`$S/oracle/mb/mb_*.rex`, the eleven probe programs
`method_bodies.rs:401`-`:413` builds).

| rows | from | to | evidence |
|---|---|---|---|
| `new` (class), `delete`, `delStr` | `loud`, `MAKESTRING` | `answers` | `rc 0`, the empty line -- the send already succeeded and only the probe's own `say` refused |
| `lower`, `space`, `translate`, `upper` | `loud`, `MAKESTRING` | `answers` | `rc 0` -- `abc`, `abc`, `ABC`, `ABC` |
| `makeArray`, `makeString`, `subWords` | `loud`, at their own name | `answers` | `rc 0` -- `abc` each; `makeArray` and `subWords` answer a one-element Array, which the probe's `say` renders through the Array's own `makeString` |
| `setText` | `loud`, `SETTEXT` | `answers` | `rc 163` -- 93.903 on both sides, the only one of the eleven that refuses |
| every other row | unchanged | unchanged | -- |

**Measured**: `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec
--test method_bodies` reads `regressions this run: 0. other drift from the
committed table: 11.` (`$S/ctrl/fast/mb.err`), and
`git diff -- corpus/method-bodies.txt` is those eleven rows and no others.
**Prediction confirmed row for row and rc for rc.**

**No row moved to `diverge`**, which is the gate. Two readings of "the
`diverge` total", both **measured**, both unchanged across the refresh:
`/bin/grep -c 'diverge' corpus/method-bodies.txt` is **7** before and after
(five of those seven lines are header prose), and
`awk -F'\t' '$4=="diverge"'` counts **2** rows before and after, both
`DateTime`. The whole-file verdict tally moved `answers` 652 → 663 and `loud`
620 → 609, with `unstable` 2, `diverge` 2 and `unanswered` 71 unchanged.

**No `MutableBuffer` row is `loud` any more**: `/bin/grep -c '^MutableBuffer.*loud'`
is 0.

---

## 6. Controls, each predicted before it ran

Predictions in `$S/ctrl/predictions.md` (C1-C7) and
`$S/ctrl/predictions-mut.md` (M1-M8), each written before the run it names.
`$S/ctrl/witness-observations.md` was written before the mutations, against
the committed witness text.

| # | control | prediction | reading |
|---|---|---|---|
| C1 | the witness against BASE's binary -- a `git archive 4ac61707 rust interpreter` extract at `$B/base/` with its own `CARGO_TARGET_DIR` (`$B/target-base`, binary sha256 `10f42ee90faa0649…`) | rc 120 on both engines, stdout empty, stderr naming `MAKESTRING` of class `MutableBuffer` | **confirmed exactly**: rc 120, 0 bytes of stdout, `rexx-exec: method "MAKESTRING" of class "MutableBuffer" is not implemented (Phase 5)` on both engines (`$S/probes/base/`) |
| C2 | the witness on this build against the oracle, both engines | rc 0, stderr 0 bytes, stdout `cmp`-identical | **confirmed**, and re-confirmed over the final tree (§7) |
| C3 | every probe on this build against the oracle, both engines | all three descriptors identical | **confirmed for 88 of 94 pairs**; the 6 exceptions are three exploration programs, §8 |
| C4 | the probe comparison can see a difference | `cmp` of two oracle stdouts that differ exits 1 | **confirmed**: exit 1, `differ: byte 1, line 1` (`$S/ctrl/c4/out.txt`) |
| C5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`, read by its `N of M matching` line | rc 0, `363 of 363 matching` | **confirmed**: rc 0, `363 of 363 matching`, 18 passed (`$S/ctrl/fast/corpus.err`; Task 3d read `362 of 362`) |
| C6 | the pin's inversion: the witness line removed from `EXPECTED_SUBSET_5C` only | `phase_5c_subset_matches_the_committed_list` red | **confirmed**: rc 101, that one test FAILED and the other 19 ok (`$S/ctrl/c6/out.txt`); `coverage.rs` restored from `$S/ctrl/c6/coverage.rs.good`, `touch`ed, then 20 passed (`$S/ctrl/c6/after.txt`) |
| C7 | the method-bodies refresh | eleven rows, no `diverge` | **confirmed**, §5 |
| C8 | the mutations, §6.2 | eight, each predicted | **seven confirmed, one partly falsified** (M3 gained a catcher the prediction did not name) |

### 6.1 The mutation instrument

Task 3d §6.1's fix is carried: **the per-mutant sha is
`sha256sum target/mutation/deps/corpus-*`, read after the corpus run**, never
`target/mutation/rexx-run` after `--lib` -- that binary is not what the
differential runs (`corpus.rs:256` runs this crate in process) and `--lib`
does not rebuild it.

**Measured, and this is the check that says the instrument sees anything at
all**: nine runs -- the unmutated control plus M1-M8 -- give **nine distinct
shas**.

```
CTL  5e66e0c0d5d62d38   M1 5f829d51fe5f7f40   M2 861a43c2a0478b64
M3   68f4a0cac9b816be   M4 ed93b80f19fa71d7   M5 e3646d985ccaeaac
M6   28f409c06286797d   M7 63fe2dab4b285663   M8 36f57667b532efc0
```

The final unmutated control, run over the restored tree after M8,
**reproduces the first control's sha exactly** (`5e66e0c0d5d62d38`) at
`363 of 363 matching` -- which is also the check that every restore was
byte-exact. `target/release/rexx-run` was `b4202e38a6060856…` before the
mutation runs and `b4202e38a6060856…` after them and after a rebuild; its
mtime never moved into the run window (`14:21:54`, the first mutation at
`14:37`). The commit's binary is `836fd0adffa39924…`, rebuilt after the §7
comment correction.

Every anchor was checked to occur **exactly once** in its file before the run
(`$S/mut/pairs`, counted by exact substring rather than by `grep -c`, which
counts lines and gave 2, 3 and 16 for the three multi-line anchors). Every
build is `--profile mutation` (`target/mutation/`), every command runs under
`memcap 8G`, and after each mutation the edited file is restored from
`$S/mut/good/`, `cmp`-checked and `touch`ed.

### 6.2 Mutations

Checks per mutant: `--lib`, STRICT `--test corpus`, `--test method_bodies`,
and the **without-witness** run in Task 3d's M1b shape -- the witness line
taken out of `corpus/phase-5c.txt` **and** named in `corpus/unfiled.txt`, so
`every_lang_program_is_run_or_named_unfiled` is satisfied and the exit status
and the matching line say the same thing.

| id | site | mutation | predicted catcher | reading |
|---|---|---|---|---|
| M1 | `line_slices` (`builtin/string.rs`) | the `CR` strip removed | STRICT corpus on `lang/string_makearray.rex` **and** the new witness; **adds no coverage** | **confirmed exactly**: `361 of 363`, the two named programs; without the witness `361 of 362`, still red on `string_makearray.rex`. `--lib` 0, method_bodies drift 0 |
| M2 | `split_slices`'s tail (`builtin/string.rs`) | `start < text.len()` → `start <= text.len()` | STRICT corpus, the new witness only | **confirmed**: `362 of 363` on `lang/mutablebuffer_conversion.rex` alone; without the witness rc **0**, `362 of 362`, 18 passed |
| M3 | `native_mutable_buffer_subwords` | `.skip(position - 1)` → `.skip(position)` | STRICT corpus, the new witness only | **red as predicted and one catcher more**: `362 of 363` on the witness alone and `362 of 362` without it, but `--test method_bodies` is **also** red -- `MutableBuffer subWords (instance arm): answers -> diverge`. The prediction named only the corpus; it did not follow that a zero-argument `subWords` reaches the same default |
| M4 | `native_mutable_buffer_settext` | the `clear` moved behind `buffer_capacity` | STRICT corpus, the new witness only | **confirmed**: `362 of 363`; without the witness `362 of 362`; `--lib` 0, method_bodies drift 0 |
| M5 | `operator_message_receiver` (`eval.rs`) | `Body::Instance { .. }` → `Body::Instance { native: None, .. }` -- a buffer stops taking the operator as a message | STRICT corpus on the new witness **and** `lang/mutablebuffer_mutators.rex`; `--lib` red; **adds no coverage** | **confirmed exactly, message and all**: `361 of 363`, both programs `[the operator `==` applied to an instance of a user class] … rust rc 120, oracle rc 0`; without the witness `361 of 362`, still red on the mutators witness; `--lib` 778 passed, 1 failed |
| M6 | `native_mutable_buffer_makearray` | the explicit-separator arm calls `line_slices` | STRICT corpus, the new witness only | **confirmed**: `362 of 363`; without the witness `362 of 362` |
| M7 | `array_of_texts` | `dimensions: None` → `slots.is_empty().then(…)`, `native_string_makearray`'s own shape | STRICT corpus, the new witness only | **confirmed**: `362 of 363`; without the witness `362 of 362` |
| M8 | `native_mutable_buffer_makestring` | the last byte dropped | STRICT corpus on the new witness, method_bodies, and `--lib`; **adds no coverage** | **confirmed**: corpus `362 of 363` and, without the witness, `362 of 362` -- so the new witness is the only *corpus program* that catches it -- while `--test method_bodies` reads `regressions this run: 5` (`lower`, `makeString`, `space`, `translate`, `upper`, each `answers -> diverge`) and `--lib` is 1 failed. The suite catches it without the witness; the differential does not |

`--test method_bodies` reads `regressions this run: 0. other drift from the
committed table: 0.` under M1, M2, M4, M5, M6 and M7.

### 6.3 Which line of the witness each mutation moves

**Predicted from the code before the run**
(`$S/ctrl/predictions-mut.md`), and the corpus report truncates both stdout
strings, so the reading below is the prediction plus what the run's own
classification says.

| id | predicted first moving line | what the run shows |
|---|---|---|
| M1 | `ma3` `2 1 1` → `2 2 1` | consistent: the two diverging programs are exactly the two that carry a `CR`-before-`LF` case |
| M2 | `ma9` `2 [ab][ab]` → `3 [ab][ab]` | consistent: `ma9` is the only field in any corpus program whose text ends with an explicit separator |
| M3 | `sw1` `3 3 [a][b][c]` → `2 2 [b][c]` | **corroborated by a second instrument**: method_bodies puts `subWords` at `diverge-stdout`, which is the same off-by-one at the same default |
| M4 | `st5` `40 40` → `40 43` | consistent: no other program reads `getBufferSize` after a `setText` |
| M5 | the witness dies at `cmp` with rc 120, `Loud::operator_operand` rather than a comparison answering `1` | **confirmed exactly**: the corpus classifier names `the operator `==` applied to an instance of a user class`, `rust rc 120, oracle rc 0` |
| M6 | `ma4` `2 2 1` → `2 1 1` | consistent |
| M7 | `mtarr` `0 0 0` → `0 1 0` | consistent |
| M8 | `say` `say a b  c` → `say a b ` | **corroborated**: the five method-bodies rows that regress are exactly the five whose probe prints a non-empty string through `MAKESTRING`; `new`, `delete` and `delStr` print the empty line and cannot move, and `makeArray`/`subWords` print through the Array's own `makeString` |

**So the witness is not one field wearing eight hats**: eight mutations name
eight different first-moving fields, in five different blocks of the output
(`ma`, `sw`, `st`, `cmp`, `say`).

---

## 7. The witness

`corpus/lang/mutablebuffer_conversion.rex`, 60 lines, 36 lines of output.
**Written and confirmed on the oracle before the crate was touched**
(`$S/oracle/w/`), then widened once (`$S/oracle/w2/`) before the mutation run,
for the reason in §7.1.

**Measured**, `$S/run_oracle.sh $S/oracle/w2`: rc **0**, stderr **0 bytes**.
**Measured**, `$S/run_crate.sh $S/oracle/w2` re-run over the final tree: rc 0
and stderr 0 bytes on both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, and
`cmp` reports stdout byte-identical to the oracle's on both.

It contains `say buf` -- the first witness in this phase that may -- and the
buffer variables are `buf`, `mt`, `one`, `two`, `three`, `crlines`, `lines`,
`sepb`, `each`, `whole`, `num`, `tail`, `wide`, `all`, `txt`, `grow`, `grow2`;
none of them `b`.

The oracle's stdout in full:

```
say a b  c
ms a b  c a b  c 1 The String class
cmp 0 1 0 1
ncmp 1 0
len 6 6
cat xa b  c a b  cy
mt [] [] 0
mtcmp 0 1 0 1
mtarr 0 0 0
mtsw 0 0
ma1 1 1 1 [a b  c]
ma2 2 [one][two]
ma3 2 1 1
ma4 2 2 1
ma5 2 [a ][  c]
ma6 6 [a][c]
ma7 1 [a b  c]
ma8 3 [a][c]
ma9 2 [ab][ab]
ma10 1 [ab]
sw0 The Array class 1
sw1 3 3 [a][b][c]
sw2 [b
c]
sw3 [b]
sw4 2 1 0
sw5 0 2
sw6 [a
b]
st1 xyz 3 10
st2 0 10
st3 12 2
st4 1 same
st5 40 40
st6 400 512
st7 0 512
```

### 7.1 What each printed value observes, and the gap that pass found

The full per-field table is `$S/ctrl/witness-observations.md`, written against
the committed witness text rather than against the exploration programs --
Task 3c's M2 is why it is a separate pass. The load-bearing rows:

* **`cmp` 1 and 3 against 2 and 4** are the regression this task exists to
  avoid: `0 1 0 1`, the identity compare on the receiver side and the string
  request on the other. M5 is the mutation that reaches them.
* **`ma3` and `ma4`** are the pair that pins the `CR` rule to the *omitted*
  argument: the same buffer, one field apart, `1` against `2`.
* **`st5` `40 40`** separates zeroing the length before `ensureCapacity` from
  raising for the sum, which would read `43`.
* **`mtarr` field 2** is `~dimension` `0` for an empty result, which is the
  field `native_string_makearray`'s own shape gets wrong (open question 1).
* **`sw0`** is `The Array class`, the measurement the plan's sketch missed.

**The gap that pass found, closed before the mutations ran**: nothing
exercised `split_slices`'s own tail rule (a text **ending** with the
separator) or its scan limit (a separator **longer** than the text). `ma2`
observes the tail rule only for `line_slices`. `ma9` and `ma10` were added for
those, confirmed on the oracle, and the `sourceline_oracle` companion
regenerated; the widening happened **before** any mutation ran, so every
mutation is measured against the committed text. M2 is red because of `ma9`,
which is the check that the widening was not decorative.

---

## 8. Corrections to prose and findings met on the way

* **`''~makeArray~dimension` diverges, and it is `String`'s, not
  `MutableBuffer`'s.** See open question 1.
* **The three exploration programs that do not agree with the oracle**, none
  of them a witness and none of them about the four rows:
  `$S/oracle/e2/f1_size.rex` uses `Array~last`, which is unbound here
  (`rexx-exec: method "LAST" of class "Array" is not implemented (Phase 5)`,
  rc 120 against the oracle's rc 0); the same probe without that one call
  (`f2_size.rex`) agrees on all three descriptors on both engines.
  `$S/oracle/reach/reach.rex` and `$S/oracle/dim/dim.rex` differ in exactly
  one field each, and that field is the `String~makeArray` divergence above.
* **`MutableBuffer::makeStringRexx` does not exist as its own C++ function.**
  `Setup.cpp:1473` spells it that way, and it resolves to the inherited
  `RexxObject::makeStringRexx` (`classes/ObjectClass.cpp:2846`), which is the
  *same* entry `Setup.cpp:1446` binds under `String`. The doc comment was
  written with the brief's spelling first and corrected to the resolved one
  after `grep` found no such definition in `MutableBufferClass.cpp`.
* **Every C++ citation in the diff was checked by running `sed -n "<line>p"`
  on it**, which is Task 3d §8's instrument. All of them land on the named
  declaration or statement.
* **`grep -c` on a multi-line pattern counts lines, not occurrences.** The
  anchor-uniqueness check first read 1, 2, 1, 3, 1, 1, 1, 16 for eight anchors
  that each occur exactly once; the exact substring count is what the run
  used.

---

## 9. Files created outside the tree

Enumerated from the filesystem, not from memory. Nothing under `$S` or `$B`
was deleted.

* `$B/base/` -- the `git archive 4ac61707 rust interpreter` extract;
  `$B/target-base/` -- its build; `$B/base-build.{out,err,status,pid}`.
* `$S/run_oracle.sh`, `$S/run_crate.sh` -- the two runners; every `runs/`
  subdirectory under an `$S/oracle/*` or `$S/probes/*` directory is a fresh
  empty run directory one of them made with `mktemp -d`.
* `$S/oracle/e/`, `$S/oracle/e2/` -- the exploration probes (`makeArray`
  semantics, the explicit separator, `subWords`, `makeString`, `setText`, and
  the array `size`/`items`/`dimension` probe).
* `$S/oracle/r/` -- the 23 refusal probes and their three descriptors, oracle
  and both engines.
* `$S/oracle/mb/` -- the eleven method-bodies probe programs on the oracle.
* `$S/oracle/cmp/`, `$S/oracle/reach/`, `$S/oracle/dim/` -- the comparison,
  reachability and array-dimension probes.
* `$S/oracle/w/`, `$S/oracle/w2/` -- the witness before and after the
  widening; `$S/probes/w/`, `$S/probes/cmp/`, `$S/probes/reach/` -- the same
  against this build; `$S/probes/base/` -- against BASE's binary.
* `$S/srclines/` -- the `sourceline_oracle` driver, its two stderr files and
  the scratch copy it was run on (the repository's own copy was never given to
  `.Package~new`).
* `$S/good/` -- pristine copies of the six tracked files this task edits.
* `$S/ctrl/` -- `predictions.md`, `predictions-mut.md`,
  `method-bodies-prediction.txt` and `witness-observations.md`, each written
  before the run it names; `c4/`, `c6/` (with `coverage.rs.good`); and the
  fast-check directories `fast/`, `fast2/` and `fast3/` (before the mutations,
  after them, and after the §8 comment correction).
* `$S/mut/` -- `mutate.py`, `pull_witness.py`, `run_one.sh`, `pairs/`,
  `good/` (the pristine copies the harness restores from), `status.txt`, the
  per-mutation directories `M1/`-`M8/`, and the two unmutated controls
  `CTL0/` and `CTL1/`.
* `$S/commit-msg.txt` -- the commit message, `git commit -F`'s argument.
* `$S/gates/` -- the gate runner, its status file and its pidfile.

---

## 10. Open questions

1. **`''~makeArray~dimension` is `1` here and `0` on the oracle, and it is
   `String`'s row rather than this task's.** **Measured** at BASE, before any
   edit, on both engines: `.array~new()~dimension` is `0` and
   `.array~of()~dimension` is `1` on both sides, and `''~makeArray~dimension`
   is `0` on the oracle and `1` here (`$S/oracle/dim/`, `$S/oracle/reach/`).
   The cause is `native_string_makearray`'s
   `dimensions: slots.is_empty().then(|| Box::from([0].as_slice()))`, which is
   `.array~of()`'s shape where `StringUtil::makearray`'s empty result is
   `.array~new()`'s. **The fix is that one expression becoming `None`**, which
   is what `array_of_texts` does for the buffer's own rows and what `mtarr`
   field 2 pins.
   **It was deliberately not taken here**: it changes a `String` row in a
   commit whose gate is about `MutableBuffer` rows, it has no witness of its
   own in `corpus/lang`, and a behaviour change riding along in another
   task's commit is the shape this phase has been bitten by. It is a
   one-expression change plus a line in `string_makearray.rex`, and it is
   Moritz's call.
2. **The oracle reads one byte before its own buffer when a `makeArray`
   separator matches at position 0.** `StringUtil::makearray`'s
   `checkCR && *(tmp - 1) == '\r'` (`:625`) is evaluated even when `tmp ==
   start`. **Measured**, oracle rc 0: `.MutableBuffer~new('0a'x)~makeArray` is
   one element of length 0, so the byte read there was not a `CR` on that run
   -- but the answer depends on what precedes the data. `line_slices` strips
   a `CR` only from a non-empty line, which agrees whenever that byte is not a
   `CR` and cannot be made to agree when it is. **No leading-`LF` case is in
   the witness**, deliberately: a differential case whose oracle answer is not
   a property of the program does not belong in the corpus. The same read
   exists for `String~makeArray`.
3. **`SETTEXT` is in no family list in the plan**, §1.1. The plan's Task 3
   paragraph is what should say which commit owns it.
4. Carried unchanged from Task 3a §6.1, 3b §7, 3c §7 and 3d §10:
   `MutableBuffer~verify` answers `counted` on every path while the builtin's
   past-the-end zero is an untagged text; `refusal-sites.tsv` cites
   constructor definitions by line; `MutableBuffer::translate` numbers its
   range `ARG_FOUR`; and Task 3c's mutant-sha claim may need a correction
   commit.
5. **Every `MutableBuffer` row in `corpus/method-bodies.txt` now reads
   `answers`**, which is the row arithmetic this phase set out to move -- and
   `rust/CLAUDE.md`'s own warning applies to it: a zero-argument send agreeing
   is not a method working. The witnesses are what say the bodies work, and
   `corpus/phase-5c.txt` now names six of them.

---

## Fast checks before the commit

Run from `rust/` in the foreground with a bounded timeout, each status read
unpiped, over the tree **as committed** (`$S/ctrl/fast3/`, re-run after the §8
comment correction; `fast/` is the same set before the mutations and `fast2/`
after them).

| check | reading |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| `cargo build --release -p rexx-exec --bin rexx-run` | rc 0, sha256 `836fd0adffa39924…` |
| `cargo test --release -p rexx-exec --lib` | rc 0, 779 passed, 0 failed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | rc 0, `363 of 363 matching`, 18 passed |
| `cargo test --release -p rexx-exec --test coverage` | rc 0, 20 passed |
| `cargo test --release -p rexx-exec --test refusal_sites` | rc 0, 5 passed -- no re-derivation needed, `corpus/refusal-sites.tsv` is not in `git status` |
| `cargo test --release -p rexx-exec --test builtin_status` | rc 0, 19 passed |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | rc 0, 1 passed |
| `cargo test --release -p rexx-exec --test method_bodies` (no refresh) | rc 0, 16 passed, `regressions this run: 0. other drift from the committed table: 0.` |
| the witness through `target/release/rexx-run`, both engines | rc 0, stderr 0 bytes, stdout `cmp`-identical to the oracle's |

Every run count is non-zero, which is the check `cargo test <name>` needs
because it exits 0 when it matches nothing.

`git status --short` names five modified tracked files and three new ones
(this report among them), and `rust/Cargo.lock` is not among them.

---

## Gates

Run from `rust/` by `$S/gates/run.sh` in the background, each status written
unpiped to `$S/gates/status.txt` as it goes -- the commit sha as its first
line, `finished` as its last -- with the pidfile `$S/gates/pid` beside it and
each gate's descriptors in `$S/gates/g<N>.{out,err}`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **G1** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **G2** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **G3** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **G4** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **G5** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G6** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G7** |
