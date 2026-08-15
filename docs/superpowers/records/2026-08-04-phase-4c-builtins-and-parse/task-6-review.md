# Task 6 review: `builtin/numeric.rs`

**Spec compliance: FAIL.** **Quality: CHANGES-REQUESTED.**

Two wrong builtin semantics, both reachable from ordinary one-line programs, both invisible to
this task's own 92,880-program sweep because of how its axes were bounded. Everything else in the
task checks out, and the parts that were hardest -- the `MAX`/`MIN` two-path split, the
`93.942` substitution, the C++ citations, the mutation proofs -- verify exactly as reported.

---

## Re-run verification, each status unpiped

```text
cargo test --offline --workspace --no-fail-fast          exit 0    1114 passed, 0 failed
cargo fmt --all --check                                  exit 0
cargo clippy --offline --workspace --all-targets -- -D warnings   exit 0
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus  exit 0
                                                         mode: STRICT (the gate), 42 of 42 matching
```

The report's numbers reproduce. 1,114 = 1,096 + 18, and the 18 is confirmed independently below
(the workspace under `--skip builtin::numeric` runs exactly 1,096).

`clippy` reported `Finished in 0.06s` from a warm target directory, which `rust/CLAUDE.md`
says to treat as provisional. It was not re-run from a clean target; that is a phase-boundary
obligation rather than this task's, and it is listed under "cannot verify" below.

---

## Findings

### 1. Critical -- `FORMAT` panics the interpreter for `expp >= 65536`

`expp` is the one width in this task that does **not** go through `builtin::buffer`. `format`
(`crates/rexx-exec/src/builtin/numeric.rs:271-273`) refuses it only past `u32::MAX`; everything
below that is handed to `rexx-num`, where `format.rs:306` builds the exponent field with
`format!("{:0width$}", ..)`. Rust's format width is a `u16`, so any `expp` at or above 65,536
that reaches a **displayed non-zero exponent** panics.

Measured, both sides under the standard wrapper and the same `ulimit -v 1048576`:

```text
say length(format(1e10,,,65536))     oracle  65539       rc 0
                                     here    <panic>     rc 101
    thread 'rexx-interp' panicked at crates/rexx-num/src/format.rs:306:25:
    Formatting argument out of range

say format(12345,,,65536,0)          oracle  1.2345E+000...0   rc 0
                                     here    <panic>           rc 101

say length(format(1,,,3000000000,0)) oracle  Error 5 System resources exhausted.  rc 251
                                     here    memory allocation of 3000000002 bytes failed  rc 134
```

The adjacent successes pin the boundary and the cause. `format(12345,,,65535,0)` matches byte for
byte. `format(1,,,70000,0)` matches, because the `exp == 0` path takes a different branch
(`format.rs:299`). And `before`/`after` -- which *do* go through `padding_width`/`buffer` -- are
correct at the same magnitudes: `format(1,65536)`, `format(1,,65536)`, `format(1,3000000000)` and
`trunc(1,3000000000)` are 4 of 4 matching. So this is specifically the unguarded argument.

The first shape needs no contrivance at all: `format(1e10,,,65536)` supplies only `expp`, at
default `DIGITS`, and gets a Rust panic message on stderr with rc 101 -- not a Rexx condition, not
Error 5, and outside the 157..253 band `phase-4-exclusions.txt` requires a non-implemented failure
to sit outside so it cannot be mistaken for one.

`format.rs:299`/`:306` are pre-existing `rexx-num` lines and are not in the diff. They were
unreachable from a Rexx program until this commit implemented `FORMAT`, and this commit's own
comment reasons explicitly about `expp` and chooses the wrong bound, so the defect is this task's.

The sweep could not see it: the `FORMAT` grid's `expp` axis is five small values. My own
1,400-program `FORMAT` cross-product missed it too for the same reason, until the boundary was
probed directly.

### 2. Critical -- `RANDOM`'s result is re-rendered under the current `NUMERIC DIGITS`

`RexxActivation::random` ends `return new_integer(minimum)` -- a `RexxInteger`, whose
`stringValue` is the plain digits and is never reshaped by `DIGITS` or `FORM`. `random`
(`numeric.rs:618-619`) instead builds `interp.number(answer, saturate(digits), form)`, so the
result captures the pair and renders exponentially whenever it is wider than `DIGITS`.

Deterministic, with no randomness involved -- a degenerate range is legal and fixed:

```text
numeric digits 3; say random(12345,12345)        oracle 12345      here 1.23E+4
numeric digits 2; say random(12345,12345)        oracle 12345      here 1.2E+4
numeric digits 3; say random(1000,1000)          oracle 1000       here 1.00E+3
numeric digits 3; numeric form engineering; say random(12345,12345)
                                                 oracle 12345      here 12.3E+3
numeric digits 3; say random(1,999999999,12345)  oracle 776163098  here 7.76E+8
numeric digits 1; say random(1,999999999,12345)  oracle 776163098  here 8E+8
```

