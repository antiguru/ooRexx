# Task 1 re-review: fix round 1

Scope: `5a801e1e9..a8d36deb8`, of which `1fdc2a9b1` and `a8d36deb8` are the
implementer's. `d2efbaf5b` (`rust/CLAUDE.md`, "a comment may not name the size
of a set") is read as governing rather than as subject.

**Verdict: all eight findings addressed. Four new false statements, three of
them born in the correction and one pre-existing sentence falsified by the rows
this round added.**

## What I verified rather than took on trust

Read-only, from `rust/`:

* `cargo test -p rexx-exec --test ir_dual` -- 9 passed, 0 failed, including
  `both_engines_agree_on_every_case_file`. So the rewritten stanzas match on
  both engines.
* The three programs this round captured, re-run against the C++ oracle from a
  fresh empty directory (not the scratchpad), three descriptors, absolute paths:
  the twelve-row logical stanza answers `1 0 0 0 1 1 1 0 0 1 1 0`, the DIGITS
  stanza `1 0 0 0 0`, the FUZZ stanza `1 0 0 0 0`. Every added byte is the
  oracle's, and the two added strict rows are `0` at both settings as claimed.
* `/bin/grep -rn "eval_compare\|eval_logical" crates/` minus `eval_logical_list`
  -- no hits. `Interp::concat`/`self.concat(` -- no hits.
