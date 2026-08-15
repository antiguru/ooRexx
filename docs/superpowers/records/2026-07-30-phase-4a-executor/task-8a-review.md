STATUS: DONE

# Task 8a review: the comparison entry points

Reviewing `5bf9b03d` and `4a320f1c` in `rexx-num`.

## Verdicts

**Spec compliance: PASS.** The funnel is total, the retype is provably pure,
the borrow is real, and the non-UTF-8 case is exercised end to end.

**Code quality: PASS.** The three tests the dispatch singled out do
discriminate, and one of them visibly caught and fixed its own
non-discriminating assertion before shipping.

**0 Critical, 0 Important, 1 Minor.** `rexx-num` 147 passed / 0 failed.

The addendum's central claim is **confirmed**; its arithmetic is **wrong**, and
the wrong number propagated into the dispatch. Detail below.

---

## 1. One implementation, not three: confirmed, and total rather than nearly

`numeric_order` and `string_order` have **exactly one call site each**, both
inside `compare_decoded`:

```
compare.rs:183:        (Some(na), Some(nb)) => numeric_order(na, nb, digits, fuzz)?,
compare.rs:188:        _ => string_order(a, b),
```

and both other public entry points are one-liners into it:

```rust
pub fn compare(...)       { compare_decoded(a.as_bytes(), None, b.as_bytes(), None, ...) }
pub fn compare_bytes(...) { compare_decoded(a, None, b, None, ...) }
```

So there is no second copy of the string rule and no path that bypasses the
funnel. This was the task's whole point and it is met exactly.

## 2. The `&str` -> `&[u8]` retype is pure, and the diff proves it better than the tests do

The report offers ten unmodified passing tests as evidence. That is weaker
evidence than it looks — passing tests show behaviour did not change *on the
cases tested*. The diff shows something stronger:

```diff
-fn string_order(a: &str, b: &str) -> Ordering {
+fn string_order(a: &[u8], b: &[u8]) -> Ordering {
-    let a = skip_leading_blanks(a.as_bytes());
-    let b = skip_leading_blanks(b.as_bytes());
+    let a = skip_leading_blanks(a);
+    let b = skip_leading_blanks(b);
```

The body already operated on bytes throughout; the only change is that the
`.as_bytes()` conversion moved from inside the function to outside it. There is
no reachable behaviour difference, on tested cases or untested ones. The strict
path changed the same way, `op.holds(a.as_bytes().cmp(b.as_bytes()))` becoming
`op.holds(a.cmp(b))`, and `[u8]::cmp` is the same lexicographic byte order the
`&str` comparison already reduced to.

Worth stating because "it still compiles and the tests pass" would not have
been enough on its own, and here something better is available.

## 3. The borrow is real, and both defeat-the-mechanism tests discriminate

`compare_decoded(a: &[u8], a_number: Option<&Number>, ...)` borrows. When
`None`, it parses into a local and takes `.as_ref()`; there is no clone
anywhere on the path, so a caller holding a parsed `Number` genuinely keeps it.

**`compare_decoded_uses_the_supplied_number_not_a_fresh_parse_of_the_bytes`
discriminates, and the interesting part is that it caught its own weak
assertion.** It supplies `Some(1)` for bytes `b"999"` and asserts equality with
`b"1"` — which fails unless the override is used. Then on the right-hand side it
notes in its own message that `1 < 5` and `1 < 999` are *both* true, so that
assertion alone would not pin anything, and adds `6 < 5` -> false where
`6 < 999` would be true. That second assertion is the one that discriminates,
and the author found the gap rather than shipping a test that could not fail.

**`a_supplied_number_does_not_affect_strict_comparison` discriminates too.**
`b"007"` with `Some(Number(7))` against `b"7"`: numerically equal, strictly not,
and cross-checked against `compare_bytes(b"007", b"7", ..., StrictEqual)` to pin
that the override changes nothing on the strict path. If the supplied `Number`
leaked into the strict comparison, "both sides are numerically 7" would wrongly
make it true.

## 4. The non-UTF-8 case is exercised end to end

