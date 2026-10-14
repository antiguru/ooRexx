# Phase 9b1 scout: Deviations re-judged, and string/number identity measured

Read-only scout, 2026-10-14, tree `ce3793971`. Oracle: `/home/moritz/dev/repos/ooRexx/build/bin/rexx`
(`rexx -v`: Open Object Rexx Version 5.3.0 r0). Crate: `rust/target/release/rexx-run` built 2026-10-13
22:19, after the last commit touching `rust/crates` (`1590a1cea`, 22:18:19). Every run: a fresh empty
directory, `memcap 4G`, the oracle additionally under `ulimit -v 1048576`. Each probe ran once per side
unless stated (they are single-threaded and allocate deterministically; `wr2` ran three times a side and
answered the same each time). Probe sources are in the appendix, because the scratchpad is in `/tmp`.

Principle (rust/CLAUDE.md, "Specialization is internal"): a difference that only the representation
explains is a defect unless Moritz ruled a Deviation for it.

## Task 1: every Deviation, sorted

(A) a representation leak 9b1 should remove; (B) an oracle bug not copied; (C) an unrelated licence.

| # | Class | Reason |
|---|---|---|
| 31 | **A** | Refused only because a short string and a SmallInt have no heap object, and heap strings do not carry the oracle's identities. That is the representation, verbatim. |
| 4 | **A, partly** (rescope; keep a narrow residue as C) | Its "we merge where the oracle separates" half is an inline/SmallInt artefact, so the principle calls it a leak. Its "we separate where the oracle merges" half (the no-op set: `left`/`right`, `~string`, `\|\|''`, whole-template PARSE) is upstream accident by its own argument, so it stays licensed. Task 2 found one more leak this row does not name: weak references (below). |
| 0 | B | Oracle `traceIndent` counter defect. |
| 1 | C | Stem `DO OVER` order. That is an unspecified order, Deviation 33's principle; it is not a representation leak, because order is not owed. |
| 2 | B | Oracle SIGSEGVs past 100,000 flat terms. 11.1 is reported in its place. |
| 3 | B | The oracle matches through its own NUL terminator. |
| 5 | n/a | Withdrawn. |
| 6 | C | GC ordering licence (the oracle's save stack). Unspecified timing, not representation. |
| 7 | C | Interleaving of two tracing threads is unspecified. |
| 8 | n/a | Withdrawn. |
| 9 | B | After a collection the oracle reads reclaimed memory. |
| 10-23 | C | Signal and scheduler design (rulings P23-P84, spec 2026-09-29). These are architecture choices about activities and threads. None is an object's representation. |
| 24 | B | The oracle SIGSEGVs or hangs on a kept context. |
| 25 | B | `DO ... TO` compares with `TheTrueObject` by pointer (Moritz 2026-10-08). See the note under Task 2: the identity model must keep `.true` distinct from a computed `1` anyway. |
| 26 | B | The oracle's block stack double-removes. |
| 27 | B | An unchecked cast in `Activity::traceInput`. |
| 28 | B | The oracle reads through another class's layout. |
| 29 | B | The oracle stores the evaluation stack (`HashCollection.cpp:1015`). |
| 30 | B, with one A-shaped clause | Reading `.nil` through the string layout is an oracle bug. Two clauses inside it lean on "not built". The first is a one-variable PARSE template assigning the `.nil` object, which the oracle defines and the crate refuses. The second is the 88.909 consumers, refused "since this crate builds no frames for the REXX package's methods". Neither is caused by a string's representation, so neither is a 9b1 target. The inventory scout should still list them as refusals of the "does not build X" shape. |
| 32 | B | Damage to a shared behaviour or class. The metaclass-`NEW` refusal answers a case the oracle corrupts. |
| 33 | C | Hash order is unspecified by Moritz's own ruling, so it is a licence and not an oracle bug. |
| 34 | B | `normalizePathName`'s `previousSlash` defect. |
| 35 | B (+C) | The oracle's `HANDLE:` never reaches its descriptor. Refusing other descriptors is the unsafe-code policy (C). |

### What removing each A entry requires

* **31.** `OBJECTNAME=` has to work on every string and number. A name set on a held value must stick on
  later reads of that variable, on an alias, and through an argument (probe `on8`: the oracle names the
  caller's value from inside a routine). `~copy` must carry the name (O2f). This is the queued note's
  "hardest case": the value is inline in a slot and the naming happens through a copy of it. There are
  only two ways to do it. Either identity is a function of the value for anything never materialized,
  and materializing interns one object per value (a value-keyed table, strong while it holds state). Or
  every value carries a provenance identity from birth. Task 2 says which oracle answers each choice
  gives up.
* **4.** Rescope the row. Keep, as an unspecified-observable licence in Deviation 33's register:
  *whether two separately produced equal strings or numbers are one object*, and the no-op set. Remove
  from it everything that makes the crate answer inconsistently about **one** value it holds. Today that
  is the weak-reference defect below, plus Deviation 31. Also extend literal interning to the
  oracle's `commonString` scope (probe C17), which needs no representation change.
* **New, not yet a Deviation: a weak reference to an inline value is cleared while the value is still
  held.** `rexx-core/src/heap.rs`, the weak pass after marking: `self.resolve(target).is_some_and(...)`
  is `None` for a non-heap `ObjRef`, so every inline referent counts as dead at every collection.
  Measured (`wr3`): `c = 'ab'||'c'; w = .WeakReference~new(c)`, two forced collections, `c` still
  assigned. The oracle answers *not cleared* and `w~value` is `c`. The crate answers *cleared*. The
  same happens for `'abc'`, `1000+1` and the literal `1000`. There is no Deviation for it. Removing it
  requires deciding when an inline referent is dead, which is the same decision as 31's.

## Task 2: oracle identity, measured, with the crate beside it

`1` means the same object. `ih` stands for `~identityHash`, compared with `==`, a strict string compare,
so NUMERIC DIGITS cannot merge two hashes.

### Literals, packages, INTERPRET (probe `idA`, `idC`)

| Probe | Oracle | Crate |
|---|---|---|
| A01 two equal long literals, one package | 1 | 1 |
| A02 two equal short (`'abc'`) literals, one package | 1 | 1 |
| A03 long literal evaluated in a loop | 1 | 1 |
| A04 same long literal in a `::ROUTINE` of the package | 1 | 1 |
| A05 same long literal in a second package (`::requires`) | 0 | 0 |
| **A06 same short literal in a second package** | **0** | **1** |
| A07 long literal vs the same literal inside INTERPRET | 0 | 0 |
| A08 two equal literals in one INTERPRET string | 1 | 1 |
| A09 one INTERPRET string executed twice | 0 | 0 |
| A21 literal `'ABC'` vs unset symbol `abc` | 1 | 1 |
| **C17 literal `'ABCDEFGHIJ'` vs unset symbol `abcdefghij`** | **1** | **0** |

Oracle mechanism. `LanguageParser` has one `strings` pool (`commonString`, LanguageParser.cpp:2269) and
one `literals` table per parser instance. A parser instance is a package, or one INTERPRET execution
(LanguageParser.cpp:788-792). Symbol names and literal text share the pool, which is why C17 is 1. The
`literals` table is keyed by text, so a constant symbol and a quoted literal with the same text are one
retriever (N06, N17 below).

### Computed strings (probe `idA`, `idC`)

| Probe | Oracle | Crate |
|---|---|---|
| **A10 `'a'\|\|'b'` vs literal `'ab'`** | **0** | **1** |
| A11 long computed vs equal literal | 0 | 0 |
| **A12 two `'a'\|\|'b'`** | **0** | **1** |
| **A13 long literal `\|\| ''`** | **1** | **0** |
| **A14 long computed `\|\| ''`** | **1** | **0** |
| **A15 long computed `~string`** | **1** | **0** |
| A16 `~copy` | 0 | 0 |
| A17 alias `b = a` | 1 | 1 |
| A18 / A19 `x` read twice (long / short computed) | 1 | 1 |
| **C18 `parse value <literal> with p`, p vs literal** | **1** | **0** |
| **C19 `parse arg pa` of a literal argument, vs literal** | **1** | **0** |
| C20 `arg(1)` of a literal argument, vs literal | 1 | 1 |

A13 and A14 come from `RexxString::concat` returning `this` for an empty operand (StringClass.cpp:1328).
A13-A15, C18 and C19 are Deviation 4's no-op family.

### Numbers (probe `idA`)

| Probe | Oracle | Crate |
|---|---|---|
| N01 `1` vs `1`; N02 `1` vs `1+0`; N03 `5+0` vs `3+2`; N15 `5` vs `5+0` | 1 | 1 |
| **N04 `101+0` vs `100+1`** | **0** | **1** |
| N05 literal `1000` twice; N08 literal `123456789012` twice; N13 `1.5` twice | 1 | 1 |
| N06 symbol `1000` vs quoted `'1000'`; N17 `'5'` vs `5` | 1 | 1 |
| **N07 `1000+0` twice** | **0** | **1** |
| N09 `7*1000` read twice; N10 its alias | 1 | 1 |
| **N11 `1000` vs `1000` in a second package** | **0** | **1** |
| N12 `50` vs `50` in a second package | 1 | 1 |
| N14 `1.5+0` twice | 0 | 0 |
| **N16 `1000` vs `1000 + 0`** | **0** | **1** |
| N18 `1000` vs `1000~string`; N19 `1001` (computed) vs its `~string`; N20 vs `\|\|''` | 0 | 0 |
| N21 two `~string` of one computed `1001` | 1 | 1 |

Oracle mechanism. A constant symbol that is an integer within 9 digits becomes a `RexxInteger`, interned
in the package's `literals` table (LanguageParser.cpp:2361-2402). Integers -10..100 are process-wide
singletons (`IntegerClass.hpp:185-192`), whatever produced them. Any other arithmetic result is a fresh
object. A `RexxInteger`'s string form (`~string`, `\|\|`, a Directory key) is a separate `stringrep`
object, cached on the integer (IntegerClass.cpp:206, 239).

### identityHash stability (probe `idC`). Every row agrees, oracle 1 and crate 1

C01-C16. A long computed string, a short computed string and a computed `1001`. Each was read twice,
assigned to another variable, passed to an internal routine, passed to a routine in a second package,
passed to a method, passed and returned, stored in a stem, an array or a directory and read back. The
hash never changed on either side.

### OBJECTNAME= (probes `on1`-`on8`, `tr1`). The crate refuses at the first `OBJECTNAME=`, rc 120, in every one

The oracle's answers. `NAMED` means the name stuck. `a String` means the object is a different one.

| Named value | same var | alias | equal literal, same pkg | computed equal | 2nd package | other |
|---|---|---|---|---|---|---|
| long literal (`on1`) | NAMED | NAMED | NAMED | a String | a String | `\|\|''` NAMED, `::ROUTINE` literal NAMED, INTERPRET literal a String |
| long computed (`on2`) | NAMED | NAMED | a String | a String | | passed and returned NAMED, **`~copy` NAMED** |
| short literal `'abc'` (`on3`) | NAMED | NAMED | NAMED | a String | a String | |
| short computed (`on4`) | NAMED | NAMED | a String | a String | | |
| literal `1000` (`on5`) | NAMED | | `1000` NAMED, `'1000'` NAMED | `1000+0` a String | a String | `~string`, `\|\|''` a String |
| literal `50` (`on6`) | NAMED | | `'50'` NAMED | `49+1` NAMED | **NAMED** | **`length(copies('x',50))` NAMED** |
| computed `1001` (`on7`) | NAMED | NAMED | | `1000+1` a String, `n+0` a String | | `~string` a String |
| named inside a routine through `use arg` (`on8`): long, `1001`, short | NAMED in the caller | | | | | |
| `x = 1` named (`tr1`) | | | | `(0+1)` ONE, `length('a')` ONE | | `.true` and `(1=1)` a String |

So naming a small integer names that integer for the whole process, across packages and builtins.
`.true` is a separate object from every `1` (Deviation 4 says so too). `~copy` copies the object
variables, the name included.

### setMethod and enhanced (probes `sm1`, `sm2`). Every row agrees

A plain string cannot be reached by `setMethod` from Rexx on either side. From a program it is 97.2
"cannot accept private message SETMETHOD from this context", rc 159, on both. `.String~inherit(...)` is
98.985 on both, so no user method can run with a plain string as `self`. An enhanced string
(`.String~enhanced(d, text)`) behaves the same on both sides for every row E01-E20, long, short and
`1000`. Its methods survive `\|\| ''`, alias, argument passing and `~copy`. They are lost on `~string`,
`upper` and `strip`, and on an equal literal. A `setMethod` it does on `self` is seen through an alias.
So among plain strings and numbers, `OBJECTNAME=` is the only identity-bearing mutation a program can
reach. `setMethod` reaches only enhanced strings, which the crate already gives a heap object.

### WeakReference (probes `wr1`, `wr2` x3, `wr3`)

Two forced collections, each after 50 allocating clauses (beyond the oracle's save-stack depth of 10).

| Referent | Oracle cleared? | Crate cleared? |
|---|---|---|
| W1 long computed, dropped | 1 | 1 |
| W2 long literal, dropped (the package still holds it) | 0 | 0 |
| W3 short computed, dropped | 1 | 1 |
| W5 `1000+1`, dropped | 1 | 1 |
| **W6 literal `1000`, dropped (the package holds it)** | **0** | **1** |
| **W7 literal `'abc'`, dropped (the package holds it)** | **0** | **1** |
| W4/W11 `5+0`, dropped; W10 literal `5`, dropped | **1** | 1 |
| W8 long computed, still held | 0 | 0 |
| **W13 short computed, still held** | **0** | **1** |
| **W14 `'abc'`, W15 `1000+1`, W16 `1000`, still held** | **0** | **1** |
| **W18 `w~value` is the held value** | **1** | **0** |

The W4/W10/W11 oracle answer is an oracle bug. `5` survives and is the same object
(`wr2` prints `same object 1`, then `5+0` again), but `checkWeakReferences` clears any referent that is
not marked (RexxMemory.cpp:469), and the -10..100 cache lives in the image's old space, which a normal
collection does not mark. The crate agrees by accident: the inline cause below.

**W13-W16 and W18 are a crate defect** (the heap.rs weak pass, Task 1).

### Collections (probe `tb1`)

| Probe | Oracle | Crate |
|---|---|---|
| K01 IdentityTable, long literal key, the same literal | 1 | 1 |
| K02 long literal key, computed equal | 0 | 0 |
| K03 short literal key, the same literal | 1 | 1 |
| **K04 short literal key, computed equal** | **0** | **1** |
| K05 short computed key, the same variable | 1 | 1 |
| **K06 short computed key recomputed; K07 vs literal** | **0** | **1** |
| K08 `1000+1` key, the same variable | 1 | 1 |
| **K09 `1000+1` recomputed; K10 vs literal `1001`** | **0** | **1** |
| K11 `5+0` key, literal `5` | 1 | 1 |
| K13 Table, computed key, literal lookup (equality) | 1 | 1 |
| K14 Table / K15 Directory: the stored index is the inserted object | 1 | 1 |
| **K16 Directory index from `1001`: is `n`** | **0** | **1** |
| **K17 Directory index from `1001`: is `n~string`** | **1** | **0** |
| K18 Table index from `1001` is `n` | 1 | 1 |
| K19/K20 Set of a computed and an equal literal: items | 1 | 1 |
| K21 Set keeps the first object | 1 | 1 |

Equality-keyed collections agree everywhere, as expected: they do not ask about identity. The
IdentityTable differences are exactly the computed-merge rows above, seen through a collection.
K16/K17 come from the Integer/`stringrep` split (a Directory index must be a string).

### The minimal rule that matches every probe

1. **Literal interning per compiled unit.** One object per text per package and per INTERPRET
   execution. Quoted literals, constant symbols and unset-symbol names share it. An integer constant
   within 9 digits is the integer form.
2. **Integers -10..100 are process-wide singletons.**
3. **Every other produced value is a fresh object.** Two computations never share an object.
4. **Identity flows** through assignment, arguments, returns, `use arg`/`arg()` and collection storage.
5. **The oracle's no-op set answers its receiver**: `\|\|` with an empty operand, `~string` on a string,
   a whole-string PARSE target, and Deviation 4's table (`strip`, `right`, `changestr` no-ops, but not
   `left`, `substr`, `translate`).
6. **An integer's string form is a separate object**, cached per integer.
7. **A weak reference to a -10..100 integer is cleared at every collection.**

Rules 5, 6 and 7 are oracle internals or bugs. Rule 7 is RexxMemory.cpp:469 with the old-space image.
Rule 6 is a class (`RexxInteger`) a program cannot even name: `~class~id` answers `String`. Rule 5 is
Deviation 4's "upstream accident". **License them as unspecified**, in Deviation 33's register.

Rule 2's process-wide reach (on6, tr1) is also an internal. The crate's SmallInt merges a superset of it,
so every rule-2 probe already agrees.

Rule 3 is the expensive one. It is the only rule behind every remaining divergence: A06, A10, A12, N04,
N07, N11, N16, K04, K06, K07, K09, K10, and on3/on4/on5/on7's "computed equal" columns once
`OBJECTNAME=` works. To match it, an inline short string or SmallInt would need a per-production
identity, which is exactly what inline representation exists to avoid (Deviation 4: entry 59 bought
6.4% on `strings`). **Recommend licensing it as well**: *whether two separately produced equal strings
or numbers are the same object is unspecified*. This is the oracle's own construction-path accident,
which Deviation 4 already argues.

What the crate **must** then guarantee, because each of these is consistency about one held value, not
a merge question:
* identity, `objectName`, weak-reference liveness and IdentityTable membership agree across every read,
  alias, argument, return and collection round trip of one value (C01-C16 hold today; on8, W13-W16 and
  W18 do not);
* `~copy` carries `objectName`;
* literals are per compiled unit (A05, A07, A09 hold for long literals; extend to C17's shared
  symbol-name pool, and decide A06/N11 for short literals and integers under the licence);
* `.true`/`.false` stay distinct from every computed `1`/`0` (tr1). Deviation 25 depends on it.

Under that licence, the own rule the open design decision describes satisfies everything that is left:
*every value can be materialized, identity stable once materialized, and an unmaterialized immutable
value's identity is its value*. One consequence must be ruled explicitly, because it is a visible answer
and not just an internal: naming `'abc'` would name every equal unmaterialized `'abc'` in the process
(on3 "computed equal", on4 "equal literal"). The oracle does exactly that for -10..100 already (on6).
The weak-reference question gets the same answer: an inline referent is live while the value can still
be produced. The simplest sound reading is "never cleared". The other reading is "cleared once the
interned materialized object is unreachable". Either way it cannot be today's "always cleared".

## Appendix: probe sources

Runner (`run.sh NAME...`): runs `probes/NAME.rex` on each side from its own `mktemp -d`. The oracle runs
as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout 60 memcap 4G
.../build/bin/rexx FILE )`, the crate as `timeout 60 memcap 4G rust/target/release/rexx-run FILE`, with
stdout, stderr and rc kept separately. No shape from `rust/corpus/oracle-crashes.txt` was run.

### `idA.rex`

```rexx
-- identity of strings and numbers; 1 = same object
a = 'a longer literal string'; b = 'a longer literal string'
say 'A01 two equal long literals, one package    ' (a~identityHash == b~identityHash)
a = 'abc'; b = 'abc'
say 'A02 two equal short literals, one package   ' (a~identityHash == b~identityHash)
do i = 1 to 2; l.i = 'a longer literal string'; end
say 'A03 long literal in a loop                  ' (l.1~identityHash == l.2~identityHash)
a = 'a longer literal string'; b = rlit()
say 'A04 long literal, same text in ::routine    ' (a~identityHash == b~identityHash)
a = 'a longer literal string'; b = otherlit()
say 'A05 long literal, same text in 2nd package  ' (a~identityHash == b~identityHash)
a = 'abc'; b = othershort()
say 'A06 short literal, same text in 2nd package ' (a~identityHash == b~identityHash)
a = 'a longer literal string'; interpret "b = 'a longer literal string'"
say 'A07 long literal vs INTERPRET literal       ' (a~identityHash == b~identityHash)
interpret "a = 'a longer literal string'; b = 'a longer literal string'"
say 'A08 two literals in one INTERPRET           ' (a~identityHash == b~identityHash)
do i = 1 to 2; interpret "l.i = 'a longer literal string'"; end
say 'A09 one INTERPRET run twice                 ' (l.1~identityHash == l.2~identityHash)
a = 'ab'; b = 'a' || 'b'
say 'A10 short computed vs literal               ' (a~identityHash == b~identityHash)
a = 'a longer literal string'; b = 'a longer literal' || ' string'
say 'A11 long computed vs literal                ' (a~identityHash == b~identityHash)
a = 'a' || 'b'; b = 'a' || 'b'
say 'A12 two short computed                      ' (a~identityHash == b~identityHash)
a = 'a longer literal string'; b = a || ''
say 'A13 long literal || empty                   ' (a~identityHash == b~identityHash)
a = 'a longer' || ' computed one'; b = a || ''
say 'A14 long computed || empty                  ' (a~identityHash == b~identityHash)
a = 'a longer' || ' computed one'; b = a~string
say 'A15 long computed ~string                   ' (a~identityHash == b~identityHash)
a = 'a longer' || ' computed one'; b = a~copy
say 'A16 long computed ~copy                     ' (a~identityHash == b~identityHash)
a = 'a longer' || ' computed one'; b = a
say 'A17 alias b = a                             ' (a~identityHash == b~identityHash)
a = 'a longer' || ' computed one'
say 'A18 x == x long computed                    ' (a~identityHash == a~identityHash)
a = 'ab' || 'c'
say 'A19 x == x short computed                   ' (a~identityHash == a~identityHash)
a = 'abc'; b = abc
say 'A20 literal vs unset symbol ABC (upper)     ' (a~identityHash == b~identityHash)
a = 'ABC'; b = abc
say 'A21 literal ABC vs unset symbol abc         ' (a~identityHash == b~identityHash)
a = 1; b = 1
say 'N01 1 vs 1                                  ' (a~identityHash == b~identityHash)
a = 1; b = 1 + 0
say 'N02 1 vs 1+0                                ' (a~identityHash == b~identityHash)
a = 5 + 0; b = 3 + 2
say 'N03 5+0 vs 3+2                              ' (a~identityHash == b~identityHash)
a = 101 + 0; b = 100 + 1
say 'N04 101+0 vs 100+1                          ' (a~identityHash == b~identityHash)
a = 1000; b = 1000
say 'N05 literal 1000 vs 1000                    ' (a~identityHash == b~identityHash)
a = 1000; b = '1000'
say 'N06 symbol 1000 vs quoted 1000              ' (a~identityHash == b~identityHash)
a = 1000 + 0; b = 1000 + 0
say 'N07 1000+0 twice                            ' (a~identityHash == b~identityHash)
a = 123456789012; b = 123456789012
say 'N08 big literal twice                       ' (a~identityHash == b~identityHash)
a = 7 * 1000
say 'N09 x == x of 7*1000                        ' (a~identityHash == a~identityHash)
a = 7 * 1000; b = a
say 'N10 alias of 7*1000                         ' (a~identityHash == b~identityHash)
a = 1000; b = otherint()
say 'N11 1000 vs 1000 in 2nd package             ' (a~identityHash == b~identityHash)
a = 50; b = otherfifty()
say 'N12 50 vs 50 in 2nd package                 ' (a~identityHash == b~identityHash)
a = 1.5; b = 1.5
say 'N13 1.5 vs 1.5                              ' (a~identityHash == b~identityHash)
a = 1.5 + 0; b = 1.5 + 0
say 'N14 1.5+0 twice                             ' (a~identityHash == b~identityHash)
a = 5; b = 5 + 0
say 'N15 literal 5 vs 5+0                        ' (a~identityHash == b~identityHash)
a = 1000; b = a + 0
say 'N16 1000 vs 1000 + 0                        ' (a~identityHash == b~identityHash)
a = '5'; b = 5
say 'N17 quoted 5 vs symbol 5 (literal table)    ' (a~identityHash == b~identityHash)
a = 1000; b = a~string
say 'N18 1000 vs its ~string                     ' (a~identityHash == b~identityHash)
a = 1000 + 1; b = a~string
say 'N19 1001 computed vs its ~string            ' (a~identityHash == b~identityHash)
a = 1000 + 1; b = a || ''
say 'N20 1001 computed vs ||empty                ' (a~identityHash == b~identityHash)
a = 1000 + 1; b = a~string; c = a~string
say 'N21 two ~string of computed 1001            ' (b~identityHash == c~identityHash)
exit
rlit: return 'a longer literal string'
::requires 'idB.rex'
```

### `idB.rex`

```rexx
::routine otherlit public
  return 'a longer literal string'
::routine othershort public
  return 'abc'
::routine otherint public
  return 1000
::routine otherfifty public
  return 50
::routine passthru public
  return arg(1)
::routine hashof public
  return arg(1)~identityHash
```

### `idC.rex`

```rexx
a = 'a longer' || ' computed one'
h1 = a~identityHash; h2 = a~identityHash
say 'C01 long computed, read twice               ' (h1 == h2)
b = a
say 'C02 after b = a                             ' (h1 == b~identityHash)
say 'C03 passed as argument (hashof in 2nd pkg)  ' (h1 == hashof(a))
say 'C04 passed and returned                     ' (h1 == passthru(a)~identityHash)
say 'C05 internal routine arg                    ' (h1 == ih(a))
say 'C06 method arg                              ' (h1 == .k~new~ih(a))
s = 'ab' || 'c'
g1 = s~identityHash
say 'C07 short computed, read twice              ' (g1 == s~identityHash)
say 'C08 short computed passed                   ' (g1 == hashof(s))
say 'C09 short computed passed and returned      ' (g1 == passthru(s)~identityHash)
n = 1000 + 1
k1 = n~identityHash
say 'C10 computed 1001 read twice                ' (k1 == n~identityHash)
say 'C11 computed 1001 passed                    ' (k1 == hashof(n))
say 'C12 computed 1001 passed and returned       ' (k1 == passthru(n)~identityHash)
x.1 = a
say 'C13 stored in stem and read back            ' (h1 == x.1~identityHash)
arr = .array~of(a)
say 'C14 stored in array and read back           ' (h1 == arr[1]~identityHash)
arr2 = .array~of(n)
say 'C15 1001 stored in array and read back      ' (k1 == arr2[1]~identityHash)
d = .directory~new; d['K'] = n
say 'C16 1001 stored in directory, read back     ' (k1 == d['K']~identityHash)
a = 'ABCDEFGHIJ'; b = abcdefghij
say 'C17 long literal vs unset symbol same text  ' (a~identityHash == b~identityHash)
parse value 'a longer literal string' with p
q = 'a longer literal string'
say 'C18 parse value of literal, whole template  ' (p~identityHash == q~identityHash)
call r1 'a longer literal string'
exit
r1: parse arg pa
  say 'C19 parse arg of literal vs literal         ' (pa~identityHash == 'a longer literal string'~identityHash)
  say 'C20 arg(1) of literal vs literal            ' (arg(1)~identityHash == 'a longer literal string'~identityHash)
  return
ih: return arg(1)~identityHash
::requires 'idB.rex'
::class k
::method ih
  use arg o
  return o~identityHash
```

### `on1.rex`

```rexx
s = 'a longer literal string'; s~objectName = 'NAMED'
say 'O1a same var        ' s~objectName
t = s
say 'O1b copy var        ' t~objectName
say 'O1c equal literal   ' 'a longer literal string'~objectName
say 'O1d computed equal  ' ('a longer literal' || ' string')~objectName
say 'O1e s || empty      ' (s || '')~objectName
say 'O1f 2nd package lit ' otherlit()~objectName
call r
interpret "say 'O1h interpret lit  ' 'a longer literal string'~objectName"
exit
r: say 'O1g internal rtn lit' 'a longer literal string'~objectName; return
::requires 'idB.rex'
```

### `on2.rex`

```rexx
s = 'a longer' || ' computed one'; s~objectName = 'NAMED'
say 'O2a same var        ' s~objectName
t = s
say 'O2b copy var        ' t~objectName
say 'O2c equal literal   ' 'a longer computed one'~objectName
say 'O2d computed equal  ' ('a longer' || ' computed one')~objectName
say 'O2e passthru        ' passthru(s)~objectName
say 'O2f s~copy          ' s~copy~objectName
::requires 'idB.rex'
```

### `on3.rex`

```rexx
s = 'abc'; s~objectName = 'NAMED'
say 'O3a same var        ' s~objectName
t = s
say 'O3b copy var        ' t~objectName
say 'O3c equal literal   ' 'abc'~objectName
say 'O3d computed equal  ' ('ab' || 'c')~objectName
say 'O3e 2nd package lit ' othershort()~objectName
::requires 'idB.rex'
```

### `on4.rex`

```rexx
s = 'ab' || 'c'; s~objectName = 'NAMED'
say 'O4a same var        ' s~objectName
t = s
say 'O4b copy var        ' t~objectName
say 'O4c equal literal   ' 'abc'~objectName
say 'O4d computed equal  ' ('ab' || 'c')~objectName
```

### `on5.rex`

```rexx
n = 1000; n~objectName = 'NAMED'
say 'O5a same var        ' n~objectName
say 'O5b equal literal   ' 1000~objectName
say 'O5c quoted literal  ' '1000'~objectName
say 'O5d 1000+0          ' (1000+0)~objectName
say 'O5e 2nd package     ' otherint()~objectName
say 'O5f n~string        ' n~string~objectName
say 'O5g n || empty      ' (n || '')~objectName
::requires 'idB.rex'
```

### `on6.rex`

```rexx
n = 50; n~objectName = 'NAMED'
say 'O6a same var        ' n~objectName
say 'O6b 49+1            ' (49+1)~objectName
say 'O6c 2nd package 50  ' otherfifty()~objectName
say 'O6d quoted 50       ' '50'~objectName
say 'O6e length of 50 ch ' length(copies('x',50))~objectName
::requires 'idB.rex'
```

### `on7.rex`

```rexx
n = 1000 + 1; n~objectName = 'NAMED'
say 'O7a same var        ' n~objectName
m = n
say 'O7b copy var        ' m~objectName
say 'O7c 1000+1 again    ' (1000+1)~objectName
say 'O7d n+0             ' (n+0)~objectName
say 'O7e n~string        ' n~string~objectName
```

### `on8.rex`

```rexx
-- named via a method running on the value: self
s = 'a longer' || ' computed one'
call nameit s
say 'O8a after routine named arg' s~objectName
n = 1000 + 1
call nameit n
say 'O8b after routine named int' n~objectName
s = 'ab' || 'c'
call nameit s
say 'O8c after routine named short' s~objectName
exit
nameit: use arg v; v~objectName = 'NAMED'; return
```

### `tr1.rex`

```rexx
x = 1; x~objectName = 'ONE'
say 'T1 .true~objectName  ' .true~objectName
say 'T2 (1=1)~objectName  ' (1=1)~objectName
say 'T3 (0+1)~objectName  ' (0+1)~objectName
say 'T4 length(a)         ' length('a')~objectName
```

### `sm1.rex`

```rexx
s = 'a longer literal string'
s~setMethod('FOO', 'return "foo"')
say 'SM1 after setMethod from program' s~hasMethod('FOO')
```

### `sm2.rex`

```rexx
d = .directory~new
d['FOO'] = 'return "foo ran"'
d['DOSET'] = 'self~setMethod("BAR", "return ''bar ran''")'
e = .String~enhanced(d, 'a longer literal string')
say 'E01 e~foo                    ' e~foo
say 'E02 e~class~id               ' e~class~id
say 'E03 e == literal             ' (e == 'a longer literal string')
say 'E04 (e||empty)~hasMethod     ' (e || '')~hasMethod('FOO')
say 'E05 e~string~hasMethod       ' e~string~hasMethod('FOO')
say 'E06 e~copy~hasMethod         ' e~copy~hasMethod('FOO')
say 'E07 literal~hasMethod        ' 'a longer literal string'~hasMethod('FOO')
say 'E08 e~upper~hasMethod        ' e~upper~hasMethod('FOO')
say 'E09 strip(e)~hasMethod       ' strip(e)~hasMethod('FOO')
t = e
say 'E10 alias~hasMethod          ' t~hasMethod('FOO')
e~doSet
say 'E11 setMethod on self, e~bar ' e~bar
say 'E12 alias t~bar              ' t~bar
say 'E13 passthru(e)~bar          ' passthru(e)~bar
say 'E14 e~defaultName            ' e~defaultName
f = .String~enhanced(d, 'abc')
say 'E15 short enhanced f~foo     ' f~foo
say 'E16 (f||empty)~hasMethod     ' (f || '')~hasMethod('FOO')
say 'E17 abc literal~hasMethod    ' 'abc'~hasMethod('FOO')
g = .String~enhanced(d, 1000)
say 'E18 enhanced 1000 g~foo      ' g~foo
say 'E19 g+0~hasMethod            ' (g+0)~hasMethod('FOO')
say 'E20 1000~hasMethod           ' 1000~hasMethod('FOO')
::requires 'idB.rex'
```

### `wr1.rex`

```rexx
a = 'a longer' || ' computed one';  wa = .WeakReference~new(a)
b = 'a longer literal string';      wb = .WeakReference~new(b)
c = 'ab' || 'c';                    wc = .WeakReference~new(c)
d = 5 + 0;                          wd = .WeakReference~new(d)
e = 1000 + 1;                       we = .WeakReference~new(e)
f = 1000;                           wf = .WeakReference~new(f)
g = 'abc';                          wg = .WeakReference~new(g)
h = 'a longer computed' || ' kept'; wh = .WeakReference~new(h)
say 'before:' (wa~value == .nil) (wb~value == .nil) (wc~value == .nil) (wd~value == .nil) (we~value == .nil) (wf~value == .nil) (wg~value == .nil) (wh~value == .nil)
drop a b c d e f g
do i = 1 to 50; z = 'pad' || i || 'x'; end
call gc 'force'
do i = 1 to 50; z = 'pad' || i || 'y'; end
call gc 'force'
say 'W1 long computed, dropped   cleared:' (wa~value == .nil)
say 'W2 long literal, dropped    cleared:' (wb~value == .nil)
say 'W3 short computed, dropped  cleared:' (wc~value == .nil)
say 'W4 5+0, dropped             cleared:' (wd~value == .nil)
say 'W5 1000+1, dropped          cleared:' (we~value == .nil)
say 'W6 literal 1000, dropped    cleared:' (wf~value == .nil)
say 'W7 short literal, dropped   cleared:' (wg~value == .nil)
say 'W8 long computed, held      cleared:' (wh~value == .nil) h
say 'W9 wh~value is h            ' (wh~value~identityHash == h~identityHash)
```

### `wr2.rex`

```rexx
d = 5;                 wd = .WeakReference~new(d)
k = 5 + 0;             wk = .WeakReference~new(k)
say 'same object' (d~identityHash == k~identityHash)
drop d k
do i = 1 to 50; z = 'pad' || i || 'x'; end
call gc 'force'
do i = 1 to 50; z = 'pad' || i || 'y'; end
call gc 'force'
say 'W10 literal 5 (literal table + cache) cleared:' (wd~value == .nil)
say 'W11 5+0 (cache)                       cleared:' (wk~value == .nil)
say 'W12 5 still usable:' (5 + 0)
```

### `wr3.rex`

```rexx
c = 'ab' || 'c';  wc = .WeakReference~new(c)
g = 'abc';        wg = .WeakReference~new(g)
n = 1000 + 1;     wn = .WeakReference~new(n)
f = 1000;         wf = .WeakReference~new(f)
b = 'a longer literal string'; wb = .WeakReference~new(b)
do i = 1 to 50; z = 'pad' || i || 'x'; end
call gc 'force'
do i = 1 to 50; z = 'pad' || i || 'y'; end
call gc 'force'
say 'W13 short computed, STILL HELD   cleared:' (wc~value == .nil) c
say 'W14 short literal, STILL HELD    cleared:' (wg~value == .nil) g
say 'W15 1000+1, STILL HELD           cleared:' (wn~value == .nil) n
say 'W16 literal 1000, STILL HELD     cleared:' (wf~value == .nil) f
say 'W17 long literal, STILL HELD     cleared:' (wb~value == .nil)
say 'W18 wc~value is c                ' (wc~value~identityHash == c~identityHash)
```

### `tb1.rex`

```rexx
it = .IdentityTable~new
it['a longer literal string'] = 'L'
say 'K01 IdT long literal key, same literal    ' it~hasIndex('a longer literal string')
say 'K02 IdT long literal key, computed equal  ' it~hasIndex('a longer literal' || ' string')
it['abc'] = 'S'
say 'K03 IdT short literal key, same literal   ' it~hasIndex('abc')
say 'K04 IdT short literal key, computed equal ' it~hasIndex('ab' || 'c')
k = 'ab' || 'c'; it2 = .IdentityTable~new; it2[k] = 'S'
say 'K05 IdT short computed key, same var      ' it2~hasIndex(k)
say 'K06 IdT short computed key, recomputed    ' it2~hasIndex('ab' || 'c')
say 'K07 IdT short computed key, literal       ' it2~hasIndex('abc')
n = 1000 + 1; it3 = .IdentityTable~new; it3[n] = 'N'
say 'K08 IdT 1001 computed, same var           ' it3~hasIndex(n)
say 'K09 IdT 1001 computed, recomputed         ' it3~hasIndex(1000 + 1)
say 'K10 IdT 1001 computed, literal 1001       ' it3~hasIndex(1001)
it3[5 + 0] = 'F'
say 'K11 IdT 5+0, literal 5                    ' it3~hasIndex(5)
say 'K12 IdT items after 1001,1001 put         ' it3~items
t = .Table~new; key = 'a longer' || ' computed one'; t[key] = 'v'
say 'K13 Table computed key, literal lookup    ' t~hasIndex('a longer computed one')
say 'K14 Table stored key is inserted object   ' (t~allIndexes[1]~identityHash == key~identityHash)
d = .Directory~new; d[key] = 'v'
say 'K15 Dir stored key is inserted object     ' (d~allIndexes[1]~identityHash == key~identityHash)
d2 = .Directory~new; d2[n] = 'v'
say 'K16 Dir key from 1001: is n               ' (d2~allIndexes[1]~identityHash == n~identityHash)
say 'K17 Dir key from 1001: is n~string        ' (d2~allIndexes[1]~identityHash == n~string~identityHash)
t2 = .Table~new; t2[n] = 'v'
say 'K18 Table key 1001 is n                   ' (t2~allIndexes[1]~identityHash == n~identityHash)
st = .Set~new; st~put('ab' || 'c'); st~put('abc')
say 'K19 Set items short computed + literal    ' st~items
st2 = .Set~new; st2~put(key); st2~put('a longer computed one')
say 'K20 Set items long computed + literal     ' st2~items
say 'K21 Set kept first object                 ' (st2~allIndexes[1]~identityHash == key~identityHash)
```

### `inh.rex`

```rexx
s = 'a longer string than seven'
.String~inherit(.setter)
s~doSet
say s~foo
::class setter mixinclass object
::method doSet
  self~setMethod('FOO', 'return "foo ran"')
```

