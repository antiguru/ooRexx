# Phase 8 L2 slice — documentation pass after the fix rounds

Read this first. It is your requirements.

## Where this fits

Phase 8's L2 slice closed at `e64202ae7`; a whole-branch review in three slices, a fix round
(F1-F11), a re-review, a residual round (X1-X5, Y1, Y2) and its re-review followed. Every
behavioural finding is fixed or explicitly parked. What is left is prose: claims in the phase's
documents that the review found false, facts the fix rounds changed, and gaps that were measured
and left and must be recorded where the project looks. You make those edits. You do not change code
or corpus programs.

Worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`. Check HEAD with
`git log -1` and read `rust/CLAUDE.md` and `docs/superpowers/records/README.md`.

## Sources of the findings, read the named items in full

All under `.superpowers/sdd/2026-09-14-phase-8/`:
* `final-review-c-claims.md` -- I1, I3, I4, I5, I6, I7 and M1-M10 (claims across the documents)
* `final-review-b-integration.md` -- B6's rows (the KNOWN GAPS "all predating Phase 8" sentence),
  B9, B12
* `final-fix-report.md` -- its "Not done" section: facts F1-F11 changed, and three divergences
  measured and left (the oracle build's `RUNPATH` empty element, `LIBRARY REXX`
  `unresolved_external` naming the program, loader/unloader)
* `fix-rereview-integration.md` -- R9 (parked: user `REQUEST` never sent), R6 (recorded in the ledger)
* `residual-fix-report.md` -- its "Not done" and "Follow-up" sections: Y1's oracle segfault and
  entry 15, `Method~new`/`Routine~new` refusing a third argument, the R10 assertion's blind spots,
  the `::ATTRIBUTE` getter-first order
* `residual-rereview.md` -- finding 4 (a class resolved through a package parent answers the
  unresolved symbol, pre-existing, silent) and its "loud, pre-existing, not counted" list
* `progress.md`'s "Z1 committed and reviewed by the controller" -- Z1's facts, the missing
  `Compiled method "NEW" with scope "Package".` traceback line (pre-existing), the entry 15
  precedent ruling, `~addRoutine`/`~addPackage` on a discarded package unmeasured
* `progress.md` -- the controller's rulings, especially "Triage of the final review" (the measured
  group partition and the `testbinaries/` evidence) and "Adjudication of the re-review residuals"

## The documents in scope

* `docs/superpowers/specs/2026-09-14-phase-8-scoping.md`
* `docs/superpowers/specs/2026-09-14-phase-8-native-api.md`
* `docs/superpowers/plans/2026-09-14-phase-8.md` (the executed L2 plan: correct false statements of
  fact; do not rewrite what the tasks asked for, which is a record of what was asked)
* `docs/superpowers/plans/phase-8-l2.md`
* `docs/superpowers/plans/phase-8-gate.md`
* `docs/superpowers/plans/phase-4-exclusions.txt`, its Phase 8 section (tests read this file:
  `rust/crates/rexx-exec/tests/{licensed_divergences,coverage,builtin_status,owners}.rs` and
  `rust/crates/rexx-inventory/src/lib.rs`; run those tests after editing it)
* `docs/superpowers/plans/2026-07-27-rust-rewrite.md`: the D-U1 block, the D5 amendment, roadmap
  row 8, D-L2, and the crate tree's `rexx-api` annotation (I6)
* `rust/corpus/phase-8.txt`'s comments
* **Not** `docs/superpowers/plans/2026-09-14-phase-8-surface.md` (the controller amends it
  separately) and **not** anything under `docs/superpowers/records/` (append-only, never retouched).

## What to do

1. **Corrections of fact.** Each finding named above that is a false or stale statement in a
   document in scope: correct it in every place the fact is stated (the reviews found facts restated
   in several documents where only some were updated; grep for each fact, and remember the files are
   hard-wrapped, so search for short distinctive fragments). Settled facts to use:
   * The API group partition, measured by the controller (progress.md, "Triage"): `METHOD` and
     `CONVERSION` load `orxmethod`, `FUNCTION` loads `orxfunction`, neither NEEDs an interpreter
     library nor imports a `Rexx*` symbol -- Phase 8. `INVOCATION` and `ProcessInvocation` load
     `orxinvocation`, which NEEDs `liborxexits.so`, which NEEDs `librexx.so.4`/`librexxapi.so.4` and
     imports `RexxCreateInterpreter` and `RexxStart` -- Phase 9, with `RexxStart` and
     `ProcessRexxStart`. `CLASSIC` loads `orxclassic` and registers `orxclassic1` through `rxfuncadd`,
     both importing the function, subcom, queue and macro-space registries -- Phase 10. Re-run the
     commands before writing them into a document (`readelf -d`, `nm -D --undefined-only` on the
     oracle checkout's `/home/moritz/dev/repos/ooRexx/build/lib`, `/bin/grep` over
     `ootest/ooRexx/API`), and put the commands beside the claim.
   * `api/` and `testbinaries/` are byte-identical between the oracle checkout and this worktree
     (`diff -rq`), and the oracle's `build/lib` holds every `testbinaries/` product.
   * Two `librxregexp.so` builds exist (I7/B9): the crate's in-crate tests load this worktree's
     `build/lib`, the corpus differential the oracle checkout's; both from identical sources, every
     measurement holds on both. Say which instrument loads which.
   * Every roadmap line citation into `2026-07-27-rust-rewrite.md` gets re-printed after your own
     edits to that file, since your insertions move lines (I3 is that failure).
2. **I4, the re-homing.** Record in `phase-4-exclusions.txt` and roadmap rows 9 and 10 that the
   groups above belong to Phases 9 and 10, with the measurement as the reason. (The surface plan's
   Task 7 will derive the partition as a test; your text should not claim that test exists.)
3. **Facts the fix rounds changed**, wherever a document in scope describes the old behaviour: the
   native string argument protocol (`REQUEST('STRING')` only, a `MAKESTRING` raise carried); the
   lineless boundary refusals against the declaring package and the result-side 93.968 that keeps
   its line; load failures in a required package naming it; `Libraries` holding only a loaded
   library, a version-refused library held and answering loaded on later asks with no routines,
   98.982 on the first ask at every load site; the search path taken once at start; `~package` of
   `EXTERNAL` methods and the shared per-procedure record for `loadExternal*` (methods per spelling,
   routines one object per table entry found caselessly, the first binder's package, bound as each
   directive resolves); a package whose translation raised is not kept and keeps no routine, method or resource
   table and no prolog (Z1); every interface slot
   `unsafe extern "C" fn`; `value_of` `unsafe`. Only where a document states the contrary or
   describes the mechanism; do not add a changelog.
4. **KNOWN GAPS**, appended to the Phase 8 section of `phase-4-exclusions.txt`, each with its
   evidence (probe path or transcript excerpt, both sides) and an owner phase or "no owner" with the
   reason: B12 (condition object `~program`/`~position`/`~traceback` attribution, predates); R9 (user
   `REQUEST`); the oracle build's `RUNPATH` empty element making its `dlopen` search the working
   directory; `LIBRARY REXX` `unresolved_external` naming the program; the crate running no package
   loader or unloader; Y1's oracle segfault (point at `rust/corpus/oracle-crashes.txt` entry 15);
   `Method~new`/`Routine~new` refusing a third argument; the `::ATTRIBUTE` getter-first order;
   Miri outside the gate and the one instrument it is for F7/F8; `collect_stress`'s L0 test blocking
   the stress check of `phase-8.txt`. Correct the "all predating Phase 8" sentence (B6).
5. **`phase-8-gate.md`**: append a section recording the final review, the fix round, the residual
   round with its follow-ups (Y1, Y2, Z1) and their re-reviews (counts, what was fixed, what was
   parked and where). **Do not write gate readings**: the controller runs the gates at the commit
   after yours and appends that section itself. No placeholder for it. Correct M1 and M9 in place.

## Constraints

* Markdown rules: concise technical prose, active voice, `*` for lists, headers capitalised on the
  first word only. No claim without evidence; if you cannot find the evidence for a correction, say
  so in the report instead of writing it.
* Every number measured or quoted from a report that measured it, with the source named. Never
  state a set's size where the set can be named (`no-set-cardinality`), except in gate readings.
* Every citation you add or keep in an edited paragraph: print the line and confirm it lands.
* Do not edit `docs/superpowers/records/`, the surface plan, code, or corpus programs. `phase-8.txt`
  comments only.
* Oracle runs, if you need one: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory, three descriptors as three files. `oracle-crashes.txt` first.
* After editing `phase-4-exclusions.txt`: `cargo test -j 4 -p rexx-exec --test licensed_divergences
  --test coverage --test builtin_status --test owners` and `cargo test -j 4 -p rexx-inventory`.
* Stage explicit paths; commit per document group with `git commit -F`, ending with:

  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD
  ```
* No subagents. Ask before editing if a correction's evidence contradicts a finding.

## Report

`.superpowers/sdd/2026-09-14-phase-8/docs-pass-report.md`, written first: one row per finding
(corrected where, with the commit; or not corrected and why), the commands you re-ran, and "Not
done". Message the controller when finished.
