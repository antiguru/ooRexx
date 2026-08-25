# Task 6 report: the native-method frame in a traceback, the operator half

**Status: DONE**

**Commits:**
`84f3e580b5e194c2a0c4c3109bcc5e12034e3988` -- the fix, the corpus, the doc correction.
`4f6359b848952f080ff4c9f5e3a9206441a4dc2b` -- the performance sitting's row set (a separate commit because its own `commit` column names the first).

## The mechanism, established by reading the oracle's C++ before writing anything

`ActivationFrame.cpp`'s `NativeActivationFrame::createStackFrame` and
`NativeActivation.cpp`'s `NativeActivation::createStackFrame` are the only two
places `Message_Translations_compiled_method_invocation` is built, and both
require a live `NativeActivation` with a receiver. That only exists when an
operator reaches its native method through a genuine `messageSend` -- which a
plain `String`/`NumberString` receiver's own arithmetic never does (the
oracle's `RexxInteger`/`NumberString`/`RexxString` implement `plus` etc. as
direct C++ virtual calls, no dispatch, no activation, no frame) but a stem
does: an unset or assigned stem has no operator method of its own, so the
oracle's `StemClass::UNKNOWN` forwards the send to the stem's default value,
and *that* forward is a real dispatch landing on the default's own native
method. Measured against the oracle, three-descriptor, fresh directory, each
program run alone:

| program | rc | frame line |
|---|---|---|
| `say b. + 1` | 215 | `       *-* Compiled method "+" with scope "String".` |
| `say b. ** 2` | 215 | `       *-* Compiled method "**" with scope "String".` |
| `say -b.` | 215 | `       *-* Compiled method "-" with scope "String".` |
| `say 'abc' + 1` | 215 | none |
| `say 2 + b.` | 215 | none |
| `say 2 - b.` | 215 | none |
| `say 2 ** b.` | 230 (26.8) | none |
| `s. = "abc"; say s. + 1` | 215 | `       *-* Compiled method "+" with scope "String".` |

`b.~class` is `The Stem class` on the oracle, so the frame's scope `"String"`
names the method that actually raises, not the receiver's own class -- the
brief's own point, and the reason the first program was already the probe for
it. The right-operand and exponent rows show the frame belongs to the
*receiver's* forward failing, not to any failure downstream of it: `2 + b.`
and `2 ** b.` both raise (on the argument/exponent, evaluated separately from
the receiver's own dispatch) with no frame either side, oracle or ours, before
and after this change.

## Implementation

`crates/rexx-exec/src/eval.rs`: `Interp::arith_left_operand` is the one
function both engines already share for a binary arithmetic operator's left
operand and a prefix operator's sole operand (entered from `eval_arithmetic`/
`eval_prefix` on the tree-walker and from `Op::Arith`/`Op::Prefix` on the IR).
Once its existing `operator_operand_gap` check has already ruled out a class
object or one of this crate's own native objects, a new predicate
`Interp::is_stem_receiver` asks whether `value` is itself a stem handle
(regardless of whether its default is set); if so, the existing
`Interp::blame_native_method` renders the frame with the operator's own
spelling as the name and `"String"` (read through a new `Interp::string_class`
accessor) as the scope, before falling through to the unchanged 41.1 raise.
`blame_native_method` moved from private to `pub(crate)` in
`crates/rexx-exec/src/dispatch.rs` so `eval.rs` -- a different module -- can
call it; nothing about its own behaviour changed.

Because the fix sits in the one function both engines call, it needed no
per-engine duplication and no `Op::Generic` fallback.

## Corpus

Three new programs, one per operator shape the brief names (infix `+`, infix
`**` so the frame's own method name is read rather than assumed, prefix `-`
whose dispatch path is different from infix `-` and happens to collide with
its frame text):

- `rust/corpus/lang/operator_frame_stem_plus.rex` -- `say b. + 1`
- `rust/corpus/lang/operator_frame_stem_power.rex` -- `say b. ** 2`
- `rust/corpus/lang/operator_frame_stem_prefix_minus.rex` -- `say -b.`

