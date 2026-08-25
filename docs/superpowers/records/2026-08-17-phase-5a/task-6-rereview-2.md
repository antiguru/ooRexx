# Task 6, re-review of fix round 2

Fix base `14e25eca3`, head `9d524ad2c`, two commits (`c351fa473` the fix, `9d524ad2c` the
sitting rows). Read-only on the checkout; every probe run from a fresh per-program
directory with absolute paths, three descriptors read separately, both engines
(`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`) against the oracle.

**Round verdict: one more narrow round.** All six findings are addressed in the code and
the prose. The widening does not over-fire anywhere I could reach: every probe below is
byte-identical on both engines, with no divergence attributable to this diff. But the round's own closing claim, that
comparison cannot raise past a receiver and so needs no wrapping, is **false and
measurable**: six comparison operators on a stem receiver raise inside the forwarded
method and the oracle emits the frame, and both engines omit it. That is the same class
the review's finding 1 named, declared closed on a premise that measurement contradicts.
Plus one false perf explanation and four prose nits.

---

## Verdicts on the six findings

| # | Verdict | Where |
|---|---|---|
| 1 | ADDRESSED (mechanism), but the round's claim of completeness is false -- see New finding A | `eval.rs:814-826`, `eval.rs:956-983`, `eval.rs:1435-1452`, `run.rs:7288-7301` |
| 2 | ADDRESSED (all four), one new instance of the same shape introduced -- see New finding E | `corpus.rs:314`, `phase-5a.txt:171,178,186`, `run.rs` block deleted |
| 3 | ADDRESSED, and the replacement measurement is correct | `eval.rs:1159-1169` |
| 4 | ADDRESSED | `run.rs:7303-7305` |
| 5 | ADDRESSED, both falsifying rows quoted verbatim and verified against the TSV | report, "Finding 5" |
| 6 | ADDRESSED, full transcript for all eleven, line numbers all check out | report, "The negative control, in full" |

Detail:

**Finding 2.** `corpus.rs:314` now reads "every entry below's stderr"; `phase-5a.txt:171`
"across the programs below"; `:178` "every program below's stderr"; `:186` "a stem in any
position"; the `run.rs` "Measured, all three positions" block was deleted with the code it
described. `/bin/grep -n "three programs\|three positions\|all three\|the three "` over the
five touched files finds none of the four back.

**Finding 3.** Measured myself: `say .environment + 1` on the oracle is **rc 0**, stdout
`The NIL object`, stderr empty. The new comment's narrower reason -- this crate registers
no operator method for a native object, whatever the oracle does for a particular one --
is the true one, and the `.environment` measurement sits beside it rather than inside a
universal claim.

**Finding 4.** `accept_header_value` (`run.rs:7171-7233`) routes `Initial`/`To`/`By` to
`header_number` and `For`/`OverFor`/`Count` to `whole_nonneg`, `Over` to the gap check --
so `header_number_body`'s "for a position that reaches it (`Initial`, `To` and `By`)" is
exactly right.

**Finding 5.** Both rows exist verbatim at `bench-baselines/phase-5a-arms.tsv:359` and
`:568`:

```
6  211763aaa  alloc4c  pinned>head  across_builds  ir  small  instructions:u  1.000000  1.000000  1.000001  5
6  211763aaa  strings  pinned>head  across_builds  tw  large  instructions:u  1.000000  0.998014  1.000000  5
```

---

## 1. Blast radius: no over-fire found

The failure mode this widening opens is a frame emitted where the oracle emits none. I
could not produce one. Every program below was run against the oracle and both engines;
"identical" means all three descriptors and the exit status match byte for byte.

**No stem anywhere (must stay frameless).** `say 'abc' + 1` (41.1), `say 1 / 0` (42.3),
`say 'x' & 1` (34.901), `say \'x'` (34.901), `do i = 'abc' to 5` (41.1). All five
identical, no frame on either side.

**Stem on the right, non-stem receiver (must stay frameless).** `say 1 + b.`,
`say 2 ** b.`, `say 1 & b.`, `s. = 'abc'; say 1 + s.`, `s. = 'abc'; say 1 & s.`,
`s. = 'abc'; say 2 ** s.`. All identical, no frame on either side -- the receiver rule
holds after the widening. `do i = 1 to b.` and `do i = b. to 5` both carry the frame on
both sides, as round 1 established.