9 of 10 probes in that set diverge. Three separate mechanisms hid it, which is worth recording
because they are the same shape:

* the sweep excludes `RANDOM` entirely (D11), so the `DIGITS` axis never met it;
* both `RANDOM` unit tests run at `DIGITS 9`, where `MaxRandomRange` (999,999,999) is exactly nine
  digits and no result can trigger exponential form;
* the status harness's probe is `RANDOM	say random(5,5)` -- one digit.

The fix is to answer text, the way `integer_path` already answers the winning object rather than a
recomputed number.

Two comments become false with it. `numeric.rs:47-53` says `ABS`, `SIGN`, `MAX`, `MIN` **and
`RANDOM`** "produce numbers, and those capture the `DIGITS`/`FORM` pair in force at the call
(D15)"; report §4 says `RexxActivation::random` is "reproduced exactly". Neither holds for the
return value.

### 3. Important -- none of the four disclosed divergences has a ledger row

`docs/superpowers/plans/phase-4-exclusions.txt` is the file the plan names as the live record
(plan line 32: "Adding a `KNOWN GAP` row needs no permission; removing one does"), and its
`KNOWN GAPS` section at line 682 states the asymmetry in as many words. D1 (`MAX`/`MIN`'s two
paths), D2 (`expp`), D3 (value-driven result size) and D4 (`ABS` under ENGINEERING) exist only in
`task-6-report.md`. The file was last touched at `41fc5474`, six commits before this one.
`TRANSLATE`'s equivalent gap does have a row (line 1293), so the mechanism is in use.
A per-task report is not the live record: it is not read by the gate, by Task 13, or by the next
implementer.

D4 in particular deserves one on its own merits -- it is an upstream defect this task diagnosed
correctly (below) and did not file, and nothing outside the report says so.

### 4. Important -- D2's disclosure understates what actually happens

`format`'s own comment says `expp` past `u32::MAX` "is this crate's answer and not the oracle's
for **that one shape**", and report §6's D2 describes an Error 5 against a `1` at rc 0. The real
divergence starts at 65,536 and is a panic, not an Error 5 (finding 1). The sentence reads as a
bounded, deliberate trade and is not one.

### 5. Minor -- one C++ paraphrase in the report is not the test it cites

Report §1(iii) says `NumberStringClass.cpp:2029` is "the same test with `exptrigger` in place of
`createdDigits`". The line is
`if (adjustedLength >= exptrigger || (adjustedLength < 0 && std::abs(numberExponent) > exptrigger * 2))`
-- the second disjunct carries an extra `adjustedLength < 0` guard that `:391` does not have. The
conclusion drawn (that `2*DIGITS+1` is read rather than fitted) is correct and `:391` is quoted
verbatim; only the restatement of the second citation is loose. No shipped code depends on it.

### 6. Minor -- `reported_value` is computed on every `format_with` call

`format.rs:276` computes it unconditionally, including on the `before: None` path where it can
never be read. It clones and reframes twice. Guarding it on `before.is_some()` costs nothing and
makes the dead-value placeholder argument to `render_integer_padded` unnecessary on that path.

---

## What was verified and holds

**The nine Step 0 behaviours.** A 129-program differential covering every row of the report's
probe table -- half-up-away-from-zero rounding, `before=0` failing for zero, `expp=0` beating
`expt=0`, `FORMAT` under `NUMERIC FORM`, `TRUNC` rounding to `DIGITS` first with no LOSTDIGITS,
`RANDOM`'s `40.33`/`40.13`/degenerate ranges, `41.1` never being raised here, and the `MAX`/`MIN`
position-dependent errors -- reports **0 mismatches**.

**Both `MAX`/`MIN` implementations and the rule that selects between them.** A 31-program
discriminating set (`1`, `+1`, `-1`, `1+0`, `10/2`, `2*3`, `01`, `1.0`, `1.`, `1e1`, `1.0+0`,
18- and 19-digit literals, `word(..)`, an assigned variable of each kind, and the same call at two
`DIGITS`) reports exactly **4 mismatches, all four of D1's own list**. `integer_object` and
`valid_under` reproduce `Scanner.cpp:1546` and `Numerics::isValid` correctly, including
`unsigned_abs() < 10^min(digits,18)` being equivalent to `<= validMaxWhole[..]`.
`RexxInteger::Min`'s deliberately-moved early return is reproduced in the right position, and
`RexxInteger::Max`'s fallback rescanning the whole list from argument 1 is too.

