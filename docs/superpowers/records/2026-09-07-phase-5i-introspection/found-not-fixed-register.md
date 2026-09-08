# Found and not fixed: the register Phase 5i exits owing

Every item here was measured during Phase 5i, is **not** Phase 5i's to fix, and has an owner.
Recorded at Moritz's request so the phase cannot close by forgetting them.

| # | What | Evidence | Severity | Owner |
| --- | --- | --- | --- | --- |
| 1 | **A message send in a multi-argument builtin's argument list corrupts that call's argument run.** `say substr(v, 1, length(.Array~id))` **panics and aborts the interpreter** at `run.rs:6151`; `say right(v, .Array~id~length)` reports `RIGHT` itself as missing an argument. The oracle answers both at rc 0. **It is already the cause of the only two `diverge` rows in `corpus/method-bodies.txt`** -- `DateTime~date` and `~timeOfDay`, whose `CoreClasses.orx` bodies are `date('F', self~standardDate, 'S')` and `time('F', self~LongTime, 'L')`. | `found-defect-builtin-argument-run.md` | **panic**, plus two live `diverge` rows | Phase 4 executor |
| 2 | **D59's four consequences**, reclassified from licensed to defects when Moritz corrected D59 on 2026-09-08: `~subclasses` retaining a dropped class; a `WeakReference` to a dropped class still answering it; `Interp::class_variables` as a permanent root; a class's `UNINIT` running later than the oracle's. D60 reopens with D59. | `d59-correction-and-exit-obligation.md` | leak + four observables | **after Phase 5i, before any new work** |
| 3 | `header_number` (`run.rs:8674`), `controlled_step_wide` (`:10379`) and `RAISE ADDITIONAL` (`:5417`) refuse loudly where the oracle answers `97.1`. Since Task 5 they are the **last** places a class object refuses. Closing them means teaching `run.rs` the operator send. | Task 5's report | loud, never wrong | Phase 4 executor |
| 4 | `.VariableReference~new` diverges: oracle rc 163 / `93.967`, this crate rc 120. One line with Task 2's `Raised::unsupported_new_method`. It is no table's row, which is why it was homeless. | Task 2's report | one line | whichever phase owns `VariableReference` |
| 5 | **Upstream candidate, one signal.** An `::ATTRIBUTE ... GET`/`SET` with a body reports its `~source` **one line late**, because `LanguageParser::hasBody` consumes the first clause without restoring the scanner's line position -- a three-line body answers `Array(2)`. This crate mirrors it deliberately. | Task 4's report | upstream | needs two more signals before filing |
| 6 | `stackFrames` taken inside an `INTERPRET` sees **four** frames here and **five** on the oracle, because a fragment runs inside the enclosing activation and pushes none. `~type`'s `INTERPRET` and `COMPILE` values are unreachable here. | Task 6's pre-flight | declared divergence | executor, if ever |
| 7 | The `Supplier` from `~methods`/`~instanceMethods` iterates in **hash order** on the oracle and **name order** here. No witness asserts it; matching means replicating the oracle's hash function. | Task 5's report | declared divergence | undecided |
| 8 | `StackFrame~executable` and `RexxContext~copy` are `Setup.cpp` rows in neither table, left loud and named rather than missed. | Task 6's pre-flight | loud | a later phase |

## Licensed, not owed

**`Object~hashCode` and `~identityHash` are licensed to diverge.** Moritz, 2026-09-08: *mirroring it
is not practical and limits our choices, and users shouldn't rely on it anyway.* Both are
address-derived; measured, **the oracle does not reproduce its own answer across two runs**, so
`corpus/introspection-arity.tsv` records them `unstable` and `corpus/method-bodies.txt` records them
`unstable  the oracle`. That verdict is the licence expressed in the instrument rather than in
prose, and `every_unstable_row_is_really_unstable` polices it by running the oracle twice and
failing if its two answers agree.

This is a **licence**, not a declined row: it is accepted permanently and nothing owes a fix.
