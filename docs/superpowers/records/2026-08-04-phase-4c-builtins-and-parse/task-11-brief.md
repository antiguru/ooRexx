### Task 11: `builtin/datatype.rs`

**Files:** create `crates/rexx-exec/src/builtin/datatype.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 4 names:** `DATATYPE SYMBOL VALUE VAR`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 1: Build the probe table**

`DATATYPE`'s option letters are its whole surface -- probe every one.
`DATATYPE.testGroup` is one of the six groups containing non-UTF-8 bytes; use `/bin/grep -a` on it.

- [ ] **Step 0: Measured 2026-08-05 -- read before writing any code**

**(a) `VALUE`'s phase split is decided by the third argument's PRESENCE, not its value, and getting this wrong is silent.**

```
myvar = 'ORIGINAL'
value('myvar')             -> ORIGINAL
value('myvar','NEWVAL')    -> ORIGINAL     returns the OLD value
value('myvar')             -> NEWVAL       the assignment took effect
value('myvar',,'')         -> .MYVAR       <-- an EMPTY third argument is still a pool lookup
value('myvar','N','')      -> .MYVAR
value('zz',,'NOSUCHPOOL')  -> 40.914
```

**A crate that ignores the third argument returns `NEWVAL` where the oracle returns `.MYVAR`** -- a wrong answer, not a loud one. The discriminator at the call site is **purely arity**.
Also: `value('nosuchvar')` returns the **uppercased name** with no error, and lookup is caseless.

**(b) `DATATYPE` is strictly ASCII, and only the option's FIRST character is read.**
The authoritative option set comes from the error insert, not the documentation: `datatype(1,'Z')` gives `93.915`, *"Method option must be one of "ABILMNOSUVWX9""*.

```
datatype(123,'n') = 1     datatype(123,'NUM') = 1     datatype(123,'NX') = 1
datatype(123,'')  = 93.915                            datatype(123,'Z')  = 93.915
```

So `'NUM'` and `'NX'` are accepted silently; only an empty string or a bad *first* character raises.
**Swept over all 256 bytes: every byte from `0x80` to `0xFF` returns 0 for `A`, `U`, `L`, `W` and `M`.** Nothing above `0x7F` is ever a letter.

**An OMITTED second argument is not an empty one, and the whole message is the requirement.** Measured 2026-08-07:

```
datatype('',)     -> CHAR       omitted: the one-argument form
datatype(1,'')    -> 93.915     empty: Method option must be one of "ABILMNOSUVWX9"; found "?".
datatype(1,'Z')   -> 93.915     Method option must be one of "ABILMNOSUVWX9"; found "Z".
```

Both raisers are rc 163.
The empty option's insert renders as **`found "?"`**, not `found ""` -- the oracle substitutes a placeholder where there is no character to show, so a crate that formats the option's own bytes emits a byte-different message.
This is the absent-versus-empty axis Task 8 had to cross for `ARG`; here it decides between an answer and an error.

**The trap: the empty string is `CHAR` in the one-argument form, but `datatype('','B')` is `1`** -- the empty string is a valid binary string.

**(c) `SYMBOL`/`VAR`: a stem with a default value makes EVERY tail report `VAR`**, including tails never assigned. `BAD` is reserved for genuinely malformed names -- `1abc` is `LIT`, not `BAD`.

- [ ] **Step 2: `VALUE` is split, and its two-argument form writes the pool**

The variable-access form (`value('name')`, `value('name', newval)`) is 4c's.
**The external-selector form (`value(name, , 'ENVIRONMENT')`) is Phase 7's and must fail loudly naming Phase 7**, not silently ignore the third argument.

**No existing `Loud` constructor fits, and the owner string is policed for its exact spelling.**
Checked 2026-08-07: `Loud::unresolved_call` (`lib.rs:517`) hardcodes `Some("4c")`; `Loud::compound_expose` and `Loud::builtin_option_object` carry no owner at all, and both docs say why.
`owned_message(name, owner)` (`lib.rs:718`) takes an arbitrary owner and renders `"{name} is not implemented ({owner})"`, so add a constructor that passes `Some("Phase 7")`.
The literal is **`"Phase 7"`, not `"7"`** -- that is how `instruction_owner` spells it (`lib.rs:856`, `:867`), how `ADDRESS`'s message spells it (`run.rs:13442`), and `coverage.rs`'s module doc requires the split table's spelling.

The two-argument form writes, so it needs the **growing** slot resolver.
That is **`Interp::slot_of`** (`plan.rs:561`), which checks plan, then `extra`, then grows, and is idempotent.
It is **not** `Plan::slot_of` (`plan.rs:528`), which is `&self -> Option<usize>` and cannot create.
An implementer who writes "if not in plan, grow" instead leaks one slot per call.

- [ ] **Step 3: Read each `<NAME>.testGroup`, write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 4 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

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

