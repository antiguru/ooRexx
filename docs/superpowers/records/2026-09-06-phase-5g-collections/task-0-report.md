# Phase 5g Task 0 — the instrument, the scopes table, and the module

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
Spec: `docs/superpowers/specs/2026-09-06-collections.md`. BASE `f5b383d6f`.

Three deliverables, all landed. **No method body is added and no row moves in
`corpus/method-bodies.txt`.**

---

## (c) `crates/rexx-exec/src/dispatch/collection.rs`

Landed with an empty `NATIVE_METHODS` slice, chained into `ObjectModel::build`
beside `string::NATIVE_METHODS` at both of `dispatch.rs`'s chain sites.

**Red control, both halves run and both confirmed.** Predicted before running:
a row naming a method the class does not answer panics at build; a row binding
a real loud name to the wrong body makes the send reach that body.

| step | predicted | measured |
|---|---|---|
| `("Array", "ZZZPROBE", Fixed(0), native_default_name)` | panic at `ObjectModel::build` | rc 101, `NATIVE_METHODS names Array~ZZZPROBE, which that class's behaviour does not answer` (`dispatch.rs:1209`) |
| `("Array", "ISEMPTY", Fixed(0), native_default_name)` | `.Array~new~isEmpty` answers a default name at rc 0 | rc 0, `an Array` |
| slice emptied again | `isEmpty` loud once more | rc 120, `method "ISEMPTY" of class "Array" is not implemented (Phase 5)`, both engines |

---

## (a) `corpus/collection-scopes.tsv`

461 rows over the fourteen classes in scope; `crates/rexx-exec/tests/collection_scopes.rs`
re-derives it on every run. 311 native, 150 Rexx.

**The scope column is the oracle's own answer**, per spec D89 —
`receiver~instanceMethod(NAME)~scope~id`, one probe program, no misses. The
`Setup.cpp` scan supplies only native-or-Rexx and the token.

**The red control for the recipe is the three rows a file-only join gets
wrong.** Predicted before reading the generated table, and all four confirmed:

| row | predicted | in the table |
|---|---|---|
| `Queue~sort`, `Queue~stableSort` | `OrderedCollection`, rexx | as predicted |
| `Set~union`, `Bag~union`, `Relation~union` | the class's own scope, rexx | as predicted |
| `Set~hasItem` | native, token `IdentityTable::hasIndexRexx`, arity 1 | as predicted |
| `CircularQueue~makeArray` | `CircularQueue`, rexx | as predicted |

`RemoveMethod` and `HideMethod` never had to be implemented, which is the
recipe change paying for itself: a name they remove is not at that scope on the
oracle either, so the lookup never reaches the stale copy.

**Red control on the comparison test.** Changing `Set~hasItem`'s committed
token to `HashCollection::hasIndexRexx` — the function it actually resolves to,
so a *true* statement — reddens `the_table_matches_the_interpreter`. The column
is a citation of what `Setup.cpp` writes, and the test holds it to that.

---

## (b) `corpus/collection-arity.tsv`

433 instance rows, each sent an argument list the oracle completes, on the
oracle and both engines, three descriptors compared. 28 seconds, reproduced
twice with identical counts.

| verdict | rows |
|---|---|
| `setup-differs` | 321 |
| `send-differs` | 90 |
| `agree` | 21 |
| `exempt` | 1 |

**The headline, and it is what sizes Phase 5h.** Crossing this against
`corpus/method-bodies.txt`'s instance rows for the same classes — all 433 join,
under `LC_ALL=C`:

| `method-bodies.txt` | instrument | rows |
|---|---|---|
| `answers` | `agree` | 21 |
| `answers` | `send-differs` | 23 |
| `answers` | `setup-differs` | 125 |
| `answers` | `exempt` | 1 |
| `loud` | `send-differs` | 67 |
| `loud` | `setup-differs` | 196 |

**The verdict column calls 170 of these rows `answers`. Sent a real argument
list, 21 of them answer.** The survey's warning was right and its size was
understated.

`setup-differs` dominating is not a harness fault, it is the measurement:
`corpus/collection-receivers.tsv` deliberately builds the richest receiver the
*oracle* can build, so a class with no store fails there. Nine of the fourteen
classes cannot hold anything — `Bag`, `CircularQueue`, `IdentityTable`, `List`,
`Properties`, `Queue`, `Relation`, `Set`, `Stem`, `Table`. `Array`,
`Directory`, `StringTable` and `Supplier` build on both sides and give real
per-row signal.

### The harness rule, and that it earned its keep

**A list is real only if the oracle completes the send** — the probe prints
`SENT` after it, and `the_oracle_completes_every_send` makes a run that does
not reach it a failure for that row rather than a data point.

**An earlier version of the rule was itself vacuous and this is worth
recording.** It asked only for oracle exit 0; the probe traps `SYNTAX` and
exits 0, so a list that *raised* satisfied it. Tightening it to `SENT` is what
found the crash below.

Between them the rule and `every_row_is_sent_something_its_arity_needs` caught
**eleven** of my own wrong argument lists across three rounds — `r~[](1)` for
a bracket send, a `call r~append 'b'` that is not valid syntax, `index` shaped
as taking nothing when it takes an item, six `Properties` rows needing typed
values, `CircularQueue~init`'s arity, `Stem~unknown`'s forwarding signature,
`CircularQueue~makeString` forwarding to its own `~string` rather than
`Array`'s, and two rows sent nothing at a non-zero arity.

**One exemption, with its reason committed**: `Properties~load` needs a file
only `~save` creates, and putting `save` in the receiver setup would make every
`Properties` row read `setup-differs`.

### A new oracle crash, found by the rule

```rexx
s = .Stem~new
s['k1'] = 'v1'
say s~index
```

SIGSEGV, rc 139, deterministic over three runs, stdout and stderr both empty.
`say s~hasItem` is the same crash. `StemClass::index`
(`classes/StemClass.cpp:430`) and `StemClass::hasItem` (`:377`) both open with
`findByValue(target)` and neither calls `requiredArgument`; `Setup.cpp`
registers each at arity 1, which is a **maximum**, so a zero-argument send
arrives with `target` null. The guarded sibling `removeItemRexx` (`:391`) is in
the same file and measured clean at rc 163, so the fix is that one check.

Same shape as the six `String` shift operators already in the file — an
unguarded required argument behind a maximum arity — so it is a second
instance of one upstream defect class. Filed as a new entry in
`corpus/oracle-crashes.txt` with the consequence for
`method_bodies.rs`'s refresh. **No upstream ticket.**

---

## Decision taken, and it was the plan's to take

**Task 0 lands before spec D92's `Bag~put`/`Set~put` commit.** The plan's risk
section says the reverse order makes the red control unfalsifiable: D92 turns
that row `loud`, and a loud row is classified without running the oracle at
all. Ordered this way the control can fail, and the prediction written before
the run — that the two rows would disagree, but through `setup-differs` rather
than `send-differs`, because the receiver setup itself calls `put` — came out
confirmed in both halves.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` **G0**.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