**Stem receiver, failure past its own conversion (the new territory).**
`s. = 1; say s. + .array`, `s. = 1; say s. + b.`, `s. = 1; say s. + .environment`,
`s. = 1; say s. + .nil`, `s. = 1; say s. + 'abc'`, `s. = 1; say s. & 'x'`,
`s. = 1; say s. & .array`, `s. = 1; say s. ** 'abc'`, `s. = 1; say s. ** 2.5`,
`s. = 1; say s. // 0`, `s. = 1; say s. % 0`, `a. = 1; b. = 'x'; say a. + b.`,
`a. = 1; b. = 'x'; say a. & b.`, `s. = 'abc'; say s. | 1`, `s. = 'abc'; say s. && 1`,
`s. = ''; say s. + 1`, `s. = .true; say s. / 0`, `s. = 1 + 0; say s. / 0` (a
NumberString-shaped default -- the oracle still says scope `"String"`),
`a. = 1; b. = a.; say b. / 0` (nested stem), `x = s.` copied out of the stem then
`say x + 1`, `numeric digits 1; say +s.` with an overflowing default,
`numeric digits 1; say s. + 0`, and the `TO`/`BY` twins of the new DO witness. All
identical, all carrying exactly one frame naming the operator actually applied.

**Stem receiver that never dispatches (must stay frameless).** `s. = .nil; say s. + 1`
and `s. = .array; say s. + 1` -- the oracle answers 97.1 with no frame, and this crate
emits no frame either. (Both diverge for an unrelated pre-existing reason; see
Out-of-Scope.)

**An error in the surrounding clause, after the operator already succeeded.**
`s. = 1; do i = s. to 5 for 'x'` (26.3), `s. = 1; if s. + 1 then nop` (34.1),
`s. = 'abc'; select; when s. then nop; ...` (34.2), `s. = 1; do i = 1 to 3 for b.`
(26.3). All identical, none frames -- the blame is not leaking out of the operator into
the clause around it.

**Frame-state leak hunt.** `blame_native_method` (`dispatch.rs:867`) is first-wins into
`Interp::failure_site`, so a blame set on a *recovered* error would corrupt a later,
unrelated traceback. Every caller of the four wrapped functions propagates the `Err`
(`eval.rs:791,899,1557`; `ir/drive.rs:1390,1492` `break 'cold Err(failure)`;
`run.rs:7195,7198,7199`) -- none swallows one, so there is no recovery path to leak from.
Measured anyway: `signal on syntax` trapping a stem-frame error and then raising a second,
untrapped one carries no stale frame; the same trapped and returning cleanly prints
nothing; `raise propagate` out of the handler reproduces the frame exactly once.
Also identical under `trace i`, `trace r`, inside an internal routine, inside a `DO`, and
through `interpret`.