**Independent sweeps.** 1,400 crossed `FORMAT` programs (20 values x 6 `before` x 4 `after` x
4 `expp` x 3 `expt` x 5 settings, sampled): **0 mismatches**. 6,000 crossed
`ABS`/`SIGN`/`TRUNC`/`MAX`/`MIN` programs (35 values including `'ff'x`, `'00'x`, `'80'x` and the
null string, x 10 settings including `DIGITS` 1/3/5/12/20, both `FORM`s, `FUZZ` 0/3, x 9 `TRUNC`
places and 13 `MAX`/`MIN` tails): **4 mismatches, one D4 and three D1**. A 20-program byte-alphabet
set aimed at the error-message rendering path: **0 mismatches**. This corroborates the shape of the
"18, all disclosed" claim.

**Both classifications of the 18.** D1 is a real value-model gap: `eval.rs` builds `1` and `'1'`
alike and this crate has no `RexxInteger`. D4 is genuinely an upstream defect --
`NumberString::copyIfNecessary` (`NumberStringClass.cpp:3654`) tests
`isScientific() != form` where `form` is `number_form()` and `FORM_SCIENTIFIC` is `false`
(`Numerics.cpp:108`), so a scientific-created number is cloned under SCIENTIFIC and *not* under
ENGINEERING, returning its cached literal text. The comparison wants
`isScientific() != (form == FORM_SCIENTIFIC)`. The report's reading is correct.

**Every C++ citation resolves to what is claimed.** `BuiltinFunctions.cpp:1993` (BUILTIN(MAX), the
three-way dispatch), `StringClass.cpp:1057-1064` (the `ArithmeticMethod` macro and its
`Error_Incorrect_method_string_nonumber`) and `:1098` (`RexxString::Max`),
`NumberStringMath.cpp:240` (`maxMin`, its `arg + 1`, its strict `rc > 0 && compResult > 0`, and
its `argCount == 0` early return), `IntegerClass.cpp:1578` (`RexxInteger::Max`, `requiredArgument(argument, arg)`
unincremented) and `:1625` (`::Min`'s comment naming `RexxInteger.testGroup` -- report B2 is exact),
`MethodArguments.hpp:99`, `Scanner.cpp:1546`, `Numerics.cpp:108`/`:112`, `Numerics.hpp:191`
(`ARGUMENT_DIGITS = 18` on 64-bit), `NumberStringClass.cpp:391`, `:3654`, `:3700`, `:2100-2118`
(`math_round_places`' three branches, sign kept on the round-away and cleared on the collapse),
`RexxActivation.hpp:593/596/601`. `RexxActivation::random`'s five-arm range mapping matches the
Rust `match` arm for arm, and `getRandomSeed`'s negative-seed raise really does precede both the
seed install and the unconditional `RANDOMIZE`, so a rejected seed correctly does not advance the
stream.

**`rexx-num`'s changes are minimal and nothing already depending on it moved.** `Number::abs`,
`Number::signum` and `FormatError::sub_code` are additive. `BeforeOversize`'s fields changed
shape, but no consumer outside `rexx-num`'s own tests reads `additional()` -- `fmt-check.rs` reads
only `code`, and `rexx-exec` had no `FormatError` path before this commit. The success path's bytes
are untouched: `reported` is read only in the raise branch, and the 1,400-program `FORMAT` sweep,
the full suite and the strict corpus gate all agree.

**Two mutations re-verified three-state, including one of the two needing the skip.** Mutated in
place, restored from my own copy, `git status` clean afterwards and the file's md5 back to
`3840d379`:

| mutation | without | with |
|---|---|---|
| `integer_path`: drop the `valid_under` check | `cargo test --workspace -- --skip builtin::numeric` **exit 0, 1096 passed** | named test alone **exit 101, 0 passed; 1 failed** |
| `Extreme::op`: `Min => CompareOp::Greater` | `--skip a_tie_keeps_the_earlier_value_at_both_ends` **exit 0, 1113 passed** | named test alone **exit 101, 0 passed; 1 failed** |

Both reproduce the report's rows. The first is the stronger claim: the pre-existing 1,096-test tree
is green under the mutation, so the new test adds coverage rather than duplicating something.

**`keyword-exempt.txt`.** The header's `4c 766 bodies` equals the file's 766 rows tagged `4c`; the
total is 772 rows (766 + 6 `defect:`). The four rows are removed outright, not re-attributed -- the
diff has no compensating additions -- and the suite is green with them gone.

---

## Cannot verify from the diff

* **The 92,880-program sweep and its 6,000-program negative control.** The generator stayed in the
  scratchpad, so the 18-vs-122 and 155-vs-0 figures are not reproducible here. My own 7,400
  independent programs are consistent with them in shape, and finding 1 shows what the `expp` axis
  could not reach.
* **`clippy` from a clean target directory.** Not re-run; `rust/CLAUDE.md` treats a same-session
  green as provisional.
* **D3's `trunc(1e999999999)` behaviour** was not probed -- a 1 GB result under the documented
  `ulimit`/reservation asymmetry cannot be compared meaningfully, which is what the report says.
* **The eleventh mutation** that the report says was tried and correctly not listed.
