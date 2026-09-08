# Found defect: `DO OVER` a `Body::Native` table binds a dead handle

**Found by Phase 5i Tasks 7+8 while gating `Package`. It is not that task's,
and it is not new: it reproduces at BASE `59d25e938` with no `Package` method
in the program.**

## What it is

`Interp::hash_collection_indexes` (`crates/rexx-exec/src/run.rs`) builds the
index strings a `DO OVER` binds for one of the interpreter's own
`Body::Native` tables — `.METHODS`, `.ROUTINES`, `.RESOURCES`, and every table
`Package`'s readers answer. It allocates one string per key and `push_temp`s
each as it goes. **A `push_temp` is not enough to keep one alive to the loop
body**: measured, a durable root is, and the same `push_temp` is not.

## The seven-byte boundary is a VISIBILITY threshold, not the mechanism

**Ran**: with method names of three bytes the loop prints every entry at rc 0;
with the same program's names at twenty-plus bytes it panics. **Read**:
`ObjRef::inline_text` (`rexx-core/src/handle.rs:184`) answers `None` above
`INLINE_TEXT`, which is 7, so a name at or below it **is not an allocation at
all** — the bytes live in the handle and there is no object for a collector to
take.

**So the boundary is where the defect becomes observable, not where it
begins.** A name short enough to inline hides an unrooted allocation by not
allocating; it says nothing about whether the rooting of the allocated ones is
sound. Everything below is about that rooting.

**What is established by running, and what is not:**

* **Ran** — the items are all still live when `hash_collection_indexes`
  returns: a check inserted at the end of that function found none dead.
* **Ran** — at the panic the handle is in the root set (`temps` and a variable
  slot) and `Heap::get` rejects it: `Heap { slot: 44, generation: 1 }`,
  a slot whose generation has moved on.
* **Ran** — adding `roots.add_global` beside the existing `push_temp` makes all
  five `Package` witnesses pass under the stress mode.
* **Ran** — putting the items in one `Array` and `push_temp`ing that does
  **not** fix it.
* **NOT established** — where between the two the temporary stops protecting
  the object. The two runs above bracket it and no third run was made, so
  "the clause boundary truncates them" is an inference from the code's own
  doc comments and **was not run**. No site in `run.rs` is named here as the
  one that drops them, because nothing was run there.

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

## Does it reach a user program?

**Yes for the program text; only under the stress instrument for the
observation.**

* **Ran** — `.METHODS`, `.ROUTINES` and `.RESOURCES` each reproduce it. Those
  are reachable from any ordinary program that declares a `::METHOD`,
  `::ROUTINE` or `::RESOURCE` and iterates the table, with no `Package` method
  and nothing unusual in it. Every table `Package`'s ten readers answer is the
  same shape.
* **Ran** — `.StringTable~new`, `.Directory~new` and `.array~of` with equally
  long members all print every entry at rc 0. So the defect is confined to
  `Body::Native` tables and does not touch the collection classes a program
  builds itself.
* **NOT observed in an ordinary run.** Every reproduction here is under
  `run_program_collect_every_alloc`. The corpus differential, which runs the
  same programs normally against the oracle, is green on all five witnesses.
  Whether an ordinary run can open the window was **not** established either
  way — it was not attempted.

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

## What Tasks 7+8 did instead, and how that could mask this

Kept every witness and kept them out of the trap:

* `package_tables.rex` and `package_writes.rex` enumerate their tables with
  `DO OVER` and key every entry under a name of at most seven bytes, so the
  bound index sits in the handle. Each program's own comment says why.
* `package_rexx.rex` cannot choose the REXX package's class names, so it
  reads all 67 back **by name** from a list it carries itself and uses
  `DO OVER` only to count. That is the stronger witness of the two anyway: it
  says which names must be there, where an enumeration says only how many.

**THE AVOIDED SHAPE IS THE ONE A LATER TASK WOULD NATURALLY WRITE, AND THAT IS
THE HAZARD THIS SECTION EXISTS TO NAME.** `do n over p~classes ; say n` over a
table whose names are ordinary length -- `Comparable`, `MutableBuffer`,
`OwnClassPublic` -- is the obvious way to witness any collection-returning
reader, and it is exactly what fails. Nothing in the tree stops a task writing
it: the corpus differential passes such a program, and only
`collect_stress.rs` reddens, at a panic whose text (`a live value`) names
neither `DO OVER` nor the table.

So a reader who sees three corpus programs iterating tables under short names
and concludes the shape is safe has drawn the wrong conclusion. **The names are
short because of this defect, not because short names are better.** When it is
fixed, the workaround should be reverted -- long names in
`package_tables.rex`, `package_writes.rex` and the two `.cls` helpers, and
`package_rexx.rex` back to enumerating -- and that revert is itself the
regression test.
