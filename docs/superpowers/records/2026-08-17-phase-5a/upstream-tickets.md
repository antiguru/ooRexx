# ooRexx defects to report upstream

Every finding below was measured on ooRexx 5.3.0, Linux x86_64, `build/bin/rexx` from a CMake
`RelWithDebInfo` build, under the wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx /abs/path/FILE )` run from a
fresh empty directory. Each is deterministic on that build; none has been reproduced on another
platform, and each report should say so.

**Duplicate search.** SourceForge's `bugs/search?q=` endpoint OR-matches its terms and returned
500-plus hits for every query, so it cannot answer "is this already filed". What was actually run is a
title scan over all 495 tickets in `rest/p/oorexx/bugs/?limit=1000`, filtered for `date`, `changestr`,
`pos`/`lastpos`, `trace`, array/`at`/subscript, and segfault/crash. **That scan can only miss a ticket
whose title does not name the area** -- which is exactly how #2018 was missed the first time, by
searching the symptom rather than the mechanism. Before filing, search each ticket's body for the C++
function named in its cause paragraph.

---

## Already filed: do not file again

* **The orphaned-`WHEN` SIGSEGV is SF #2018**, "Segfault instead of error message for incorrect select
  statement", open, filed 2025-05-16 by perolovjonsson. Our four-line plain-`SELECT` reproducer, the
  `EndIf.cpp:142` crash site and a verified guard were posted as comment #4 on 2026-07-30. Nothing to
  add unless the maintainers ask.
* **The missing `>>>` lines on a controlled loop's re-tested pass are SF #1850**, "TRACE issues",
  **status accepted**, filed 2022-11-21. Its second section describes exactly that symptom, with the
  same expected-output shape. See item 5 below, which is the *other* half of the same code path and is
  not in that ticket.
* `NUMERIC DIGITS` above 1000 exhausting memory is a resource-exhaustion hazard, not a defect. Do not
  file it.

---

## 1. `DATE('M','0','D')` segfaults: day zero passes the day-of-year range check

**Title.** `DATE` accepts day-of-year 0 and then reads `monthNames[-1]`, segfaulting on output style
`M`

**Description.**

    say date('M','0','D')

exits with SIGSEGV, rc 139, printing nothing. Measured deterministic, three runs of three.

The day-of-year range check in `RexxBuiltinFunctionDATE`
(`interpreter/expression/BuiltinFunctions.cpp:1163`) reads
`yearday < 0 || yearday > YEAR_DAYS + 1 || ...`. Day zero is one below the first valid day, but the
check only rejects strictly negative values, so `0` passes. `RexxDateTime::setDay`
(`interpreter/classes/support/RexxDateTime.cpp:457-475`) then computes `month = 0` from a zero day
count, and the `M` output style reads `monthNames[month - 1]` at `RexxDateTime.cpp:526` -- that is
`monthNames[-1]`, one element before the array, whose bytes form an invalid pointer that the
subsequent string read dereferences.

The other output styles reachable from the same `month == 0` state do not crash, and the contrast is
what identifies the faulting read: `D`, `B`, `W`, `F` and `T` read `monthStarts[-1]`, which on this
build holds `0`, so they return stable but wrong values (`D` gives `0`, `B` gives `739615`, `W` gives
`Wednesday`, `F` gives `63902736000000000`, `T` gives `1767139200`). `M` is the only style whose
out-of-bounds read is dereferenced as a pointer rather than used as an integer.

The fix is presumably to reject `yearday < 1` rather than `yearday < 0`, but note that the other five
styles are silently wrong on the same input today, so a range-check fix changes their answers too.

Not a duplicate of #1729, which is `invalid` and concerns the documented digit limits of the second
argument.

---

## 2. `StringUtil::pos` searches one position past its window, and `CHANGESTR` segfaults on the
   past-the-end case

**Title.** `POS`'s fourth argument does not bound the search: `StringUtil::pos` scans one position
past the window, giving wrong answers and a `CHANGESTR` segfault

**Description.** Two symptoms, one off-by-one, so they are filed together; split them if the
maintainers prefer.

**Symptom one, a wrong answer.** `POS(needle, haystack, start, range)`'s `range` does not bound where
a match may fit, nor where it may begin:

    say pos('an','axan',1,3)    /* 3 -- the match ends at position 4, outside the range */
    say pos('an','zxan',1,3)    /* 0 */

Same start, same range; the haystacks differ only in a decoy `a` at position 1 that is not itself a
match. `StringUtil::pos` (`interpreter/classes/support/StringUtil.cpp`) sets `endpointer` to one past
the last position at which the whole needle fits, `memchr`s for the first byte over
`endpointer - haypointer` bytes, and on a candidate whose first byte matched but whose whole did not,
rescans from `haypointer + 1` **with that same length**, measured from the rejected candidate rather
than from where the scan resumes. Every rescan therefore ends one position past `endpointer`.

**The oracle's own twin is the proof this is a defect and not an extension.** `caselessPos` walks
`_range - needle_length + 1` probes one at a time, so `'axan'~caselessPos('an',1,3)` is `0` where
`'axan'~pos('an',1,3)` is `3`. `LASTPOS` uses a different primitive and is clean, checked by a
16-by-10 start-by-range sweep.

The overrun is one position and does not accumulate (`axaxan` is `0` at range 4 and `5` at range 5),
it holds for longer needles (`axxabc` gives 4 and `zxxabc` gives 0, both at range 5), and a one-byte
needle takes an early return before the loop and cannot overrun.

**Symptom two, a segfault.** When the search runs to the end of the haystack, the position one past
the window is the byte past the string itself, where the C++ reads the `RexxString`'s NUL terminator.
A needle whose last byte is `'00'x` therefore matches off the end:

    say pos('a'||'00'x, 'aa')          /* 2, over a two-byte haystack */
    say changestr('a'||'00'x, 'aa', 'ZZZ')   /* SIGSEGV, rc 139 */

`POS` and `COUNTSTR` merely report that position and survive. `CHANGESTR` copies the haystack up to
and including the matched needle, and copying through a match that runs past the buffer walks off the
end of the allocation.

Not a duplicate of #2010 (`ChangeStr` wrong message for a negative fourth argument) or #2012
(`Insert` with -1), both of which are argument-validation messages rather than this scan bound.

---

## 3. One clause queuing the same `CALL ON` condition twice segfaults on the second delivery

**Title.** Queuing the same `CALL ON` condition name twice in one clause segfaults when the boundary
drains the second copy

**Description.**

    call on user c1 name h1
    zr = ra() + ra()
    say 'after' zr
    exit 0
    ra:
    raise user c1 return 1
    h1:
    say 'h1' sigl
    return

`ra` raises the trapped condition once per call and the clause calls it twice, so both are queued
before the clause reaches its boundary. The interpreter prints `h1 2` on stdout -- the first delivery,
which completes normally -- and then dies with SIGSEGV, rc 139. So what fails is the boundary reaching
the *second* copy of an already-delivered condition name, not the queuing of it.

**The neighbouring shapes are clean and bound the defect to the repeated name.** The same clause with
two differently named conditions answers correctly: `zr = ra() + rb()` prints `h1 3`, `h2 3`, then
`after 3`, both handlers reporting the raising clause's own line, and a three-name version delivers
all three in the order queued. Draining a boundary in queue order therefore works; only a repeated
name in one boundary fails.

Found while establishing what a clause boundary does with more than one pending condition. No crash
site was captured for this one; a maintainer reproducing it under a debugger would add that.

---

## 4. `ArrayClass::validateIndex` mixes an array's item count with its raw slot array, segfaulting on
   a leading empty slot

**Title.** `~at` with a spread array whose leading slot is empty segfaults: `validateIndex` takes
`items()` alongside `data()`

**Description.**

    a = (1,2)
    say a~at((,2))

exits with SIGSEGV, rc 139. Measured deterministic, three runs of three, and the same for
`a~at((,,3))`.

`ArrayClass::validateIndex` spreads a lone array argument into the subscript list by taking its item
count alongside its slot array (`interpreter/classes/ArrayClass.cpp:1219-1226`):
`indexCount = indirect->items()` and `index = indirect->data()`. But `items()` counts the non-empty
slots while `data()` returns the raw slot array, so an array whose leading slot is empty and which
holds exactly one item hands `validateSingleDimensionIndex` an `indexCount` of 1 with
`index[0] == OREF_NULL`. Line 1264 then calls `index[0]->requiredPositive(argPosition)` on that null.

**The neighbours are all clean and identify the shape exactly.** `a~at((1,))` answers `1` (one item,
leading slot filled); `a~at((1,,3))` raises 93.926 (two items, so the count is rejected before either
subscript is read); `a~at((,))` raises 93.901 (no items at all). The crashing shape is any array whose
first filled slot is not slot one and which holds exactly one item.

---

## 5. A comment on SF #1850, not a new ticket: the trace indent counter has two disagreeing exit paths

**Where this goes.** SF #1850 "TRACE issues", status accepted, already covers the missing `>>>` lines
on a controlled loop's re-tested pass. The indentation symptom below comes out of the same two
functions in `BaseDoInstruction.cpp` and belongs as a comment there rather than as a separate ticket.

**Description.** `settings.traceIndent`
(`interpreter/execution/ActivationSettings.hpp:188`) is a mutable counter, not a computed property.
`newBlockInstruction` increments it (`interpreter/execution/RexxActivation.hpp:308`) and `DoBlock`'s
constructor saves the pre-increment value (`interpreter/instructions/DoBlock.cpp:71`). **The two exit
paths from a loop disagree about how to undo that.**

* Normal termination goes `RexxBaseBlockInstruction::terminate` ->
  `terminateBlockInstruction(doblock->getIndent())`
  (`interpreter/instructions/BaseDoInstruction.cpp:161`), an absolute restore of the saved value
  (`RexxActivation.hpp:306`).
* A failed control test goes `RexxInstructionBaseLoop::endLoop`
  (`interpreter/instructions/BaseDoInstruction.cpp:377`) -> `popBlockInstruction()` and then a bare
  `unindent()`, with no restore. `unindent()` clamps at zero (`RexxActivation.hpp:318`), and that
  clamp is why the defect is invisible at top level.

**Symptom.** Any repetitive `DO` or `LOOP` that completes at least one body pass and then ends because
its control test failed -- count exhausted, `WHILE` false, `UNTIL` true alike -- leaves the trace
indent two columns low for every later clause. Zero-trip loops and loops left by `LEAVE` do not.
Minimal reproducer, with no `INTERPRET` or `CALL` involved:

    do
    do jj = 1 to 1
    nop
    end
    say 1/0
    end

The raise is printed at indent 0 where lexical depth is 2.

**Which enclosing constructs absorb the stray decrement rather than propagating it:** exactly those
that restore from a saved `DoBlock` -- any repetitive `DO` or `LOOP`, `SELECT`/`OTHERWISE`, and any
`DO` carrying a `LABEL`, including a non-repetitive one, because
`interpreter/instructions/SimpleDoInstruction.cpp:78-89` creates the saved block only when a `LABEL`
is present. An unlabelled plain `DO` propagates it outward, which is why the reproducer above uses
one.

State the mechanism rather than a rule about which constructs decrement. The behaviour is emergent
from an imperative counter with inconsistent exit paths, and four separate attempts to state a
declarative construct-by-construct rule here were each wrong.

---

## Filing notes

* Mark each post as submitted by Claude Code, alongside the account posting it.
* Items 1 to 4 are memory-safety defects and each has a self-contained reproducer; item 5 is an
  output-correctness defect and a comment on an existing accepted ticket.
* Items 1, 2 and 4 name a specific line as the faulting read and those citations should be re-read
  against the revision being filed against, since all were taken from a local 5.3.0 checkout.
* Item 3 has no crash site captured. Say so in the ticket rather than leaving it implied.

---

## New, found 2026-08-24: the value-less entry-assignment message reads past its argument list

**Title:** `Directory`/`StringTable` value-less `NAME=` message stores a stale stack object instead
of removing the entry

**Description.**

A collection assignment message sent with an explicit empty argument list -- `d~"MYTHING="()` --
stores whatever object happens to sit one slot past the message's argument list, rather than
removing the entry. It is deterministic, exits 0, and writes nothing to stderr, so the wrong value
propagates silently.

```rexx
d = .directory~new
d~"A="()
say "A is:" d~A            /* prints: A is: STDQUE */
```

`STDQUE` is an interpreter-internal name, not anything the program mentions. That the read is live
rather than constant shows up when an expression is evaluated in between:

```rexx
d = .directory~new
say "FIRST MARKER"
d~"A="()
say "A is:" d~A            /* prints: A is: FIRST MARKER */
say "SECOND DIFFERENT MARKER"
d~"B="()
say "B is:" d~B            /* prints: B is: SECOND DIFFERENT MARKER */
```

The stored value tracks the preceding expression, so what is being read is the evaluation stack.

**Cause.** `StringHashCollection::unknown` (`interpreter/classes/support/HashCollection.cpp:1015`)
is declared with an `argCount` parameter that its body never reads. The assignment branch takes
`RexxObject *value = arguments[0];` at `:1026` and passes it to `setEntryRexx`. Both callers supply
a count that is discarded: `processUnknown` at `:989` forwards the interpreter's own array and
count, and `unknownRexx` at `:966` forwards `argumentList->messageArgs()` with
`argumentList->messageArgCount()`.

**The intended behaviour is documented one function away.** `setEntry` at `:854` opens with "set
entry is a little different than put, in that the value argument is optional. no argument is a
remove operation", and removes the entry when handed `OREF_NULL`. So a value-less `NAME=` send is
meant to remove the entry; the missing count check is precisely what stops it reaching that path.
The comment at `:1017`, "The arguments have already been validated by the base Object method", does
not hold for the count.

**Suggested fix.** In the assignment branch, use `argCount == 0 ? OREF_NULL : arguments[0]`, which
routes a value-less send into `setEntry`'s own documented removal path.

**Affects** `Directory` and `StringTable` alike: `Directory`'s behaviour inherits the method through
`InheritInstanceMethods(StringTable)` (`interpreter/memory/Setup.cpp:933`), and the `Unknown` method
is added once, at `:883`.

**Reproduced** three runs of three on ooRexx 5.3.0, Linux x86_64, `RelWithDebInfo`, under the
standard wrapper from a fresh empty directory. Not reproduced on another platform.

**Duplicate check owed before filing:** search ticket bodies for `HashCollection`, `setEntry` and
`unknown`, not just titles -- the title scan cannot see a ticket that names the symptom instead of
the mechanism.