* `error.rs`: `Failure` is `Loud`/`Raised`/`Exited`; `/bin/grep -n ObjRef
  crates/rexx-exec/src/error.rs` returns three lines (import, a doc mention,
  `Exited(Option<ObjRef>)`). `Loud` is `struct Loud { message: String }`
  (`lib.rs:463`). `Raised`'s fields are `Cow<'static, str>`, two `u16`s,
  `Vec<Substitution>` with `type Substitution = Vec<u8>` (`error.rs:141`),
  `Option<Vec<u8>>` twice, `Delivery`.
* `concat_values` returns `Ok` on every path; `compare_values` fails only
  through `.map_err(Raised::from)?`; `logical_values` only through
  `Raised::not_logical` and `Loud::binary_operator`.
* `compare_op`'s match arms, read one by one against the doc above them.
* `CompareOp` has twelve variants (`crates/rexx-num/src/compare.rs:52`), and
  `compare_decoded`'s strict path is `Ok(op.holds(a.cmp(b)))` (`compare.rs:154`)
  -- byte equality with no padding, which is what `test_case_when`'s new reason
  rests on.
* `apply_binary` is `pub(crate)` (`eval.rs:1015`) and `test_case_when`'s
  `case_text` is `&[u8]` (`run.rs:7032`), so both halves of the new reason hold.
* `compile.rs` emits `Op::TraceClause`, `Op::TraceLiteral`, `Op::TraceRead`,
  `Op::TraceOperator`, and `>=>`/`>>>` come from `Op::Store`/`Op::Say` -- so the
  `arithmetic` family stanza's replacement sentence is true of its own rows.

## Per finding

### 1 -- addressed

`eval.rs`'s module doc no longer counts or enumerates the frame-opening sites
and states the mechanism instead. `run.rs:14460` and `rexx-core/src/roots.rs`
lost the same count.

**The `roots.rs` argument survived intact.** Both halves are still there and
still connected: truncation is load-bearing *because* a raise leaves an inner
`pop_frame` unreached and `step_in_temps_frame`'s outer watermark heals it, and
"do not add a balance assertion **without first making every such site pop on
its own path**" still names the precondition the count used to quantify. The
count was never doing work in the inference; only "there are sites like this"
was, and that is preserved.

### 2 -- addressed

`eval.rs:18` and `run.rs`'s `numeric_less` doc both lost "twelve"; the second
also had `eval_compare` corrected to `compare_values`, and that citation is
accurate -- `compare_values`' doc does state the no-second-comparison rule.

**On the instruction the implementer went against** (`eval.rs`'s "`CompareOp` has
only twelve variants", excluded from the sweep on the grounds that it is correct
about `rexx-num`'s enum): the replacement, "**Several operators share one
`CompareOp`**", does still say what that sentence existed to say. The sentence's
job was to explain why this function is a many-to-one map rather than a rename,
and the paragraph then enumerates exactly which operators share -- so the
mechanism is intact and the reader who wants the number can read
`compare.rs:52`. What was lost is only the "and here is how much smaller the
target set is" framing, which is the part the rule is about. The de-count is
defensible. Its **execution** is not: the same edit widened a neighbouring
quantifier into a false claim -- see NF-1.

### 3 -- addressed

All six dangling references gone; the grep is clean over the whole of `crates/`.

**`test_case_when`'s replacement reason is true.** The old reason ("needs a
second `eval.rs` visibility bump") had indeed stopped being true --
`apply_binary` is `pub(crate)`. The new one has three claims and all three hold:
routing would answer the same (`CompareOp::StrictEqual` is `a.cmp(b).is_eq()`,
byte equality, which is what `text == case_text` computes); it is reachable from
here; and only one side is a value, because `case_text` arrives as `&[u8]` and
would have to be allocated into an `ObjRef` for `compare_values` to render
straight back to bytes already in hand.

### 4 -- addressed on substance; the comment beside it overstates

The stanza now runs all three operators over all four operand combinations. The
hole is closed in both directions: `say 1 | 1` -> `1` reddens an implementation
computing `Or` as `Xor`, and `say 1 && 1` -> `0` reddens `Xor` as `Or`. Verified
against the oracle, not only against the harness.

Two problems came with the fix: NF-2 (the file's own header now misdescribes
what the file is worth) and NF-4 (the stanza comment's counterfactual).

### 5 -- addressed

`1000000` and `1000100` differ in the fifth significant digit and the comment
now says fifth; `1000000`/`1000001` differ in the seventh, also correct. The
comment quotes the numbers instead of numbering positions, which is the fix that
does not rot. The implementer found and fixed two more of the same shape
(the concatenation and nesting stanzas' row numbering) unprompted.

### 6 -- addressed

The "different route through `eval.rs`" clause is gone and the replacement
("where they differ is in what computes the value behind the line") is true on
both engines: `eval_arithmetic` against `apply_binary` on the tree-walker,
`Op::Arith` against `Op::Binary` compiled.

The family stanza's comment is also now true of its rows. I checked the stronger
half of it -- "on the compiled engine every line here comes from an op" -- against
`compile.rs`: the clause lines come from `Op::TraceClause`, `>L>` from
`Op::TraceLiteral`, `>V>` from `Op::TraceRead`, `>O>` from `Op::TraceOperator`,
`>=>` from `Op::Store` and `>>>` from `Op::Say`, and every clause in that stanza
promotes, so no `Op::EvalExpr` is in it.

### 7 -- addressed, and the argument holds

The change is now documented in three places (module doc, the arm itself, the
commit message). The argument as written stands up:

* `pop_frame` truncates to the watermark the arm took itself, so it can discard
  only what that arm rooted. Checked against `roots.rs:141`.
* The claim the doc rests on -- "`Failure::Exited` is the variant that carries an
  `ObjRef`" -- is **true**, and I checked it structurally rather than by grep:
  `Loud` holds a `String`, `Raised` holds a `Cow<str>`, two `u16`s, `Vec<Vec<u8>>`
  and two `Option<Vec<u8>>` plus a `Delivery`. No root can travel out on a
  `Raised` or a `Loud`.
* The other half -- "the functions `apply_binary` dispatches to build only
  `Raised` and `Loud`" -- is true by reading all three bodies. None of them calls
  `eval`, which is the only constructor of `Exited` (`eval_call`).

So popping unconditionally is safe, and it is stated as a property of the three
functions rather than as a reachability claim, which is the right form.

### 8 -- addressed

The strict row now exists at both settings in both stanzas, and both answers are
oracle-verified. The comments claim a control the rows now actually provide.

## On the new rule (`d2efbaf5b`)

**Applied where it applies.** Every set-size count in the prose this task wrote
is gone: `eval.rs`'s module doc, the collapsed arm's comment, `compare_values`,
`apply_binary`, `compare_op` (doc and `unreachable!` message), all four family
predicates, `is_native_binary`, `corpus_shape_tests::arithmetic`, and the
`operators` header and stanza comments. I greped `5a801e1e9`'s own added lines
for number words and found nothing left that is a set's size -- what remains
("two operands", "the two ops", "two registers", "the two engines") is ordinary
prose, exactly as the second commit message claims.

**Not over-applied to a measurement.** `Three mutations of the promoted path`,
`Seven of the eighteen mapping rows survived a mutation run`, `rc 245`, the error
numbers (34.901, 41.1, 26.8), the DIGITS/FUZZ settings and "one space and ...
four" (a description of a program's own text) all kept their numbers. I found no
case where a number that was evidence was removed.

Two wobbles, neither a false statement:

* `a8d36deb8` turned "the two comparison families" into "**both** comparison
  families". "Both" asserts the same cardinality with a determiner instead of a
  numeral, and rots the same way a numeral would. If the rule means what its
  commit message says, this is a rename rather than a fix.
* `operators`' header opens "by kind" while its sibling `arithmetic`, edited in
  the same commit, still opens "in four kinds" -- and `arithmetic` also still says
  "all seven operators" twice. The two files' headers are now written to
  different rules.

## New false statements, most severe first

### NF-1. `compare_op`'s doc now makes a false claim about `\==` (and `\=`)

`crates/rexx-exec/src/eval.rs:1235`:

> ... and the backslash-negated forms invert their positive counterpart's sense
> rather than getting a `CompareOp` of their own: `\>` ("not greater than") is
> `LessEqual`, `\<` is `GreaterEqual`, and their strict siblings `\>>`/`\<<` map
> the same way onto `StrictLessEqual`/`StrictGreaterEqual`.

The old text was "the **four** backslash-negated forms", and the four were
exactly the four the colon then names. Deleting the number widened the subject
to *every* backslash-negated comparison operator, and there are six:

* `StrictBackslashEqual` (`\==`) maps to `CompareOp::StrictNotEqual`
  (`eval.rs:1256`), and **nothing else maps to `StrictNotEqual`**. It is a
  `CompareOp` of its own by any reading, and it does not invert `==`'s sense
  through another operator's `CompareOp` the way `\>` inverts `>`'s.
* `BackslashEqual` (`\=`) maps to `NotEqual` -- which the *same sentence's first
  clause* has just finished describing as `\=`'s own mapping alongside `<>` and
  `><`. The second clause therefore contradicts the first about `\=`.

This is the failure mode the rule is meant to prevent, arriving through the fix
for it: the count was not just a number, it was the quantifier's bound. The
rule-compliant repair is to name the set -- "`\>`/`\<` and their strict siblings
`\>>`/`\<<` invert ..." -- not to delete the bound and leave a bare plural.

### NF-2. `tests/ir_dual_cases/operators` now catches a mutation its own header says it cannot

Header, lines 34-46, unchanged by this round:

> **What this file is worth was measured rather than assumed, and it is not
> mutation-catching.** Three mutations of the promoted path were each re-run
> with this file moved out of the directory, and every one was caught anyway
> ... **So the reason to keep these rows is that they pin oracle bytes** ...

That measurement was taken against the file as it stood *before* this round.
This round added `say 1 | 1` precisely because the review showed an
implementation computing `Or` as `Xor` passed the stanza unchanged. As far as I
can determine, that row is now the only thing in the workspace that reddens that
mutation:

* `eval.rs`'s own truth-table unit test
  (`the_three_logical_operators_and_their_truth_tables`, `eval.rs:2089`) has
  `1 & 1`, `1 & 0`, `0 | 0`, `1 && 1`, `1 && 0` -- no true/true `|` row, so `Or`
  computed as `Xor` passes it.
* `/bin/grep -rnE 'b"[^"]*[^|]\|[^|][^"]*"'` over `crates/` returns only
  `say (0 | 0)` and `say (1 | 'x')` (which raises). No `1 | 1` anywhere in Rust.
* No corpus program contains a Rexx `|` at all (checked all 81 files under
  `rust/corpus`; the only single pipes in the tree are markdown table borders).
* The ooTest sweep and the population sweeps are engine-against-engine, and a
  mutation inside `logical_values` moves both engines identically, so they
  structurally cannot see it -- which is the header's own argument, turned around.

I did not run the mutation (read-only), so this is inference from the four
points above rather than a measurement. But either way the header's sentence is
no longer supported by the measurement it cites, and it is load-bearing in the
worst direction: it is the argument a future reader would use to decide these
rows are safe to thin.

The correct repair is one sentence, not a re-measurement: say that the three
mutations measured were caught elsewhere, and that the operand-combination rows
were added because a fourth was not.

### NF-3. The report's "Three counts in `eval.rs` I did not touch" is an under-count

`task-1-report.md`, fix-round section: "**Three counts in `eval.rs` I did not
touch, and why.** All three predate this task (`git blame`: ...)", then three
bullets -- `arith_general`'s `unreachable!`, `small_int_arith`'s "`/` is the one
of the seven", and the "Seven of the eighteen mapping rows" test comment.

At least two more counts of *the same arithmetic set* were left, and neither is
in the list:

* `eval.rs:707` -- `eval_arithmetic`'s own doc comment: "**The seven arithmetic
  operators**, sharing one operand-evaluation and error-conversion path." This is
  the most prominent one in the file, it sits on the function this whole task
  split around, and it is the direct analogue of `is_arithmetic`'s doc, which
  *was* de-counted three hundred lines below it.
* `eval.rs:1143` -- `small_int_arith`'s doc, "**All seven arithmetic operators**
  are here", a separate sentence from the `/`-clause the report quotes.

Beyond the arithmetic set the file also still carries "the three bare-symbol
reads" (114, 256, 471), "the three admissible names" (481), "three `Ended` arms"
(629), "the five keywords" (1061), "all three division operators" (1637) and
"`logical_value` gives all four" (2102). So the honest form of that paragraph is
"eval.rs is full of these and the sweep is bigger than this round", which is what
the paragraph's *conclusion* says -- the enumeration in front of it is what is
wrong, and per this project's own ledger the fix is to delete the list rather
than extend it.

The **boundary itself** ("the sweep's scope is this task's own prose") is
defensible and I would not widen it. Nothing left behind is false; the file is
merely inconsistent with itself, and `arith_general`'s `unreachable!` message
counting a set three lines from `is_arithmetic`'s de-counted doc is the sharpest
instance. It is a follow-up, not a blocker.

### NF-4. The logical stanza's counterfactual is false in one direction

`tests/ir_dual_cases/operators:171`:

> `|` and `&&` agree everywhere except on two true operands, so `say 1 | 1` and
> `say 1 && 1` are the pair that tells exclusive-or from inclusive -- **without
> both, an implementation computing one as the other answers every row
> identically.**

The first two clauses are true. The last one is not, and the stanza's own
history disproves it: before this round the stanza had `say 1 && 1` -> `0` and no
`say 1 | 1`, and an implementation computing `Xor` as `Or` answered `1` there and
**failed**. Each of the two rows catches one direction on its own; what needs
both is covering both directions.

`1fdc2a9b1`'s commit message carries the blunter, plainly false form -- "an
implementation computing **either** operator as the other passed it unchanged" --
where it cannot be edited. It came from the original review's finding 4, which
made the same over-broad claim; the round propagated it rather than checking it.
The rows are right and the fix was the right fix; only the reason given for it
overstates.
