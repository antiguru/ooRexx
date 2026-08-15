### Task 2: Builtin dispatch, the arity table, and the 40.x error family

**Files:**
* Create: `crates/rexx-exec/src/builtin/mod.rs`, `crates/rexx-exec/src/builtin/string.rs` (holding `LENGTH` alone)
* Modify: `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/error.rs`, `rust/corpus/builtin-status.txt`

**Read "Shared facts every builtin task needs" -- it is restated in your brief and carries the dispatch signature, the argument model, the existing 40.x raisers, the allocation rule, the probe safety rules and the verify block.**

- [ ] **Step 1: Measure the 40.x family**

Measured 2026-08-04, rc **216** in every case:

```
     1 *-* say substr('abc')
Error 40 running <path> line 1:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
```

| probe | sub-code | secondary text |
|---|---|---|
| `substr('abc')` | 40.3 | `Not enough arguments in invocation of SUBSTR; minimum expected is 2.` |
| `substr('abc','x')` | 40.12 | `SUBSTR argument 2 must be a whole number; found "x".` |
| `substr('abc',2,3,'pq')` | 40.23 | `SUBSTR argument 4 must be a single character; found "pq".` |

Probe further, because guessing a sub-code ships a whole family wrong: too many arguments; a missing *required* argument in a middle position (`substr('abc',,2)`); a negative where non-negative is required; and the same type error on a builtin **not** named `SUBSTR`, to confirm the name is interpolated.
The name in the message is uppercased while the `*-*` echo carries the source spelling -- confirm both.

- [ ] **Step 2: Write the failing tests**

`say length('abc')` prints `3`, exit 0; and `say length()` produces the measured 40.3 bytes.
Both fail: `dispatch` does not exist.

- [ ] **Step 3: Build `builtin/mod.rs` and hook it in at the right place**

**The hook goes *after* argument evaluation, not at the label-lookup fallback.**
`run.rs`'s `resolve_and_run_call` looks up the label, returns `Loud::unresolved_call` if absent, and evaluates the argument expressions **after** that point.
The first revision put the builtin step between the lookup and the loud fallback, which is upstream of the evaluation it claimed to consume unchanged.
Restructure so the name resolves to one of three outcomes *before* the loud return, then evaluate arguments once, then dispatch.

**Answer these three for the builtin path explicitly, because the label path answers them and the builtin path must not inherit the answers by accident:** whether `SIGL` is set, whether `>A>` argument trace lines fire, and whether the activation depth counter increments.
Measure each on the oracle and state the answer in the code's own doc comment.

The name set the dispatch covers is `NAMES` minus the **15 whole exclusions**, which is **66** -- not `NAMES` minus `EXCLUDED_BUILTINS`, which is 63, because `VALUE`, `ADDRESS` and `QUEUED` are partial rows that must dispatch.
Read the list from `rexx_inventory` (Task 1 moved it there); do not copy it.

**Only `LENGTH` is implemented here**, and it goes straight into `builtin/string.rs`, which this task creates.
A one-builtin file is not a placeholder: `dispatch` needs one real name to prove the chain end to end, and staging it through `mod.rs` buys a rename and a diff that says nothing.

- [ ] **Step 4: Leave the loud fallback in place**

A name that is neither a label nor a builtin still returns `Loud::unresolved_call`. Task 13 replaces it.

- [ ] **Step 5: Discharge Task 1's Step 4.3**

Delete `LENGTH`'s dispatch arm, run `builtin_status.rs`, and confirm its row flips `implemented` -> `loud` on its own.
Restore from a copy, not `git checkout --`, and confirm `git status` is clean.
**This is the falsification that proves the status harness observes the interpreter rather than a name table**, and Task 1 could not run it.

- [ ] **Step 6: Re-run the status harness, commit the flipped row, verify and commit**

---



---

## Shared facts every builtin task needs

**Tasks 2, 3, 4, 5, 6, 10, 11 and 12 each restate this block in their own body.**
It is here once so the eight agree; it is *in* each task because briefs are extracted per heading and nothing outside a task's own section is visible to its implementer.

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
What is new is the *type* family -- 40.12 whole number, 40.23 single character, and the others Task 2 measures.

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
