STATUS: DONE

# Review of Task 15a, commit f8ab5386 (assertion-row extractor, rexx-extract)

## Verdicts

**Spec compliance: PASS.** No written brief exists for this task (it was
split out of Task 15 and dispatched directly, per the report's own
opening line); judged against that dispatch, the file-scope discipline
(`rust/crates/rexx-extract/` only, confirmed exactly four files touched),
and the project's own D-decision-grade rigor (oracle verification, an
enforced invariant, a real pre-existing bug found and fixed rather than
worked around). **Code quality: PASS.** 0 blocking, 0 major. One
precision correction worth recording (below) to a claim in the commit
message and in your own dispatch, not a defect in the code: the "all 388"
framing overstates the risk for about a sixth of `CONCATENATION`'s
assertions, though the engineering conclusion it argues for is exactly
right regardless.

## Inputs

- `.superpowers/sdd/2026-07-30-phase-4a-executor/task-15a-report.md` (no
  separate brief file exists for this task).
- `git show f8ab5386` (full diff: `src/lib.rs`,
  `src/bin/rexx-extract-assertions.rs`, `tests/extract.rs`,
  `tests/extract_assertions.rs`).
- The actual `.testGroup` sources this all runs against
  (`ootest/ooRexx/base/expressions/*.testGroup`, gitignored, not part of
  any archive -- read directly from the checkout).
- The pre-existing `rexx-extract` binary and its `render()` function,
  untouched by this commit apart from benefiting from the shared fix.

## Zeroth check: is the old rendering really executing nothing?

Read `render()` (`src/bin/rexx-extract.rs`, pre-existing, untouched):
`"/* extracted from {group} */\n::routine main public\n{body}\n::class
shim public\n{SHIM_METHODS}"`. `::routine main public` is the *first*
thing in the file after a comment, so the program's prolog -- everything
before the first `::` directive -- is empty. Confirmed by building the
exact shape myself with a deliberately wrong assertion (`self~assertSame(1
+ 1, 999)`, which the shim's own `assertSame` would `exit 1` on if it ever
ran) and running it wrapped:

```
rc=0, no output at all
```

Not "fails to compile" or "fails at runtime" -- the process runs clean and
prints nothing, confirming the method body never executes at all. Every
"100.0% extractable" figure the old rendering ever reported was measuring
whether text was accepted as syntax, never whether anything in it ran.
This is what makes rebuilding L1 as data rather than programs the right
call, not a stylistic preference.

## Pressure point 1: sequential `NUMERIC DIGITS`/`FORM`, not read in isolation

Read `scan_method_for_assertions` directly: it iterates `body.lines()`
once, top to bottom, and `digits`/`form` are plain mutable locals
overwritten each time a `numeric digits`/`numeric form` line is reached --
there is no separate pass, no regex scanning the whole method for the
"nearest" setting, nothing that could read a setting out of position. So
"sequential" is what the code structurally does, not merely what the test
names hope it does.

