# criterion 6 over every ooTest group file run whole, at d9e17e5c8, from rust/:
# RAYON_NUM_THREADS=4 REXX_SHARING_LOG=LOG memcap 8G cargo test --release -p rexx-exec --features sharing --test concurrency_tests -- --exact sharing::sharing_fraction_over_every_ootest_group --nocapture
# (at the default 32 threads the run was OOM-killed at the 8G cap)

groups 388

| | objects | shared |
|---|---|---|
| bootstrap | 105536 | 66 |
| program | 1872392 | 157 |

not run, reaching rxapi:

- API/classic/CLASSIC.testGroup
- API/oo/FUNCTION.testGroup
- API/oo/INVOCATION.testGroup
- API/oo/ProcessInvocation.testGroup
- API/oo/ProcessRexxStart.testGroup
- API/oo/RexxStart.testGroup
- base/bif/RXQUEUE.testGroup
- base/class/RexxQueue.testGroup
- base/keyword/ADDRESS.testGroup
- base/rexxutil/Macrospace.testGroup
- base/rexxutil/platform/unix/SysGetMessage.testGroup
- base/runtime.objects/environmentEntries.testGroup
- extensions/platform/unix/rxunixsys/SysUnix.testGroup
- utilities/rxqueue/rxQueue.testGroup

