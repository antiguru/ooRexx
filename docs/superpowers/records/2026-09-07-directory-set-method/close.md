# `Directory~setMethod` / `~unsetMethod` -- close

Base `1310582e9`. Plan:
`docs/superpowers/plans/2026-09-07-directory-set-method.md`.

## What moved

`corpus/collection-arity.tsv`: `agree` 418 to **422**, `send-differs` 14 to
**10**, `setup-differs` 0. The four rows are `Directory`/`Properties` times
`setMethod`/`unsetMethod`.

`corpus/method-bodies.txt`: the same four rows, `loud` to `answers rc 168`.
Nothing else drifted -- the refresh diff is eight lines across the two tables.

Strict corpus 437 to **439**: `lang/directory_set_method.rex` and
`lang/method_new.rex`, each filed in all four places.

## The prerequisite check paid, and shrank the task

The leftovers close recorded `Method~new` as an additional cost of the
`Method`-object form. Read against the tree first, it was the smaller half:
`compile_method_source` (`dispatch.rs:5048`) already builds a complete
first-class `Method` -- native instance, annotation table, body recorded in
`table_method_bodies` -- and five call sites use it, including
`Object~setMethod` compiling a source string. `native_executable_new` already
validated its arguments correctly (`Method new` read `answers rc 168` before
this change) and then refused. Wiring it was a delegation and a class branch
so `Routine~new`, which shares the function, keeps refusing.

The store was the other half of the saving. `DirectoryClass.cpp:496` builds
the method table with `new_string_table()`, so it is the Phase 5h Task 1 store
with different pool names, not a structure of its own. Measured before
anything was written: a directory given the methods `AAA` then `MMM`
enumerates them `MMM AAA`, and a plain directory given those two names as
ordinary entries answers `MMM AAA` too. A `Half` name set threaded through the
mechanics was the whole storage change.

## Three things the recorded specification did not have

**`setMethod('UNKNOWN', ...)` is not a table entry.** It replaces a dedicated
field and answers when nothing else does. It is not counted and not
enumerated -- measured, a directory holding one ordinary entry and an
`UNKNOWN` method answers `items` 1 and `allIndexes` `p` -- and `hasIndex` does
not consult it while `[]` does. It is also run differently: with the index as
its ONE argument and under the name `UNKNOWN`, where a method-table entry is
run with none.

**`EMPTY` does not clear the method table, and the C++ says it does.**
`DirectoryClass::empty()` clears both halves and the unknown method, but the
Rexx-visible `EMPTY` is `HashCollection::emptyRexx`, which calls
`contents->empty()` directly and never reaches the virtual. Measured: after
`k~empty`, `items` is 1 and `k['M']` still answers 2. Reading either the code
or the behaviour alone gets this wrong in a different direction; the override
is dead for the Rexx path.

**A collision destroys rather than shadows.** `setMethodRexx` ends with
`contents->remove(entryname)` on both its branches. Measured: `g['AAA'] =
'value'`, then `setMethod('AAA', 'return 1')`, then `unsetMethod('AAA')`
leaves the directory EMPTY -- the ordinary entry does not come back. The
leftovers close had recorded this as the method winning a merge.

## Where the merge went

Most of it concentrated rather than scattering. `pairs()` feeds `allIndexes`,
`allItems`, `makeArray`, `hasItem`, `index`, `removeItem` and `supplier`, so
appending the method half there covers seven read points at once -- and the
oracle's `getIndex`, which runs every method looking for a matching value,
falls out of it rather than being written. `store_at`, `entry` and the
`UNKNOWN`-message read share one `merged_get`. `items` and `isEmpty` take a
separate count that does NOT run the methods, which is the oracle's split.

`insert` on the contents half drops a method of the same name (`put`'s
semantics), and `take_merged` answers what a read would -- possibly running a
method, or the unknown method -- before dropping the name from both halves.

The method table's entries are read out before any of them runs: a body may
write to the directory, measured at 1, 2, 3 over three reads of one entry, and
a walk interleaved with that would follow a chain its own callee had moved.

## What is preserved rather than fixed

`StringTable` keeps `Object`'s private pair and is untouched: `setMethodRexx`
is declared only in `DirectoryClass.cpp`, and a `setMethod` send to a
StringTable falls through to its `unknown`. The crate already agreed on all
three arms before this change and still does -- 97 for a plain object, 97 for
a `Table`, `.nil` for a `StringTable` -- which is why the override is
registered at `Directory` and reaches only it and `Properties`.

Two gaps in `compile_method_source` are carried unchanged, both already true
of `Object~setMethod`: an unparseable source is oracle rc 14 and a crate
refusal, and a source carrying a directive is accepted by the oracle and
refused here. `Routine~new` and the three-argument `Method~new` still refuse,
as they did before.

## Adjacent, found and not taken

`condition('O')` answers a `Directory` and the crate refuses it, so the arity
harness's own syntax trap cannot run under `rexx-run`. `DO ... OVER` a
`Directory` is likewise unimplemented, which is what the remaining
`Directory subset` and `Properties subset` rows are waiting on.

## Verification

Nine probe programs, each diffed against the oracle on both engines by exit
status and stdout: the core surface, ordering, collision, removal, `self` and
argument count, `empty`, `copy`, `setEntry`, the no-method form, the `UNKNOWN`
handler and its argument, the 97/97/`.nil`/91 restriction matrix, the class
hierarchy, and a growth case of 40 methods whose scrambled bucket order
matches exactly. All agree.

## Gates

Over `2b4db37e0`: G1 0, G2 0, G3 0, G4 0, G5 0, **G6 101**, G7 0.

The G6 failure was `queued_empty` -- the whole program is `say queued()` --
and it is not this commit's. The same binary passed that case in G3, G4 and
G5 minutes earlier in the same run, and `REXX_PHASE_GATE=5c` cannot change
what `queued()` reads. Re-running the full G6 command alone: rc 0,
`failed-suites=0`. The session queue is shared per user across processes, so
a concurrently-running suite's `push` is visible to a bare `queued()` in
another. This is the same surface `QUEUED`'s partial exclusion already names
as un-differentiable cross-process, and it is an argument for giving that
harness a queue of its own.

A follow-up commit extends the witness with the case-sensitivity asymmetry
`setMethod`/`put`/`setEntry` have -- `setMethod('unknown', ...)` upper-cases
into the UNKNOWN slot, `d['m'] = 5` removes a method by the index AS WRITTEN
and so leaves an upper-cased `M` standing beside a new `m`, while
`setEntry('m', 9)` upper-cases first and does reach it. Measured on both
engines.

Over that commit, `bdb56f652`: all seven zero, `failed-suites=0` on each of
the five suite-running gates. G6 among them, which is the third reading
agreeing that `queued_empty` was transient.
