# `Directory~setMethod` / `~unsetMethod`

Base: `1310582e9`. This is the task Phase 5h's leftovers close named and did
not start. Its specification there was measured but partial; everything below
was re-measured against the oracle at this base, and the C++ was read
alongside, because the two disagree in one place that matters.

## The rows

Four `send-differs` rows in `corpus/collection-arity.tsv` (of fourteen) and
four `loud` rows in `corpus/method-bodies.txt`:

| table | rows |
| --- | --- |
| arity | `Directory setMethod`, `Directory unsetMethod`, `Properties setMethod`, `Properties unsetMethod` |
| bodies | the same four |

`Properties` comes free: `Properties: Directory` (measured `~superClasses`),
and `corpus/collection-scopes.tsv` puts both methods at scope `Directory`,
implemented by `DirectoryClass::setMethodRexx` / `unsetMethodRexx`.

`StringTable` is **not** in scope and must not change. It is a sibling
(`StringTable: Object MapCollection`), `setMethodRexx` exists only in
`DirectoryClass.cpp`, and a `setMethod` send to a StringTable falls through to
its `unknown`, answering `.nil`. Measured, and **the crate already agrees**:
oracle and both engines answer 97 for a plain `Object`, 97 for a `Table`, and
`.nil` for a `StringTable`. Only the `Directory` arm is missing.

Two of the four arity rows are gated on something else:
`corpus/collection-arguments.tsv:156` sends
`.Method~new("MM","return 7")`, and `Method~new` is a stub.

## `Method~new` is not reflection work

`native_executable_new` (`dispatch.rs:10208`) validates its arguments and then
returns `unbuilt_new`. The validation is already right -- the `Method new`
method-body row reads `answers rc 168`. What is missing is the build, and the
builder already exists: `compile_method_source` (`dispatch.rs:5048`) parses the
source, mints a native instance of the `Method` class, attaches an annotation
table and records the body, and five call sites use it today. `Object~setMethod`
compiles a source string through it right now.

Work: `native_executable_new` branches on the receiving class and delegates to
`compile_method_source` for `Method`. `Routine` shares the function
(`dispatch.rs:1032`) and keeps refusing.

## The measured specification

Every line below was run at this base. `d` is a `.Directory~new`.

**A method entry is an ordinary entry whose item is the method's result,
recomputed on every read.** After `d['plain'] = 1` and
`d~setMethod('GREET', 'return "hi"')`: `items` 2, `allItems` `1 hi`,
`allIndexes` `plain GREET`, `d['GREET']` and `d~greet` `hi`, `hasIndex`
1, `hasEntry` 1, `entry` `hi`, `hasItem('hi')` 1, `index('hi')` `GREET`,
`isEmpty` 0, `makeArray` `plain GREET` (indexes), supplier
`plain=1 GREET=hi`.

Recomputed, not cached, and the body may mutate the directory while a read is
in progress: with `d['n'] = 0` and
`setMethod('TICK', 'self["n"] = self["n"] + 1; return self["n"]')`, three
successive reads answer 1, 2, 3.

**The body runs with the directory as `self` and with no arguments.**
`setMethod('SELFTEST', 'return self~class~id "/" self["plain"]')` answers
`Directory / 7`. `arg()` inside the body is `0` even for `d~args(1,2)` --
`method->run(..., this, name, NULL, 0, v)` at `DirectoryClass.cpp:216`.

**The name is upper-cased**; the index family is not. `setMethod('lower', ...)`
shows as `LOWER`, while `d['plain']` keeps its case.

**The method table is a second `StringTable` and the merge is an append.**
`DirectoryClass.cpp:480` creates it with `new_string_table()`; `allIndexes`
appends `methodTable->allIndexes()` after the contents, `allItems` appends
each method's result, `items` adds the two counts, and `supplier` appends
`(values, indexes)`. So each half is in its own store order. Measured:
inserting `AAA`(method), `zzz`, `MMM`(method), `bbb` answers
`zzz bbb MMM AAA` -- and a plain directory given `AAA` then `MMM` also
answers `MMM AAA`, so the method half needs no geometry of its own. It is the
Phase 5h Task 1 store.

