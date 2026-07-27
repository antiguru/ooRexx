# L1 coverage measurement (Task 0.4, Step 6)

**This is the real, full-suite measurement**, not a sample: the SVN checkout
of `code-0/test/trunk` completed during this task (the network came back
partway through), landing at `../ootest` — 409 `.testGroup` files, 20 MB.
An earlier draft of this file measured a 4-file local sample under a
"PROVISIONAL" label before the checkout was available; that draft has been
replaced by the numbers below. See "Re-running this measurement" at the
bottom for the exact command.

## Result

```
409 groups, 14122 test methods, 12176 extractable (86.2%)
```

**86.2% ≥ 40% — comfortably clears the D8 threshold the plan sets for L1
viability.** (This report only measures and records the number; per
instructions, the D8 ladder decision itself is left to the main session.)

| File | Total | Extractable | Percentage |
|---|---|---|---|
| ../ootest/misc/Advanced.testGroup | 5 | 0 | 0.0% |
| ../ootest/misc/SampleOLEObject.testGroup | 2 | 2 | 100.0% |
| ../ootest/misc/SimpleWithOneTimeSetup.testGroup | 3 | 3 | 100.0% |
| ../ootest/misc/SimpleWithSomeSetup.testGroup | 3 | 3 | 100.0% |
| ../ootest/misc/Simplest.testGroup | 2 | 2 | 100.0% |
| ../ootest/misc/template.testGroup | 2 | 2 | 100.0% |
| ../ootest/misc/templateAPI.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/API/classic/CLASSIC.testGroup | 9 | 7 | 77.8% |
| ../ootest/ooRexx/API/oo/CONVERSION.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/API/oo/FUNCTION.testGroup | 57 | 49 | 86.0% |
| ../ootest/ooRexx/API/oo/INVOCATION.testGroup | 55 | 0 | 0.0% |
| ../ootest/ooRexx/API/oo/METHOD.testGroup | 124 | 120 | 96.8% |
| ../ootest/ooRexx/API/oo/ProcessInvocation.testGroup | 54 | 0 | 0.0% |
| ../ootest/ooRexx/API/oo/ProcessRexxStart.testGroup | 20 | 0 | 0.0% |
| ../ootest/ooRexx/API/oo/RexxStart.testGroup | 20 | 0 | 0.0% |
| ../ootest/ooRexx/SimpleTests.testGroup | 7 | 7 | 100.0% |
| ../ootest/ooRexx/base/bif/ABBREV.testGroup | 117 | 117 | 100.0% |
| ../ootest/ooRexx/base/bif/ABS.testGroup | 49 | 49 | 100.0% |
| ../ootest/ooRexx/base/bif/ADDRESS.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/ARG.testGroup | 17 | 17 | 100.0% |
| ../ootest/ooRexx/base/bif/B2X.testGroup | 41 | 41 | 100.0% |
| ../ootest/ooRexx/base/bif/BEEP.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/bif/BITAND.testGroup | 101 | 101 | 100.0% |
| ../ootest/ooRexx/base/bif/BITOR.testGroup | 124 | 124 | 100.0% |
| ../ootest/ooRexx/base/bif/BITXOR.testGroup | 136 | 136 | 100.0% |
| ../ootest/ooRexx/base/bif/C2D.testGroup | 105 | 105 | 100.0% |
| ../ootest/ooRexx/base/bif/C2X.testGroup | 34 | 34 | 100.0% |
| ../ootest/ooRexx/base/bif/CENTER.testGroup | 97 | 97 | 100.0% |
| ../ootest/ooRexx/base/bif/CENTRE.testGroup | 97 | 97 | 100.0% |
| ../ootest/ooRexx/base/bif/CHANGESTR.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/bif/CHARIN.testGroup | 13 | 4 | 30.8% |
| ../ootest/ooRexx/base/bif/CHAROUT.testGroup | 39 | 2 | 5.1% |
| ../ootest/ooRexx/base/bif/CHARS.testGroup | 10 | 2 | 20.0% |
| ../ootest/ooRexx/base/bif/COMPARE.testGroup | 166 | 166 | 100.0% |
| ../ootest/ooRexx/base/bif/CONDITION.testGroup | 14 | 10 | 71.4% |
| ../ootest/ooRexx/base/bif/COPIES.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/bif/COUNTSTR.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/bif/D2C.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/D2X.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/DATATYPE.testGroup | 7 | 7 | 100.0% |
| ../ootest/ooRexx/base/bif/DATE.testGroup | 27 | 19 | 70.4% |
| ../ootest/ooRexx/base/bif/DELSTR.testGroup | 11 | 11 | 100.0% |
| ../ootest/ooRexx/base/bif/DELWORD.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/DIGITS.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/ERRORTEXT.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/FILESPEC.testGroup | 12 | 12 | 100.0% |
| ../ootest/ooRexx/base/bif/FORM.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/FORMAT.testGroup | 767 | 767 | 100.0% |
| ../ootest/ooRexx/base/bif/FUZZ.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/GC.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/bif/INSERT.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/LASTPOS.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/bif/LEFT.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/LENGTH.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/LINEIN.testGroup | 4 | 0 | 0.0% |
| ../ootest/ooRexx/base/bif/LINEOUT.testGroup | 17 | 3 | 17.6% |
| ../ootest/ooRexx/base/bif/LINES.testGroup | 18 | 15 | 83.3% |
| ../ootest/ooRexx/base/bif/LOWER.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/MAX.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/MIN.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/OVERLAY.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/POS.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/bif/QUALIFY.testGroup | 19 | 19 | 100.0% |
| ../ootest/ooRexx/base/bif/QUEUED.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/bif/RANDOM.testGroup | 11 | 10 | 90.9% |
| ../ootest/ooRexx/base/bif/REVERSE.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/bif/RIGHT.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/RXQUEUE.testGroup | 18 | 11 | 61.1% |
| ../ootest/ooRexx/base/bif/SIGN.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/SOURCELINE.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/base/bif/SPACE.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/STREAM.testGroup | 40 | 21 | 52.5% |
| ../ootest/ooRexx/base/bif/STRIP.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/bif/SUBSTR.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/SUBWORD.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/SYMBOL.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/TIME.testGroup | 286 | 278 | 97.2% |
| ../ootest/ooRexx/base/bif/TRANSLATE.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/bif/TRUNC.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/UPPER.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/VALUE.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/VAR.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/VERIFY.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/bif/WORD.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/WORDINDEX.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/WORDLENGTH.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/WORDPOS.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/WORDS.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/X2B.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/X2C.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/bif/X2D.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/bif/XRANGE.testGroup | 21 | 14 | 66.7% |
| ../ootest/ooRexx/base/class/Alarm.testGroup | 6 | 5 | 83.3% |
| ../ootest/ooRexx/base/class/Array.testGroup | 141 | 124 | 87.9% |
| ../ootest/ooRexx/base/class/Bag.testGroup | 34 | 34 | 100.0% |
| ../ootest/ooRexx/base/class/CircularQueue.testGroup | 59 | 56 | 94.9% |
| ../ootest/ooRexx/base/class/Class.testGroup | 107 | 104 | 97.2% |
| ../ootest/ooRexx/base/class/CollectionMethods.testGroup | 10 | 1 | 10.0% |
| ../ootest/ooRexx/base/class/CollectionSetlikeMethods.testGroup | 9 | 4 | 44.4% |
| ../ootest/ooRexx/base/class/Comparator.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/class/DateTime.testGroup | 22 | 8 | 36.4% |
| ../ootest/ooRexx/base/class/Directory.testGroup | 55 | 55 | 100.0% |
| ../ootest/ooRexx/base/class/EventSemaphore.testGroup | 9 | 8 | 88.9% |
| ../ootest/ooRexx/base/class/File.testGroup | 64 | 48 | 75.0% |
| ../ootest/ooRexx/base/class/IdentityTable.testGroup | 43 | 22 | 51.2% |
| ../ootest/ooRexx/base/class/List.testGroup | 55 | 51 | 92.7% |
| ../ootest/ooRexx/base/class/Message.testGroup | 70 | 54 | 77.1% |
| ../ootest/ooRexx/base/class/Method.testGroup | 56 | 43 | 76.8% |
| ../ootest/ooRexx/base/class/MethodArgs.testGroup | 10 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/Monitor.testGroup | 7 | 7 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/append.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/brackets.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessChangestr.testGroup | 13 | 13 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessContains.testGroup | 38 | 38 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessContainsWord.testGroup | 22 | 22 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessCountstr.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessLastpos.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessMatch.testGroup | 12 | 12 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessMatchChar.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessPos.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/caselessWordPos.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/changestr.testGroup | 15 | 15 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/contains.testGroup | 25 | 25 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/containsWord.testGroup | 22 | 22 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/countstr.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/delStr.testGroup | 11 | 11 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/delete.testGroup | 11 | 11 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/delword.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/getbuffersize.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/insert.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/lastpos.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/length.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/lower.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/match.testGroup | 12 | 12 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/matchChar.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/new.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/overlay.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/pos.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/replaceAt.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/setText.testGroup | 4 | 3 | 75.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/setbuffersize.testGroup | 6 | 6 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/space.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/string.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/subWord.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/subWords.testGroup | 1 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/subchar.testGroup | 6 | 6 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/substr.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/translate.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/upper.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/verify.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/word.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/wordindex.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/wordlength.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/wordpos.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutableBuffer/words.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/MutexSemaphore.testGroup | 8 | 7 | 87.5% |
| ../ootest/ooRexx/base/class/Object.testGroup | 164 | 157 | 95.7% |
| ../ootest/ooRexx/base/class/Orderable.testGroup | 15 | 13 | 86.7% |
| ../ootest/ooRexx/base/class/Package.testGroup | 80 | 76 | 95.0% |
| ../ootest/ooRexx/base/class/Package_Options.testGroup | 54 | 52 | 96.3% |
| ../ootest/ooRexx/base/class/Properties.testGroup | 63 | 58 | 92.1% |
| ../ootest/ooRexx/base/class/Queue.testGroup | 43 | 37 | 86.0% |
| ../ootest/ooRexx/base/class/QueueRGF.testGroup | 45 | 43 | 95.6% |
| ../ootest/ooRexx/base/class/Relation.testGroup | 45 | 45 | 100.0% |
| ../ootest/ooRexx/base/class/RexxContext.testGroup | 15 | 12 | 80.0% |
| ../ootest/ooRexx/base/class/RexxInfo.testGroup | 21 | 17 | 81.0% |
| ../ootest/ooRexx/base/class/RexxInteger.testGroup | 20 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/RexxQueue.testGroup | 48 | 39 | 81.2% |
| ../ootest/ooRexx/base/class/Routine.testGroup | 60 | 48 | 80.0% |
| ../ootest/ooRexx/base/class/Set.testGroup | 30 | 30 | 100.0% |
| ../ootest/ooRexx/base/class/Singleton.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/StackFrame.testGroup | 11 | 6 | 54.5% |
| ../ootest/ooRexx/base/class/Stem.testGroup | 52 | 52 | 100.0% |
| ../ootest/ooRexx/base/class/Stream.testGroup | 107 | 33 | 30.8% |
| ../ootest/ooRexx/base/class/String/String.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/String/abbrev.testGroup | 117 | 117 | 100.0% |
| ../ootest/ooRexx/base/class/String/abs.testGroup | 46 | 46 | 100.0% |
| ../ootest/ooRexx/base/class/String/append.testGroup | 3 | 3 | 100.0% |
| ../ootest/ooRexx/base/class/String/arithmetic.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/b2x.testGroup | 41 | 41 | 100.0% |
| ../ootest/ooRexx/base/class/String/bitand.testGroup | 102 | 102 | 100.0% |
| ../ootest/ooRexx/base/class/String/bitor.testGroup | 124 | 124 | 100.0% |
| ../ootest/ooRexx/base/class/String/bitxor.testGroup | 136 | 136 | 100.0% |
| ../ootest/ooRexx/base/class/String/brackets.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/c2d.testGroup | 106 | 106 | 100.0% |
| ../ootest/ooRexx/base/class/String/c2x.testGroup | 34 | 34 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessAbbrev.testGroup | 119 | 119 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessChangestr.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessCompare.testGroup | 161 | 161 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessCompareTo.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessContains.testGroup | 17 | 17 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessContainsWord.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessCountstr.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessEquals.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessLastpos.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessMatch.testGroup | 12 | 12 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessMatchChar.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessPos.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/caselessWordPos.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/ceiling.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/String/center.testGroup | 85 | 85 | 100.0% |
| ../ootest/ooRexx/base/class/String/centre.testGroup | 85 | 85 | 100.0% |
| ../ootest/ooRexx/base/class/String/changestr.testGroup | 10 | 10 | 100.0% |
| ../ootest/ooRexx/base/class/String/compare.testGroup | 166 | 166 | 100.0% |
| ../ootest/ooRexx/base/class/String/compareTo.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/comparisonOperators.testGroup | 4 | 3 | 75.0% |
| ../ootest/ooRexx/base/class/String/concatenationOperators.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/contains.testGroup | 15 | 15 | 100.0% |
| ../ootest/ooRexx/base/class/String/containsWord.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/copies.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/String/countstr.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/d2c.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/d2x.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/datatype.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/class/String/delstr.testGroup | 10 | 10 | 100.0% |
| ../ootest/ooRexx/base/class/String/delword.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/encode_decodeBase64.testGroup | 24 | 15 | 62.5% |
| ../ootest/ooRexx/base/class/String/endsWith.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/base/class/String/equals.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/floor.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/String/format.testGroup | 771 | 771 | 100.0% |
| ../ootest/ooRexx/base/class/String/iif.testGroup | 11 | 10 | 90.9% |
| ../ootest/ooRexx/base/class/String/insert.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/lastpos.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/left.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/length.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/String/logicalOperators.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/lower.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/makearray.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/makestring.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/match.testGroup | 12 | 12 | 100.0% |
| ../ootest/ooRexx/base/class/String/matchChar.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/max.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/min.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/modulo.testGroup | 14 | 14 | 100.0% |
| ../ootest/ooRexx/base/class/String/new.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/overlay.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/pos.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/replaceat.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/class/String/reverse.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/String/right.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/round.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/sign.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/space.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/class/String/startsWith.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/base/class/String/strip.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/String/subWords.testGroup | 11 | 10 | 90.9% |
| ../ootest/ooRexx/base/class/String/subchar.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/substr.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/subword.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/translate.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/class/String/trunc.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/upper.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/verify.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/class/String/word.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/wordindex.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/wordlength.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/wordpos.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/words.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/x2b.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/x2c.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/String/x2d.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/class/Table.testGroup | 44 | 44 | 100.0% |
| ../ootest/ooRexx/base/class/Ticker.testGroup | 30 | 29 | 96.7% |
| ../ootest/ooRexx/base/class/TimeSpan.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/base/class/Validate.testGroup | 101 | 101 | 100.0% |
| ../ootest/ooRexx/base/class/WeakReference.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/class/collections/array.testGroup | 41 | 40 | 97.6% |
| ../ootest/ooRexx/base/class/collections/bag.testGroup | 11 | 8 | 72.7% |
| ../ootest/ooRexx/base/class/collections/circularqueue.testGroup | 7 | 7 | 100.0% |
| ../ootest/ooRexx/base/class/collections/directory.testGroup | 29 | 29 | 100.0% |
| ../ootest/ooRexx/base/class/collections/list.testGroup | 19 | 19 | 100.0% |
| ../ootest/ooRexx/base/class/collections/properties.testGroup | 11 | 11 | 100.0% |
| ../ootest/ooRexx/base/class/collections/queue.testGroup | 22 | 22 | 100.0% |
| ../ootest/ooRexx/base/class/collections/stringtable.testGroup | 44 | 44 | 100.0% |
| ../ootest/ooRexx/base/directives/ANNOTATE.testGroup | 51 | 6 | 11.8% |
| ../ootest/ooRexx/base/directives/ATTRIBUTE.testGroup | 84 | 17 | 20.2% |
| ../ootest/ooRexx/base/directives/CLASS.testGroup | 54 | 0 | 0.0% |
| ../ootest/ooRexx/base/directives/CONSTANT.testGroup | 29 | 22 | 75.9% |
| ../ootest/ooRexx/base/directives/METHOD.testGroup | 78 | 24 | 30.8% |
| ../ootest/ooRexx/base/directives/OPTIONS.testGroup | 87 | 3 | 3.4% |
| ../ootest/ooRexx/base/directives/REQUIRES.testGroup | 33 | 0 | 0.0% |
| ../ootest/ooRexx/base/directives/RESOURCE.testGroup | 13 | 3 | 23.1% |
| ../ootest/ooRexx/base/directives/ROUTINE.testGroup | 30 | 6 | 20.0% |
| ../ootest/ooRexx/base/expressions/ADDITION.testGroup | 198 | 198 | 100.0% |
| ../ootest/ooRexx/base/expressions/COMPOSITE.testGroup | 28 | 28 | 100.0% |
| ../ootest/ooRexx/base/expressions/CONCATENATION.testGroup | 3 | 3 | 100.0% |
| ../ootest/ooRexx/base/expressions/DIVISION.testGroup | 313 | 313 | 100.0% |
| ../ootest/ooRexx/base/expressions/EXPONENT.testGroup | 97 | 97 | 100.0% |
| ../ootest/ooRexx/base/expressions/Literals.testGroup | 43 | 2 | 4.7% |
| ../ootest/ooRexx/base/expressions/MULTIPLICATION.testGroup | 143 | 143 | 100.0% |
| ../ootest/ooRexx/base/expressions/PRECEDENCE.testGroup | 1365 | 1365 | 100.0% |
| ../ootest/ooRexx/base/expressions/REMAINDER.testGroup | 293 | 293 | 100.0% |
| ../ootest/ooRexx/base/expressions/SPECIAL.testGroup | 96 | 96 | 100.0% |
| ../ootest/ooRexx/base/expressions/SUBTRACTION.testGroup | 300 | 300 | 100.0% |
| ../ootest/ooRexx/base/keyword/ADDRESS.testGroup | 80 | 30 | 37.5% |
| ../ootest/ooRexx/base/keyword/ASSIGNMENT.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/keyword/Assignments.testGroup | 11 | 11 | 100.0% |
| ../ootest/ooRexx/base/keyword/CALL.testGroup | 25 | 22 | 88.0% |
| ../ootest/ooRexx/base/keyword/DO.testGroup | 132 | 130 | 98.5% |
| ../ootest/ooRexx/base/keyword/DoControlled.testGroup | 60 | 8 | 13.3% |
| ../ootest/ooRexx/base/keyword/DoOther.testGroup | 28 | 17 | 60.7% |
| ../ootest/ooRexx/base/keyword/DoOver.testGroup | 36 | 6 | 16.7% |
| ../ootest/ooRexx/base/keyword/DoWith.testGroup | 37 | 8 | 21.6% |
| ../ootest/ooRexx/base/keyword/EXPOSE.testGroup | 16 | 0 | 0.0% |
| ../ootest/ooRexx/base/keyword/FORWARD.testGroup | 22 | 22 | 100.0% |
| ../ootest/ooRexx/base/keyword/GUARD.testGroup | 24 | 6 | 25.0% |
| ../ootest/ooRexx/base/keyword/IF.testGroup | 168 | 168 | 100.0% |
| ../ootest/ooRexx/base/keyword/INTERPRET.testGroup | 11 | 8 | 72.7% |
| ../ootest/ooRexx/base/keyword/ITERATE.testGroup | 31 | 31 | 100.0% |
| ../ootest/ooRexx/base/keyword/LABEL.testGroup | 42 | 1 | 2.4% |
| ../ootest/ooRexx/base/keyword/LEAVE.testGroup | 23 | 23 | 100.0% |
| ../ootest/ooRexx/base/keyword/LOOP.testGroup | 20 | 17 | 85.0% |
| ../ootest/ooRexx/base/keyword/LOSTDIGITS.testGroup | 11 | 0 | 0.0% |
| ../ootest/ooRexx/base/keyword/LabelOption.testGroup | 7 | 4 | 57.1% |
| ../ootest/ooRexx/base/keyword/LoopControlled.testGroup | 60 | 8 | 13.3% |
| ../ootest/ooRexx/base/keyword/LoopOther.testGroup | 28 | 17 | 60.7% |
| ../ootest/ooRexx/base/keyword/LoopOver.testGroup | 36 | 6 | 16.7% |
| ../ootest/ooRexx/base/keyword/LoopWith.testGroup | 37 | 8 | 21.6% |
| ../ootest/ooRexx/base/keyword/NOP.testGroup | 4 | 2 | 50.0% |
| ../ootest/ooRexx/base/keyword/NUMERIC.testGroup | 99 | 92 | 92.9% |
| ../ootest/ooRexx/base/keyword/PARSE.testGroup | 682 | 682 | 100.0% |
| ../ootest/ooRexx/base/keyword/RAISE.testGroup | 22 | 13 | 59.1% |
| ../ootest/ooRexx/base/keyword/REPLY.testGroup | 18 | 12 | 66.7% |
| ../ootest/ooRexx/base/keyword/SAY.testGroup | 2 | 1 | 50.0% |
| ../ootest/ooRexx/base/keyword/SELECT.testGroup | 44 | 28 | 63.6% |
| ../ootest/ooRexx/base/keyword/SIGNAL.testGroup | 33 | 11 | 33.3% |
| ../ootest/ooRexx/base/keyword/SelectCase.testGroup | 12 | 7 | 58.3% |
| ../ootest/ooRexx/base/keyword/ShortCircuitAnd.testGroup | 6 | 6 | 100.0% |
| ../ootest/ooRexx/base/keyword/TRACE.testGroup | 57 | 16 | 28.1% |
| ../ootest/ooRexx/base/keyword/TRACE_TraceObject.testGroup | 11 | 7 | 63.6% |
| ../ootest/ooRexx/base/keyword/USE.testGroup | 38 | 17 | 44.7% |
| ../ootest/ooRexx/base/keyword/USELOCAL.testGroup | 9 | 0 | 0.0% |
| ../ootest/ooRexx/base/keyword/VarRef.testGroup | 43 | 22 | 51.2% |
| ../ootest/ooRexx/base/rexxutil/Macrospace.testGroup | 39 | 17 | 43.6% |
| ../ootest/ooRexx/base/rexxutil/SysDumpVariables.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/SysFileDateTime.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/SysFileSearch.testGroup | 14 | 7 | 50.0% |
| ../ootest/ooRexx/base/rexxutil/SysFileTree.testGroup | 26 | 19 | 73.1% |
| ../ootest/ooRexx/base/rexxutil/SysFileXXX.testGroup | 43 | 32 | 74.4% |
| ../ootest/ooRexx/base/rexxutil/SysFormatMessage.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/SysSearchPath.testGroup | 10 | 9 | 90.0% |
| ../ootest/ooRexx/base/rexxutil/SysSleep.testGroup | 7 | 5 | 71.4% |
| ../ootest/ooRexx/base/rexxutil/SysStemCopy.testGroup | 12 | 0 | 0.0% |
| ../ootest/ooRexx/base/rexxutil/SysStemDelete.testGroup | 5 | 2 | 40.0% |
| ../ootest/ooRexx/base/rexxutil/SysStemInsert.testGroup | 5 | 5 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/SysStemSort.testGroup | 16 | 13 | 81.2% |
| ../ootest/ooRexx/base/rexxutil/platform/unix/SysGetMessage.testGroup | 11 | 3 | 27.3% |
| ../ootest/ooRexx/base/rexxutil/platform/unix/tilde.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysBootDrive.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysCurPos.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysCurState.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysDrive.testGroup | 7 | 4 | 57.1% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysDriveMap.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysFileTree.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysGetXxxPathName.testGroup | 9 | 9 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysIni.testGroup | 16 | 1 | 6.2% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysIsFileDirectory.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysSystemDirectory.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysTextScreenRead.testGroup | 10 | 3 | 30.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysTextScreenSize.testGroup | 21 | 15 | 71.4% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysUnicode.testGroup | 43 | 43 | 100.0% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysWinVer.testGroup | 3 | 2 | 66.7% |
| ../ootest/ooRexx/base/rexxutil/platform/windows/SysWin_xxx_Printer.testGroup | 17 | 16 | 94.1% |
| ../ootest/ooRexx/base/runtime.objects/environmentEntries.testGroup | 15 | 6 | 40.0% |
| ../ootest/ooRexx/base/security.manager/SecurityManager.testGroup | 29 | 0 | 0.0% |
| ../ootest/ooRexx/base/source.file/SourceFile.testGroup | 8 | 8 | 100.0% |
| ../ootest/ooRexx/base/source.file/incorrectCharacters.testGroup | 1 | 0 | 0.0% |
| ../ootest/ooRexx/base/source.file/whiteSpace.testGroup | 13 | 13 | 100.0% |
| ../ootest/ooRexx/base/special.variables/RESULT_RC_SIGL.testGroup | 7 | 4 | 57.1% |
| ../ootest/ooRexx/doc/rexxref/chapter5/Section1.testGroup | 3 | 3 | 100.0% |
| ../ootest/ooRexx/doc/rexxref/chapter7/Section4.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/extensions/dateparser/DateFormatter.testGroup | 31 | 31 | 100.0% |
| ../ootest/ooRexx/extensions/dateparser/DateParser.testGroup | 112 | 112 | 100.0% |
| ../ootest/ooRexx/extensions/hostemu/hostemu.testGroup | 16 | 1 | 6.2% |
| ../ootest/ooRexx/extensions/json/json.testGroup | 33 | 28 | 84.8% |
| ../ootest/ooRexx/extensions/json/json_02.testGroup | 227 | 183 | 80.6% |
| ../ootest/ooRexx/extensions/platform/unix/ncurses/ncurses.testGroup | 1 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/unix/rxunixsys/SysUnix.testGroup | 16 | 4 | 25.0% |
| ../ootest/ooRexx/extensions/platform/windows/ole/ExcelQuickTest.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/extensions/platform/windows/ole/OLEObject.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/windows/ole/OLEVariant.testGroup | 54 | 54 | 100.0% |
| ../ootest/ooRexx/extensions/platform/windows/ole/Printers.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/windows/ole/RexxProcess.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/windows/ole/SpecialFolders.testGroup | 1 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/windows/oodialog/Basic.testGroup | 5 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/windows/rxwinsys/Clipboard.testGroup | 0 | 0 | 0.0% |
| ../ootest/ooRexx/extensions/platform/windows/rxwinsys/WindowsEventLog.testGroup | 50 | 46 | 92.0% |
| ../ootest/ooRexx/extensions/rxmath/RxMath.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/extensions/rxregexp/rxregexp.testGroup | 33 | 20 | 60.6% |
| ../ootest/ooRexx/extensions/rxsock/socketClass.testGroup | 23 | 14 | 60.9% |
| ../ootest/ooRexx/extensions/yaml/yaml.testGroup | 50 | 50 | 100.0% |
| ../ootest/ooRexx/regressions/bug1853738.testGroup | 4 | 4 | 100.0% |
| ../ootest/ooRexx/regressions/bug2003_guard_when.testGroup | 1 | 0 | 0.0% |
| ../ootest/ooRexx/regressions/bug2061_newline_misalignment.testGroup | 2 | 2 | 100.0% |
| ../ootest/ooRexx/samples/samples.testGroup | 29 | 0 | 0.0% |
| ../ootest/ooRexx/samples/scclient.testGroup | 2 | 1 | 50.0% |
| ../ootest/ooRexx/samples/scserver.testGroup | 2 | 1 | 50.0% |
| ../ootest/ooRexx/samples/sfclient.testGroup | 2 | 1 | 50.0% |
| ../ootest/ooRexx/samples/sfserver.testGroup | 2 | 1 | 50.0% |
| ../ootest/ooRexx/samples/windows/adsi.testGroup | 6 | 0 | 0.0% |
| ../ootest/ooRexx/samples/windows/fileNameDialog_demo.testGroup | 5 | 0 | 0.0% |
| ../ootest/ooRexx/samples/windows/samples.testGroup | 1 | 0 | 0.0% |
| ../ootest/ooRexx/samples/windows/wmi.testGroup | 4 | 0 | 0.0% |
| ../ootest/ooRexx/utilities/rexx/rexx_command.testGroup | 30 | 0 | 0.0% |
| ../ootest/ooRexx/utilities/rexxc/rexxc.testGroup | 8 | 0 | 0.0% |
| ../ootest/ooRexx/utilities/rxapi.testGroup | 1 | 1 | 100.0% |
| ../ootest/ooRexx/utilities/rxqueue/rxQueue.testGroup | 9 | 6 | 66.7% |
| ../ootest/ooRexx/utilities/rxsubcom/rxsubcom.testGroup | 13 | 0 | 0.0% |
| **Total** | **14122** | **12176** | **86.2%** |

