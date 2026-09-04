# Phase 5e — `MutableBuffer` gets state, and hollowness gets a gate

**Measured 2026-09-04 at `dc20a0f18`.** Decided by Moritz as D-P7-2: a 5c follow-up of its own,
before Phase 7 starts, rather than folding the work into Phase 7.

This phase exists because of the rule added to `rust/CLAUDE.md` the same day — **an empty method is
not an implementation, and building one lays a trap for the next phase.** `MutableBuffer` is the
instance that produced the rule, and it is on Phase 7's critical path.

---

## 1. What is actually wrong

Phase 5c built *constructors*, not bodies. For `MutableBuffer` that landed as a working constructor
over an instance with no contents, and every reader refusing:

```text
.MutableBuffer~new              oracle: The MutableBuffer class    crate: same, rc 0
say .MutableBuffer~new('abc')   oracle: abc                        crate: rc 120, MAKESTRING loud
~length ~string ~endsWith ~delstr, and the rest
                                oracle answers each               crate: each rc 120 loud
```

`dispatch.rs`'s own doc states the design rather than hiding it:

> The buffer's contents are not kept, and nothing that would read them answers -- which is what
> keeps the rendering honest, since the oracle renders a buffer as its contents rather than as a
> default name

**That was defensible and it is not what this phase is fixing.** Refusing every reader is what kept
a stateless constructor from ever giving a *wrong* answer. What it did not do is stop a later reader
counting the class as built — Phase 7 was scoped on `File`'s 57 `answers` rows, and `File` cannot
construct at all, because `File~init` → `normalizePathSyntax` → `.mutableBuffer~new(path)` and then
`~length`.

**One row of the 52 is `new` (class arm) and answers; the other 51 are instance and are `loud`.**

---

## 2. Capacity is observable, so storage is not just "hold the text"

The obvious implementation — an object variable carrying a string — is wrong, and this is the
measurement that says so:

```text
b = .MutableBuffer~new('abc')
  len 3 cap 256                         <- a default capacity, not the text's length
b~setBufferSize(500)
  len 3 cap 500
c = .MutableBuffer~new('abc', 999)
  len 3 cap 999                         <- the constructor's second argument
b~append('defgh')
  len 8 cap 500                         <- capacity survives a growing append
```

`getBufferSize` and `setBufferSize` make capacity a **first-class observable with a default of
256**, independent of length, and the oracle's `ensureCapacity`
(`classes/MutableBufferClass.hpp:69`) is what maintains it. Any representation this phase picks has
to carry length and capacity separately and reproduce that default.

---

## D-numbers

**D80 — where the bytes live.** Three shapes, and the width assertion in `rexx-core/src/body.rs` is
the constraint on all of them:

* an object variable in `Body::Instance`'s `pools`, holding text plus a second variable for
  capacity — cheapest, no core change, and every mutation reallocates;
* a new boxed `Body` variant carrying `Bytes` + capacity, following `Body::Native`'s own precedent
  (*"Boxed, and that is the decision the assertion below asks for"*) — matches the oracle's shape
  and costs one pointer;
* `NativeObject`'s existing `entries` map — general, and the wrong shape for a byte buffer.

**Not decided here.** The implementer measures the mutation-heavy path (`append` in a loop) on both
candidates before choosing, and records the figure. `String`'s own 112 loud rows suggest the byte
machinery may already exist; that was not checked and checking it is the first task.

**D81 — the phase's id and the `method-owner` column.** `class-set.txt` gives `MutableBuffer`
`method-owner` `5c`, and `5c` is in `CLOSED_PHASES`. Either this phase re-owns those rows to `5e`,
or it leaves them under a closed phase and relies on the method-body table alone. Re-owning is
preferred — the owner column is what `gate_table_c.rs` reads, and a phase that moves rows should own
them — but it changes a committed table and needs its own control.

**D82 — how far the class goes.** Moritz chose the 5c-follow-up option, whose text was *"fixes the
class rather than the five methods `File` happens to need"*. So the target is all 51 instance rows,
not the five on `File`'s path. A task may still stage them, and the gate below is stated per-row so
a partial landing is visible rather than hidden.

---

## 3. The gate

**Every one of `MutableBuffer`'s 51 instance rows moves `loud` → `answers` in
`corpus/method-bodies.txt`, and none moves to `diverge`.** That table already runs in the gate and
already refuses a `loud` → `diverge` move, so the new requirement is the direction it does not
currently enforce.

**Plus the reachability the phase exists for**: `.File~new('/tmp')~name` answers byte-identically to
the oracle. That is the one-line proof that the trap is gone, and it belongs in `corpus/lang/` as a
program the differential runs, not in a test binary.

---

## 4. The hollowness check, which is the phase's second deliverable

**No gate in this tree distinguishes "the row answers" from "the method works",** and the same
blindness has now cost two phases' scoping — table C's `hasMethod` readbacks in 5c, and
`method-bodies.txt`'s raising-constructor rows in Phase 7's survey. Both were found by a human
opening a probe.

The discriminator is available and cheap: **a row may not be `answers` if neither side reached a
body.** For the raising-constructor shape that is exactly "the oracle's stdout is empty" — the
probe's `say` lines never ran, so agreement is agreement on the raise. `File`'s 50 instance rows are
that, and `corpus/gate-tables/methods/file__instance.rex` says so in its own header.

**The verdict such a row should carry is `unanswered`, not `answers`** — the value
`gate_tables::UNANSWERED` already exists for the case where a probe the oracle never reached must
not read as satisfied.

Shown to fail: make a class's construction expression raise, and confirm its rows move to
`unanswered` rather than staying `answers`.

**This will move rows in the committed table, and that is the point.** The count is not predicted
here; whoever lands it reports the measured before/after and names every class that moves.

---

## 5. What this phase does not do

* **Not `File`, not `Stream`, not any Phase 7 work.** It ends when `File` can construct; making
  `File`'s own methods answer is Phase 7's.
* **Not the other 571 loud rows under `5c`.** `String` 112, `Queue` 34, `Package` 33 and the rest
  stay as they are. Some are recorded decisions — Task 3 left `String`'s operator-message rows and
  Task 5 left `Package`'s — and this phase does not reopen them.
* **Not a change to the method-body gate's drift rule.** `loud` → `answers` stays ungated as
  progress; §4 adds a verdict, not a requirement that rows advance.
* **Not `SAY` through `.OUTPUT`.** D-P7-1 deferred it and this phase does not touch it.

## 6. Risks

* **The hollowness check may move rows on classes nobody is looking at**, and a large move would be
  a second finding rather than a failure. It should be landed and measured *before* the
  `MutableBuffer` work, so the two are not confounded in one table diff.
* **Re-owning rows to `5e` (D81) touches a committed table** whose drift is itself gated. It needs
  the same control discipline as any table change.
* **`getBufferSize`'s 256 default is measured on this host only.** Whether it is a compile-time
  constant or derived was not read out of `MutableBufferClass.cpp`, and the implementer should read
  it rather than hardcode what one probe showed.