| group | outcome | bootstrap objects | bootstrap shared | program objects | program shared |
|---|---|---|---|---|---|
| API/oo/CONVERSION.testGroup | pass, rc 0 | 272 | 0 | 7818 | 0 |
| API/oo/METHOD.testGroup | failure, rc 1 | 272 | 0 | 23045 | 0 |
| SimpleTests.testGroup | pass, rc 0 | 272 | 0 | 1493 | 0 |
| base/bif/ABBREV.testGroup | pass, rc 0 | 272 | 0 | 5418 | 0 |
| base/bif/ABS.testGroup | pass, rc 0 | 272 | 0 | 3003 | 0 |
| base/bif/ADDRESS.testGroup | pass, rc 0 | 272 | 0 | 1340 | 0 |
| base/bif/ARG.testGroup | failure, rc 1 | 272 | 0 | 1997 | 0 |
| base/bif/B2X.testGroup | pass, rc 0 | 272 | 0 | 3292 | 0 |
| base/bif/BEEP.testGroup | pass, rc 0 | 272 | 0 | 1860 | 0 |
| base/bif/BITAND.testGroup | pass, rc 0 | 272 | 0 | 4844 | 0 |
| base/bif/BITOR.testGroup | pass, rc 0 | 272 | 0 | 5533 | 0 |
| base/bif/BITXOR.testGroup | pass, rc 0 | 272 | 0 | 6219 | 0 |
| base/bif/C2D.testGroup | pass, rc 0 | 272 | 0 | 6820 | 0 |
| base/bif/C2X.testGroup | pass, rc 0 | 272 | 0 | 2185 | 0 |
| base/bif/CENTER.testGroup | pass, rc 0 | 272 | 0 | 5014 | 0 |
| base/bif/CENTRE.testGroup | pass, rc 0 | 272 | 0 | 5014 | 0 |
| base/bif/CHANGESTR.testGroup | pass, rc 0 | 272 | 0 | 1595 | 0 |
| base/bif/CHARIN.testGroup | pass, rc 0 | 272 | 0 | 2242 | 0 |
| base/bif/CHAROUT.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 3270 | 0 |
| base/bif/CHARS.testGroup | failure, rc 1 | 272 | 0 | 2587 | 0 |
| base/bif/COMPARE.testGroup | pass, rc 0 | 272 | 0 | 6288 | 0 |
| base/bif/CONDITION.testGroup | error, rc 2 | 272 | 0 | 3801 | 0 |
| base/bif/COPIES.testGroup | pass, rc 0 | 272 | 0 | 25402 | 0 |
| base/bif/COUNTSTR.testGroup | pass, rc 0 | 272 | 0 | 1429 | 0 |
| base/bif/D2C.testGroup | pass, rc 0 | 272 | 0 | 2953 | 0 |
| base/bif/D2X.testGroup | pass, rc 0 | 272 | 0 | 3153 | 0 |
| base/bif/DATATYPE.testGroup | pass, rc 0 | 272 | 0 | 51861 | 0 |
| base/bif/DATE.testGroup | pass, rc 0 | 272 | 0 | 5946 | 0 |
| base/bif/DELSTR.testGroup | pass, rc 0 | 272 | 0 | 4379 | 0 |
| base/bif/DELWORD.testGroup | pass, rc 0 | 272 | 0 | 3295 | 0 |
| base/bif/DIGITS.testGroup | pass, rc 0 | 272 | 0 | 2069 | 0 |
| base/bif/ERRORTEXT.testGroup | pass, rc 0 | 272 | 0 | 1575 | 0 |
| base/bif/FILESPEC.testGroup | pass, rc 0 | 272 | 0 | 2042 | 0 |
| base/bif/FORM.testGroup | pass, rc 0 | 272 | 0 | 1346 | 0 |
| base/bif/FORMAT.testGroup | pass, rc 0 | 272 | 0 | 30053 | 0 |
| base/bif/FUZZ.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/bif/GC.testGroup | pass, rc 0 | 272 | 0 | 1642 | 0 |
| base/bif/INSERT.testGroup | pass, rc 0 | 272 | 0 | 6688 | 0 |
| base/bif/LASTPOS.testGroup | pass, rc 0 | 272 | 0 | 6562 | 0 |
| base/bif/LEFT.testGroup | pass, rc 0 | 272 | 0 | 3034 | 0 |
| base/bif/LENGTH.testGroup | pass, rc 0 | 272 | 0 | 2053 | 0 |
| base/bif/LINEIN.testGroup | pass, rc 0 | 272 | 0 | 1501 | 0 |
| base/bif/LINEOUT.testGroup | refused at environment symbol ".STDQUE" is not implemented (Phase 10) | 272 | 0 | 2871 | 0 |
| base/bif/LINES.testGroup | failure, rc 1 | 272 | 0 | 2999 | 0 |
| base/bif/LOWER.testGroup | pass, rc 0 | 272 | 0 | 1350 | 0 |
| base/bif/MAX.testGroup | pass, rc 0 | 272 | 0 | 1629 | 0 |
| base/bif/MIN.testGroup | pass, rc 0 | 272 | 0 | 1544 | 0 |
| base/bif/OVERLAY.testGroup | pass, rc 0 | 272 | 0 | 3750 | 0 |
| base/bif/POS.testGroup | pass, rc 0 | 272 | 0 | 4272 | 0 |
| base/bif/QUALIFY.testGroup | pass, rc 0 | 272 | 0 | 2183 | 0 |
| base/bif/QUEUED.testGroup | pass, rc 0 | 272 | 0 | 1626 | 0 |
| base/bif/RANDOM.testGroup | pass, rc 0 | 272 | 0 | 2421 | 0 |
| base/bif/REVERSE.testGroup | pass, rc 0 | 272 | 0 | 1998 | 0 |
| base/bif/RIGHT.testGroup | pass, rc 0 | 272 | 0 | 3692 | 0 |
| base/bif/SIGN.testGroup | pass, rc 0 | 272 | 0 | 3146 | 0 |
| base/bif/SOURCELINE.testGroup | pass, rc 0 | 272 | 0 | 2328 | 0 |
| base/bif/SPACE.testGroup | pass, rc 0 | 272 | 0 | 3632 | 0 |
| base/bif/STREAM.testGroup | error, rc 2 | 272 | 0 | 5874 | 0 |
| base/bif/STRIP.testGroup | pass, rc 0 | 272 | 0 | 5812 | 0 |
| base/bif/SUBSTR.testGroup | pass, rc 0 | 272 | 0 | 5492 | 0 |
| base/bif/SUBWORD.testGroup | pass, rc 0 | 272 | 0 | 7260 | 0 |
| base/bif/SYMBOL.testGroup | pass, rc 0 | 272 | 0 | 1958 | 0 |
| base/bif/TIME.testGroup | refused at the run exceeded its deadline | 272 | 0 | 26360 | 0 |
| base/bif/TRANSLATE.testGroup | pass, rc 0 | 272 | 0 | 2759 | 0 |
| base/bif/TRUNC.testGroup | pass, rc 0 | 272 | 0 | 2856 | 0 |
| base/bif/UPPER.testGroup | pass, rc 0 | 272 | 0 | 1350 | 0 |
| base/bif/VALUE.testGroup | pass, rc 0 | 272 | 0 | 3152 | 0 |
| base/bif/VAR.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/bif/VERIFY.testGroup | pass, rc 0 | 272 | 0 | 4829 | 0 |
| base/bif/WORD.testGroup | pass, rc 0 | 272 | 0 | 3923 | 0 |
| base/bif/WORDINDEX.testGroup | pass, rc 0 | 272 | 0 | 3950 | 0 |
| base/bif/WORDLENGTH.testGroup | pass, rc 0 | 272 | 0 | 3895 | 0 |
| base/bif/WORDPOS.testGroup | pass, rc 0 | 272 | 0 | 6811 | 0 |
| base/bif/WORDS.testGroup | pass, rc 0 | 272 | 0 | 2335 | 0 |
| base/bif/X2B.testGroup | pass, rc 0 | 272 | 0 | 4155 | 0 |
| base/bif/X2C.testGroup | pass, rc 0 | 272 | 0 | 3785 | 0 |
| base/bif/X2D.testGroup | pass, rc 0 | 272 | 0 | 4627 | 0 |
| base/bif/XRANGE.testGroup | pass, rc 0 | 272 | 0 | 5039 | 0 |
| base/class/Alarm.testGroup | pass, rc 0 | 272 | 3 | 2245 | 22 |
| base/class/Array.testGroup | error, rc 2 | 272 | 0 | 30267 | 0 |
| base/class/Bag.testGroup | failure, rc 1 | 272 | 0 | 10334 | 0 |
| base/class/CircularQueue.testGroup | failure, rc 1 | 272 | 0 | 10461 | 0 |
| base/class/Class.testGroup | refused at method "TEST1" of class "TESTDEFINE1" is not implemented (Phase 9) | 272 | 0 | 3623 | 0 |
| base/class/CollectionMethods.testGroup | error, rc 2 | 272 | 0 | 13253 | 0 |
| base/class/CollectionSetlikeMethods.testGroup | pass, rc 0 | 272 | 0 | 5683 | 0 |
| base/class/Comparator.testGroup | pass, rc 0 | 272 | 0 | 1570 | 0 |
| base/class/DateTime.testGroup | refused at DO is not implemented | 272 | 0 | 5356 | 0 |
| base/class/Directory.testGroup | pass, rc 0 | 272 | 0 | 9729 | 0 |
| base/class/EventSemaphore.testGroup | pass, rc 0 | 272 | 0 | 2000 | 4 |
| base/class/File.testGroup | refused at routine "SYSGETFILEDATETIME" is not implemented (Phase 10) | 272 | 0 | 3385 | 0 |
| base/class/IdentityTable.testGroup | failure, rc 1 | 272 | 0 | 5039 | 0 |
| base/class/List.testGroup | pass, rc 0 | 272 | 0 | 11339 | 0 |
| base/class/Message.testGroup | refused at method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | 272 | 20 | 4173 | 45 |
| base/class/Method.testGroup | refused at DO is not implemented | 272 | 0 | 610 | 0 |
| base/class/MethodArgs.testGroup | refused at DO is not implemented | 272 | 0 | 5714 | 2 |
| base/class/Monitor.testGroup | pass, rc 0 | 272 | 0 | 1820 | 0 |
| base/class/MutableBuffer/append.testGroup | pass, rc 0 | 272 | 0 | 1655 | 0 |
| base/class/MutableBuffer/brackets.testGroup | pass, rc 0 | 272 | 0 | 2347 | 0 |
| base/class/MutableBuffer/caselessChangestr.testGroup | pass, rc 0 | 272 | 0 | 2451 | 0 |
| base/class/MutableBuffer/caselessContains.testGroup | pass, rc 0 | 272 | 0 | 2850 | 0 |
| base/class/MutableBuffer/caselessContainsWord.testGroup | pass, rc 0 | 272 | 0 | 1850 | 0 |
| base/class/MutableBuffer/caselessCountstr.testGroup | pass, rc 0 | 272 | 0 | 1346 | 0 |
| base/class/MutableBuffer/caselessLastpos.testGroup | pass, rc 0 | 272 | 0 | 7011 | 0 |
| base/class/MutableBuffer/caselessMatch.testGroup | pass, rc 0 | 272 | 0 | 2385 | 0 |
| base/class/MutableBuffer/caselessMatchChar.testGroup | pass, rc 0 | 272 | 0 | 1728 | 0 |
| base/class/MutableBuffer/caselessPos.testGroup | pass, rc 0 | 272 | 0 | 4229 | 0 |
| base/class/MutableBuffer/caselessWordPos.testGroup | pass, rc 0 | 272 | 0 | 7152 | 0 |
| base/class/MutableBuffer/changestr.testGroup | pass, rc 0 | 272 | 0 | 2256 | 0 |
| base/class/MutableBuffer/contains.testGroup | pass, rc 0 | 272 | 0 | 2490 | 0 |
| base/class/MutableBuffer/containsWord.testGroup | pass, rc 0 | 272 | 0 | 1850 | 0 |
| base/class/MutableBuffer/countstr.testGroup | pass, rc 0 | 272 | 0 | 1433 | 0 |
| base/class/MutableBuffer/delStr.testGroup | pass, rc 0 | 272 | 0 | 2085 | 0 |
| base/class/MutableBuffer/delete.testGroup | pass, rc 0 | 272 | 0 | 2085 | 0 |
| base/class/MutableBuffer/delword.testGroup | pass, rc 0 | 272 | 0 | 3403 | 0 |
| base/class/MutableBuffer/getbuffersize.testGroup | pass, rc 0 | 272 | 0 | 1346 | 0 |
| base/class/MutableBuffer/insert.testGroup | pass, rc 0 | 272 | 0 | 10699 | 0 |
| base/class/MutableBuffer/lastpos.testGroup | pass, rc 0 | 272 | 0 | 7775 | 0 |
| base/class/MutableBuffer/length.testGroup | pass, rc 0 | 272 | 0 | 1995 | 0 |
| base/class/MutableBuffer/lower.testGroup | pass, rc 0 | 272 | 0 | 1355 | 0 |
| base/class/MutableBuffer/match.testGroup | pass, rc 0 | 272 | 0 | 2374 | 0 |
| base/class/MutableBuffer/matchChar.testGroup | pass, rc 0 | 272 | 0 | 1728 | 0 |
| base/class/MutableBuffer/new.testGroup | pass, rc 0 | 272 | 0 | 1718 | 0 |
| base/class/MutableBuffer/overlay.testGroup | pass, rc 0 | 272 | 0 | 4632 | 0 |
| base/class/MutableBuffer/pos.testGroup | pass, rc 0 | 272 | 0 | 4797 | 0 |
| base/class/MutableBuffer/replaceAt.testGroup | pass, rc 0 | 272 | 0 | 2028 | 0 |
| base/class/MutableBuffer/setText.testGroup | pass, rc 0 | 272 | 0 | 1655 | 0 |
| base/class/MutableBuffer/setbuffersize.testGroup | pass, rc 0 | 272 | 0 | 1735 | 0 |
| base/class/MutableBuffer/space.testGroup | pass, rc 0 | 272 | 0 | 8456 | 0 |
| base/class/MutableBuffer/string.testGroup | pass, rc 0 | 272 | 0 | 1371 | 0 |
| base/class/MutableBuffer/subWord.testGroup | pass, rc 0 | 272 | 0 | 7286 | 0 |
| base/class/MutableBuffer/subWords.testGroup | pass, rc 0 | 272 | 0 | 7621 | 0 |
| base/class/MutableBuffer/subchar.testGroup | pass, rc 0 | 272 | 0 | 1749 | 0 |
| base/class/MutableBuffer/substr.testGroup | pass, rc 0 | 272 | 0 | 6535 | 0 |
| base/class/MutableBuffer/translate.testGroup | pass, rc 0 | 272 | 0 | 2826 | 0 |
| base/class/MutableBuffer/upper.testGroup | pass, rc 0 | 272 | 0 | 1355 | 0 |
| base/class/MutableBuffer/verify.testGroup | pass, rc 0 | 272 | 0 | 5140 | 0 |
| base/class/MutableBuffer/word.testGroup | pass, rc 0 | 272 | 0 | 4002 | 0 |
| base/class/MutableBuffer/wordindex.testGroup | pass, rc 0 | 272 | 0 | 4034 | 0 |
| base/class/MutableBuffer/wordlength.testGroup | pass, rc 0 | 272 | 0 | 3951 | 0 |
| base/class/MutableBuffer/wordpos.testGroup | pass, rc 0 | 272 | 0 | 7053 | 0 |
| base/class/MutableBuffer/words.testGroup | pass, rc 0 | 272 | 0 | 2353 | 0 |
| base/class/MutexSemaphore.testGroup | pass, rc 0 | 272 | 0 | 1822 | 2 |
| base/class/Object.testGroup | refused at method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | 272 | 0 | 6557 | 2 |
| base/class/Orderable.testGroup | pass, rc 0 | 272 | 0 | 4569 | 0 |
| base/class/Package.testGroup | refused at method "NEW" of class "Routine" is not implemented (Phase 9) | 272 | 0 | 5823 | 0 |
| base/class/Package_Options.testGroup | refused at a package settings write is not implemented (Phase 10) | 272 | 0 | 2276 | 0 |
| base/class/Properties.testGroup | pass, rc 0 | 272 | 0 | 6905 | 0 |
| base/class/Queue.testGroup | failure, rc 1 | 272 | 0 | 10715 | 0 |
| base/class/QueueRGF.testGroup | error, rc 2 | 272 | 0 | 8518 | 0 |
| base/class/Relation.testGroup | pass, rc 0 | 272 | 0 | 11555 | 0 |
| base/class/RexxContext.testGroup | refused at CONDITION option "O" answers a Directory, which is not implemented | 272 | 0 | 1536 | 0 |
| base/class/RexxInfo.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 1621 | 0 |
| base/class/RexxInteger.testGroup | failure, rc 1 | 272 | 0 | 12428 | 0 |
| base/class/Routine.testGroup | refused at DO is not implemented | 272 | 0 | 609 | 0 |
| base/class/Set.testGroup | pass, rc 0 | 272 | 0 | 7569 | 0 |
| base/class/Singleton.testGroup | refused at method "COPY" of class "TEST" is not implemented (Phase 9) | 272 | 0 | 1210 | 0 |
| base/class/StackFrame.testGroup | refused at method "EXECUTABLE" of class "StackFrame" is not implemented (Phase 9) | 272 | 0 | 1832 | 0 |
| base/class/Stem.testGroup | pass, rc 0 | 272 | 0 | 7230 | 0 |
| base/class/Stream.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 62862 | 0 |
| base/class/String/String.testGroup | pass, rc 0 | 272 | 0 | 1386 | 0 |
| base/class/String/abbrev.testGroup | pass, rc 0 | 272 | 0 | 5434 | 0 |
| base/class/String/abs.testGroup | pass, rc 0 | 272 | 0 | 2822 | 0 |
| base/class/String/append.testGroup | pass, rc 0 | 272 | 0 | 1533 | 0 |
| base/class/String/arithmetic.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/b2x.testGroup | pass, rc 0 | 272 | 0 | 3356 | 0 |
| base/class/String/bitand.testGroup | pass, rc 0 | 272 | 0 | 4938 | 0 |
| base/class/String/bitor.testGroup | pass, rc 0 | 272 | 0 | 5814 | 0 |
| base/class/String/bitxor.testGroup | pass, rc 0 | 272 | 0 | 6529 | 0 |
| base/class/String/brackets.testGroup | pass, rc 0 | 272 | 0 | 6959 | 0 |
| base/class/String/c2d.testGroup | pass, rc 0 | 272 | 0 | 7046 | 0 |
| base/class/String/c2x.testGroup | pass, rc 0 | 272 | 0 | 2189 | 0 |
| base/class/String/caselessAbbrev.testGroup | pass, rc 0 | 272 | 0 | 5487 | 0 |
| base/class/String/caselessChangestr.testGroup | pass, rc 0 | 272 | 0 | 1575 | 0 |
| base/class/String/caselessCompare.testGroup | pass, rc 0 | 272 | 0 | 6360 | 0 |
| base/class/String/caselessCompareTo.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/caselessContains.testGroup | pass, rc 0 | 272 | 0 | 4971 | 0 |
| base/class/String/caselessContainsWord.testGroup | pass, rc 0 | 272 | 0 | 13697 | 0 |
| base/class/String/caselessCountstr.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/caselessEquals.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/caselessLastpos.testGroup | pass, rc 0 | 272 | 0 | 6897 | 0 |
| base/class/String/caselessMatch.testGroup | pass, rc 0 | 272 | 0 | 2372 | 0 |
| base/class/String/caselessMatchChar.testGroup | pass, rc 0 | 272 | 0 | 1722 | 0 |
| base/class/String/caselessPos.testGroup | pass, rc 0 | 272 | 0 | 4332 | 0 |
| base/class/String/caselessWordPos.testGroup | pass, rc 0 | 272 | 0 | 7023 | 0 |
| base/class/String/ceiling.testGroup | pass, rc 0 | 272 | 0 | 1644 | 0 |
| base/class/String/center.testGroup | pass, rc 0 | 272 | 0 | 4072 | 0 |
| base/class/String/centre.testGroup | pass, rc 0 | 272 | 0 | 4072 | 0 |
| base/class/String/changestr.testGroup | pass, rc 0 | 272 | 0 | 1625 | 0 |
| base/class/String/compare.testGroup | pass, rc 0 | 272 | 0 | 6467 | 0 |
| base/class/String/compareTo.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/comparisonOperators.testGroup | failure, rc 1 | 272 | 0 | 2039 | 0 |
| base/class/String/concatenationOperators.testGroup | pass, rc 0 | 272 | 0 | 1348 | 0 |
| base/class/String/contains.testGroup | pass, rc 0 | 272 | 0 | 4920 | 0 |
| base/class/String/containsWord.testGroup | pass, rc 0 | 272 | 0 | 7020 | 0 |
| base/class/String/copies.testGroup | pass, rc 0 | 272 | 0 | 23252 | 0 |
| base/class/String/countstr.testGroup | pass, rc 0 | 272 | 0 | 1429 | 0 |
| base/class/String/d2c.testGroup | pass, rc 0 | 272 | 0 | 2969 | 0 |
| base/class/String/d2x.testGroup | pass, rc 0 | 272 | 0 | 3193 | 0 |
| base/class/String/datatype.testGroup | pass, rc 0 | 272 | 0 | 51391 | 0 |
| base/class/String/delstr.testGroup | pass, rc 0 | 272 | 0 | 4368 | 0 |
| base/class/String/delword.testGroup | pass, rc 0 | 272 | 0 | 3351 | 0 |
| base/class/String/encode_decodeBase64.testGroup | pass, rc 0 | 272 | 0 | 12949 | 0 |
| base/class/String/endsWith.testGroup | pass, rc 0 | 272 | 0 | 1558 | 0 |
| base/class/String/equals.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/floor.testGroup | pass, rc 0 | 272 | 0 | 1644 | 0 |
| base/class/String/format.testGroup | pass, rc 0 | 272 | 0 | 31896 | 0 |
| base/class/String/iif.testGroup | pass, rc 0 | 272 | 0 | 2274 | 0 |
| base/class/String/insert.testGroup | pass, rc 0 | 272 | 0 | 6783 | 0 |
| base/class/String/lastpos.testGroup | pass, rc 0 | 272 | 0 | 6876 | 0 |
| base/class/String/left.testGroup | pass, rc 0 | 272 | 0 | 3078 | 0 |
| base/class/String/length.testGroup | pass, rc 0 | 272 | 0 | 2077 | 0 |
| base/class/String/logicalOperators.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/lower.testGroup | pass, rc 0 | 272 | 0 | 1350 | 0 |
| base/class/String/makearray.testGroup | pass, rc 0 | 272 | 0 | 1350 | 0 |
| base/class/String/makestring.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/match.testGroup | pass, rc 0 | 272 | 0 | 2361 | 0 |
| base/class/String/matchChar.testGroup | pass, rc 0 | 272 | 0 | 1722 | 0 |
| base/class/String/max.testGroup | pass, rc 0 | 272 | 0 | 1653 | 0 |
| base/class/String/min.testGroup | pass, rc 0 | 272 | 0 | 1569 | 0 |
| base/class/String/modulo.testGroup | pass, rc 0 | 272 | 0 | 2508 | 0 |
| base/class/String/new.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/overlay.testGroup | pass, rc 0 | 272 | 0 | 3779 | 0 |
| base/class/String/pos.testGroup | pass, rc 0 | 272 | 0 | 4316 | 0 |
| base/class/String/replaceat.testGroup | pass, rc 0 | 272 | 0 | 1989 | 0 |
| base/class/String/reverse.testGroup | pass, rc 0 | 272 | 0 | 1980 | 0 |
| base/class/String/right.testGroup | pass, rc 0 | 272 | 0 | 3697 | 0 |
| base/class/String/round.testGroup | pass, rc 0 | 272 | 0 | 1780 | 0 |
| base/class/String/sign.testGroup | pass, rc 0 | 272 | 0 | 3179 | 0 |
| base/class/String/space.testGroup | pass, rc 0 | 272 | 0 | 8433 | 0 |
| base/class/String/startsWith.testGroup | pass, rc 0 | 272 | 0 | 1558 | 0 |
| base/class/String/strip.testGroup | pass, rc 0 | 272 | 0 | 5809 | 0 |
| base/class/String/subWords.testGroup | pass, rc 0 | 272 | 0 | 6322 | 0 |
| base/class/String/subchar.testGroup | pass, rc 0 | 272 | 0 | 1341 | 0 |
| base/class/String/substr.testGroup | pass, rc 0 | 272 | 0 | 5614 | 0 |
| base/class/String/subword.testGroup | pass, rc 0 | 272 | 0 | 5161 | 0 |
| base/class/String/translate.testGroup | pass, rc 0 | 272 | 0 | 2810 | 0 |
| base/class/String/trunc.testGroup | pass, rc 0 | 272 | 0 | 2881 | 0 |
| base/class/String/upper.testGroup | pass, rc 0 | 272 | 0 | 1350 | 0 |
| base/class/String/verify.testGroup | pass, rc 0 | 272 | 0 | 5101 | 0 |
| base/class/String/word.testGroup | pass, rc 0 | 272 | 0 | 3975 | 0 |
| base/class/String/wordindex.testGroup | pass, rc 0 | 272 | 0 | 4017 | 0 |
| base/class/String/wordlength.testGroup | pass, rc 0 | 272 | 0 | 3960 | 0 |
| base/class/String/wordpos.testGroup | pass, rc 0 | 272 | 0 | 6999 | 0 |
| base/class/String/words.testGroup | pass, rc 0 | 272 | 0 | 2337 | 0 |
| base/class/String/x2b.testGroup | pass, rc 0 | 272 | 0 | 4233 | 0 |
| base/class/String/x2c.testGroup | pass, rc 0 | 272 | 0 | 3880 | 0 |
| base/class/String/x2d.testGroup | pass, rc 0 | 272 | 0 | 4343 | 0 |
| base/class/Table.testGroup | pass, rc 0 | 272 | 0 | 9849 | 0 |
| base/class/Ticker.testGroup | pass, rc 0 | 272 | 0 | 3469 | 38 |
| base/class/TimeSpan.testGroup | pass, rc 0 | 272 | 0 | 3229 | 0 |
| base/class/Validate.testGroup | error, rc 2 | 272 | 0 | 10343 | 0 |
| base/class/WeakReference.testGroup | pass, rc 0 | 272 | 0 | 130256 | 0 |
| base/class/collections/array.testGroup | pass, rc 0 | 272 | 0 | 2762 | 0 |
| base/class/collections/bag.testGroup | pass, rc 0 | 272 | 0 | 2092 | 0 |
| base/class/collections/circularqueue.testGroup | pass, rc 0 | 272 | 0 | 1604 | 0 |
| base/class/collections/directory.testGroup | failure, rc 1 | 272 | 0 | 2988 | 0 |
| base/class/collections/list.testGroup | pass, rc 0 | 272 | 0 | 1931 | 0 |
| base/class/collections/properties.testGroup | pass, rc 0 | 272 | 0 | 1986 | 0 |
| base/class/collections/queue.testGroup | failure, rc 1 | 272 | 0 | 2427 | 0 |
| base/class/collections/stringtable.testGroup | pass, rc 0 | 272 | 0 | 3970 | 0 |
| base/directives/ANNOTATE.testGroup | refused at test does not parse here: 19.923: String or symbol expected. is not implemented (Phase 5) | 272 | 0 | 2257 | 0 |
| base/directives/ATTRIBUTE.testGroup | refused at test does not parse here: 25.925: Invalid subkeyword found. is not implemented (Phase 5) | 272 | 0 | 3440 | 0 |
| base/directives/CLASS.testGroup | refused at test does not parse here: 25.901: Invalid subkeyword found. is not implemented (Phase 5) | 272 | 0 | 2364 | 0 |
| base/directives/CONSTANT.testGroup | refused at constant_TestGroup does not parse here: 19.916: String or symbol expected as ::CONSTANT value. is not implemented (Phase 5) | 272 | 0 | 1839 | 0 |
| base/directives/METHOD.testGroup | refused at test does not parse here: 25.902: Invalid subkeyword found. is not implemented (Phase 5) | 272 | 0 | 2763 | 0 |
| base/directives/OPTIONS.testGroup | refused at test does not parse here: 25.927: Invalid subkeyword found. is not implemented (Phase 5) | 272 | 0 | 3107 | 0 |
| base/directives/REQUIRES.testGroup | refused at method "NEW" of class "Routine" is not implemented (Phase 9) | 272 | 0 | 1896 | 0 |
| base/directives/RESOURCE.testGroup | refused at method "ITEMS" of class "StringTable" is not implemented (Phase 9) | 272 | 0 | 1463 | 0 |
| base/directives/ROUTINE.testGroup | refused at test does not parse here: 19.903: String or symbol expected after ::ROUTINE keyword. is not implemented (Phase 5) | 272 | 0 | 1775 | 0 |
| base/expressions/ADDITION.testGroup | pass, rc 0 | 272 | 0 | 6802 | 0 |
| base/expressions/COMPOSITE.testGroup | pass, rc 0 | 272 | 0 | 1932 | 0 |
| base/expressions/CONCATENATION.testGroup | pass, rc 0 | 272 | 0 | 2724 | 0 |
| base/expressions/DIVISION.testGroup | pass, rc 0 | 272 | 0 | 13653 | 0 |
| base/expressions/EXPONENT.testGroup | pass, rc 0 | 272 | 0 | 4625 | 0 |
| base/expressions/Literals.testGroup | refused at method "NEW" of class "Routine" is not implemented (Phase 9) | 272 | 0 | 2070 | 0 |
| base/expressions/MULTIPLICATION.testGroup | pass, rc 0 | 272 | 0 | 7614 | 0 |
| base/expressions/PRECEDENCE.testGroup | pass, rc 0 | 272 | 0 | 44799 | 0 |
| base/expressions/REMAINDER.testGroup | pass, rc 0 | 272 | 0 | 16112 | 0 |
| base/expressions/SPECIAL.testGroup | pass, rc 0 | 272 | 0 | 7762 | 0 |
| base/expressions/SUBTRACTION.testGroup | pass, rc 0 | 272 | 0 | 9546 | 0 |
| base/keyword/ASSIGNMENT.testGroup | pass, rc 0 | 272 | 0 | 4023 | 0 |
| base/keyword/Assignments.testGroup | pass, rc 0 | 272 | 0 | 3093 | 0 |
| base/keyword/CALL.testGroup | refused at test does not parse here: 19.2: String or symbol expected after CALL keyword. is not implemented (Phase 5) | 272 | 0 | 2320 | 0 |
| base/keyword/DO.testGroup | refused at DO is not implemented | 272 | 0 | 3982 | 0 |
| base/keyword/DoControlled.testGroup | refused at test does not parse here: 35.1: Invalid expression. is not implemented (Phase 5) | 272 | 0 | 2400 | 0 |
| base/keyword/DoOther.testGroup | refused at test does not parse here: 27.1: Only one WHILE or UNTIL condition can be used on the same loop. is not implemented (Phase 5) | 272 | 0 | 1730 | 0 |
| base/keyword/DoOver.testGroup | refused at test does not parse here: 31.3: Name starts with number or ".". is not implemented (Phase 5) | 272 | 0 | 1918 | 0 |
| base/keyword/DoWith.testGroup | refused at DO is not implemented | 272 | 0 | 1934 | 0 |
| base/keyword/EXPOSE.testGroup | refused at method "NEW" of class "Routine" is not implemented (Phase 9) | 272 | 0 | 1492 | 0 |
| base/keyword/FORWARD.testGroup | pass, rc 0 | 272 | 0 | 1910 | 0 |
| base/keyword/GUARD.testGroup | refused at test does not parse here: 25.913: Invalid subkeyword found. is not implemented (Phase 5) | 272 | 0 | 1660 | 0 |
| base/keyword/IF.testGroup | pass, rc 0 | 272 | 0 | 6684 | 0 |
| base/keyword/INTERPRET.testGroup | refused at test does not parse here: 35.912: Missing expression following INTERPRET keyword. is not implemented (Phase 5) | 272 | 0 | 1672 | 0 |
| base/keyword/ITERATE.testGroup | pass, rc 0 | 272 | 0 | 2885 | 0 |
| base/keyword/LABEL.testGroup | refused at test does not parse here: 47.2: Unexpected label. is not implemented (Phase 5) | 272 | 0 | 2043 | 0 |
| base/keyword/LEAVE.testGroup | pass, rc 0 | 272 | 0 | 2299 | 0 |
| base/keyword/LOOP.testGroup | refused at LOOP is not implemented | 272 | 0 | 1593 | 0 |
| base/keyword/LOSTDIGITS.testGroup | error, rc 2 | 272 | 0 | 3458 | 0 |
| base/keyword/LabelOption.testGroup | pass, rc 0 | 272 | 0 | 1827 | 0 |
| base/keyword/LoopControlled.testGroup | refused at test does not parse here: 35.1: Invalid expression. is not implemented (Phase 5) | 272 | 0 | 2400 | 0 |
| base/keyword/LoopOther.testGroup | refused at test does not parse here: 27.1: Only one WHILE or UNTIL condition can be used on the same loop. is not implemented (Phase 5) | 272 | 0 | 1730 | 0 |
| base/keyword/LoopOver.testGroup | refused at test does not parse here: 31.3: Name starts with number or ".". is not implemented (Phase 5) | 272 | 0 | 1918 | 0 |
| base/keyword/LoopWith.testGroup | refused at LOOP is not implemented | 272 | 0 | 1932 | 0 |
| base/keyword/NOP.testGroup | refused at test does not parse here: 21.901: Invalid data on end of clause. is not implemented (Phase 5) | 272 | 0 | 1247 | 0 |
| base/keyword/NUMERIC.testGroup | refused at method "NEW" of class "Routine" is not implemented (Phase 9) | 272 | 0 | 5692 | 0 |
| base/keyword/PARSE.testGroup | failure, rc 1 | 272 | 0 | 29699 | 0 |
| base/keyword/RAISE.testGroup | failure, rc 1 | 272 | 19 | 4107 | 5 |
| base/keyword/REPLY.testGroup | pass, rc 0 | 272 | 15 | 2500 | 29 |
| base/keyword/SAY.testGroup | pass, rc 0 | 272 | 0 | 1382 | 0 |
| base/keyword/SELECT.testGroup | refused at test does not parse here: 7.2: WHEN or OTHERWISE expected. is not implemented (Phase 5) | 272 | 0 | 2321 | 0 |
| base/keyword/SIGNAL.testGroup | refused at method "NEW" of class "Routine" is not implemented (Phase 9) | 272 | 0 | 2302 | 0 |
| base/keyword/SelectCase.testGroup | refused at test does not parse here: 35.933: Missing expression following CASE keyword of a SELECT instruction. is not implemented (Phase 5) | 272 | 0 | 1779 | 0 |
| base/keyword/ShortCircuitAnd.testGroup | pass, rc 0 | 272 | 0 | 1463 | 0 |
| base/keyword/TRACE.testGroup | refused at DO is not implemented | 272 | 0 | 10677 | 0 |
| base/keyword/TRACE_TraceObject.testGroup | refused at DO is not implemented | 272 | 9 | 1776 | 2 |
| base/keyword/USE.testGroup | refused at test does not parse here: 35.930: Invalid or missing expression following "=" token of a USE ARG instruction. is not implemented (Phase 5) | 272 | 0 | 1974 | 0 |
| base/keyword/USELOCAL.testGroup | refused at test does not parse here: 99.910: USE LOCAL must be the first instruction executed after a method invocation. is not implemented (Phase 5) | 272 | 0 | 1347 | 0 |
| base/keyword/VarRef.testGroup | refused at USE LOCAL in a ::METHOD body is not implemented (Phase 5) | 272 | 0 | 2213 | 0 |
| base/rexxutil/SysDumpVariables.testGroup | refused at routine "SYSDUMPVARIABLES" is not implemented (Phase 10) | 272 | 0 | 1265 | 0 |
| base/rexxutil/SysFileDateTime.testGroup | refused at routine "SYSGETFILEDATETIME" is not implemented (Phase 10) | 272 | 0 | 1318 | 0 |
| base/rexxutil/SysFileSearch.testGroup | refused at routine "SYSFILESEARCH" is not implemented (Phase 10) | 272 | 0 | 1477 | 0 |
| base/rexxutil/SysFileTree.testGroup | no test ran, rc 3 | 272 | 0 | 798 | 0 |
| base/rexxutil/SysFileXXX.testGroup | refused at routine "SYSFILECOPY" is not implemented (Phase 10) | 272 | 0 | 2182 | 0 |
| base/rexxutil/SysFormatMessage.testGroup | refused at routine "SYSFORMATMESSAGE" is not implemented (Phase 10) | 272 | 0 | 1270 | 0 |
| base/rexxutil/SysSearchPath.testGroup | refused at routine "SYSSEARCHPATH" is not implemented (Phase 10) | 272 | 0 | 1362 | 0 |
| base/rexxutil/SysSleep.testGroup | failure, rc 1 | 272 | 0 | 2120 | 6 |
| base/rexxutil/SysStemCopy.testGroup | refused at routine "SYSSTEMCOPY" is not implemented (Phase 10) | 272 | 0 | 1410 | 0 |
| base/rexxutil/SysStemDelete.testGroup | refused at routine "SYSSTEMDELETE" is not implemented (Phase 10) | 272 | 0 | 1267 | 0 |
| base/rexxutil/SysStemInsert.testGroup | refused at routine "SYSSTEMINSERT" is not implemented (Phase 10) | 272 | 0 | 1259 | 0 |
| base/rexxutil/SysStemSort.testGroup | refused at routine "SYSSTEMSORT" is not implemented (Phase 10) | 272 | 0 | 1530 | 0 |
| base/rexxutil/platform/unix/tilde.testGroup | refused at routine "SYSTEMPFILENAME" is not implemented (Phase 10) | 272 | 0 | 1234 | 0 |
| base/rexxutil/platform/windows/SysBootDrive.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysCurPos.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysCurState.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysDrive.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysDriveMap.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysFileTree.testGroup | no test ran, rc 0 | 272 | 0 | 933 | 0 |
| base/rexxutil/platform/windows/SysGetXxxPathName.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysIni.testGroup | no test ran, rc 0 | 272 | 0 | 957 | 0 |
| base/rexxutil/platform/windows/SysIsFileDirectory.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysSystemDirectory.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysTextScreenRead.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysTextScreenSize.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysUnicode.testGroup | no test ran, rc 0 | 272 | 0 | 922 | 0 |
| base/rexxutil/platform/windows/SysWinVer.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/rexxutil/platform/windows/SysWin_xxx_Printer.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| base/security.manager/SecurityManager.testGroup | refused at USE LOCAL in a ::METHOD body is not implemented (Phase 5) | 272 | 0 | 4209 | 0 |
| base/source.file/SourceFile.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 1372 | 0 |
| base/source.file/incorrectCharacters.testGroup | error, rc 2 | 272 | 0 | 1566 | 0 |
| base/source.file/whiteSpace.testGroup | pass, rc 0 | 272 | 0 | 1804 | 0 |
| base/special.variables/RESULT_RC_SIGL.testGroup | failure, rc 1 | 272 | 0 | 1816 | 0 |
| doc/rexxref/chapter5/Section1.testGroup | refused at method "OBJECTNAME=" of class "Object" is not implemented (Phase 9) | 272 | 0 | 1236 | 0 |
| doc/rexxref/chapter7/Section4.testGroup | no test ran, rc 0 | 272 | 0 | 901 | 0 |
| extensions/dateparser/DateFormatter.testGroup | no test ran, rc 3 | 272 | 0 | 901 | 0 |
| extensions/dateparser/DateParser.testGroup | no test ran, rc 3 | 272 | 0 | 901 | 0 |
| extensions/hostemu/hostemu.testGroup | refused at the LIBRARY REXX entry point "rexx_clear_queue" is not implemented (Phase 10) | 272 | 0 | 1771 | 0 |
| extensions/json/json.testGroup | no test ran, rc 3 | 272 | 0 | 901 | 0 |
| extensions/json/json_02.testGroup | no test ran, rc 3 | 272 | 0 | 901 | 0 |
| extensions/platform/unix/ncurses/ncurses.testGroup | no test ran, rc 3 | 272 | 0 | 1511 | 0 |
| extensions/platform/windows/ole/ExcelQuickTest.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/ole/OLEObject.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/ole/OLEVariant.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/ole/Printers.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/ole/RexxProcess.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/ole/SpecialFolders.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/oodialog/Basic.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| extensions/platform/windows/rxwinsys/Clipboard.testGroup | no test ran, rc 0 | 272 | 0 | 933 | 0 |
| extensions/platform/windows/rxwinsys/WindowsEventLog.testGroup | no test ran, rc 0 | 272 | 0 | 949 | 0 |
| extensions/rxmath/RxMath.testGroup | pass, rc 0 | 272 | 0 | 13027 | 0 |
| extensions/rxregexp/rxregexp.testGroup | pass, rc 0 | 272 | 0 | 60663 | 0 |
| extensions/rxsock/socketClass.testGroup | no test ran, rc 3 | 272 | 0 | 901 | 0 |
| extensions/yaml/yaml.testGroup | no test ran, rc 3 | 272 | 0 | 901 | 0 |
| regressions/bug1853738.testGroup | pass, rc 0 | 272 | 0 | 1445 | 0 |
| regressions/bug2003_guard_when.testGroup | error, rc 2 | 272 | 0 | 1511 | 0 |
| regressions/bug2061_newline_misalignment.testGroup | no test ran, rc 0 | 272 | 0 | 921 | 0 |
| samples/samples.testGroup | no test ran, rc 3 | 272 | 0 | 798 | 0 |
| samples/scclient.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 668 | 0 |
| samples/scserver.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 667 | 0 |
| samples/sfclient.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 668 | 0 |
| samples/sfserver.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 667 | 0 |
| samples/windows/adsi.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 699 | 0 |
| samples/windows/fileNameDialog_demo.testGroup | no test ran, rc 0 | 272 | 0 | 947 | 0 |
| samples/windows/samples.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 699 | 0 |
| samples/windows/wmi.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 698 | 0 |
| utilities/rexx/rexx_command.testGroup | error, rc 2 | 272 | 0 | 5910 | 0 |
| utilities/rexxc/rexxc.testGroup | error, rc 2 | 272 | 0 | 2723 | 0 |
| utilities/rxapi.testGroup | refused at method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9) | 272 | 0 | 1225 | 0 |
| utilities/rxsubcom/rxsubcom.testGroup | failure, rc 1 | 272 | 0 | 2766 | 0 |