## Two real bugs the full run found (fixed in `rexx-extract`)

Both only surface once you point the tool at the actual suite instead of a
handful of files — the sample used before the checkout completed didn't hit
either.

1. **Non-UTF-8 source.** `ooRexx/base/bif/C2X.testGroup` is ISO-8859 text
   that embeds a literal `0xAA` byte inside a string argument to `C2X()`
   (testing hex-conversion of raw high bytes). `std::fs::read_to_string`
   rejected it outright and aborted the whole run. Fixed by reading bytes
   and using `String::from_utf8_lossy` — safe here because `extract()` only
   looks for ASCII markers (`::method`, `self~`); a lossy-decoded string
   literal payload doesn't change where those markers fall.
2. **Path-unsafe method names.** `ooRexx/base/keyword/Assignments.testGroup`
   has `::method "test_/="` and `::method "test_//="` (testing the `/=` and
   `//=` operators) — the quoted name itself contains `/`, which is a path
   separator. Building `<group>_<method>.rex` from the raw name therefore
   tried to write into a nonexistent subdirectory and failed. Fixed by
   sanitizing the method-name component: anything outside
   `[A-Za-z0-9_-]` becomes `_`.

## Why the raw number is probably a little optimistic, and by how much

`touches_fixture` (as specified) only flags fixture access sent as
`self~<message>`. It does **not** recognize the other common idiom: `setUp`
stores fixture state in an exposed instance variable, and test methods do
`expose <var>` then call `<var>~...` directly, never through `self~`. Since
`extract()` never sees a `self~` message in that body, it reports no fixture
use, and the method is marked extractable — even though the emitted
`.rex` wraps it in `::routine main public`, where `expose` isn't even legal
(it's a method-only instruction), so the program would fail to parse, not
just fail an assertion.

Checked against the full 12,176-file extraction output (files that mention
`expose` anywhere in the extracted body, as a proxy for this blind spot):

- 491 of 12,176 extracted files (**4.0%**) contain `expose`.
- Treating all of those as actually fixture-dependent: adjusted extractable
  = 12,176 − 491 = **11,685**, adjusted percentage = 11,685 / 14,122 =
  **82.7%**.

So the correction is small at full-suite scale (86.2% → 82.7%, both well
above 40%) — **not** the ~4x drop an earlier draft of this file (measured
against just `json_02.testGroup`, `json_01_Claude.testGroup`, and
`yaml.testGroup`) suggested. Those three files turned out to be
unrepresentative outliers: they're ~80% `expose`-based, while the suite as a
whole is ~4%. That's itself worth remembering when eyeballing individual
rows above — a handful of files (the `json`/`yaml`/`OLE`/directive-heavy
groups) carry most of the remaining `expose` risk, not the corpus broadly.

## Other things this run surfaced in the spec (not fixed — flagging, not redesigning)

- **No comment-awareness.** `extract()` is a line-by-line scanner with no
  concept of Rexx's `/* ... */` block comments. Any `.testGroup` with a
  commented-out `::method test...` block (seen in the older-framework
  `Assert.testUnit`, not part of this SVN tree but present in the sibling
  `ootRexxUnit`-style scratchpad checkout) will have those dead methods
  counted as live and extracted into `.rex` files with no corresponding real
  test. Not observed to matter at scale in this suite, but it's a real gap.
- **`ASSERTIONS` has no `expectCondition`.** Only `expectSyntax` is listed,
  even though `expectCondition` is the same style of assertion (expect a
  condition to be raised) and appears in ooRexxUnit-family test code. Methods
  using it are (correctly, given the list) marked fixture-dependent, which
  looks like an omission rather than an intentional exclusion.
- **Shim completeness was underspecified.** Step 5's example Rexx snippet
  shows only `::method assertEquals`, while the surrounding prose says the
  shim "must define exactly the assertion messages listed in `ASSERTIONS`"
  (11 names). The snippet was read as illustrative rather than exhaustive;
  the shipped binary emits all 11 shim methods. Worth confirming that
  reading was intended.

## Re-running this measurement

```bash
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
# already checked out at ../ootest for this run; re-checkout only if it's
# missing or stale:
svn checkout --non-interactive --trust-server-cert \
  https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest
cd rust && cargo run --release -p rexx-extract --bin rexx-extract -- \
  --suite ../ootest --out ../rust/corpus/extracted --report ../docs/superpowers/plans/l1-coverage.md
```
