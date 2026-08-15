### Task 6: `builtin/numeric.rs`

**Files:** create `crates/rexx-exec/src/builtin/numeric.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 7 names:** `ABS FORMAT MAX MIN RANDOM SIGN TRUNC`.

**Read "Shared facts every builtin task needs", restated in your brief -- in particular D15, which this family is the one most exposed to.**

- [ ] **Step 0: Measured behaviours the reference does not give you -- read before writing any code**

Surveyed and re-verified 2026-08-05. **Three rc values in this family where earlier ones had two:** `40.x` -> 216, `93.x` -> 163, **`41.1` -> 215**.

**(a) `MAX`/`MIN` have ZERO error coverage in the whole suite, and their error depends on argument POSITION.**
Zero `expectSyntax` in `bif/MAX.testGroup`, `bif/MIN.testGroup`, `class/String/max.testGroup` or `class/String/min.testGroup` -- verified with a positive control, since 93.903/93.904 *are* asserted at `directives/METHOD.testGroup:278` and `keyword/VarRef.testGroup:127,143`. **Your probes are the only evidence for all of it.**

```
max()          -> 40.3
max(5)         -> 5
max('a',1,3)   -> 93.943   MAX method target must be a number; found "a".
max(1,'a',3)   -> 93.904   Method argument 1 must be a number; found "a".
max(1,2,'a')   -> 93.904
max(1,,3)      -> 93.903   Missing argument in method; argument 0 is required.
```

Three things to get right: **argument 1 raises a different number from arguments 2+**; the `93.904` insert is **off by one against Rexx's own numbering** (the bad value in `max(1,'a',3)` is argument *2* and the message says *1*); and `93.903` says **"argument 0 is required"**, literally zero, measured rather than mistranscribed.

*Flagged as inference, not measurement:* the "method target" wording suggests dispatch as `arg1~max(arg2,…)`, which would explain both the split and the off-by-one. The behaviour is measured; **the mechanism is unconfirmed in the C++ -- read it rather than trusting this.**

**(b) `41.1` is never these builtins' own error.** `SIGN.testGroup`'s two `41.1` cases raise *before* `SIGN` is entered:

```
sign(-1E1234567890)   -> 41.1    rc 215   the unary minus is arithmetic, raised first
sign('-1E1234567890') -> 93.943  rc 163   quoted, no arithmetic, reaches SIGN
abs(17+'c')           -> 41.1    rc 215   the addition raises; nothing to do with ABS
```

**Wiring `41.1` into `SIGN` because its test group asserts it is wrong.**

**(c) `FORMAT` rounds half-up away from zero, not banker's.** `format(2.5,,0)` is **3**, `format(3.5,,0)` is 4, `format(-2.5,,0)` is -3, `format(1.245,,2)` is 1.25.
A banker's-rounding implementation gives 2 for the first.

**(d) `FORMAT`'s `before=0` always fails, even for zero.** `format(1,0)` and `format(0,0)` are both **93.942**, "Integer part of "0" is too large for 0 spaces" -- while `format(0)` is `0`.

**(e) `expp=0` suppresses exponential and BEATS `expt=0`, which forces it.**

```
format(12345,,,0)     = 12345          format(12345,,,,0)   = 1.2345E+4
format(12345,,,0,0)   = 12345          format(12345,,,2,0)  = 1.2345E+04
format(12345,,,4,0)   = 1.2345E+0004   format(1e10,,,,20)   = 10000000000
```

`FORMAT` also honours `NUMERIC FORM`: `format(1e10,,,,0)` is `1E+10` under SCIENTIFIC and `10E+9` under ENGINEERING.
All four optional arguments reject negatives with 93.906, and `format(1,,,,)` with every optional explicitly omitted is legal at rc 0.

**(f) `TRUNC` rounds its input to `DIGITS` FIRST, and never raises LOSTDIGITS.**
Measured: `numeric digits 3; trunc(123456,2)` is **123000.00**. `trunc(1e20)` at digits 9 is `100000000000000000000` -- never exponential.
`keyword/LOSTDIGITS.testGroup:388-391` asserts TRUNC does **not** raise LOSTDIGITS, with the reason in a source comment: the arithmetic builtins round their arguments before processing. **Nothing in `TRUNC.testGroup` says either thing.**

**(g) `RANDOM`'s negative first argument is 40.33, not 40.13.**
`random(-1)` is **40.33**, "RANDOM argument 1 ("-1") must be less than or equal to argument 2 ("")" -- and argument 2's insert is the **empty string** because it was omitted. The zero-or-positive argument (40.13) is the **seed**: `random(1,2,-1)`.
Degenerate ranges are legal: `random(5,5)` is 5, `random(0,0)` is 0. `random(5,1)` is 40.33; `random(1.5)` is 40.12.

**(h) Validation order, measured for `TRUNC` and `FORMAT` only: argument-2 TYPE > argument-1 TARGET > argument-2 RANGE.**

```
trunc('AB.CD','V') -> 40.12     format(1,'x')  -> 40.12
trunc('AB.CD',-1)  -> 93.943    format('a',-1) -> 93.943
trunc(1.5,-1)      -> 93.906    format(1,-1)   -> 93.906
```

*Flagged:* measured for those two only. No value-before-length inversion was found like `D2X`/`D2C`'s, but the analogous probe could not be constructed here -- **establish the order per builtin, as Task 5 had to.**

**(i) D15 holds across `FORM` as well as `DIGITS`.** A value created under `DIGITS 9`/`SCIENTIFIC` keeps `1.23456789E+11` after either setting changes; recomputing gives `123456789012` or `123.456789E+9`.
The exponential trigger is `exp >= DIGITS` positive and `exp >= 2*DIGITS+1` negative -- *flagged: a fit to four data points, not read from the source.*

**What the suite checks:** 181 `expectSyntax` calls, 8 distinct numbers, against 876 `assertSame`. `111×93.942 · 34×93.906 · 19×93.943 · 11×40.12 · 2×41.1 · 1×40.33 · 1×40.3 · 1×40.13`. Per group: ABS `40.3 93.943` · FORMAT `93.906 93.942` · **MAX none** · **MIN none** · RANDOM `40.12 40.13 40.33` · SIGN `41.1 93.943` (both of which do not exercise SIGN) · TRUNC `40.12 93.943`.

**Cross-file, and one false lead killed:** `class/RexxInteger.testGroup:301-307` requires `number~trunc` to be a RexxInteger equal to `integer~trunc(0)`, and `:415` requires MIN's result to contain no `E`; neither is in `TRUNC.testGroup`.
**`bif/DATE.testGroup` has 725 word-boundary hits for `FORMAT` and zero `format(` calls** -- they are the English word and the date-option name. **A bare name grep is badly misleading for `FORMAT` and `MIN`; require the `(`.**

- [ ] **Step 1: Build the probe table, including at least one D15 probe per numeric builtin**

Every result here obeys D15: a value's rendering is fixed when the value is created.
**A probe cannot see a D15 violation unless `DIGITS` or `FORM` changes between the value's creation and its rendering**, so each numeric builtin needs at least one probe that changes one of them in between.

`FORMAT` is the largest single builtin in the group -- `FORMAT.testGroup` has **767** `::method` bodies, the biggest in `base/bif` -- so budget for it.
Never set `NUMERIC DIGITS` above 1000.

- [ ] **Step 2: `RANDOM`'s requirement is a stream, not a seed**

`RANDOM.testGroup` seeds once and makes **99 further unseeded calls**, re-seeds, repeats, and requires all 100 to match.
**A generator that re-seeds on every call satisfies "seedable and deterministic" and fails this.**
Pin the stream property in a unit test.
`RANDOM` must **not** appear in any corpus program (D11), and the group's 8 `expectSyntax` cases (six 40.12, one 40.13, one 40.33) pin its argument validation.

- [ ] **Step 3: Read each `<NAME>.testGroup`, write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 7 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

---



---

## Shared facts every builtin task needs

**This block is a controller obligation, not a task's own text, and that distinction is load-bearing.**
Briefs are extracted per heading, so nothing outside a task's own section reaches its implementer.
Tasks 2, 3, 4, 5, 6, 10, 11 and 12 each say "restated in your brief" -- **the controller makes that true by appending this section to the generated brief file before dispatching.**
An earlier revision said each task "restates this block in their own body", which was false: no task body contained it and the extractor could not supply it.
That is the same mechanism that cost this project five decisions recorded where the implementer never read them.

The append is bounded by this heading and the next `## `, and asserts the block still contains `pub(crate) fn dispatch(`, `not_enough_arguments` and `REXX_CORPUS_GATE=1` before writing -- so a reorganisation of this section fails loudly rather than shipping a brief with a silently truncated span.

**The dispatch interface.**

```rust
pub(crate) fn dispatch(
    interp: &mut Interp,
    name: &[u8],            // already upcased by the caller
    args: &[Option<ObjRef>],
) -> Option<Result<ObjRef, Failure>>
```

`None` means "not a builtin name", which is what lets resolution fall through to `::routine`.
`Some(Err(..))` is a raised condition, including the 40.x family.

**Arguments arrive already evaluated.** `resolve_and_run_call` (`src/run.rs`) evaluates the argument expressions into `Vec<Option<Argument>>`, where `Argument` is `lib.rs:1340`'s private two-variant enum.
Pass `Argument::value()`, which yields `ObjRef`; that method exists for this and its own doc names `ARG()` as a caller.
The `Reference` variant's alias data is `USE ARG >`'s business and no builtin takes a variable reference.
**An omitted position stays `None` rather than being closed up** -- the rule 4b established for `call sub 1,,3`.

**The arity rows** live beside the dispatch as `(min, max)` per name, `max` as `Option<usize>` for the variadic ones.

**The 40.x raisers already half exist.** `error.rs` has `not_enough_arguments` (40.3) and `too_many_arguments` (40.4) from 4b's `USE STRICT ARG`.
**Reuse them; do not write a second pair.**
What is new is the *type* family.

**The argument-error families, measured 2026-08-05 by Task 2. An earlier revision of this block got the third row's family wrong.**

| probe | error | rc |
|---|---|---|
| `substr('abc')` | 40.3 `Not enough arguments in invocation of SUBSTR; minimum expected is 2.` | 216 |
| `substr('abc',,2)` | **40.5** `Missing argument in invocation of SUBSTR; argument 2 is required.` | 216 |
| `substr('abc',2,-1)` | **93.923** `Invalid length argument specified; found "-1".` | **163** |
| `substr('abc','x')` | 40.12 `SUBSTR argument 2 must be a whole number; found "x".` | 216 |
| `substr('abc',2,3,'pq')` | 40.23 `SUBSTR argument 4 must be a single character; found "pq".` | 216 |

**A negative where a non-negative is required is not a 40.x error at all** -- it is **93.923**, "Incorrect call to method", at **rc 163** rather than 216.
An earlier revision listed it among the 40.x probes to take, which would have shipped the wrong code and the wrong exit status across all seven family tasks.
**Probe the family, do not assume it from the neighbouring row.**

**Trailing omitted arguments are not arguments, and this changes the argument model.**
Measured: `q(1,,2,,)` gives `arg()` = **3**, so `length('abc',)` prints `3` while `length(,)` is 40.3.
Only **interior** omissions reach `dispatch` as `None`; trailing ones are dropped before it sees them.

**`check_arity` is a count check, not a shape check, and your builtin must check its own positions.**
`(min, max)` cannot express which positions are required, because required-ness is **conditional on what comes after**.
Measured: `date()` and `date('S')` both succeed, so `DATE`'s minimum is 0 -- yet `date('S',,'S')` is **40.5**, "argument 2 is required", because supplying position 3 makes position 2 mandatory.
The shared machinery will not raise this for you.
**Probe each of your builtins with an interior omission before every optional position**, and raise 40.5 where the oracle does.

**Rexx strings are byte strings, and every probe alphabet must say so.**
Measured at Task 3, and it cost two Critical findings: the error raisers rendered a value through UTF-8, so a byte `>= 0x80` in `found "..."` became U+FFFD where the oracle emits the raw byte, and control bytes stayed raw where the oracle emits `?`.
**A 62,144-program differential sweep reported zero mismatches and could not have found it**: its operand corpus was seven printable-ASCII strings, with zero hex literals and zero bytes above `0x7F`.
Nine committed ooTest cases already reached the defect.

So: **every probe set in this phase includes a byte `>= 0x80`, a control byte, and the null string**, and any sweep reports **the alphabet it drew from** beside the case count.
A count without its alphabet is not a coverage claim -- the same shape as a count without its scan.
This binds Task 5 hardest, since `B2X`, `C2X`, `X2C` and `D2C` are *about* bytes above `0x7F`.

**Never render a Rexx value through `String::from_utf8_lossy` on a path whose bytes are compared.**
It is silent, it is lossy in exactly one direction, and it looks correct in every ASCII test.

**Cross the axes; widening one is not enough.**
Measured at Task 3, and it cost a **silent wrong answer** that two separate corpora both reported clean.
`verify('abcde','','00'x)` is `1` on the oracle and was `0` here, because two C++ branch tests ask *opposite* questions -- an empty reference asks `VERIFY_MATCH`, a non-empty one `VERIFY_NOMATCH`.
Corpus A had **8** empty-reference `verify` programs and no `0x00` option; corpus C had **384** `0x00` options and no empty reference.
**Neither axis was missing. Each corpus varied one and held the other at a safe value**, so the defect at their intersection was invisible to both while their case counts summed to something that looked like coverage.

So a family task's corpus must **cross every argument position's alphabet with every option value**, not vary one position at a time.
And prove the crossing earns its place the way Task 3 did: **run the new corpus against the build that had the bug.**
Its 72 mismatches, against 0 from the two older corpora on that same build, is what turned "this corpus can fail" into "this corpus catches something the others could not".

**Enumerate a builtin's branches from the C++, not from its documentation or from probing.**
Task 3 found 14 empty-argument branches across the string builtins that way.
A branch you did not know exists is one your probes cannot be varied against.

**The ooTest suite lives in *this* repo, not under the oracle.**
`/home/moritz/dev/repos/ooRexx/ootest` **does not exist**; the C++ tree carries no suite at all beyond three stray `.testGroup` files under `extensions/`.
The path is `ootest/ooRexx/base/<group>/` relative to the repository root.
Worth stating because "the oracle is at `/home/moritz/dev/repos/ooRexx`" and "read the ooTest group" sit next to each other in every brief, and an absolute path built from the two is wrong.

**A builtin's test group is not the only place its behaviour is asserted.**
Measured at Task 4: `DELWORD`'s whitespace rule -- the deleted word takes the run *after* it while the run *before* it survives byte for byte, tab-vs-blank identity included -- is asserted in `base/source.file/whiteSpace.testGroup`, **not** in `DELWORD.testGroup`.
So `/bin/grep -a` the whole of `ootest/ooRexx/base/` for your builtin's name, not just its own file.

**And the bytes those cases test are not in the source, which matters to Task 15's extractor.**
Measured: `whiteSpace.testGroup` contains **zero 0x09 bytes**.
`TAB` is a Rexx *variable* -- `TAB = "09"x` at `:63`, `PLANK = " "` at `:66`, `TAB2 = TAB||TAB` -- so the tab exists only in the data at run time.
A scan for whitespace **literals** finds nothing there and silently concludes the tab-separator corpus does not exist.

For a probe author: build separator probes from `"09"x` and the other byte values directly, and never read "no literal tabs in the suite" as evidence about the oracle's separator set.
**For the `base/bif` extractor: whole-body extraction over a file like this needs the variable assignments resolved, not just the assertion lines matched** -- the same class of modelling requirement as `base/expressions` needing the `NUMERIC DIGITS` setting carried forward, and it must be handled or the affected bodies dropped explicitly rather than extracted wrongly.

**Argument *type* and argument *range* are validated in different layers and raise different errors.**
Measured: `word('a b c',1.5)` is **40.12 at rc 216** from the BIF wrapper's integer conversion, while `word('a b c',0)` and `word('a b c',-12)` are **93.924 at rc 163** from the String method's `positionArgument`.
Different number, different exit code, same argument.
**Probe a bad *type* and a bad *range* for every numeric position you take** -- a probe set testing only one kind cannot see the other.

**Validation order is observable, and the C++ order is often structural rather than intended.**
Measured: `subword('SUBWORD','30'x,'30'x)` -- position 0 *and* length 0 -- raises 93.924 rather than returning `''`, because `positionArgument` is called before the `count == 0` test.
An implementation that early-returns on a zero length gets it wrong at rc 0.
Read the function body for the order; it is documented nowhere else.

**Measure whether a 40.12 or 40.23 message substitutes the rendered value or the source spelling.**
The neighbouring 88.928 raiser in `error.rs` documents having measured exactly this distinction, and it is invisible until an argument's two forms differ -- `'007'` against `7`, or a number whose `DIGITS` rendering is not its literal text.
Task 2 did not record which it is, and the first family task that raises a typed error owes the measurement.

**A quoted literal target reaches the builtin table, and it is case-sensitive.**
Measured: `"LENGTH"('abc')` is `3`; `"length"('abc')` is **43.1 rc 213**, `Could not find routine "length"`.
So the caller upcases a *symbol* target and does **not** upcase a quoted one -- `dispatch` receives the name already upcased only on the symbol path.

**Allocation.** Every builtin returning a string allocates, and every such site goes through `Interp::alloc_with`, never `Heap::alloc_with_uncollected` or `Heap::alloc`.
**A builtin's result must be rooted before any subsequent allocation.**

**D15.** A value's rendering is fixed when the value is created.
A builtin producing a number captures the `DIGITS`/`FORM` pair in force at creation; formatting it later with `settings.digits()` is wrong.
**A probe cannot see this unless `DIGITS` or `FORM` changes between creation and rendering** -- construct at least one probe per numeric builtin that does.

**Probe safety, restated because these are the two that bite this work:**

* **Run every probe from a fresh empty subdirectory of the scratchpad, with absolute paths.**
  The scratchpad root is on the oracle's external-routine search path.
  A probe of a not-yet-implemented builtin name reaches exactly that search, and a stale `.rex` file will be found and run.
  Measured: 44.1 rc 212 from the root against 43.1 rc 213 from a clean directory -- different error, different rc, different meaning.
* **Wrap every oracle call** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx ABSOLUTE_PATH )`.

**The verify block every task ends with**, and no task may substitute a bare "run the tests":

```bash
cd rust
cargo test --offline --workspace --no-fail-fast
cargo fmt --all --check
cargo clippy --offline --workspace --all-targets -- -D warnings
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus
```

Read each exit status **unpiped**.
`REXX_CORPUS_GATE=1` matters: without it `corpus.rs` reports mismatches and still passes (`!gate || mismatches.is_empty()`), so a builtin that diverges byte-for-byte from the oracle leaves `cargo test --workspace` green.

---
