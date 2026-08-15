# Task 3c review: the parser's parenthesis depth counter

Reviewing commit `6285d98f`, "Add a depth counter on the parser's parenthesis
recursion, raising 11.1".

## Verdicts

**Spec compliance: PASS.** Every step of the plan's Task 3c is done, including
Step 4's survey, and the one thing the plan said was not achievable (exact
depth parity) is stated as unachievable rather than faked.

**Code quality: PASS.** The counter is in the right place, decrements on the
error path as well as the success path, cannot be reset mid-descent, and has no
off-by-one. No `unsafe`.

**0 Critical, 3 Medium, 3 Minor.** Nothing blocks. The two Mediums are wrong
numbers in documentation, not wrong code, and the third is a scope-adjacent
measurement that changes how the deferred work should be prioritised.

## Method

Read the plan's Task 3c section, the report, and `git show 6285d98f`. Then
re-measured every load-bearing number rather than accepting it.

All parser measurements were taken from a **clean export of the commit**
(`git archive 6285d98f` into a scratch directory), not from the working tree,
because the working tree carries three other agents' uncommitted changes
including one to `examples/depth_probe.rs`. Every build was run unpiped with
`BUILD_EXIT` checked before any number was read off the resulting binary. That
precaution earned its keep immediately: my first attempt measured the working
tree's binary, which was two minutes older than the commit, and would have
produced nonsense.

Oracle invocations wrapped as `( ulimit -v 1048576; build/bin/rexx FILE )`.

Read-only on the repository apart from this file. Two extra probes
(`select_probe`, `calls_sized`) were written **into the scratch export only**,
never into the repository.

## What reproduced

Everything the report claims, when measured under the conditions it measured
under.

**The oracle's noisy bracket is real.** This was the item flagged as most
likely to be an artifact, so I tested a midpoint the report did not name:

```
oracle n=39900:  rc 0    0    0
oracle n=39925:  rc 245  0    245     <- genuinely non-deterministic
oracle n=39950:  rc 245  245  245
```

Identical file, three runs each. At 39,950 the message is exactly
`Error 11 … Control stack full.` / `Error 11.1:  Insufficient control stack
space; cannot continue execution.`; at 39,900 it prints `a`. So the bracket
`[39,900, 39,950]` with a non-deterministic middle is a property of the oracle,
not of a hasty bisection, and reporting it as a range rather than an integer was
the right call.

**The counter's boundary is exact.** `paren_sized` at 49,999 and 50,000 parses;
50,001 and 100,000 raise `11.1`. No off-by-one.

**The counter cannot be reset mid-descent.** `paren_depth` lives on `Parser`,
and `Parser::new` is called in the free functions `parse_expression`,
`parse_paren_expression`, `parse_constant_expression` and friends. I checked
every caller of the two that consume a `(`: all are in `instruction.rs` at
clause level, none is reachable from inside an expression parse. So a fresh
`Parser` never appears part-way down a nesting, and the field's doc comment
claim is accurate.

**`nested_do` is not recursive**: 100,000 nested `do`/`end` parses cleanly on a
default thread, confirming `translate_block`'s `Vec<Frame>`.

**The pre-fix cliffs reproduce exactly** when I rebuild with `expr.rs` from
`6285d98f^` and the commit's own probe: `paren_default` 337 ok / 338 abort, and
`paren_sized` 88,800 ok / 89,000 abort. Both are the report's numbers to the
integer, which says its method was sound.

**`prefix_chain`** 1,150 ok / 1,200 abort, as reported.

**The corpus claim holds with room to spare.** Scanning 12,103 `.rex` files
under `rust/corpus/` and `rust/corpus-l1/`, the deepest parenthesis nesting
anywhere is **5** (`corpus-l1/SELECT_test_14.rex`). Against a limit of 50,000
and a default-thread cliff of ~331, "no corpus program goes anywhere near
either cliff" is a considerable understatement.

**`cargo test -p rexx-parse`** on the clean export: 396 passed, 0 failed,
including both new `deep.rs` tests.