Checked the test that is supposed to prove this
(`digits_changing_mid_method_is_carried_sequentially_not_read_in_
isolation`) discriminates the two designs it claims to, not just
happens to pass: it asserts the two rows come out with **different**
digits (9 then 18) and different expected text, which a
read-the-whole-file-in-isolation implementation (one global "the digits
setting" rather than a per-line-position one) could not produce -- it
would give both rows the same value. Ran it: passes on the shipped code.
`form_changing_mid_method_is_carried_sequentially` is the same shape for
`FORM`, which the brief never named (see Pressure point 3).

Also checked the `blocked_reason.is_none()` guard around every `digits`/
`form` update: once a method hits an unsupported statement, further
`NUMERIC` lines are seen but no longer applied. This cannot matter for
correctness (no more rows will be emitted from that method regardless),
but it is the right conservative choice and not dead code -- it stops the
state from silently drifting in a way a later change to this function
might accidentally start relying on.

## Pressure point 2: the `CONCATENATION` prelude, and a precision correction

Read `CONCATENATION.testGroup`'s `test_1` directly: seven assignments
(`a`..`g`, distinct byte strings with deliberate blank-padding
differences) followed by 388 `assertSame` calls comparing them with `==`,
`=`, and their negations. Confirmed the load-bearing fact independently:
an unset Rexx variable renders as its own upcased name, so `a` through `g`
unset are seven *distinct* one-character strings ("A" through "G"), and
that is enough by itself to make a **subset** of the 388 assertions pass
by coincidence with no prelude at all.

**But not all 388, and I want to correct this precisely rather than wave
it through, because the commit message and your own dispatch both state
it as "all 388" and the actual mechanism is narrower.** Built the
concrete counter-example rather than reasoning about it in the abstract:
of the 388 `assertSame` calls, 56 use strict `==`/`\==` and 332 use
non-strict `=`/`\=`. Ran three representative lines from the actual file
with `a`..`g` deliberately left unset, wrapped against the oracle:

```
line 62 (a==a)(b==a)...(g==a), expects '1 0 0 0 0 0 0': actual matches   -- silently passes
line 76 (a\==a)(b\==a)...,     expects '0 1 1 1 1 1 1': actual matches   -- silently passes
line 71 (a=c)(b=c)...(g=c),    expects '0 0 1 1 0 1 0': actual is '0 0 1 0 0 0 0' -- WRONG, fails loudly
```

The reason: with `a`..`g` all unset, every one of them is a distinct
single-character string, so *any* comparison operator between two of them
reduces to plain diagonal (equal only to itself) -- `==` and `=` cannot be
told apart on unset variables, because there is no blank-padding
difference left for `=`'s looser rule to matter. The *real* assigned
values do have such differences (`c="abcdefg "`, `d=" abcdefg"`, deliberately
padded so that `c=d` is true under non-strict `=` despite being false
under `==`), which is exactly what line 71 is testing. So a non-strict-`=`
row that depends on one of those real cross-variable equalities gets a
visibly wrong answer without the prelude, not a silent pass -- a real
evaluator comparing `'0 0 1 0 0 0 0'` against the expected `'0 0 1 1 0 1
0'` reports a clear failure, for the wrong reason (missing prelude) but
loudly.

**This does not change the fix, which is exactly right regardless:** every
row from this method needs the real prelude to be evaluated
*meaningfully*, whether its failure mode without one would be a silent
false pass or a loud false failure, and the shipped code attaches the
prelude to all 388 unconditionally rather than trying to distinguish the
two cases. It only changes how the risk should be described: "some of the
388 would silently pass" is accurate and already a strong enough reason
on its own; "all 388 would silently pass" is not, and repeating it forward
(as the commit message and the dispatch both do) would let a future reader
believe a stronger, false claim about which specific rows are dangerous in
which way.

## Pressure point 3: `rows + dropped == calls`, enforced or merely computed?

Read both call sites: `rexx-extract-assertions.rs`'s `main()` and
`tests/extract_assertions.rs`'s
`every_assert_same_in_base_expressions_is_a_row_or_an_accounted_for_drop`
both use `assert_eq!`, a real panic/failure on violation, not a printed
comparison a reader could miss. Then checked it is not merely computed but
survives a real regression, rather than accepting the report's narrative
that it once did: reverted the single-quote fix in a scratch copy
(`.trim_matches(['"', '\''])` back to `.trim_matches('"')`) and ran both
enforcement sites against `base/expressions` again.

```
test:   thread panicked: MULTIPLICATION.testGroup: 184 rows + 2 dropped != 1050 assertSame calls
binary: thread 'main' panicked: same assertion, same file, same message
```

Both fire, independently, with the exact file and the exact shortfall
named. This is genuine mutation testing, not a restatement of the
report's claim: the invariant is enforced at both sites it lives at, and
a regression of the bug it originally caught reintroduces a loud, specific
failure rather than a silent undercount.

**All ten dropped rows checked against the actual source, not just the
report's list:**

* `Literals::test_hexadecimal_single` -- two `do ... over` loops, one
  `assertSame` inside each = 2 dropped.
* `Literals::test_hexadecimal_double`/`test_binary_single`/
  `test_binary_nibble`/`test_binary_ones` -- one `assertSame` each inside a
  `do`/`do ... over` loop = 1 dropped apiece, 4 total.
* `MULTIPLICATION::test_bug1339` -- **read the full method, not just the
  loop**: the two `assertSame` calls that get dropped (`435600*435600*...`
  and `65536*65536*...`) are not inside the nested `do m`/`do i` loops at
  all -- they sit textually *after* both `end`s close, and would evaluate
  fine with `digits=100` carried through correctly. They are dropped only
  because the scanner has no notion of a block closing: once `do m = 2 to
  3` sets `blocked_reason`, nothing un-blocks it for the rest of the
  method, so these two get swept up conservatively rather than mis-handled.
  Confirmed this is deliberate over-caution, not a bug: the alternative
  (tracking block nesting to un-block after a matching `end`) is real
  scope this task correctly declined, and dropping two extractable-but-
  unrecognised assertions is the safe direction to err in.
* `SPECIAL::test_37` -- blocked at `h=''; lta=lt; c.lta=f1(3)+f1(4);`,
  which `simple_assignment` correctly refuses (an unquoted `;` joins
  multiple clauses on one line, which the scanner cannot safely fold into
  one prelude entry verbatim), dropping the two `assertSame` calls that
  follow = 2.

6 + 2 + 2 = 10, matching exactly, and every one of the ten is a real,
traceable reason rather than an unexplained residual.

## Verification

Ran everything myself rather than trusting the report's numbers:

- `cargo test -p rexx-extract --no-fail-fast`: exit 0, **14/14** (3
  pre-existing + 11 new), matches exactly.
- `cargo clippy -p rexx-extract --all-targets -- -D warnings`: exit 0.
- `cargo fmt -p rexx-extract -- --check`: exit 0.
- `rexx-extract-assertions --suite ootest/ooRexx/base/expressions`:
  reproduced the full per-group table digit for digit -- 4,269 calls,
  4,259 rows, 10 dropped, 1,226 `PRECEDENCE`, 388 `CONCATENATION`, 1,048
  `MULTIPLICATION`, and the same ten blocked-method lines with the same
  reasons.
- The pre-existing `rexx-extract` binary (old rendering mode, benefiting
  only from the shared `extract()` fix): re-ran it over the same suite,
  **2,888 test methods, 2,847 extractable (98.6%)**, with
  `MULTIPLICATION.testGroup` now at 151/151 (100.0%) -- matches the
  report exactly, and confirms the fix helps the old mode too, not just
  the new one.
- The flagged staleness of `docs/superpowers/plans/l1-coverage.md`: no
  longer true. It currently reads 151/151 for `MULTIPLICATION.testGroup`
  and carries a note explaining the 143-vs-151 discrepancy as a worked
  example -- consistent with your own message saying you re-ran the
  measurement and updated it (14,122/86.2% to 24,581/90.5%). The report's
  flag was correct when written and has since been closed by your own
  follow-up, not by this task.

## Code quality notes

- `parse_assert_same`'s hand-written paren/quote/comma scanner is
  conservative in exactly the right direction: it returns `None` (drop,
  don't guess) on anything outside the one shape it commits to handling,
  rather than attempting a best-effort split that could silently
  misattribute which text is the expression and which is the expected
  value.
- The `blocked` bookkeeping's "no entry when `dropped == 0`" rule
  (confirmed via `a_trailing_return_with_nothing_after_it_reports_no_
  blocked_method`) is a real, deliberate design point and not an obvious
  default: a naive implementation would record every unsupported statement
  as blocked regardless of whether anything downstream was actually lost,
  which would misreport harmless trailing statements (a bare `return` at
  the end of a method, which two groups' `test_198`/`test_293` both end
  on) as if they had cost something.
- `find_test_groups` is correctly factored out as a shared, sorted walk
  rather than duplicating or reusing the pre-existing binary's own
  `walk()` (unsorted) -- a deterministic order matters for the whole-corpus
  tests pinning exact totals, which an unsorted walk would not
  reliably provide across filesystems.

## Scratch cleanup

Two scratch extractions (`git archive f8ab5386`), one mutated to revert
the quote fix for the enforcement check above. `ootest/` is gitignored
and not included in a `git archive`, unlike `interpreter/` -- needed one
new symlink for it, created once against a path that did not already
exist, confirmed with `git status` before and after that the real
repository's `ootest/` was never touched. No commits, branches or
worktrees against the real repository.
