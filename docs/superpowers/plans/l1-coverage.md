# L1 coverage measurement (Task 0.4, Step 6)

> **PROVISIONAL — NOT the full-suite measurement D8 requires.**
> There is no network in this environment (DNS is down), so the SVN checkout
> of `code-0/test/trunk` specified in the plan could not be done. The numbers
> below are measured against four files available locally, not the ooTest
> suite. **Do not use this to make the D8 call.** Re-run the real command
> (given at the bottom) once the network is back, and replace this file.

## What was measured

Four `.testGroup`/`.testUnit`-shaped files reachable without the network:

| File | Source |
|---|---|
| `extensions/json/json_02.testGroup` | in-repo |
| `extensions/json/json_01_Claude.testGroup` | in-repo |
| `extensions/yaml/yaml.testGroup` | in-repo |
| `framework.tests/Assert.testUnit` | salvaged partial SVN checkout of the ooTest `framework/` tree, at `ootest-fw/` in this session's scratchpad |

`Assert.testUnit` belongs to the older **ooRexxUnit** framework (`OOREXXUNIT.CLS`,
`docs-ooRexxUnit.txt` sit next to it), not **ooTest** (`::requires 'ooTest.frm'`,
used by the json/yaml files) — a different, earlier test framework with the
same `::class ... ::method "test_xxx"` shape. `rexx-extract`'s suite walk only
matches `*.testGroup`, per the plan's Step 5 spec, so a *copy* of
`Assert.testUnit` was renamed to `Assert.testGroup` to be picked up by this
one measurement run. The shipped binary was not changed to accept `.testUnit`.
**If the real suite under `code-0/test/trunk` contains other `.testUnit`
files, the literal Step 6 command will silently skip them.**

Command actually run:

```bash
cd rust && cargo build --release -p rexx-extract
./target/release/rexx-extract \
  --suite <scratchpad>/l1-sample --out <scratchpad>/l1-extracted \
  --report <scratchpad>/l1-coverage-sample.md
```

where `<scratchpad>/l1-sample/` held the four files above (three copied
in-repo files, one renamed copy of `Assert.testUnit`).

## Raw result (literal spec, as implemented)

| File | Total | Extractable | Percentage |
|---|---|---|---|
| `Assert.testGroup` (was `Assert.testUnit`) | 48 | 46 | 95.8% |
| `json_01_Claude.testGroup` | 227 | 183 | 80.6% |
| `json_02.testGroup` | 227 | 183 | 80.6% |
| `yaml.testGroup` | 50 | 50 | 100.0% |
| **Total** | **552** | **462** | **83.7%** |

**This raw number is misleadingly high — see the correction below before
treating 83.7% as informative.**

## Why the raw number overstates viability

`touches_fixture` (as specified in the plan) only flags a method as
fixture-dependent when it sends a non-assertion `self~<message>`. Real ooTest
methods overwhelmingly access fixture state a different way: `setUp` builds
an object and stores it via `expose <var>` into an *exposed instance
variable*, and every test method does `expose <var>` then calls `<var>~...`
directly — never through `self~`. `touches_fixture` never sees a `self~`
message in these bodies at all, so it reports them as fixture-free, and the
binary marks them extractable and wraps them in `::routine main public` —
where `expose` is not even legal (it's a method-only instruction), so the
emitted `.rex` would fail to parse, not just fail an assertion.

Counting how many of the 462 "extractable" files actually contain `expose`
in their extracted body:

| File | Extractable | Contains `expose` |
|---|---|---|
| `Assert.testGroup` | 46 | 0 |
| `json_01_Claude.testGroup` | 183 | 151 |
| `json_02.testGroup` | 183 | 147 |
| `yaml.testGroup` | 50 | 42 |
| **Total** | **462** | **340** |

Treating every `expose`-using method as fixture-dependent (a corrected
heuristic, **not implemented in the shipped binary** — this is a manual
illustrative recount, done outside `rexx-extract`):

- Adjusted extractable: 462 − 340 = **122**
- Adjusted percentage: 122 / 552 = **22.1%**

That is a ~4x drop from the raw 83.7%, and it falls below the plan's 40%
D8 threshold on this sample. This sample is four files, not the suite, so
this is not a substitute for the real measurement — but it is reason to
distrust the raw 83.7% figure specifically, independent of sample size.

## Other things this sample exposed in the spec

- **No comment-awareness.** `extract()` is a line-by-line scanner with no
  concept of Rexx's `/* ... */` block comments. `Assert.testUnit` has 20
  `::method "test_fail_..."` definitions inside one large commented-out block
  (lines 352–426 of the source); all 20 were counted as real methods and
  extracted into `.rex` files that don't correspond to any live test. This
  inflates both the numerator and denominator whenever a source file has
  commented-out methods.
- **`ASSERTIONS` omits `expectCondition`.** `Assert.testUnit` has
  `self~expectCondition("NOVALUE")` alongside `self~expectSyntax(13.1)`; only
  the latter is in the plan's `ASSERTIONS` list. `test_expectCondition` is
  therefore (correctly, given the list) marked fixture-dependent — but this
  looks like an omission in the list rather than an intentional exclusion,
  since it's the same kind of "expect a condition to be raised" assertion as
  `expectSyntax`.
- **Shim completeness was underspecified.** Step 5's example Rexx snippet
  shows only `::method assertEquals`, but its surrounding prose says the shim
  "must define exactly the assertion messages listed in `ASSERTIONS`" (11
  names). The snippet was read as illustrative rather than exhaustive, and
  the shipped binary emits all 11 shim methods. Worth confirming that reading
  was intended.

## Re-running the real measurement

Once the network is back:

```bash
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
svn checkout --non-interactive --trust-server-cert \
  https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest
cd rust && cargo run --release -p rexx-extract --bin rexx-extract -- \
  --suite ../ootest --out ../rust/corpus/extracted --report ../docs/superpowers/plans/l1-coverage.md
```

This overwrites this file with the real, full-suite numbers. Given the
`expose` finding above, it would be worth also re-running the manual
`expose`-in-body recount (or, better, fixing `touches_fixture` to detect
`expose <var>` and re-measuring with the corrected heuristic) before the D8
call is made — the raw number this tool produces is not reliable on its own.