## Findings

### M1 (Medium) The default-thread cliff is stale for the shipped code, and the fix is what moved it

The report, `tests/deep.rs`'s module doc, the `a_shallow_paren_nesting…` test
doc, and `depth_probe.rs`'s module doc all state **337 parses / 338 aborts** as
this parser's default-thread cliff. On the shipped code it is **331 / 332**.

A/B, same probe, same machine, only `expr.rs` differing:

| `expr.rs` from | deepest surviving | first aborting |
|---|---|---|
| `6285d98f^` (no counter) | 337 | 338 |
| `6285d98f` (shipped) | 331 | 332 |

Five runs at each boundary point, deterministic both sides. So the counter's
own field and check cost about six levels of default-thread depth, ~1.8%.

Two things follow. The quoted number is a measured property of a parser that
no longer exists, in four places, one of which is a test whose stated purpose is
to document the gap accurately. And the direction is the uncomfortable one:
**the fix makes the unprotected case slightly worse.** That is inherent to
adding a check, it is tiny, and it changes no conclusion, which is exactly why
it should be written down rather than left for someone to rediscover as a
contradiction.

Not dangerous: `a_shallow_paren_nesting_still_parses_on_a_default_stack_thread`
parses 300, so its margin went from 37 levels to 31. Still comfortable.

Fix: quote 331/332 as the shipped figure, keep 337/338 as the pre-fix one if
the contrast is worth keeping, and say the counter costs the difference.

### M2 (Medium) "Shallower than plain grouping parens" is backwards

In the report's Step 4 and in `depth_probe.rs`'s module doc, the nested-call
cliff is described as "shallower than plain parens". It is deeper.

Bisected like-for-like on one build, one machine, same probe binary:

```
paren_default: deepest surviving = 331
nested_calls:  deepest surviving = 349
```

Nested calls survive 18 levels further. This is also backwards on the report's
own numbers, without any measurement of mine: it records parens at 337/338 and
nested calls at 350-360, and 355 > 337.

The conclusion the claim was supporting is still correct — nested calls are the
shallowest recursion the counter does **not** guard, prefix chains being three
times deeper — so only the comparison is wrong, not the priority. Worth fixing
precisely because it is the kind of sentence a later task will act on.

### M3 (Medium, scope-adjacent) The nested-call gap reaches the sized path too, and its oracle cliff is now known

Not a defect in this commit: the plan scoped Task 3c to the grouping-paren
recursion, and deferring the rest was right. But the report's stated reason for
deferring — "neither construct's oracle cliff is known" — is no longer true for
nested calls, and what the measurement shows makes the deferred item more
urgent than "a known gap of the same shape" suggests.

`say f(f(f(…'a'…)))`, all three measured:

| depth | oracle | ours, default 2 MiB | ours, sized 512 MiB |
|---|---|---|---|
| 349 | parses, rc 213 (43.1 at run time) | **abort** | parses |
| 10,000 | parses, rc 213 | abort | parses |
| 39,900 | **rc 245, Error 11.1** | abort | parses |
| 50,001 | rc 245, Error 11.1 | abort | parses (counter does not apply) |
| 90,625 | rc 245, Error 11.1 | abort | parses |
| 92,187 | rc 245, Error 11.1 | abort | **abort, rc 134, no message** |

Two readings, and the second is the one that matters.

The oracle **parses** 10,000 nested calls and fails only at run time with 43.1;
a default-thread embedder of `rexx-parse` aborts the process at 349. That is a
divergence of more than an order of magnitude on input the oracle finds
unremarkable.

And above roughly 92,000, **the sized path aborts with no message where the
oracle cleanly reports 11.1** — the exact failure mode D19 and this task exist
to eliminate, still reachable through the `arg_list` arm rather than the
grouping-paren arm. `50,001` parsing on the sized thread is the direct
demonstration that the counter does not cover this path, which the report says
and which I confirmed rather than assumed.

So the follow-up is not "same shape, unknown cliff, low priority". It is a
measured divergence with a known oracle answer (11.1, the same condition
already wired up), on the shipped sized path. Recommend scheduling it with that
framing.

