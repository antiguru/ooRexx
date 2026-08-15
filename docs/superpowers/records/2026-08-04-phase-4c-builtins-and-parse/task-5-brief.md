### Task 5: `builtin/convert.rs`

**Files:** create `crates/rexx-exec/src/builtin/convert.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 12 names:** `B2X BITAND BITOR BITXOR C2D C2X D2C D2X X2B X2C X2D XRANGE`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 0: Eight measured behaviours the reference does not give you -- read before writing any code**

Surveyed and re-verified 2026-08-05. Each is a **silent wrong answer** if missed, and the obvious implementation is wrong for most of them.

**(a) `NUMERIC DIGITS` bounds the RESULT, not the input length.** This is the one most likely to be got wrong in both directions:

```rexx
numeric digits 9;  say c2d(copies('00'x,10)||'01'x)   -- 1,  eleven bytes, fine
numeric digits 9;  say c2d('ffffffff'x)               -- 93.936, four bytes
numeric digits 1;  say c2d('ff'x,1)                   -- -1, fits
numeric digits 1;  say c2d('7f'x,1)                   -- 93.936, 127 does not
```

Eleven bytes succeed and four fail under the same setting.
`C2D`/`X2D` are sensitive on **output** (93.936 / 93.935); `D2X`/`D2C` on **input** (93.928 / 93.929), where the *argument* must be a valid whole number under the current setting before conversion starts.
For all-`FF` input the bound is `floor(DIGITS / log10(256))` -- so **`DIGITS 1` admits zero bytes**, since one byte is already 255.
Both message texts name the setting.

**(b) The length argument is a right-aligned window that truncates from the LEFT, silently.**
`c2d('01020304'x,2)` is **772** (`0x0304`), `d2x(4096,2)` is `00`, `x2d('80',1)` is `0`. **No error.** An implementation that raises one breaks all of them.

**(c) The length argument also switches the read to SIGNED, and the window sets the sign bit.**

| | no length | `,1` | `,2` |
|---|---|---|---|
| `c2d('80'x)` | 128 | **-128** | 128 |
| `x2d('80')` | 128 | **0** | **-128** |

Same bytes, three answers, and **`C2D` and `X2D` disagree with each other**.
`d2x(-1)` and `d2c(-1)` without a length are 93.927, "Length must be specified to convert a negative value."

**(d) `BITAND`/`BITOR`/`BITXOR` with unequal lengths and no pad pass the longer string's tail through UNCHANGED.**
`c2x(bitand('ffff'x,'00'x))` is **`00FF`** -- the tail survives. Supply a pad and it is combined: `c2x(bitand('ffff'x,'00'x,'00'x))` is `0000`. One argument is legal: `bitand('ffff'x)` is `FFFF`.
**There is no default pad; there is a passthrough.** Defaulting to `'00'x` for `BITAND` is the obvious implementation and it is wrong.

**(e) Hex and binary string whitespace is `{0x20, 0x09}` -- blank and tab only**, the same set as the word separators. `x2c('41'||'09'x||'42')` is `4142`; LF is 93.933. Leading or trailing whitespace is 93.931.

**(f) Grouping: the FIRST group sets the residue, every LATER group must be an exact multiple** (2 for hex, 4 for binary), and the first group is left-padded rather than rejected.
`x2c('414')` is `0414`; `x2c('4 1424')` is `041424`; `x2c('414 2434')` is `04142434`; `x2c('414 243')` is 93.976.
*This rule is inferred from eight cases, not read from `validateGroupedSet` -- confirm it against the C++ before relying on it.*

**(g) `X2C` and `X2B` disagree on odd input.** `x2c('414')` pads to a whole byte (`0414`); `x2b('414')` gives 12 bits, unpadded. `b2x` pads to a multiple of 4 bits.

**(h) `XRANGE` is variadic over PAIRS and a class name consumes one slot.**
`xrange('a','b','c','d')` is `abcd` -- two ranges concatenated. `xrange('digit','z')` is **not** digits-through-`z`: `'z'` starts a *new* range running to `0xFF`. `length(xrange())` is 256.
The 12 POSIX class names are case-insensitive (`BuiltinFunctions.cpp:1639-1648`), and **`cntrl` contains a leading NUL** -- `length(xrange('cntrl'))` is 33 and it begins `00010203`, so anything using `strlen` truncates it to nothing.
Argument asymmetry: argument 1 takes a class name **or** a single character (40.28); argument 2 takes a single character **only** (40.23).

**Validation order: every `40.x` argument-conversion check precedes every `93.9xx` content check, on all arguments.**
`d2c('abc','def')` is **40.12**, not 93.929 -- the *length*'s type error beats the *value*'s. `x2d('ZZ','zz')` is 40.12 while `x2d('ZZ',4)` is 93.933.
`40.x` is rc **216**; `93.9xx` is rc **163**.

**What the suite checks, so you know what it cannot catch.** 212 `expectSyntax` calls over 17 distinct numbers across the twelve groups, against 899 `assertSame`.
Gaps, each established with a positive control: **`93.977` (binary grouping) is raised by the implementation and tested nowhere** in `ootest/base`, though its hex twin 93.976 is tested in four places; `BITAND`/`BITOR` test exactly one error number each and nothing asserts their 40.4 at four arguments; `C2X` tests only 40.4.

**Two behaviours are asserted outside these groups**: `class/RexxInteger.testGroup:276-359` requires `d2x`/`c2d`/`x2d` to return a **RexxInteger** and `RexxInteger~d2x` to equal `NumberString~d2x`; `expressions/Literals.testGroup:162` ties `.String~xdigit~x2c` to the character classes.

**A probe warning from this survey, arriving from an unexpected direction.** `bitor ('0000'x,...)` **with a space** parses as concatenation with the uninitialised symbol `BITOR`, and produces plausible-looking output (`4249544F52…` is `"BITOR "`). The literal-syntax trap is not confined to `x` and `b`; any builtin name followed by a space and a parenthesis is a symbol, not a call.

- [ ] **Step 1: Build the probe table, with two hazards specific to this family**

* **Rexx literal syntax will silently change your probe.** A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal.
  `say x2d('ff')` is fine; `y = 'ff'; say x2d(y)` is fine; `say x'41'` is a *literal*, not a call.
  This family is where that bites.
* **`NUMERIC DIGITS` interacts with `C2D`, `D2C`, `D2X` and `X2D`.** Probe each at a non-default `DIGITS`, and **never above 1000**.

Also: the bit builtins take a pad character; `XRANGE`'s arguments are single characters, so 40.23 is its live error.

**These are the six groups whose `.testGroup` files contain non-UTF-8 bytes** (`C2X`, `COPIES`, `D2C`, `DATATYPE`, `DELSTR`, `INSERT`), so use `/bin/grep -a` when searching them and expect high bytes in the expected values.

- [ ] **Step 2: Read each `<NAME>.testGroup`**

- [ ] **Step 3: Write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 12 read `implemented` and none `divergent`**

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
