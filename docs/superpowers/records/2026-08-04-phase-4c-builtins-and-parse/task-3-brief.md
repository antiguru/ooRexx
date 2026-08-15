### Task 3: `builtin/string.rs`

**Files:** modify `crates/rexx-exec/src/builtin/string.rs` (created by Task 2, holding `LENGTH`), `crates/rexx-exec/src/builtin/mod.rs` (dispatch and arity rows only), `rust/corpus/builtin-status.txt`.

**The 22 names to add:** `ABBREV CENTER CENTRE CHANGESTR COMPARE COPIES COUNTSTR DELSTR INSERT LASTPOS LEFT OVERLAY POS REVERSE RIGHT SPACE STRIP SUBSTR TRANSLATE VERIFY LOWER UPPER`.
`LENGTH` is already here and is not rewritten.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 1: Build the probe table before writing any code**

For each name: the documented base case; **every optional argument position separately** (a one-argument probe cannot distinguish a builtin that honours its optional arguments from one that ignores them); at least one probe with a **non-default pad character** where one is taken; the empty string and the `f(,2)` omitted-position forms; and the boundary values the ooTest group uses.

**Run each probe from a fresh empty scratchpad subdirectory with absolute paths.**
A probe run from the scratchpad root can return a stale file's output -- measured, 44.1 rc 212 against the true 43.1 rc 213 -- and a wrong expected value transcribed here becomes an implementation written to match it, with test and code agreeing forever.

- [ ] **Step 2: Read `ootest/ooRexx/base/bif/<NAME>.testGroup` for each name**

It is the reference for edge cases and the same file Task 15's L1 harness will run.
A case it covers that the implementation misses surfaces at Task 15 as an exempt row that should not be there.
`STRIP` and `VERIFY` take option letters -- probe each letter and an invalid one.
`CENTER` and `CENTRE` are one function under two names and dispatch to one implementation.

- [ ] **Step 3: Write failing tests from the table, then implement**

- [ ] **Step 4: Re-run the status harness and assert this family's completeness**

All 23 of this file's names must read `implemented`, and none may read `divergent`.
**Assert the family's list specifically**, not just that the file validates: without it, implementing 20 of 23 and committing 20 flipped rows is green.

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
