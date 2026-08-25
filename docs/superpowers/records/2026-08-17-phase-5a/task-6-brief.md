## Task 6: the native-method frame in a traceback, the operator half

**Goal.** An error raised inside a native method that an **operator** invoked emits the
`*-* Compiled method "X" with scope "Y".` frame line, as the oracle does.

**Why here.** It is the `diverge-stderr` cell's live example, it is small, and it is upstream of every
later task whose probe raises through an operator. Measured: `say "a"~length(1,2)` is **identical on
both sides today, frame line included** -- the explicit-send half landed with the old plan's Task 5 --
while `say b. + 1` with `b.` untouched is **rc 215 with empty stdout on both sides** and the oracle's
stderr opens with `       *-* Compiled method "+" with scope "String".` where ours does not.

**Build.** The operator path raises through a method frame carrying the operator's own name and the
scope of the class whose method it is. The frame's leading whitespace and the absent line number are
part of the bytes.

**And the const's own doc block is corrected in this commit.** `corpus.rs`'s
`raw_stderr_comparison_only_names_programs_the_subset_actually_runs` carries a doc saying
`RAW_STDERR_COMPARISON` is empty and the test therefore vacuous. The superseded plan's tasks left
entries in it, so that sentence is already false before this task appends to it -- and it names a
set's size and frames itself historically, which the Global Constraints strike.

**Verification, runnable now.** Oracle-differential on `say b. + 1`, on `say b. ** 2` and on
`say -b.`, both engines, compared **raw**. Each program joins `phase-5a.txt` and
`RAW_STDERR_COMPARISON` in this commit.

**A comparison and a concatenation cannot serve here, and that was measured rather than assumed.**
`say b. > 1` prints `1` and `say b. || 1` prints `B.1`, both at rc 0; so do the `Array`-operand forms
`say 1 > .array~of(1)`, `say "a" > .array~of(1)`, `say 1 = .array~of(1)` and `say 1 || .array~of(1)`.
And `say .array~of(1) + 1` does raise -- error 97, **with no frame line**, because no method was found
so there is no method activation to name. The reason is the one Task 14 states from the other side:
an arithmetic operator must convert its operand to a *number* and that conversion can fail, while
comparison and concatenation need only a *string*, and every object has one. So the second and third
programs vary the operator's **shape** instead: `**`, so the frame's method name is read rather than
assumed, and prefix `-`, a different dispatch path whose frame name collides with infix minus.
Measured, each is rc 215 on both engines and differs from the oracle in exactly one line --
`       *-* Compiled method "**" with scope "String".` and
`       *-* Compiled method "-" with scope "String".`.

**Negative control, and it fires on the corpus rather than on table C.** Drop the frame line and those
three corpus programs redden byte-exactly under the gate. **Table C has no row for this mechanism** --
the spec names the operator-frame line as one of two mechanisms with no documented section, and
concept rows are one per section id, so a control written against a table C row would demonstrate
nothing. That is recorded in Task 5 as well, so the absence is stated in both places rather than
inferred from a control that never fires.

**What it cannot see, and the program that covers the half of it that is reachable.** A frame
emitted with the wrong scope name -- the receiver's class where the oracle prints the scope of the
method that actually raised -- is caught only by a probe where those two names differ. **This task's
first program already is that probe, and it is named here so nobody adds a fourth looking for one:**
measured, `b.~class` is `The Stem class` on the oracle while `say b. + 1`'s frame reads scope
`String`, because the stem forwards the operator to its default value and the method that raises is
`String`'s. The assigned form `s. = "abc"` then `say s. + 1` is the same shape and is rc 215 on both
sides with the frame line the only difference. A build that names the source receiver's class instead
of the running method's scope reddens either one. What stays uncovered is the stricter case:
the *frame's own* receiver having a class different from the scope of the method it is running, which
needs an inherited method on an instance and is therefore 5b's.

**Done when** the three operator programs match byte for byte on both engines and the control is
recorded. Sitting required: `rexx-exec/src/`.

---