`a_non_utf8_operand_compares_correctly_where_str_could_not_express_it` uses a
lone `0xC3` — a two-byte lead with no continuation, which `&str` cannot hold at
all, so this is the case that makes the new entry point necessary rather than
merely tidier. It covers four distinct things: identical and different
non-UTF-8 bytes under `Equal`, `StrictEqual`, a leading blank stripped in front
of a non-UTF-8 byte (the rule inspects bytes, never decoded characters), and a
numeric operator falling back to the string rule rather than erroring.

That last one is the semantically important one: `parse_bytes` is
`from_utf8(bytes).ok().and_then(Number::parse)`, so invalid UTF-8 yields `None`
and joins the "not a number" path, which is D15's collapse of both parse
failures into one outcome.

## 5. The pow claim: substance confirmed, count wrong

**Confirmed, and Task 7 is not implementing anything twice.**
`pub fn pow(&self, exponent: &Number, digits: u64) -> Result<Number, ArithError>`
takes two already-parsed `Number`s, so a non-numeric operand cannot be
represented as an argument. No call this crate can make reaches that routing,
and therefore no test it can write. The 41.1-versus-26.8 split is decided by
whichever caller parses the operands and picks the error — `rexx-exec`'s.

All six transcripts re-verified against the oracle, wrapped:

```
2**'x'    rc 230   Error 26.8:  Operand to the right of the power operator (**)
                                must be a whole number; found "x".
2**2.5    rc 230   Error 26.8
'y'**2    rc 215   Error 41.1:  Nonnumeric value ("y") used in arithmetic operation.
'y'**'x'  rc 215   Error 41.1                       (base checked first)
2**-1     rc 0     0.5
0**0      rc 0     1
```

`2**'x'` giving **26.8 rather than 41.1** is the counter-intuitive linchpin and
it reproduces: a non-numeric *exponent* is a whole-number complaint, a
non-numeric *base* is a nonnumeric-value complaint. That asymmetry is exactly
what the caller has to route and what `pow` cannot see.

### m1 (Minor). It is three rows and three, not two and four

The report's heading says "two of six rows are `rexx-num`'s, four structurally
cannot be", and the test is named
`the_two_pow_asymmetry_rows_that_are_actually_this_crates_to_own`. The
dispatch repeated "four of the six". Counting the rows:

| row | reaches `pow`? | owner |
|---|---|---|
| `2**'x'` | no, exponent unparseable | caller |
| `'y'**2` | no, base unparseable | caller |
| `'y'**'x'` | no, both unparseable | caller |
| `2**2.5` | yes | `rexx-num` |
| `2**-1` | yes | `rexx-num` |
| `0**0` | yes | `rexx-num` |

**Three and three.** The report's own bullet list agrees with the table — it
groups `2**2.5` with `2**-1`/`0**0` as this crate's — so the heading contradicts
the body directly below it.

**No coverage is missing.** The "two" is accurate about *newly pinned* rows:
the new test pins `2**-1 -> 0.5` and `2**2.5 -> (26, 8)`, and `0**0 -> 1` was
already pinned at `tests/pow.rs:25` pre-existing. So the fix is to the wording,
not to the tests: the crate owns three rows and all three are pinned.

Worth correcting anyway because the number is now in three places — the report
heading, the test name, and the dispatch — and "four rows are the caller's"
would have Task 7 looking for a fourth that does not exist.

---

## One observation carried from the report, not a finding against this task

The report flags that `rexx-exec/src/error.rs`'s
`impl From<ArithError> for Raised` reads `let number = error.code();` —
consuming `error`, since `code(self)` takes ownership — and then uses
`error.additional()` afterwards, which will be a use-of-moved-value the moment
`mod error;` is wired in. I confirmed `code` takes `self` by value and
`sub_code`/`additional` take `&self`, so rewriting to call `sub_code()` first
resolves it as a side effect. That file belongs to Task 7 and was correctly left
alone; noting it so it is not met as a fresh compile error at wiring time.

## Method

Read both commits and the report including its addendum. Oracle probes wrapped
as `( ulimit -v 1048576; build/bin/rexx FILE )`. Call sites established by grep
over `rexx-num/src` rather than by reading the entry points alone. The
retype's purity established from the diff rather than from the passing-tests
claim. Nothing in the repository was modified.
