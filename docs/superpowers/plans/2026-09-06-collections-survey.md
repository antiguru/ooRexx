# The collection classes — a survey before the phase is planned

Queued by Moritz on 2026-09-06 as the phase after 5f. This is measurement, not
a plan: it exists because the obvious way to scope the phase is from
`corpus/method-bodies.txt`'s `loud` count, and that count is wrong by a factor
of two in a direction that would repeat the Phase 5c `File` mistake.

Classes surveyed: `Array List Queue CircularQueue Bag Set Table IdentityTable
Relation Directory StringTable Stem Properties RexxQueue`.

## 1. What the table says, and what running says

479 instance rows across those classes. 289 read `loud`. The remaining 188 read
`answers` — and of those, **21 answered at rc 0**. The other 167 record a
nonzero rc on both sides:

| evidence | rows |
|---|---|
| `rc 163` | 136 |
| `rc 168` | 30 |
| `rc 165` | 1 |
| `rc 0` | 21 |

`method_bodies.rs` documents exactly this: "A method needing arguments is sent
none, so for such a row this is agreement about an arity error rather than
about a result." The harness is not wrong; the row means less than its verdict
name suggests. **So the unbuilt surface is not 289 of 479. It is closer to 458
of 479, and the table cannot tell which.**

### It is already hiding a live divergence

`Bag~put` and `Set~put` both read `answers` with evidence `rc 163`. Sent an
actual argument they are wrong:

```rexx
b = .Bag~new
b~put('x')
say b~items
```

Oracle rc 0, `1`. This crate rc 168, `Error 88.901: Missing argument; argument
index is required.` — `put` has been given `Table`'s two-argument signature
where `Bag` and `Set` take one. That row will read `answers` for as long as
nobody sends it an argument.

**The first task of the phase is therefore an instrument, not an
implementation**: re-probe every collection row with a real argument list, and
record which of the 188 `answers` rows survive it. Phase 5c's own lesson was
that existence-shaped evidence is satisfied by a shell; this is the same lesson
one layer in, where *arity*-shaped evidence is satisfied by a wrong signature.

## 2. The classes are in three tiers, and only one is a binding job

Probed with each class's simplest put/get against the oracle, both engines.

**State works today** — `Array`, `Directory`, `StringTable`. `a[1] = 'x'`
round-trips and `a~items` is 1; `d['k'] = 'v'` round-trips.

**Storage exists, methods are missing or wrong** — `Queue`, `List`, `Bag`,
`Set`. `Queue~queue` and `List~append` refuse with the Phase 5 not-implemented
message; `Bag~put`/`Set~put` are the divergence above.

**No backing storage at all** — `Table`, `IdentityTable`, `Relation`,
`Properties` answer "a message send to a value that is not a hash collection is
not implemented", and `CircularQueue` says the same for "not an array". The
constructor hands back an instance the accessors do not recognise.

That third tier is the `MutableBuffer` shape: **no method on those five can be
bound until they have state**, exactly as D80 had to give `MutableBuffer` its
`native: Option<Box<BufferState>>` before any of its 52 rows could move. The
storage work comes first and is not optional.

## 3. The leverage is real, but it is over names rather than bodies

The 289 `loud` rows carry only 62 distinct method names, and 34 of those appear
on three or more classes — `allItems`, `allIndexes`, `makeArray`, `index` and
`empty` on 14 each; `isEmpty`, `hasItem`, `hasIndex`, `remove`, `removeItem`
and `supplier` on 13; `items` and `of` on 12.

**A shared name is not a shared body.** `remove` is index-removal on an `Array`,
entry-removal on a `Directory`, and item-with-value removal on a `Relation`;
`index` and `hasIndex` mean different things on a `Bag`/`Set` than on a
`Table`. `of` is a class method. So the ratio flatters: 62 names is a bound on
the number of *cores*, not a count of them, and the survey has not established
how many shapes hide behind each name.

## 4. Open for Moritz

1. **Does the phase include the third tier's storage, or is that its own
   phase?** It is the Phase 5c follow-up's whole shape repeated over five
   classes, and that took tasks 0 through 4 for one class.
2. **The 13-class receiver sweep** the 5c follow-up left open (105 rows, moves
   0 today) overlaps this survey's classes and should be folded in or closed.
3. `Bag~put`/`Set~put` is a shipped divergence found by this survey rather than
   by a gate. It is small and self-contained; it does not have to wait for the
   phase.