**Spelling.** `say <0xAC>s.` (the not-sign spelling of prefix `\`) frames as
`Compiled method "\"` on the oracle and here -- the canonical spelling, not the source
byte.

## New finding A -- Important: comparison is the same unmet class, and the round declares it closed on a false premise

The report says, under a heading that advertises the check:

> **Comparison and concatenation are not wrapped, and this was checked rather than
> assumed.** [...] `Interp::compare_values` does carry one `Result`-typed call,
> `rexx_num::compare_numbers`, but its only fallible step [...] is documented [...] as
> unable to overflow given its own precondition [...] So neither function has an `Err` for
> `blame_stem_forwarded_operator` to ever see, and wrapping either would add a call that
> never fires.

and repeats it under "What I could not close": "comparison and concatenation cannot raise
past a receiver at all, so there is nothing there to close."

Measured, both engines, oracle vs. rust:

```
numeric digits 1
s. = '9.9E999999999'
say s. > 1
```

```
oracle rc 214                            rust rc 214 (ir and tree-walker)
       *-* Compiled method ">" with scope "String".      <-- absent here
     3 *-* say s. > 1                                         3 *-* say s. > 1
Error 42 ... Arithmetic overflow/underflow.              (identical)
Error 42.901: Arithmetic overflow; exponent ("1000000000") exceeds 9 digits.
```

The same divergence, differing from the oracle by exactly the frame line and nothing else,
on `>`, `<`, `>=`, `<=`, `=` and `\=`, with the overflowing value as the receiver
(`s. > 1`) and with the receiver a valid stem and the *right* operand overflowing
(`s. = 1; t. = '9.9E999999999'; say s. > t.`). `1 > s.` (non-stem receiver) correctly
carries no frame on either side, and the strict operators `==` and `>>` are textual, raise
nothing, and are correctly frameless.

So `compare_numbers` demonstrably *does* answer `Err` -- this crate raised 42.901 through
`eval.rs:1381` (`rexx_num::compare_numbers(...).map_err(Raised::from)?`) in every program
above. The concatenation half of the claim is true and I confirmed it (`s. = 'abc'; say
s. || 1` and the overflowing default both rc 0); the comparison half is false.

**Not a regression** -- `bench-baselines/pinned/rexx-run-15a1ffa98` omits the same line, so
this branch did not introduce it. It is in scope because the round asserts, in its own
report and as a checked result, that the class is closed when six operators of it are not.
The fix is the same escape hatch already applied four times: wrap `compare_values`
(`eval.rs:1300`) and blame on `Err` with `left_value`, plus witnesses on the same terms as
the five this round added.

## New finding B -- Important: the perf explanation is false, and its own data contradicts it

Two statements:

* "costs one extra function-call layer and one extra (cheap, allocation-free) shape check
  on the failing path, **and nothing on any path that succeeds**."
* "**Nothing this round's diff added runs on any path `arith`'s benchmark exercises**: the
  wrapping only adds work on a failing path, and `arith`'s own axis never fails."

Both are false on the face of the diff. `arith_general` (`eval.rs:961-983`) now executes
`let result = self.arith_general_body(...)` and then `if result.is_err()` unconditionally
-- the test runs on every successful arithmetic operation, not only on failing ones. And
`bench-programs/arith.rex` is `i / 3`, `a * a - 1`, `i / 7`, `c ** 2 // 5`, `total + b + d`
over 500,000 iterations, none of which stays inside `arith_small_int`, so `arith_general`
is precisely the path it exercises.

The sitting's own numbers say the same thing. At round 1's sitting all four
`arith pinned>head across_builds instructions:u` rows were exactly `1.000000` with min
equal to max. At this round's they are `1.001006` (tw/small), `1.001036` (tw/large),
`1.001283` (ir/small), `1.001294` (ir/large), each with min equal to max to six decimals.
And `git log --oneline 211763aaa..c351fa473 -- rust/crates rust/Cargo.toml rust/Cargo.lock`
lists exactly one commit: `c351fa473`. The pin did not move; nothing else touched the
crates between the two sittings.

The magnitude stays under the 1% threshold, so no gate finding and no control build is
owed. What is owed is a true sentence. Whether the +0.10-0.13% is the added branch or the
layout shift around it is undetermined -- attributing it needs the do-nothing control the
project's own rule requires -- but "nothing this round's diff added runs on any path
arith's benchmark exercises" is not one of the two possibilities.

## New finding C -- Minor: `eval.rs:1053` now contradicts the paragraph under it

`arith_left_operand`'s doc still carries, unchanged:

> ... and each is a receiver of that unary operator exactly as this function's own operand
> is, which is why it shares [`Interp::blame_stem_forwarded_operator`] with it below.

`arith_left_operand` no longer calls `blame_stem_forwarded_operator` -- the call was
deleted this round -- and the very next paragraph the round added says so: "**This
function's own failure carries no frame by itself.**" Under either reading of "it shares X
with it", this function is one of the two sharers, and it is not. The sharers are
`arith_general`, `apply_prefix`, `logical_values` and `header_number`.

## New finding D -- Minor: historical framing at `eval.rs:973-975`

> Also `blame_stem_forwarded_operator`'s own no-op case: a non-stem receiver reaches this
> **exactly as before**, since the predicate it runs is unchanged.

Strike "exactly as before" and the sentence stops saying anything -- which is the deciding
test the constraint names. It is also not quite true: a non-stem receiver now reaches the
call on *any* `Err` where before it reached it only on the left operand's own `NotNumeric`
(`say 1 / 0` newly reaches it). What is unchanged is the answer, not the reaching. Say
that: "a non-stem receiver is a no-op here, whatever raised."

## New finding E -- Minor: a new set-size phrase, at `eval.rs:972`

> ... `s. = 1; say s. + .array` is 41.1 with the frame even though it is the *right*
> operand's own conversion that fails -- **all three** past `arith_left_operand`'s own
> return.

"all three" counts the three examples the comment has just written, which is the
"counting what you just wrote" side of finding 2's own deciding test. "each of them past
`arith_left_operand`'s own return" carries the same meaning with no count.

## New finding F -- Minor: the report's perf table breaks its own convention twice

The table is headed "median (min..max) **where they differ**", so a bare cell asserts
min = median = max.

* `alloc4c | ... | 1.000000` (ir) -- `alloc4c/ir/small` is `1.000000` with min `0.999999`.
  They differ.
* `arith | 1.001006 | ...` (tw) -- tw/small is `1.001006` and tw/large is `1.001036`. The
  cell reports one of the two medians and drops the other; the ir cell correctly gives the
  span `1.001283-1.001294`.

This is the same shape as finding 5, in the round that corrected finding 5.

## New finding G -- Minor, unfixable: `c351fa473`'s message miscounts

> Five witnesses [...] join phase-5a.txt, RAW_STDERR_COMPARISON and EXPECTED_SUBSET_5A on
> the same terms as **the eleven already there**

Six were already there; eleven is the total after. The message is in the commit and cannot
be edited -- recorded, not actionable.

## New finding H -- Minor: `operator_frame_stem_divide_by_zero.rex`'s comment misnames the raiser

> `s.` converts fine (its default is "1") and **the divisor's own arithmetic overflow** is
> what raises

The divisor is `0`; it does not overflow. The division raises 42.3, whose catalogue family
is "Arithmetic overflow/underflow". "the division's own zero-divisor check is what raises"
is the true sentence. Same text in
`crates/rexx-parse/tests/sourceline_oracle/operator_frame_stem_divide_by_zero.txt`.

---

## 2. Duplication: one mechanism, four thin adapters -- not an Important finding

The substantive logic is in one place: `blame_stem_forwarded_operator` (`eval.rs:1112`),
which holds the predicate, the scope lookup and the render, and is called from all four
sites. What repeats is a three-line adapter:

```rust
let result = self.X_body(..);
if result.is_err() { self.blame_stem_forwarded_operator(<op>, <receiver>); }
result
```

at `eval.rs:815-825`, `eval.rs:962-980`, `eval.rs:1441-1450` and `run.rs:7288-7300`, each
with a different operand pair (`op.spelling()`/`b"+"`, `value`/`left_value`). It is not
verbatim duplication of a logic block -- the block is the helper, and the helper is shared.
The `_body` split is what makes a single blame point possible at all and is the right
shape.

If it is worth collapsing, the shape is one generic on `Interp`:

```rust
fn blamed<T>(&mut self, op: &[u8], value: ObjRef, result: Result<T, Failure>) -> Result<T, Failure>
```

reducing each site to `let r = self.x_body(..); self.blamed(op, value, r)`. Optional, and
it would also be the natural place to hang New finding A's fifth caller.

## 3. The five witnesses, and the corpus arithmetic

Measured directly, not through the harness, both engines, three descriptors read
separately. All five byte-identical:

| program | rc | frame |
|---|---|---|
| `s. = 1; say s. / 0` | 214 | `Compiled method "/" with scope "String".` |
| `s. = 1; say s. ** 999999999999` | 230 | `Compiled method "**" ...` |
| `s. = 'abc'; say s. & 1` | 222 | `Compiled method "&" ...` |
| `s. = 'abc'; say \s.` | 222 | `Compiled method "\" ...` |
| `numeric digits 1; s. = '9.9E999999999'; do i = s. to 5` | 214 | `Compiled method "+" ...` |

The five corpus files match those programs, each has a `sourceline_oracle/*.txt` with a
`count` header equal to its line count (9, 10, 9, 8, 11), and
`sourceline_matches_the_interpreter_for_every_corpus_program` panics on a missing
expectation file, so a `.rex` added without one is structurally red.

Corpus arithmetic checks out: `phase-4a.txt` 31 + `phase-4b.txt` 12 + `phase-4c.txt` 12 +
`phase-5a.txt` 62 = 117 non-comment lines, no duplicates; the base commit's `phase-5a.txt`
had 57, giving 112. `cargo test -p rexx-exec --test corpus` reports `117 of 117 matching`
in report mode with `corpus_differential ... ok`.

## 4. The control transcript

Rather than only re-reading the pasted transcript, I ran the **stronger** control the brief
offers: `bench-baselines/pinned/rexx-run-15a1ffa98`, a real build of the phase's starting
commit, on all eleven corpus programs against the oracle. Result, for every one of the
eleven: stdout identical, exit status identical, stderr differing by exactly one line --
the `Compiled method "X" with scope "String".` frame, and nothing else. That is the same
verdict the report's mutation transcript reports, obtained without a mutation, and it also
answers "did this branch introduce this line" affirmatively for all eleven.

The transcript in the report is real failing output, not a summary: it pastes both sides'
full stderr per program. Its line numbers (`plus` 8, `power` 8, `prefix_minus` 7,
`do_initial` 8, `do_to` 7, `do_by` 7, `divide_by_zero` 9, `power_exponent_range` 10,
`logical_and` 9, `prefix_not` 8, `do_exponent_range` 10) all match the oracle output I
captured. The mutation named -- `return;` as the first statement of
`blame_stem_forwarded_operator` -- disables the mechanism itself and nothing adjacent: it
is the sole producer of that line, `blame_native_method` being reached from nowhere else
for an operator.

At HEAD all eleven are byte-identical on **both** engines (I ran `ir` and `tree-walker`
separately, not just the harness default).

## 5. The comment-prose pass

House method: contiguous comment blocks collapsed to one line for base and head of
`eval.rs`, `run.rs`, `tests/corpus.rs`, `tests/coverage.rs`; the two collapsed files
diffed; every added line read, no keyword filter.

New prose claims, each checked by running the program it cites:

* `s. = 'abc'; say \s.` is 34.901 with the frame -- true.
* `s. = 1; say s. / 0` is 42.3 with the frame -- true.
* `s. = 1; say s. ** 999999999999` is 26.8 with the frame, the base converting fine --
  true.
* `s. = 1; say s. + .array` is 41.1 with the frame -- true.
* `s. = 1; say s. & 'x'` is 34.901 with the frame -- true.
* `numeric digits 1; s. = '9.9E999999999'; do i = s. to 5` is 42.901 with the frame --
  true.
* `s. = .nil; say s. + 1` is 97.1 with no frame; a class object likewise -- true.
* `.environment + 1` is rc 0 on the oracle -- true.
* `is_stem_receiver` is safe to call without the gap check having run first -- true by the
  code (`heap.get` is `None` for a class handle, and no native body is `Stem`).
* `say 2 + b.` carries no frame regardless of which operand fails -- true.
* `header_number_body` is reached only from `Initial`/`To`/`By` -- true.

Defects the pass found: findings C, D, E above. Nothing else new is false. ASCII holds
across every file the diff touches (the only non-ASCII in `run.rs` is four pre-existing
`…` at lines 1146, 1187, 1237, 1599, outside the diff).

## 6. The process finding

Confirmed, not re-litigated. `c351fa473`'s message says "Finding 5 and 6 are addressed in
the report only". The report **now** contains both: the "Finding 5" section with the
corrected claim and the two falsifying rows quoted verbatim, and "The negative control, in
full, all eleven corpus programs" with per-program oracle/rust stderr. Both were written
after the commit, at the reviewer's request, so the commit message's assertion was true
only after the fact. `task-6-report.md` is untracked, which is why the commit could not
carry it.

## Gate commands I ran myself, unpiped

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0 (re-run against a
  touched `eval.rs` so it was not a cached no-op).
* `cargo test -p rexx-exec --test coverage` -- 17 passed, 0 failed.
* `cargo test -p rexx-exec --test corpus` -- 17 passed, 0 failed, `117 of 117 matching`.
* `cargo test -p rexx-parse --test sourceline_oracle` -- 1 passed.

Full suite not re-run, per the brief.

## Out-of-Scope Observations

Pre-existing, identical at `rexx-run-15a1ffa98`, unrelated to this diff -- recorded, not
blocking:

* `s. = .nil; say s. + 1` -- oracle 97.1 rc 159 "does not understand message +"; this
  crate 41.1 rc 215 quoting `"The NIL object"`. The `is_stem_receiver` doc states the
  oracle's answer correctly but the crate does not produce it. Neither side frames, so the
  frame mechanism is right; the error number is not.
* `s. = .array; say s. + 1` and `say .environment + 1` -- loud Phase 5 refusals at rc 120
  where the oracle answers 97.1 rc 159 and rc 0 respectively.
* `.c~new~m` and `call on syntax` -- loud Phase 5 refusals, noted only because they were
  in my probe set.
* `run.rs:7242` still reads "measured, and measured in all three positions". It is a
  measurement, so it keeps its number under the constraint, and it is outside the diff --
  but it sits four lines from where finding 2 struck the same words.