**A write to either half removes the name from the other.**
`setMethodRexx` ends with `contents->remove(entryname)` on **both** its
branches, and `DirectoryClass::put` removes from the method table first.
Measured: `g['AAA'] = 'value'` then `setMethod('AAA', 'return 1')` leaves
`allIndexes` `AAA`, `items` 1, `g['AAA']` 1 -- and `unsetMethod('AAA')` then
leaves the directory **empty**, because the contents entry was destroyed
rather than shadowed. In the other direction, `e~setMethod('M','return 2')`
then `e['M'] = 5` answers 5, and `unsetMethod('M')` afterwards still answers
5.

`unsetMethod` removes from the method table only, never from the contents.

**`setMethod('UNKNOWN', ...)` is special** and the leftovers close missed it.
It replaces a dedicated `unknownMethod` field rather than joining the method
table (`DirectoryClass.cpp:486`). Measured with `d['p'] = 1` and
`setMethod('UNKNOWN', 'return "caught"')`: `d['nosuch']` and `d~nosuch` are
`caught`, `d['p']` is still 1, and it is neither counted nor enumerated --
`items` 1, `allIndexes` `p`, `allItems` `1`, `hasIndex('nosuch')` 0.
`unsetMethod('UNKNOWN')` clears it. `get` consults contents, then the method
table, then the unknown method, in that order.

**`copy` carries the method table, independently.** `f~setMethod('M', ...)`,
`g = f~copy`, `g~unsetMethod('M')` leaves `f['M']` 2 and `g['M']` `.nil`.
This is the store-sharing defect that `duplicate_collection_stores` already
fixes for the contents; the second store needs the same treatment.

**`EMPTY` does not clear the method table**, and this is where the C++ and the
behaviour part company. `DirectoryClass::empty()` clears both halves and the
unknown method -- but the Rexx-visible `EMPTY` is
`HashCollection::emptyRexx`, which calls `contents->empty()` directly and
never reaches the virtual. Measured: after `e~empty`, `items` is 1 and
`e['M']` is still 2. `DirectoryClass::empty()` is dead for the Rexx path.
Implement the measured behaviour, not the override.

**Returns and argument errors.** Both methods return nothing, so an
assignment raises 91: `v = d~setMethod('X','return 1')` is rc 91 on the
oracle while the bare statement is rc 0. `stringArgument(name, "index")`
makes the index a named argument: `setMethod()` and `unsetMethod()` are 88.
`setMethod('A')` alone is the removal form and is rc 0. A third argument is
93 (arity 2). A non-string index that has a string value is accepted:
`setMethod(5, 'return 1')` and `unsetMethod(5)` are both rc 0.

## Tasks

1. **`Method~new`.** Branch `native_executable_new` on the receiving class and
   delegate to `compile_method_source`; `Routine` keeps refusing. Witness the
   built object answering `~scope` and `~annotations` as it already does for a
   compiled method.
2. **The second store.** `store_of` and `install_store` take the five pool
   variable names rather than closing over them, so a `Directory` can hold a
   contents store and a method store at the same scope. One further slot holds
   the unknown method. `insert`, `take`, `walk` and `probe` are unchanged --
   they already take a `Store` by value.
3. **The writes.** Register `SETMETHOD` and `UNSETMETHOD` at `Directory`,
   ahead of `Object`'s. Upper-case the name; route `UNKNOWN` to its own slot;
   compile a source argument through `compile_method_source` and accept a
   `Method` object as it stands; remove the name from the contents on both
   branches; return `Ok(None)`.
4. **The reads.** Merge at every Directory-reachable read point in `hash.rs`:
   `store_at`, `store_put`, `all_indexes`, `all_items`, `make_array`, `items`,
   `is_empty`, `has_index`, `has_item`, `index`, `remove`, `remove_item`,
   `supplier`, the `entry` family, `unknown`, and the `store_indexes` /
   `store_item` walkers `environment.rs` uses. Each is guarded on the method
   store being present, so a `Table` pays one absent-slot check.
5. **`copy`.** `duplicate_collection_stores` duplicates the method store and
   carries the unknown method.
6. **Witnesses and tables.** A corpus witness per group above, filed in the
   four places; `collection-arity.tsv` re-derived (14 `send-differs` to 10);
   `method-bodies.txt` refreshed (four `loud` to `answers`).

## Gates

The seven, over the commit, by the commit-before-gating protocol.

## Adjacent, not in scope

`condition('O')` answers a `Directory` and the crate refuses it, which is why
the arity harness's own syntax trap cannot run under `rexx-run`. Noted for
whoever owns the condition surface.
