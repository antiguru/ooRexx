# Found defect: `DO OVER` a `Body::Native` table binds a dead handle

**Found by Phase 5i Tasks 7+8 while gating `Package`. It is not that task's,
and it is not new: it reproduces at BASE `59d25e938` with no `Package` method
in the program.**

## What it is

`Interp::hash_collection_indexes` (`crates/rexx-exec/src/run.rs`) builds the
index strings a `DO OVER` binds for one of the interpreter's own
`Body::Native` tables — `.METHODS`, `.ROUTINES`, `.RESOURCES`, and every table
`Package`'s readers answer. It allocates one string per key and `push_temp`s
each as it goes. **Those temporaries do not survive into the loop body.** Once
an index is long enough to need a heap object — more than
`rexx_core::INLINE_TEXT`, seven bytes — the collector takes it and the loop
binds a stale handle.

## Reproduction, run at BASE and at this task's head

Under `rexx_exec::run_program_collect_every_alloc`, which is what
`tests/collect_stress.rs` runs:

```rexx
call dump 'm', .methods
exit 0
dump: procedure
  use arg label, table
  do name over table
    say label '['name'] is a' table[name]~class~id
  end
  return
::method AVeryLongMethodNameOne
  return 1
::method AVeryLongMethodNameTwo
  return 2
::method AVeryLongMethodNameThree
  return 3
```

panics at `Interp::not_in_arena`, `a live value`. The same program with the
methods named `MA`, `MB`, `MC` prints all three entries and exits 0 — those
names fit in the handle and no object is allocated for them.

Measured 2026-09-08 in the gate worktree pinned to `59d25e938`: the long-name
form PANICs and the short-name form answers
`m [MA] is a Method / m [MB] is a Method / m [MC] is a Method`.

## What it is not

* **Not the `Package` readers.** The program above sends no `Package` method.
* **Not `DO OVER` in general.** `do n over .array~of(<three long strings>)`,
  and the same over a `.StringTable~new` and a `.Directory~new` with long
  keys, all print every entry and exit 0. Those reach
  `Interp::over_items`' converted-target branch, where the index strings are
  the collection's own and stay reachable through the variable naming it.
* **Not visible without the stress mode.** An ordinary run collects far less
  often and the window may never open; nothing in the corpus differential sees
  it.

## What was established about the mechanism, and what was not

`hash_collection_indexes` returns items that are all still live — a check
inserted at the end of that function found none dead. They die later, while
their handles are still on the temporaries stack, which is the part that does
not add up and was not resolved.

**A durable root fixes it**, measured: adding
`self.roots.add_global(&format!("…{n}"), item)` beside the existing
`push_temp` makes all five `Package` witnesses pass under the stress mode.
That is a diagnostic, not a fix — it leaks a global per index.

**Wrapping the items in one `Array` and `push_temp`ing that does not fix it**,
measured — which is the same shape the converted-target branch already uses,
so that branch is very likely to be equally unprotected and to be passing only
because its items are reachable from the target the source named.

## What a fix would take

A root tied to the loop's lifetime rather than to the clause's:
`RootSet::park`/`release` around `LoopState::OverItems`, released wherever a
loop state is discarded — normal exit, `LEAVE`, `ITERATE` out of an enclosing
loop, `SIGNAL` out, and an activation unwind, in **both** engines. That is
loop-control work in `run.rs` and was out of Tasks 7+8's scope.

## What Tasks 7+8 did instead

Kept every witness and kept them out of the trap:

* `package_tables.rex` and `package_writes.rex` enumerate their tables with
  `DO OVER` and key every entry under a name of at most seven bytes, so the
  bound index sits in the handle. Each program's own comment says why.
* `package_rexx.rex` cannot choose the REXX package's class names, so it
  reads all 67 back **by name** from a list it carries itself and uses
  `DO OVER` only to count. That is the stronger witness of the two anyway: it
  says which names must be there, where an enumeration says only how many.
