### Task 12: `builtin/datetime.rs`

**Files:** create `crates/rexx-exec/src/builtin/datetime.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 2 names:** `DATE TIME`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 0: The clock is cached per clause -- measured 2026-08-05**

**Two `time('L')` calls in ONE clause return identical values across a real CPU burn; in two clauses they differ.**

```
say time("L") burn() time("L")   ->  14:47:06.798648 | 14:47:06.798648
n1 = time("L"); zz = burn(); n2 = time("L")  ->  14:47:06.866899 | 14:47:06.918360
```

Same burn, same resolution; only the clause boundary changes the answer. `DATE('T')` is cached the same way.
**The probe must put both reads inside one clause** -- a version with the burn between two statements cannot distinguish caching from a live read, and my first attempt at this probe made exactly that mistake.

**`TIME('R')` semantics, measured across real burns:** the **first** `TIME('R')` in a program returns **0**; a later one returns **elapsed since the last reset**, not since program start, and resets the clock.

```
first time('R')            0
time('E') after ~0.11s     0.109014
time('R') after another    0.219483   = the sum of BOTH burns
time('E') immediately      0.000023   so that R did reset
```

The `0.219 ≈ 0.109 + 0.110` arithmetic is what pins "since last reset"; since-start would have read ~0.33 by the third sample.

**`TIME('C')` is `2:43pm`** -- lowercase meridiem, no leading zero. `TIME('Z')` and `TIME('')` are 40.904.

**Both option sets come from the error insert, not the documentation, and `TIME`'s is the one this brief otherwise leaves unlisted.** Measured 2026-08-07, both rc 216:

```
say time('Z')   Error 40.904:  TIME argument 1 must be one of CEFHLMNORST; found "Z".
say date('J')   Error 40.904:  DATE argument 1 must be one of BDEFILMNOSTUW; found "J".
```

`TIME`'s eleven are `C E F H L M N O R S T`; this brief names only `C`, `E`, `L` and `R`, so `F H M N O S T` are unprobed and Step 1's "probe every one" applies to both names, not just `DATE`.
`DATE`'s thirteen `B D E F I L M N O S T U W` are exactly the two lists below with nothing left over, which is what makes those lists complete rather than merely long.

**The empty option renders differently here than in Task 11's builtin, and an implementer normalising the two would be wrong.** `time('')` and `date('')` report `found ""`, where `datatype(1,'')`'s 93.915 reports `found "?"`. Two error paths, two spellings for the same empty input; pin whichever one you raise.

**`DATE`: pin the CONVERSION form, which is deterministic, not any no-argument form.**
`date('S','2026-08-05','I')` is `20260805`; `date('W','20260805','S')` is `Wednesday`; a malformed or impossible input is **40.19**.
Host- or locale-dependent: `L`, `M`, `W` (names), `T`, `F` (absolute clocks). Deterministic given a fixed date: `B`, `D`, `E`, `I`, `N`, `O`, `S`, `U`.
**`DATE('C')` and `DATE('J')` are rejected with 40.904 on this build** despite existing in other Rexx dialects -- pin that, because an implementer working from a generic reference will add them.

- [ ] **Step 1: Neither may appear in a corpus program (D11), so the unit tests are the whole gate**

`TIME('R')` is **stateful** -- it resets an elapsed-time clock, and `rexxcps.rex` depends on it.
Pin the **reset semantics**, not a value: after a reset, a later `TIME('R')` returns elapsed-since-reset rather than elapsed-since-start.

**Two probes inside the same second cannot distinguish a live clock read from a cached one.**
Construct the probe so they can -- separate the reads by a measurable interval, or drive the state rather than the clock.

`DATE`'s option letters are its surface; probe every one, and note which are locale- or host-dependent.
`DATE.testGroup` and `TIME.testGroup` spell 50 directives `::METHOD` uppercase, so scan them case-insensitively.

- [ ] **Step 2: Write failing tests, then implement**

- [ ] **Step 3: Re-run the status harness; assert both read `implemented` and neither `divergent`**

- [ ] **Step 4: Run the shared verify block and commit**

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

**A fourth shape, and the only one that silently corrupted work already committed: the suite has TWO syntax-assertion forms.**
`~expectSyntax(` asserts a **run-time** error; `~assertSyntaxError(` compiles a fragment and asserts a **translation-time** one.
Measured across `ootest/ooRexx/base`: **4,654 and 733**.
The split is by error timing, not by taste -- `bif/` groups test run-time errors and carry **zero** `assertSyntaxError`, while directive and keyword groups carry many:

| group | `expectSyntax` | `assertSyntaxError` |
|---|---|---|
| `bif/MAX`, and every other `bif/` group surveyed | as reported | **0** |
| `keyword/PARSE` | 19 | 0 |
| `keyword/ADDRESS` | 20 | **16** |
| `directives/ROUTINE` | **0** | **22** |

**Every `bif/` figure in this plan therefore stands**, including "`MAX`/`MIN` have zero error coverage".
`keyword/ADDRESS`'s real total is **36, not the 20 recorded earlier**, and `directives/ROUTINE` would have read as "zero error cases" on the wrong scan alone.
**Scan for both forms whenever the subject is a directive or a keyword.**

**Scope of the doubt, checked rather than assumed, because a justified retraction that is not bounded turns into blanket loss of confidence.**
Per directory: `base/bif` **1,230 / 0**, `base/expressions` **405 / 33**, and `base/keyword` **282 / 400** -- the keyword group carries *more* of the unscanned form than the scanned one.

**The committed L1 infrastructure is nevertheless sound, and this was verified rather than hoped.**
`rexx-extract`'s keyword extractor already names `assertSyntaxError` as a `DropReason`: such bodies are **dropped and counted**, so `rows + dropped == calls` closes over them.
Its own comment records that rewriting those calls to `NOP` to admit the bodies was measured and **declined**, because it "would report a body as passing after deleting the checks it was written to make."
So the defect is confined to **survey prose in this plan**; `corpus/keyword-exempt.txt`'s 772 rows and Task 15's `base/bif` model are unaffected, the latter because `base/bif` carries none of the second form at all.

**A third shape of false lead: looking in `bif/` alone, for anything that is both an instruction and a function.**
Measured at Task 9/10: `bif/ADDRESS.testGroup` has **1 method and 2 assertions**, which reads as "essentially untested" -- while `keyword/ADDRESS.testGroup` has **97 methods and 222 assertions**, and tests the swap explicitly at `:1028`.
`TRACE` has **no `bif/` group at all** and 77 methods under `keyword/`.
The three shapes now seen are **bare-word inflation** (`ADDRESS` 707 hits against 208 for the syntax), **harness boilerplate** (407 of 440 `parse source`), and **this one**. All three inflate or deflate a coverage claim by more than an order of magnitude.

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

**A probe's own scaffolding can change the answer without failing, and this has now happened three times in this phase.**

* A helper `::routine` is **not neutral scaffolding**: it has its own variable pool, so evaluating `SYMBOL`/`VAR` inside one reported `LIT`/`0` for every name including assigned ones -- silently inverting every result rather than erroring. **Anything variable-pool-sensitive must be probed in the pool under test.**
* `TRACE()` reports the setting it is running under, so a probe wrapped in `trace i` reports the wrapper.
* A builtin name followed by a space and a parenthesis is a *symbol*, and `b`/`x` before a quoted string is a *literal* -- both return plausible bytes from a program that never made the call.

**The common shape: the harness changes the answer instead of failing.** Ask what your scaffolding contributes before trusting any probe whose result looks uniform.

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

