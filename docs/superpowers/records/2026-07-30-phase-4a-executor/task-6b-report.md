STATUS: DONE

# Task 6b report: expose `SymbolId`'s index

Commit `180875a9`, "Expose SymbolId's index, so a per-body table can be a Vec".
Two files, both `rexx-parse`. `rexx-exec` untouched.

`rexx-parse` **399 passed / 0 failed**, clippy clean, fmt clean. Workspace minus
`rexx-exec` **603 passed / 0 failed / 3 ignored**, clippy clean. See "The
workspace is red and it is not this" below.

## The argument, verified rather than inherited

The dispatch's claim is that exposing the index reveals no invariant that is not
already public. Checked in `token.rs` rather than accepted:

* `intern` computes `SymbolId(u32::try_from(self.names.len()).expect(...))`
  **before** pushing, so ids are dense, zero-based and in interning order.
* The public `name` does `&self.names[id.0 as usize]` — the raw value is
  already a direct index into `names`, on a public path.
* `len` returns `self.names.len()`, so a caller can size a `Vec` exactly.

So the guarantee was already load-bearing in public API. Confirmed.

## One correction to the dispatch's wording, which changed the doc comment

The dispatch says "`name()` already panics on an id from a different table".
That is true only when the value is **out of range**. An id from another table
whose raw value happens to be in range returns a different symbol, silently,
with no panic anywhere.

That distinction is the whole reason this accessor deserves a warning, so the
doc comment separates the two cases rather than repeating the stronger claim:

> * If the value is out of the other table's range, indexing panics, and so
>   does `name`.
> * **If it is in range, you get a different symbol and no complaint at all.**
>   That is the case to design against, and this accessor makes it easier to
>   reach, since a raw `usize` can be carried anywhere a `SymbolId` could not.

`Fragment` is named as the concrete way to hit it, per the dispatch: it carries
its own `SymbolTable` built fresh by every `parse_interpret`, so id 7 in a
fragment and id 7 in the enclosing program are unrelated symbols, and a
fragment's names have to be resolved through `fragment.symbols.name(id)`.

## The name

Kept `index`, as suggested. The alternatives I weighed were `as_usize` and
`to_index`. `index` wins because it says *what the number is* rather than what
type it has, which is exactly what the doc comment then has to qualify, and it
reads well at the call site: `plan.by_symbol[id.index()]`. It also sits beside
`name`, `len` and `is_empty` in this file without looking foreign.

## The test, and proof it pins something

`symbol_ids_are_dense_and_zero_based` in `token/tests.rs`. It asserts:

* five distinct symbols get indices exactly `0..5`;
* `len()` is exactly one past the largest index, since that is the number a
  caller sizes its `Vec` by, and "at least" would not do;
* re-interning a differently-cased spelling returns the original id and does
  **not** consume an index, so case-insensitive interning cannot make the range
  sparse;
* `index()` addresses the same entry `name()` does, which is the property that
  makes a parallel `Vec` line up with the table at all.

**Verified by mutation rather than by watching it go green.** Changing `intern`
to assign `SymbolId(names.len() * 2)` in a scratch copy fails it:

```
assertion `left == right` failed
  left: [0, 2, 4, 6, 8]
 right: [0, 1, 2, 3, 4]
```

and **every other test in the crate still passes** under that mutation. So
without this test, a change making ids sparse would surface only inside a
consumer's `Vec` — as a panic if out of range, or as a wrong slot if not.

## The workspace is red and it is not this

`cargo test --workspace` fails to compile `rexx-exec`:

```
error[E0063]: missing field `settings` in initializer of `Activation`
```

That is Task 6's in-flight work: `activation.rs` and `plan.rs` are untracked and
`lib.rs` is modified in the working tree. My change adds an inherent method and
a test and cannot affect it. Confirmed by exclusion:
`cargo test --workspace --exclude rexx-exec` is 603 passed / 0 failed, and
clippy the same. Nothing of mine is staged outside `rexx-parse`.

## Notes for Task 6

* `SymbolId::index()` and `SymbolTable::len()` together are everything needed
  for `by_symbol: Vec<Option<usize>>` sized at `symbols.len()`.
* `Option<usize>` rather than `usize` still matters: not every interned symbol
  is a variable. Keywords, labels and constants are interned too, so a dense
  `Vec` over the whole table will have holes, and the holes must be
  distinguishable from slot 0.
* The audit finding stands: the constraint that forced the `HashMap` was
  recorded only in Task 3's report and in `rexx-exec/src/lib.rs`'s comment. That
  comment can now be updated to say the accessor exists, which is Task 6's to
  do since it owns that file today.

## Method note

Mutation testing needed a scratch copy that could build, and `rexx-parse`'s test
binary reaches further into the C++ tree than the build script does — it
`include_str!`s `CoreClasses.orx` and `StreamClasses.orx` as well as needing
`rexxmsg.xml` and `BuiltinFunctions.cpp`. Copying files one at a time cost two
failed builds before I symlinked `interpreter/` into the scratch root, which is
outside the repository and so is not the mistake the dispatch warned about. The
dispatch's own suggestion, `git archive <commit> | tar -x`, would have carried
the whole tree in one step and is the better move — it just does not apply here,
since the change under test was uncommitted.