Each joins, in the same commit: `rust/corpus/phase-5a.txt` (the subset),
`RAW_STDERR_COMPARISON` in `crates/rexx-exec/tests/corpus.rs` (raw comparison,
required because `normalize_stderr`'s `*-*` marker is the same three bytes
the frame line opens with -- the default comparison would collapse the
frame's own leading run of spaces, exactly the bytes under test), and
`EXPECTED_SUBSET_5A` in `crates/rexx-exec/tests/coverage.rs` (structural row
the ownership-moves-in-one-commit rule requires; `phase_5a_subset_matches_the_committed_list`
reddened until this was added). `crates/rexx-parse/tests/sourceline_oracle/`
gained one `.txt` per program, generated with the documented `srclines.rex`
driver against the live oracle (counts 8, 8 and 7 lines respectively,
matching the `.rex` files' own line counts).

**The doc-block correction the brief calls for.** `corpus.rs`'s
`raw_stderr_comparison_only_names_programs_the_subset_actually_runs` carried
a paragraph calling `RAW_STDERR_COMPARISON` empty and the test therefore
vacuous. Both were already false before this commit -- the superseded plan's
Task 7 had left three entries there (`method_trace_invocation.rex`,
`method_trace_nested.rex`, `method_attribute_set_body.rex`) -- and the
paragraph named a set's size and framed itself historically, both of which
the global constraints strike. Replaced with a sentence stating the current,
timeless property (the test is load-bearing because the const holds entries)
rather than a corrected count.

## The five gate commands, from `rust/`, unpiped, run after the commit above

1. `cargo fmt --all --check` -- exit 0.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0. (Hit and
   fixed a `clippy::doc_lazy_continuation` false positive along the way: a
   wrapped sentence happened to put `+ 1\`` at the start of a doc-comment
   line, which CommonMark reads as an unordered-list bullet; reworded so no
   line starts with `+`, `-`, `*` or a digit-period, not suppressed.)
3. `cargo test --release --workspace` -- exit 0. 98 test binaries, every one
   `test result: ok`, 0 `test result: FAILED` anywhere in the output.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit 0. Same 98/98
   ok, 0 FAILED; `corpus_differential` reports `109 of 109 matching` (106 +
   the three new programs).
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` --
   exit 0. 98/98 ok, 0 FAILED.

All five commands' exit statuses were read directly off `$?` of the command
itself, never through a pipe.

## The three programs, oracle vs. both engines, byte for byte

For each, ran the oracle wrapper and `rexx-run` under both `REXX_ENGINE=ir`
and `REXX_ENGINE=tree-walker`, from a fresh empty directory, absolute paths,
stdout/stderr/exit status read as three separate descriptors, then `diff`ed
each pair:

```
=== operator_frame_stem_plus.rex ===          oracle_rc=215  tw_rc=215  ir_rc=215
diff oracle_out  tw_out   -> no output (stdout tw OK)
diff oracle_err  tw_err   -> no output (stderr tw OK)
diff oracle_out  ir_out   -> no output (stdout ir OK)
diff oracle_err  ir_err   -> no output (stderr ir OK)

=== operator_frame_stem_power.rex ===         oracle_rc=215  tw_rc=215  ir_rc=215
(all four diffs empty)

=== operator_frame_stem_prefix_minus.rex ===  oracle_rc=215  tw_rc=215  ir_rc=215
(all four diffs empty)
```

Oracle stderr for the first: `       *-* Compiled method "+" with scope
"String".` then the clause echo and the 41/41.1 lines; the other two differ
only in the operator's own spelling in that first line. Both engines produce
these bytes exactly.

## Negative control, run

Edited `arith_left_operand` in place to `if false && self.is_stem_receiver(value)`,
rebuilt `target/release/rexx-run`, and ran
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus corpus_differential -- --nocapture`:

```
106 of 109 matching
mismatches (3):
  [UNCLASSIFIED] lang/operator_frame_stem_plus.rex: stderr differ
      rust:   stderr="     8 *-* say b. + 1\nError 41 running .../operator_frame_stem_plus.rex line 8: ..."
      oracle: stderr="       *-* Compiled method \"+\" with scope \"String\".\n     8 *-* say b. + 1\n..."
  [UNCLASSIFIED] lang/operator_frame_stem_power.rex: stderr differ  (same shape, "**")
  [UNCLASSIFIED] lang/operator_frame_stem_prefix_minus.rex: stderr differ  (same shape, "-")
by owner:
  UNCLASSIFIED: 3

thread 'corpus_differential' panicked: STRICT (REXX_CORPUS_GATE) mode: 3 of 109
corpus programs disagree with the oracle
test corpus_differential ... FAILED
```

All three reddened byte-exactly under the gate, missing exactly the frame
line, nothing else. Restored the real check (`if self.is_stem_receiver(value)`),
rebuilt, reran the identical command: `109 of 109 matching`, `test
corpus_differential ... ok`.

Table C carries no row for this mechanism (no documented section names it),
so the control had to fire on the corpus rather than on `gate_table_c.rs` --
recorded in Task 5 as well, per the brief.

## Performance sitting

`src/` changed in `rexx-exec` (`dispatch.rs`, `eval.rs`), so the guard
applies.

**Staleness check.** `git merge-base --is-ancestor 15a1ffa98 HEAD` confirms
the pin's commit is an ancestor. The literal
`git diff 15a1ffa98 HEAD -- rust/crates rust/Cargo.toml Cargo.toml` is **not**
empty (8828 lines) -- but every byte of it is test/corpus content from this
same plan's own Tasks 1/4/5 (`crates/rexx-exec/tests/*`, `corpus/*`) plus an
unrelated `rexx-extract`/`rexx-parse` doc-extraction addition, none of which
is one of the four crates the guard names. Restricting the same diff to
exactly what the guard measures --
`rust/crates/rexx-exec/src rust/crates/rexx-core/src rust/crates/rexx-classes/src rust/crates/rexx-lib/src rust/Cargo.toml Cargo.toml`
-- is **empty**: zero commits have touched any of those four crates' `src/`
between the pin and this task's base. Since none of those paths changed, the
pinned binary is unaffected by anything upstream of this task's own edit, and
the pin is valid for this sitting. (What this check could not see on its own:
a change to `rust/Cargo.toml` or `Cargo.toml` that altered dependency
versions or build flags without touching a `src/` file would also matter and
would have shown up in the same restricted diff; it did not.)

**Command:**
```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task 6 --commit 84f3e580b --baseline bench-baselines/phase-5a-arms.tsv
```
Exit 0, appended 312 rows, committed as `4f6359b84`.

**`instructions:u`, `pinned>head across_builds`, every axis and engine:**

| axis | tw | ir |
|---|---|---|
| alloc4c | 1.000000 | 1.000000 |
| arith | 1.000453 | 1.000542 |
| compound | 1.000000 | 1.000000 |
| emptyloop | 1.000000 | 1.000000 |
| strings | 1.000000 (large: min 0.998014) | 1.000000 |
| varlookup | 1.000000 | 1.000000 |

Every ratio is under the 1% threshold -- not a finding, per the constraint.
Expected: the added check runs only inside `arith_left_operand`'s
already-failing (`NotNumeric`) branch, and none of these six axes' hot loops
raise, so the change adds no work to any path they exercise.

## What this task does not close

The brief names it explicitly: a frame emitted with the wrong *scope* --
naming the frame's own receiver's class where the oracle names the scope of
the method actually running -- needs an inherited method on an instance,
which does not exist until 5b. Nothing in this task's own scope was left
undone; the three required programs match byte for byte on both engines, the
control fires and is restored, and all five gates are green.

# Fix round 1

**Commits:**
`211763aaae70a5ca2f365380fd585fbdc2a1eb22` -- findings 1-4.
`14e25eca3aad6141f46388356fa00e7590f0cde0` -- this round's performance sitting row set.

## Finding 1: `is_stem_receiver` over-fired

The review measured `s. = .nil` then `say s. + 1`: the oracle raises 97.1 ("does
not understand message +") with no frame, because `.nil` answers no operator at
all and the stem's forward never reaches a method to raise from. My original
predicate fired on *any* stem regardless of its default, so it added the frame
here too -- a genuine new divergence, confirmed against the pinned
`rexx-run-15a1ffa98` (which does not emit it).

**Fix.** `Interp::is_stem_receiver` (`eval.rs`) now recurses through a new
`Interp::stem_default_is_string_or_number`, narrowing to exactly the shapes a
native operator answers: an unset stem (its own name is a `String`) or a stem
whose default -- chasing nested stems the way `to_number` and
`operator_operand_gap` both do -- is itself `String`/`Number`-valued. `.nil`, a
class object and one of this crate's own native objects all answer `false`.

**This crate's pre-existing divergence on that program is untouched, as
ruled.** `s. = .nil; say s. + 1` is still `41.1` here against the oracle's
`97.1` -- not this task's to close -- but carries no frame line either side
now, matching the oracle on that byte.

**In-crate test, shown failing before and passing after**, both engines
(`eval::object_operand_tests::a_nil_defaulted_stem_carries_no_operator_frame`):

Before (temporarily reverted `is_stem_receiver` to the unnarrowed
`matches!(..., Some(Body::Stem { .. }))`):
```
thread 'eval::object_operand_tests::a_nil_defaulted_stem_carries_no_operator_frame' panicked:
a `.nil`-defaulted stem's own arithmetic failure must carry no operator-forwarded
frame -- no method ever ran to raise from -- but got "       *-* Compiled method
\"+\" with scope \"String\".\n     2 *-* say s. + 1\nError 41 running /t.rex line
2:  Bad arithmetic conversion.\nError 41.1:  Nonnumeric value (\"The NIL object\")
used in arithmetic operation.\n"
test eval::object_operand_tests::a_nil_defaulted_stem_carries_no_operator_frame ... FAILED
```
After (restored):
```
test eval::object_operand_tests::a_nil_defaulted_stem_carries_no_operator_frame ... ok
```

## Finding 3: the DO-header call site, fixed rather than recorded

Measured (mine, matching the review's): `do i = b. to 5`, `do i = 1 to b.` and
`do i = 1 by b. to 3` each differed from the oracle in exactly the frame line,
on both engines, before this round. `Interp::header_number` (`run.rs`) rounds
every header position through a real unary `+` on the oracle -- this function's
own pre-existing doc already said so -- so a stem in any position is a receiver
of that unary operator exactly as `arith_left_operand`'s own operand is. This
is the same call, confirmed by inspection and by measurement, not a different
one: `header_number` already ran its own `operator_operand_gap` check in the
identical shape `arith_left_operand` does, immediately before calling
`arith_operand`.

**Fix.** Extracted the frame-rendering logic that lived inline in
`arith_left_operand` into `Interp::blame_stem_forwarded_operator` (`eval.rs`,
`pub(crate)`), called from both `arith_left_operand` and `header_number`.
Three new corpus programs, one per header position, joined `phase-5a.txt`,
`RAW_STDERR_COMPARISON` and `EXPECTED_SUBSET_5A` on the same terms as the
first three: `lang/operator_frame_stem_do_initial.rex`,
`lang/operator_frame_stem_do_to.rex`, `lang/operator_frame_stem_do_by.rex`.

**All three, both engines, oracle vs. rust, full stderr, raw:**
```
=== operator_frame_stem_do_initial.rex === oracle_rc=215 rust_rc=215 (tw and ir)
oracle:        *-* Compiled method "+" with scope "String".
     8 *-* do i = b. to 5
Error 41 running .../operator_frame_stem_do_initial.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust (tw and ir): byte-identical to the above -- diff empty on both.

=== operator_frame_stem_do_to.rex === oracle_rc=215 rust_rc=215 (tw and ir)
oracle:        *-* Compiled method "+" with scope "String".
     7 *-* do i = 1 to b.
Error 41 running .../operator_frame_stem_do_to.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust (tw and ir): byte-identical -- diff empty on both.

=== operator_frame_stem_do_by.rex === oracle_rc=215 rust_rc=215 (tw and ir)
oracle:        *-* Compiled method "+" with scope "String".
     7 *-* do i = 1 by b. to 3
Error 41 running .../operator_frame_stem_do_by.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust (tw and ir): byte-identical -- diff empty on both.
```
Corpus gate: **112 of 112** (106 + the original three + these three).

## Finding 2: the false reason for the `RAW_STDERR_COMPARISON` entries

I had written that normalisation would erase the frame's leading whitespace.
Measured (mine, confirming the review's): `normalize_line`
(`tests/support/mod.rs`) copies bytes `0..marker_end` -- the leading indent
*and* the `*-*` marker -- verbatim always, and only collapses the run of
spaces *after* the marker. `od -c` on the oracle's own stderr for all six
programs shows exactly one space in that post-marker gap, both before and
after the marker, on every one -- so normalisation is currently a no-op on
all six programs' stderr, not a hazard raw mode is closing.

**Corrected in both `corpus.rs` and `phase-5a.txt`.** The reason raw mode
belongs here is now stated as: it asserts the frame's own bytes -- leading
whitespace and absent line number included -- rather than resting on what
DEVIATION 0's normaliser happens to do to them today; the no-op is stated as
a measurement beside it, not as the justification.

## Finding 4: historical framing and set-size phrases

Struck: `dispatch.rs`'s "`pub(crate)` since Phase 5a Task 6" (historical
framing -- restated as what calls it, not when); `eval.rs`'s "one of the
**two** object shapes" (set-size, gone with the rewritten predicate anyway);
`eval.rs`'s "The one predicate" and "and unchanged here"; `phase-5a.txt`'s
"The three vary" (now "Varies the operator across the three programs").

## Finding 5: the staleness test

The plan's staleness test is fixed at `5e4f84964` (Moritz) to
`git log --oneline <pin>..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`,
every listed commit required to be one the ledger records, replacing the
`git diff ... empty` form my original report reasoned around. Re-run against
this round's base and again against this round's own commit:

```
git merge-base --is-ancestor 15a1ffa98 HEAD   -> ancestor, confirmed
git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml
```
lists 21 commits before this round's own landed (22 after `211763aaa`), every
one this plan's own Task 1/4/5 work (and their fix rounds) or this task's --
spot-checked five of them against `progress.md` (13 hits) and recognised the
rest by message from this session's own `git log`. None is a foreign or
rebased commit. Pin valid.

Also noted: my original report's restricted pathspec omitted `rexx-parse/src`,
which the `rexx-run` binary links. The new test's own pathspec is wider than
mine was (`rust/crates` unrestricted, plus `rust/Cargo.lock`) and this round's
run used it as specified, so the gap does not recur.

## Also correct: the "adds no work" claim

My original report's "so the change adds no work to any executed path" is
contradicted by its own figures two lines above it: `arith` read
`1.000453`/`1.000542` (tw/ir), min equal to max across 5 rounds -- a small,
deterministic, non-zero effect, not zero. Both readings were and are under the
1% threshold, procedurally fine either way; the claim should have said "a
layout effect below the reporting threshold," not "no work."

**This round's own sitting is consistent with that reading being layout, not
added work**: rerun at commit `211763aaa` (rounds=5, same six axes, both
engines), every `pinned>head across_builds instructions:u` ratio -- `arith`
included -- is now exactly `1.000000`, min equal to max. Nothing in this
round's diff touches `arith`'s own hot path either, so the earlier non-zero
reading and this round's zero one are both explained by the code's shape
shifting around a cold path, not by any work added to or removed from a hot
one.

## Verification this round, summarized

* Five gates, unpiped, re-run after `211763aaa`: `cargo fmt --all --check`
  exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0;
  `cargo test --release --workspace` exit 0, 98/98 `ok`, 0 `FAILED`;
  `REXX_CORPUS_GATE=1 cargo test --release --workspace` exit 0, 98/98 `ok`,
  `112 of 112 matching`; `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace
  --no-fail-fast` exit 0, 98/98 `ok`, 0 `FAILED`.
* Finding 1's test shown failing then passing, both engines (above).
* Negative control re-run in full for all six corpus programs (disabled
  `blame_stem_forwarded_operator` at its one call-in point, rebuilt, diffed
  oracle vs. rust directly -- not through the harness's bounded excerpt):
  every one reddens by exactly the frame line, nothing else; restored,
  rebuilt, gate green at 112 of 112.
* Finding 3's three DO programs measured on both engines against the oracle,
  before (differ by exactly the frame line) and after (byte-identical),
  shown above in full.

## What I could not close

Nothing new. The 5b-owned scope-naming case from the original report is
unchanged and still out of scope here.

# Fix round 2

**Commits:**
`c351fa473a294b771774f349316fb50efb530313` -- finding 1's fix, the five new witnesses, and findings 2-4.
`9d524ad2c459c58dbdfe4f684c2b618be20ef9de` -- this round's performance sitting row set.

## Finding 1: the branch taken, and what it cost

**Took the flag/blame-once shape**, not a new mechanism. The frame belongs to the
*receiver* of a forwarded operator, for however long its whole evaluation runs, not to
the one step (the receiver's own `41.1` conversion) that round 1's fix blamed. Four call
sites now blame once, over their own whole computation, on any `Err`:

* `Interp::arith_general` (`eval.rs`) -- the seven arithmetic operators.
* `Interp::apply_prefix` (`eval.rs`) -- prefix `+`, `-` and `\`.
* `Interp::logical_values` (`eval.rs`) -- `&`, `|`, `&&`.
* `Interp::header_number` (`run.rs`) -- a controlled `DO`'s initial/`TO`/`BY` value.

Each was split into itself (now a thin wrapper) and a `_body` twin holding the original
computation; the wrapper calls `Interp::blame_stem_forwarded_operator` once, after the
body returns, when the body's result is `Err`. `blame_stem_forwarded_operator` still
begins by asking `Interp::is_stem_receiver` of the *receiver* value -- unchanged from
round 1 -- so a non-stem receiver (a plain string, a class object, one of this crate's
own native objects) costs one extra function-call layer and one extra (cheap, allocation-
free) shape check on the failing path. A path that succeeds pays the wrapper's own
`is_err()` test, which runs on every call whether the body succeeded or failed; what
success skips is the shape check and the frame render, not the branch. (Sentence
corrected in fix round 4; the finding is fix round 3's finding 2.) The inline
call inside `Interp::arith_left_operand`'s own `NotNumeric` branch, which round 1 added,
is removed -- it is now the wrapping caller's job, not this narrower function's.

**What this reaches.** Every operator this crate's own arithmetic, prefix, logical and
DO-header code can raise from, at any step past the receiver reaching a real forwarded
method: the receiver's own conversion (round 1's case), the *other* operand's or
exponent's conversion, and the arithmetic/range/logical check past both. Measured on all
eleven corpus programs, both engines (below).

**What this does not reach**, unchanged from round 1's own report: the 5b-owned case
where the frame's own receiver's class differs from the scope of the method actually
running (an inherited method on an instance, which this phase builds no instances for).

**Comparison and concatenation are not wrapped, and this was checked rather than
assumed.** `Interp::concat_values`'s own body has no failing step at all -- it renders
both operands (infallible) and joins the bytes. `Interp::compare_values` does carry one
`Result`-typed call, `rexx_num::compare_numbers`, but its only fallible step,
`Number::sub` inside `numeric_order`, is documented at that call site (`rexx-num/src/
compare.rs`, pre-existing, not touched by this task) as unable to overflow given its own
precondition -- both operands already fit the working precision by the time it runs, and
subtracting can only shrink or preserve an in-range magnitude. So neither function has an
`Err` for `blame_stem_forwarded_operator` to ever see, and wrapping either would add a
call that never fires.

## The five witnesses, both engines, oracle vs. rust, full stderr

Run from a fresh directory, `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`, diffed against
the oracle directly (not through the corpus harness's bounded excerpt). All five diffs
are empty on both engines.

```
=== s. = 1 \n say s. / 0 ===                                    oracle_rc=214 rust_rc=214 (tw, ir)
       *-* Compiled method "/" with scope "String".
     2 *-* say s. / 0
Error 42 running .../w1.rex line 2:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.

=== s. = 1 \n say s. ** 999999999999 ===                        oracle_rc=230 rust_rc=230 (tw, ir)
       *-* Compiled method "**" with scope "String".
     2 *-* say s. ** 999999999999
Error 26 running .../w2b.rex line 2:  Invalid whole number.
Error 26.8:  Operand to the right of the power operator (**) must be a whole number; found "999999999999".

=== s. = 'abc' \n say s. & 1 ===                                oracle_rc=222 rust_rc=222 (tw, ir)
       *-* Compiled method "&" with scope "String".
     2 *-* say s. & 1
Error 34 running .../w3.rex line 2:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "abc".

=== s. = 'abc' \n say \s. ===                                   oracle_rc=222 rust_rc=222 (tw, ir)
       *-* Compiled method "\" with scope "String".
     2 *-* say \s.
Error 34 running .../w4.rex line 2:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "abc".

=== numeric digits 1 \n s. = '9.9E999999999' \n do i = s. to 5 \n end ===   oracle_rc=214 rust_rc=214 (tw, ir)
       *-* Compiled method "+" with scope "String".
     3 *-* do i = s. to 5
Error 42 running .../w5.rex line 3:  Arithmetic overflow/underflow.
Error 42.901:  Arithmetic overflow; exponent ("1000000000") exceeds 9 digits.
```

Bonus verification (not corpus programs, not required by the brief, run only to stress
the "blame on any `Err`, not only the receiver's own" design before trusting it): `s. = 1;
say s. + .array` (right operand fails, receiver is a valid stem) and `s. = 1; say s. &
'x'` (same, for `&`) both carry the frame on the oracle and now on this crate too, both
engines -- confirming the mechanism is not narrower than the five required witnesses,
without adding corpus entries beyond the five the brief asked for.

Corpus: `lang/operator_frame_stem_divide_by_zero.rex`,
`lang/operator_frame_stem_power_exponent_range.rex`, `lang/operator_frame_stem_logical_and.rex`,
`lang/operator_frame_stem_prefix_not.rex`, `lang/operator_frame_stem_do_exponent_range.rex` --
joined `phase-5a.txt`, `RAW_STDERR_COMPARISON` (`corpus.rs`) and `EXPECTED_SUBSET_5A`
(`coverage.rs`) on the same terms as the six already there. Gate: **117 of 117** (112 + 5).

## The five gate commands, unpiped, run after `c351fa473`

1. `cargo fmt --all --check` -- exit 0.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.
3. `cargo test --release --workspace` -- exit 0. 98 test binaries, all `ok`, 0 `FAILED`.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit 0. 98/98 `ok`, 0 `FAILED`;
   `corpus_differential` reports `117 of 117 matching`.
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` -- exit 0. 98/98
   `ok`, 0 `FAILED`.

## The negative control, in full, all eleven corpus programs

Disabled `Interp::blame_stem_forwarded_operator` at its one definition (`return;` as the
function's first statement), rebuilt `target/release/rexx-run`, ran
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus corpus_differential --
--nocapture`: `106 of 117 matching`, all eleven `operator_frame_stem_*` programs
`[UNCLASSIFIED]` mismatches, `test corpus_differential ... FAILED`. Diffed each program's
own stderr (oracle vs. rust) directly, full text, not the harness's bounded excerpt:

```
=== operator_frame_stem_plus.rex ===
oracle:        *-* Compiled method "+" with scope "String".
     8 *-* say b. + 1
Error 41 running .../operator_frame_stem_plus.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust:      8 *-* say b. + 1
Error 41 running .../operator_frame_stem_plus.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_power.rex ===
oracle:        *-* Compiled method "**" with scope "String".
     8 *-* say b. ** 2
Error 41 running .../operator_frame_stem_power.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust:      8 *-* say b. ** 2
Error 41 running .../operator_frame_stem_power.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_prefix_minus.rex ===
oracle:        *-* Compiled method "-" with scope "String".
     7 *-* say -b.
Error 41 running .../operator_frame_stem_prefix_minus.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust:      7 *-* say -b.
Error 41 running .../operator_frame_stem_prefix_minus.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_do_initial.rex ===
oracle:        *-* Compiled method "+" with scope "String".
     8 *-* do i = b. to 5
Error 41 running .../operator_frame_stem_do_initial.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust:      8 *-* do i = b. to 5
Error 41 running .../operator_frame_stem_do_initial.rex line 8:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_do_to.rex ===
oracle:        *-* Compiled method "+" with scope "String".
     7 *-* do i = 1 to b.
Error 41 running .../operator_frame_stem_do_to.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust:      7 *-* do i = 1 to b.
Error 41 running .../operator_frame_stem_do_to.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_do_by.rex ===
oracle:        *-* Compiled method "+" with scope "String".
     7 *-* do i = 1 by b. to 3
Error 41 running .../operator_frame_stem_do_by.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
rust:      7 *-* do i = 1 by b. to 3
Error 41 running .../operator_frame_stem_do_by.rex line 7:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_divide_by_zero.rex ===
oracle:        *-* Compiled method "/" with scope "String".
     9 *-* say s. / 0
Error 42 running .../operator_frame_stem_divide_by_zero.rex line 9:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
rust:      9 *-* say s. / 0
Error 42 running .../operator_frame_stem_divide_by_zero.rex line 9:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_power_exponent_range.rex ===
oracle:        *-* Compiled method "**" with scope "String".
    10 *-* say s. ** 999999999999
Error 26 running .../operator_frame_stem_power_exponent_range.rex line 10:  Invalid whole number.
Error 26.8:  Operand to the right of the power operator (**) must be a whole number; found "999999999999".
rust:     10 *-* say s. ** 999999999999
Error 26 running .../operator_frame_stem_power_exponent_range.rex line 10:  Invalid whole number.
Error 26.8:  Operand to the right of the power operator (**) must be a whole number; found "999999999999".
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_logical_and.rex ===
oracle:        *-* Compiled method "&" with scope "String".
     9 *-* say s. & 1
Error 34 running .../operator_frame_stem_logical_and.rex line 9:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "abc".
rust:      9 *-* say s. & 1
Error 34 running .../operator_frame_stem_logical_and.rex line 9:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "abc".
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_prefix_not.rex ===
oracle:        *-* Compiled method "\" with scope "String".
     8 *-* say \s.
Error 34 running .../operator_frame_stem_prefix_not.rex line 8:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "abc".
rust:      8 *-* say \s.
Error 34 running .../operator_frame_stem_prefix_not.rex line 8:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "abc".
diff: missing exactly the frame line, nothing else.

=== operator_frame_stem_do_exponent_range.rex ===
oracle:        *-* Compiled method "+" with scope "String".
    10 *-* do i = s. to 5
Error 42 running .../operator_frame_stem_do_exponent_range.rex line 10:  Arithmetic overflow/underflow.
Error 42.901:  Arithmetic overflow; exponent ("1000000000") exceeds 9 digits.
rust:     10 *-* do i = s. to 5
Error 42 running .../operator_frame_stem_do_exponent_range.rex line 10:  Arithmetic overflow/underflow.
Error 42.901:  Arithmetic overflow; exponent ("1000000000") exceeds 9 digits.
diff: missing exactly the frame line, nothing else.
```

Restored `blame_stem_forwarded_operator` to its real body, rebuilt, reran the identical
command: `117 of 117 matching`, `test corpus_differential ... ok`.

## Finding 2: set-size phrases

Struck "these three programs'" (`corpus.rs`, then at line 314) -> "every entry below's";
"all three programs' stderr" (`phase-5a.txt:178`) -> "every program below's stderr";
"across the three programs" (`phase-5a.txt:171`, survived round 1) -> "across the
programs below"; "any of the three positions" (`phase-5a.txt:186`, not named by the
review but the same defect) -> "any position"; "Measured, all three positions" (`run.rs`,
was line 7292) -- removed along with the rest of that comment block in finding 1's own
rewrite, since the code there changed anyway.

## Finding 3: the false premise at (was) `eval.rs:1097`

The doc for `stem_default_is_string_or_number` claimed "one of this crate's own native
objects answer none of the arithmetic operators" as a blanket fact. Measured: `.environment
+ 1` is rc 0 on the oracle, `Directory` forwarding through its own `UNKNOWN` -- so that is
false for at least one native-object shape. Corrected to state the true, narrower reason
this crate's own predicate answers `false` for one: this crate registers no operator
method for a native object, regardless of what the oracle itself does for a particular
one of them, with the `.environment` measurement stated beside it rather than folded into
the universal claim.

## Finding 4: `run.rs`'s "each header position is rounded..." (was line 7288)

Reworded along with finding 1's own restructuring of that function -- the comment now
names `Interp::header_number`'s own doc (which already scopes itself to `Initial`/`To`/`By`)
rather than repeating "each header position" in a spot that could be read as covering
`FOR`/a bare repeat count/`OVER`, none of which reach this function.

## Finding 5: the sitting's false "min equal to max" claim

The original report's fix-round-1 addendum said "every `pinned>head across_builds
instructions:u` ratio -- `arith` included -- is now exactly `1.000000`, min equal to
max." That is false for two rows in that same sitting's own data (commit `211763aaa`):

```
6  211763aaa  alloc4c  pinned>head  across_builds  ir  small  instructions:u  1.000000  1.000000  1.000001  5
6  211763aaa  strings  pinned>head  across_builds  tw  large  instructions:u  1.000000  0.998014  1.000000  5
```

`alloc4c/ir/small`'s max is `1.000001`, not `1.000000`; `strings/tw/large`'s min is
`0.998014`. The median is `1.000000` for both and every ratio stays under the 1%
threshold either way, so the conclusion (no hot-path cost) stands -- what needed
correcting was the "min equal to max" claim, not the conclusion drawn from it.

## Performance sitting

`src/` changed again (`eval.rs`, `run.rs`), so the guard applies. Staleness test (the
commits-based one Task 6's own review fixed, `5e4f84964`): `15a1ffa98` is still an
ancestor of HEAD, and `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml
rust/Cargo.lock Cargo.toml` lists this round's own `c351fa473` on top of the commits
already recognised in round 1's report -- the perf-row commit itself touches none of
those paths, so it does not appear in the list. None of the listed commits is foreign.
Pin valid.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task 6 --commit c351fa473 --baseline bench-baselines/phase-5a-arms.tsv
```
Exit 0, appended 312 rows, committed as `9d524ad2c`.

**`instructions:u`, `pinned>head across_builds`, median (min..max) where they differ:**

| axis | tw | ir |
|---|---|---|
| alloc4c | 1.000000 (small max 1.007947; large min 0.992316) | 1.000000 |
| arith | 1.001006 | 1.001283-1.001294 |
| compound | 1.000000 | 1.000000 |
| emptyloop | 1.000000 | 1.000000 |
| strings | 1.000000 | 1.000000 |
| varlookup | 1.000000 | 1.000000 |

Every ratio stays under the 1% threshold -- not a finding, per the constraint, so no
interleaved control build was run. `arith` is the one axis that has moved on every
sitting this task has run: `1.00045`-`1.00055` at the original commit, exactly
`1.000000` after round 1's fix, `1.001006`-`1.001294` here -- bouncing around zero rather
than growing, which is consistent with a layout sensitivity in `arith`'s own benchmark
rather than an accumulating hot-path cost. What this round's diff added *does* run on the
path `arith`'s benchmark exercises: `arith_general`'s wrapper evaluates `is_err()` on every
call, and `bench-programs/arith.rex` is all `arith_general` work. What a successful call
skips is `is_stem_receiver`'s shape check and the frame render, not the branch itself.
(Sentence corrected in fix round 4; the finding is fix round 3's finding 2.)
`alloc4c/tw`'s small/large spread (`0.992316` to
`1.007947`) is noise around a median of exactly `1.0`, the same shape finding 5 named for
round 1's sitting.

## What I could not close

Nothing new past round 1's own residual (the 5b-owned scope-naming case). The mechanism
now covers every operator this crate's own arithmetic, prefix, logical and DO-header
paths can raise from; comparison and concatenation cannot raise past a receiver at all,
so there is nothing there to close.


# Fix round 3

**Commits:**
`56c842cb03efacf942caa9113535e22fd43a80e8` -- finding 1's fix, the comparison witness, findings 2-3.
`6572d67cc54d6a8aa88a8d779543b9741a464138` -- this round's performance sitting row set.

This section's own text was written and appended to the tracked report *before* the first
commit above landed, per this round's own instruction; the sitting section further below
was completed once the sitting the first commit made necessary had actually run.

## Finding 1: comparison, and which branch

**Wrapped it -- the fifth adapter, not a new mechanism.** The false premise (comparison
needs only a string and so cannot fail) was the plan's, corrected at `be81ce689`: the
operator families split by whether the operator *needs a number*, not by whether it is
arithmetic -- arithmetic always, comparison when both operands are numeric, concatenation
never. `Interp::compare_values` (`eval.rs`) now has the same shape as the other four:
split into itself (a thin wrapper) and `compare_values_body` (the original computation),
blaming once on any `Err` from the body.

**The escape hatch did not trigger.** Measured, all frameless before this change and
still frameless after, both engines:

```
numeric digits 1; s. = '9.9E999999999'; say s. == 1     rc 0, "0", no frame (strict; never reaches compare_numbers)
numeric digits 1; s. = '9.9E999999999'; say s. >> 1     rc 0, "1", no frame (strict; never reaches compare_numbers)
s. = 'abc'; say s. > 1                                   rc 0, "1", no frame (non-numeric fallback; compare_strings, not compare_numbers)
numeric digits 1; s. = '9.9E999999999'; say 1 > s.       rc 214, WITH the same underlying overflow, but no frame -- the receiver `1` is not a stem
```

The fourth line is the one that mattered: `1 > s.` **does** raise (`compare_numbers`
overflows regardless of which side supplied the huge operand), but `is_stem_receiver(1)`
is `false`, so `blame_stem_forwarded_operator` is a no-op and the program stays frameless
-- exactly the same shape `arith_general` already had for `say 2 + b.`. No new predicate
was needed; the existing one already discriminates on the receiver, not on which operand
overflowed.

**One corpus program for the family**, per the ruling: `lang/operator_frame_stem_compare_overflow.rex`
(`say s. > 1` under `numeric digits 1` with `s. = '9.9E999999999'`), joined on the same
terms as the eleven already there. `<`, `>=`, `<=`, `=` and `\=` were measured to reach the
identical `compare_values` path and carry the identical frame (below); they do not each
get their own corpus entry, per the ruling that one program covering the shared path is
enough when the report says which operators reach it and how that was checked. Gate:
**118 of 118** (117 + 1).

## The comparison witnesses, both engines, oracle vs. rust

```
numeric digits 1; s. = '9.9E999999999'; say s. > 1    oracle_rc=214 rust_rc=214 (tw, ir) -- MATCH
numeric digits 1; s. = '9.9E999999999'; say s. < 1    oracle_rc=214 rust_rc=214 -- MATCH
numeric digits 1; s. = '9.9E999999999'; say s. >= 1   oracle_rc=214 rust_rc=214 -- MATCH
numeric digits 1; s. = '9.9E999999999'; say s. <= 1   oracle_rc=214 rust_rc=214 -- MATCH
numeric digits 1; s. = '9.9E999999999'; say s. = 1    oracle_rc=214 rust_rc=214 -- MATCH
numeric digits 1; s. = '9.9E999999999'; say s. \= 1   oracle_rc=214 rust_rc=214 -- MATCH
```
Stderr for every row: `       *-* Compiled method "<op>" with scope "String".` then the
same `Error 42`/`42.901` pair `operator_frame_stem_do_exponent_range.rex` already carries,
substituting only the operator's own spelling and the clause text. All six diffed empty
against the oracle on the tree-walker engine (the `<`, `>=`, `<=`, `=`, `\=` rows above);
`>` (the corpus program) was run on both engines, matching on each.

## The five gate commands, unpiped, run after this round's commit

1. `cargo fmt --all --check` -- exit 0.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.
3. `cargo test --release --workspace` -- exit 0. 98 test binaries, all `ok`, 0 `FAILED`.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit 0. 98/98 `ok`, 0 `FAILED`;
   `corpus_differential` reports `118 of 118 matching`.
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` -- exit 0. 98/98
   `ok`, 0 `FAILED`.

## The negative control, extended, in full, all twelve corpus programs

Disabled `Interp::blame_stem_forwarded_operator` at its one definition, rebuilt, ran
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus corpus_differential --
--nocapture`: `106 of 118 matching`, all twelve `operator_frame_stem_*` programs
`[UNCLASSIFIED]`, `test corpus_differential ... FAILED`. Diffed each program directly
against the oracle (full stderr, not the harness excerpt) -- the eleven from rounds 1-2
reddened identically to their own prior transcripts (already in this report), and the new
one:

```
=== operator_frame_stem_compare_overflow.rex ===
oracle:        *-* Compiled method ">" with scope "String".
    17 *-* say s. > 1
Error 42 running .../operator_frame_stem_compare_overflow.rex line 17:  Arithmetic overflow/underflow.
Error 42.901:  Arithmetic overflow; exponent ("1000000000") exceeds 9 digits.
rust:     17 *-* say s. > 1
Error 42 running .../operator_frame_stem_compare_overflow.rex line 17:  Arithmetic overflow/underflow.
Error 42.901:  Arithmetic overflow; exponent ("1000000000") exceeds 9 digits.
diff: missing exactly the frame line, nothing else.
```

Restored, rebuilt, reran the identical command: `118 of 118 matching`, `test
corpus_differential ... ok`.

## Finding 2: two false performance sentences, corrected

`task-6-report.md:436` said "and nothing on any path that succeeds"; `:727` said "Nothing
this round's diff added runs on any path `arith`'s benchmark exercises". Fix round 4
replaced both sentences at those lines; what follows is why they were wrong, and the
replacements say it there. Both are false in
the same way: `arith_general`'s wrapper runs `if result.is_err()` **unconditionally**, on
every call, whether the body succeeded or failed, and `bench-programs/arith.rex` is all
`arith_general` work, so that branch runs on every clause the axis measures. What is true,
and what those sentences should have said: the added work on a *successful* path is one
predicate evaluation (the `is_err()` check) per arithmetic result; what is skipped on
success is the expensive part -- `is_stem_receiver`'s shape check and the frame render --
not the branch itself. Measured, across the single commit `211763aaa..c351fa473`: `arith`
went from `1.000000` to `1.001006`-`1.001294`, this round's own sitting, shown row by row
below rather than summarised here. That is
consistent with the one predicate being real, attributable work rather than pure layout
noise -- but *how much* of it is the branch and how much is code layout shifting again
needs an interleaved do-nothing control, which was not run and is not owed: the ratio
stays under the 1% threshold either way, so nothing is gated on separating the two causes.

## Finding 3: prose

* `eval.rs:1053` (as it read after round 2) said `arith_left_operand` shares
  `blame_stem_forwarded_operator` "with it below", which round 2's own edit had already
  made false -- the call moved to `arith_general`/`header_number`, not
  `arith_left_operand`. Reworded to name `Interp::header_number` and `Interp::arith_general`
  as the two callers.
* `eval.rs:973`'s "a non-stem receiver reaches this exactly as before" was historical
  framing and slightly false: the *predicate* is unchanged, but the *call site* is not --
  a non-stem receiver now reaches `blame_stem_forwarded_operator` from every failing path
  of `arith_general`, where round 1's inline call only reached it from
  `arith_left_operand`'s own narrower branch. Reworded to say that distinction directly.
* `eval.rs:972`'s "all three past `arith_left_operand`'s own return" named the size of the
  set of examples just listed, the same shape finding 2 struck in round 2. Reworded to
  "each of them past `arith_left_operand`'s own return".
* The report's own performance table (round 2's) broke its stated "(min..max) where they
  differ" convention twice: `alloc4c/ir/small`'s min is `0.999999` against a max of
  `1.000000`, which differs and was not shown; and `arith/tw`'s row collapsed `small`
  (`1.001006`) and `large` (`1.001036`) into one figure, silently dropping `large`'s own
  value. This round's own table (below) lists every axis/engine/size cell's median with
  its min and max, rather than a per-axis summary that can hide a differing row.
* `operator_frame_stem_divide_by_zero.rex` and its `sourceline_oracle` twin said "the
  divisor's own arithmetic overflow" -- the divisor is `0`; what raises is the division's
  own zero check, not an overflow. Corrected in both files (the `sourceline_oracle` twin
  regenerated from the corrected `.rex`, same driver as every other fixture in this task,
  line count now 10 where it was 9).

**Recorded, not fixable:** `c351fa473`'s own commit message says the five witnesses
"join the eleven already there". Six were there at that commit (the three operator
programs plus the three DO-header programs from round 1), not eleven -- the commit
message miscounted its own base. The commit cannot be edited; this is the correction,
placed where the next reader will see it.

**A second instance, this round's own:** `56c842cb0`'s title calls comparison "the sixth
operator family that needs a number". Nothing in this task establishes six families or
six of anything -- the established language is three *families* (arithmetic, comparison,
concatenation, per `be81ce689`) and five *call sites* now sharing
`blame_stem_forwarded_operator` (`arith_general`, `apply_prefix`, `logical_values`,
`header_number`, `compare_values`). "Sixth" matches neither count and was written without
checking either. The commit is already made; recorded here rather than amended.

## Performance sitting

`src/` changed again (`eval.rs`'s `compare_values`), so a fresh sitting is owed for this
round's own commit -- reusing round 2's would measure a binary that does not contain this
round's change. Staleness test re-run: `15a1ffa98` still an ancestor of HEAD,
`git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock
Cargo.toml` lists `56c842cb0` on top of the commits already recognised in earlier rounds,
none foreign. Pin valid.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task 6 --commit 56c842cb0 --baseline bench-baselines/phase-5a-arms.tsv
```
Exit 0, appended 312 rows, committed as `6572d67cc54d6a8aa88a8d779543b9741a464138`.

**Every row, `pinned>head across_builds instructions:u`, read directly from the appended
rows rather than summarised -- no row hidden by a collapse this time:**

```
axis        engine  size   median     min        max
alloc4c     tw      small  1.000000   1.000000   1.000000
alloc4c     ir      small  1.000000   1.000000   1.000001
alloc4c     tw      large  1.000000   1.000000   1.000000
alloc4c     ir      large  1.000000   1.000000   1.000000
arith       tw      small  1.001006   1.001006   1.001006
arith       ir      small  1.001284   1.001283   1.001284
arith       tw      large  1.001036   1.001036   1.001036
arith       ir      large  1.001294   1.001294   1.001294
compound    tw      small  1.000000   1.000000   1.000000
compound    ir      small  1.000000   1.000000   1.000000
compound    tw      large  1.000000   1.000000   1.000000
compound    ir      large  1.000000   1.000000   1.000000
emptyloop   tw      small  1.000000   1.000000   1.000000
emptyloop   ir      small  1.000000   1.000000   1.000000
emptyloop   tw      large  1.000000   1.000000   1.000000
emptyloop   ir      large  1.000000   1.000000   1.000000
strings     tw      small  1.000000   1.000000   1.000000
strings     ir      small  1.000000   1.000000   1.000000
strings     tw      large  1.000000   1.000000   1.000000
strings     ir      large  1.000000   1.000000   1.000000
varlookup   tw      small  1.000000   1.000000   1.000000
varlookup   ir      small  1.000000   1.000000   1.000000
varlookup   tw      large  1.000000   1.000000   1.000000
varlookup   ir      large  1.000000   1.000000   1.000000
```

**`arith` did not move.** `bench-programs/arith.rex` runs `arith_general` on every clause
it measures and reaches `compare_values` on none: it contains no comparison operator, and
its `do i = 1 to n` bound test is answered by the integer fast path or
`Interp::controlled_within_wide` (`run.rs:8916-8925`), never by `compare_values`. So
wrapping `compare_values` had no path of `arith`'s to add work to.
An adapter landing elsewhere with no move on this axis is what wrapping a *different*
function predicts, not evidence the wrapping itself is free -- finding 2 above already
corrected that overclaim once. The table above is the record of what the sitting measured;
per-row numbers are there to be read rather than restated in a sentence here, which is the
step this report has got wrong in each of the three preceding rounds. Every ratio in that
table stays under the 1% threshold on `instructions:u`, the instrument the constraint
names; no interleaved control is owed.

## What I could not close

Nothing new. The 5b-owned scope-naming case remains the one residual, unchanged since the
original report.

# Fix round 4

Committed as `c27b46ea475d54b8280c408801eee6c9d4c09c77`, four files, 26 insertions and 26
deletions. This section was written before that commit, so nothing in its message describes
a file that did not already say it.

Prose only. **No line of code changed.** The whole diff is comment text plus the fixture
copy of one comment: four files, and every changed line in `git diff` is a comment line or
a line of the `sourceline_oracle` twin that mirrors one. The mechanism, the witness, the
corpus arithmetic and the sitting rows are untouched.

## The rule this round applied, once, to four findings

An enumeration in prose goes stale at the next commit that adds a member, and nothing
tells you. So for NEW-1 through NEW-4 the fix is not a better count or a longer list: the
enumeration is deleted, the sentence says what the set is and what puts something in it,
and it points at the code where that set is written down -- where something fails when it
drifts. A list the compiler or a test enforces is fine; prose is not that.

## In the tree

**NEW-1 and NEW-2 -- `rust/corpus/phase-5a.txt:213-219`.**

Was: "`Interp::compare_values` joins the same blame-on-any-failure shape the other **four**
adapters already have. One program stands for the **six** non-strict comparison operators
that reach `compare_values`'s numeric path (`>` here; `<`, `>=`, `<=`, `=` and `\=` were
measured to carry the identical frame)."

Now: "`Interp::compare_values` joins the same blame-on-any-failure shape the other adapters
have. One program stands for the operators that reach `compare_values`'s numeric path --
`>` here, and an operator is on that path when `is_comparison` admits it and
`is_strict_compare` rejects it, both in `eval.rs`, which is where that set is written down
in code rather than in prose."

Both counts are gone and neither is replaced by another number. "already" went with them:
strike it and the sentence says the same thing about the code, so it was decoration.

One copy of the false six is out of reach and stays wrong: `56c842cb0`'s commit message
says "the sixth operator family" in its title and "the six non-strict operators that reach
it" in its body. Nothing here can edit that, so the tree and the history now disagree on
this point, and the tree is the one that is right.

Read before writing it, rather than taken from the finding: `is_comparison`
(`eval.rs:1902`) is the guard on `apply_binary`'s comparison arm, and `is_strict_compare`
(`eval.rs:2018`) is what `compare_values_body` branches on to decide whether `to_number` is
called at all. So "admitted by the first and rejected by the second" is the property that
actually decides the numeric path, not a restatement of the operators that happen to be on
it today.

**NEW-3 -- `rust/corpus/lang/operator_frame_stem_compare_overflow.rex:6-10` and its twin
`rust/crates/rexx-parse/tests/sourceline_oracle/operator_frame_stem_compare_overflow.txt:7-11`.**

Was: "One program stands for the family: `>` here, and `<`, `>=`, `<=`, `=` and `\=` all
reach the identical `compare_values` path and were measured to carry the same frame. Strict
`==`/`>>` and the non-numeric fallback (`compare_strings`) never reach `compare_numbers` at
all and stay frameless, checked separately."

Now: "One program stands for the family: `>` here, and an operator joins it by being a
comparison `is_strict_compare` (`eval.rs`) rejects -- code, not prose, is where that family
is written down. The strict family and the non-numeric fallback (`compare_strings`) never
reach `compare_numbers` at all and stay frameless, checked separately."

The strict side lost its two-operator sample for the same reason the non-strict side lost
its six: a reader takes a list offered as a family for the family.

Two mechanical points about this edit, because the file is executable and mirrored:

* The replacement was reflowed to hold the file at the same length `wc -l` reported before
  it, with `say s. > 1` still on line 17, so the line number in the traceback the corpus
  differential compares is unmoved.
* The twin was regenerated as `count <the .rex's line count>` followed by the file
  verbatim. That construction was verified before being used: applied to the *unedited*
  `.rex` it reproduces the committed fixture byte for byte, and it does the same for
  `operator_frame_stem_divide_by_zero`, so it is the shape the capture driver produces and
  not a guess. The check is live rather than decorative -- with one word of the twin
  altered, `cargo test --release -p rexx-parse --test sourceline_oracle` exits 101 and
  reports `FAILED. 0 passed; 1 failed`; the tree was then restored and `cmp` confirmed it
  byte-identical to the backup.

**NEW-4 -- `rust/crates/rexx-exec/src/eval.rs:1099-1105`.**

Was: "[`Interp::arith_general`], [`Interp::apply_prefix`] and [`Interp::logical_values`]
each call this on any failure of their own, and `Interp::header_number` (`run.rs`) does the
same for the other receiver of a real unary `+` this crate models" -- a roll of the callers,
which this task's own new call site had already made incomplete, and the one that would rot
fastest since every future adapter adds to it.

Now: "A caller is an adapter that evaluates one operator whose receiver is `value`, and it
calls this on any failure of its own -- `Interp::header_number` (`run.rs`) among them, where
a controlled `DO` header position is not written as an operator but is rounded through a
real unary `+` whose receiver it is."

`header_number` stays named because the membership rule alone would not obviously reach a
`DO` header, and "among them" marks the naming as partial rather than exhaustive. I checked
the rule at each call site I could find by searching `crates/` for the helper's name --
`eval.rs:822`, `:977`, `:1322`, `:1476` and `run.rs:7298`, each an `if result.is_err()`
immediately after a `_body` call -- and it holds at all of them. That is a text search, not
the compiler; what makes it trustworthy here is that the helper is `pub(crate)` and called
as a method, so no call to it can sit outside the crate I searched. The point of the
rewrite is that the sentence no longer carries that list, so a new caller cannot falsify
it, and a search like mine going stale costs nothing.

**NEW-5 -- `rust/crates/rexx-exec/src/eval.rs:973-975`.**

Was: "`blame_stem_forwarded_operator`'s own predicate is unchanged, but this call site is
not: a non-stem receiver now reaches it on any failure here, where the removed inline call
inside `arith_left_operand` only reached it from that function's own narrower branch."

Now: "The receiver test is `blame_stem_forwarded_operator`'s own, so this site gates on
failure alone and hands it every failing path here, leaving a non-stem receiver for that
predicate to discard."

The deciding test applied: nothing in the replacement is relative to a past state, and with
no historical framing to strike the sentence still says something about the code -- where
the receiver test lives, and that the call site does not repeat it. Verified against the
code rather than against the old sentence: the site's only condition is `result.is_err()`
(`eval.rs:976`), and `blame_stem_forwarded_operator` opens with
`if !self.is_stem_receiver(value) { return; }` (`eval.rs:1115`).

## In the report

**NEW-6 -- `:436` and `:727`, corrected where they stand.** Round 3 recorded finding 2 as
fixed while both sentences were still standing verbatim; the report is git-ignored, so
nothing prevented editing them and nothing excuses not having.

* `:436` was "...(cheap, allocation-free) shape check on the failing path, and nothing on
  any path that succeeds." It is now "...on the failing path. A path that succeeds pays the
  wrapper's own `is_err()` test, which runs on every call whether the body succeeded or
  failed; what success skips is the shape check and the frame render, not the branch."
* `:727` was "Nothing this round's diff added runs on any path `arith`'s benchmark
  exercises: the wrapping only adds work on a failing path, and `arith`'s own axis never
  fails." It is now "What this round's diff added *does* run on the path `arith`'s benchmark
  exercises: `arith_general`'s wrapper evaluates `is_err()` on every call, and
  `bench-programs/arith.rex` is all `arith_general` work. What a successful call skips is
  `is_stem_receiver`'s shape check and the frame render, not the branch itself."

Each replacement carries a one-clause pointer naming the round that found it, so a reader
landing on either line sees the correction and its provenance without having to reach the
round-3 section. Round 3's own citation of the second line was updated from `:724` to
`:727`, which is where the sentence sits after the first correction lengthened `:436`.

**NEW-7 and NEW-8 -- the prose around the sitting table, deleted rather than corrected.**
The table itself matches the TSV cell for cell and is the record; three consecutive rounds
have now put a false sentence on top of it, so the sentence is the defect.

* `:853` was "`arith` went from `1.000000` (min equal to max on all four rows) to
  `1.001006`-`1.001294` (again min equal to max on all four rows, this round's own sitting,
  shown below)". Both parenthetical summaries are gone; the sentence now ends "shown row by
  row below rather than summarised here."
* The paragraph that read "Only `alloc4c/ir/small` differs from an exact `1.000000`..." is
  deleted outright. It existed only to describe rows that are printed directly above it.
* The `arith` paragraph keeps its substantive conclusion and drops the per-row precision
  claim ("the same four values to six decimal places"). It now says why the axis could not
  move -- `bench-programs/arith.rex` runs `arith_general` on every clause it measures and
  reaches `compare_values` on none, having no comparison operator, and its `do i = 1 to n`
  bound test being answered by the integer fast path or `Interp::controlled_within_wide`
  (`run.rs:8916-8925`) instead. I read both of those rather than carrying them over: the
  benchmark has no comparison operator in it, and `run.rs:8916-8925` is a `match` that
  answers the bound test with a small-integer comparison or `controlled_within_wide`,
  reaching `compare_values` from neither arm.
* The threshold sentence is kept and scoped to what it covers: every ratio in that table
  stays under 1% on `instructions:u`, the instrument the constraint names. Checked by
  reading the table's own cells -- the largest is `1.001294`, the smallest `1.000000`. It
  says nothing about the `cycles:u` arms, which the constraint does not treat as a result on
  their own.

One claim dropped rather than restated: the earlier `arith` paragraph credited the table
with showing that wrapping `compare_values` "specifically added nothing to `arith`'s own
path". A null result on an axis that never reaches the wrapped function is what the
mechanism predicts either way, so the table cannot distinguish "added nothing" from "had
nothing to add to"; the surviving sentence says the second, which is what was actually
established.

## Also recorded, not fixed

Everything below is outside this task's diff and was confirmed by the re-review to be
identical on the pinned pre-task build `bench-baselines/pinned/rexx-run-15a1ffa98`, so none
of it is a regression this round introduced or could close.

* **`s. = .nil` with a comparison silently answers where the oracle raises.** `s. = .nil;
  say s. > 1` is rc 159 with `97.1 Object "The NIL object" does not understand message ">"`
  on the oracle; this crate prints `1` and exits 0, on both engines and on the pinned build.
  This is worse in shape than the arithmetic sibling: `s. = .nil; say s. + 1` at least
  fails, with 41.1 where the oracle gives 97.1, and `is_stem_receiver`'s own doc already
  records the oracle's 97.1 for that case. A wrong error is loud; a wrong answer at rc 0 is
  not, and no differential row will catch it while the corpus contains no program of that
  shape. Whether it belongs to Phase 5b is not this task's call.
* **`eval.rs:3271`** -- "`apply_prefix`, `arith_general` and `compare_values`, all four of
  which ..." is a set-size phrase in a test-module doc. It predates this task's diff, and
  the count is right today.
* `s. = .environment; say s. > 1` and `s. = .array; say s. > 1` both take the declared
  Phase-5 loud refusal, unchanged by this round.

## The five gate commands, from `rust/`, unpiped, run after these edits

Each was run with its output redirected to a file, never piped, so the status read is the
command's own and not a pipe's.

```
cargo fmt --all --check                                              exit 0
cargo clippy --workspace --all-targets -- -D warnings                exit 0
cargo test --release --workspace                                     exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                  exit 0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    exit 0
```

Both corpus-gated runs report `118 of 118 matching`, and the `--no-fail-fast` run contains
no `test result: FAILED` line at all, which is the count that matters when nothing stops at
the first catcher. `tests/sourceline_oracle.rs` ran inside the release suite and passed,
which is the gate on the regenerated twin.

## No sitting was run, and why

This round changed no code. Every changed line is a comment or the fixture copy of one, so
no instruction is added to or removed from any executed path and there is nothing for the
`instructions:u` instrument to see. What the diff does move is `eval.rs`'s debug line table,
since the edits are not length-preserving in that file -- and that is not something the
instrument reads.

Said plainly, because it is the weaker half of the claim: I did not rebuild both arms and
compare the binaries' `.text`, so this rests on comments not reaching codegen rather than on
a measurement that they did not. The brief rules the sitting out for this round on exactly
that ground.

## What I could not close

Nothing was left open from the findings. What sits under "Also recorded, not fixed" above
stays open by instruction, not by obstacle -- all of it is outside this diff. The `.nil`
comparison is the entry there worth someone's attention, being a silent wrong answer rather
than a loud one.