### m1 (Minor) `parse_constant_expression`'s own `(` is not counted

`expr.rs:328`, inside `parse_constant_expression` (`RAISE`, `FORWARD`, `USE ARG`
defaults, `ADDRESS … WITH`), has a second `TokenKind::LeftParen` arm that calls
`full_subexpression` with no depth check. Harmless in practice: everything
nested inside it descends through `subterm`'s guarded arm, so only the outermost
level is uncounted and the effective limit for those four constructs is 50,001
rather than 50,000. Worth one clause in `MAX_PAREN_DEPTH`'s doc, since a reader
of a constant named "MAX" reasonably expects one number.

### m2 (Minor, now closed) `SELECT`/`WHEN` was read but not measured, and it is safe

The report flags this honestly as read-but-not-measured. I measured it, so the
caveat can be dropped: `select` / `when 1=1 then` nested to 100,000, one `nop`,
100,000 matching `end`s, parses cleanly on a **default 2 MiB thread**.

(A first attempt with `select` / `otherwise` was rejected with 7.1 before any
nesting happened, which would have looked like a pass while measuring nothing.
The shape above is the one that actually nests.)

### m3 (Minor) The new test covers the sized path only

`a_paren_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting` runs on
an explicit 512 MiB thread, which is right and necessary. The consequence worth
a sentence: nothing in the suite would notice if `MAX_PAREN_DEPTH` were raised
above the sized-thread native cliff, because the test only asserts that 100,000
raises 11.1, and 100,000 is past the limit by a factor of two. A second
assertion that the limit is below the measured native cliff — a plain
`const` comparison, not a run — would pin the property the doc comment argues
for.

## The two judgement calls

### Is documenting rather than closing the small-stack gap right?

**Yes, and the report undersells its own case.** Three reasons.

The exposed party is a library embedder who parses on an unsized thread. Every
consumer in this tree is already on a sized one: `rexx-exec`'s public entry
point by D19, and `deep.rs`'s own deep test explicitly.

A limit low enough to protect a 2 MiB thread would have to be around 300. That
would refuse, with a Rexx condition, input the oracle accepts by two orders of
magnitude — converting "aborts on absurd input" into "wrong answer on
reasonable input", which is strictly worse against criterion 1, and it would
make the deviation permanent rather than exotic.

And a stack-aware counter needs either a remaining-stack query, which has no
safe stable API without a platform crate, or a per-frame byte estimate
calibrated per build. This phase has now produced wrong per-level stack figures
twice in one day, by two different agents. Making a correctness-critical branch
depend on that class of number would be a poor trade.

The better long-term answer is not a stack-aware counter but a documented
minimum stack for `rexx-parse`, or a sized entry point of its own so an
embedder cannot get this wrong by default. Worth recording as a follow-up; not
this task's.

### Are the two unfixed Step 4 gaps rightly left recorded?

**Prefix chains: yes, clearly.** 1,150 deep on a default thread, no oracle
measurement, and fixing it needs a second counter in `message_subterm`, which
never reaches `subterm`'s check.

**Nested calls: right call, wrong priority.** Right, because the plan scoped
this task to the grouping-paren recursion and because a fix needs its own oracle
cliff. But that cliff is now measured (M3), it is the shallowest unguarded
recursion in the parser, and it aborts the sized path where the oracle reports a
condition. Deferring it is fine; leaving it filed as an equal-severity sibling
of the prefix-chain gap is not.

## What I did not verify

* The exact integers 350-360 and 1,150-1,200 as *brackets* rather than as
  bisected points — I confirmed each side of both and bisected only
  `nested_calls`.
* `cargo clippy` and `cargo fmt` on the export: not re-run, since the commit
  touches one source file plus an example and a test, and the report's claim is
  cheap for the next `--workspace` run to falsify.
* Whether the oracle's paren bracket moves on a different machine or under a
  different `ulimit`; it is a C++ stack artifact and the commit already says so.
