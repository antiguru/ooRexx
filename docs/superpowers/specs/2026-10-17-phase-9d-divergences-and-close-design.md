# Phase 9d: divergences, reachable refusals, the queued inventory, and the Phase 9 close

Parent spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md`, sections 3, 7, 8, 9
and 10. This document narrows and extends the parent's 9d row, and where the two disagree, this one
holds. Phase state at writing: 9c closed at `0e27a36ca` and was pushed at `71ab69cfe`
(`docs/superpowers/plans/phase-9c-gate.md`). Review: `.superpowers/sdd/2026-10-17-phase-9d-spec-review.md`.

## 1. Intent

Phase 9 ends when the interpreter is conformant enough to move on to Phase 10. Moritz, 2026-10-17:
"all but rxapi and some C interfaces implemented, deviations documented." For 9d that means three
things:
* every ooTest test with no exclusion row that the oracle passes either passes here or is
  licensed;
* every refusal a program can reach is either implemented or ruled;
* every known divergence, tested or not, has a disposition on record.

The corpus of real programs and `RxFuncAdd` move to Phase 10
(`.superpowers/sdd/queued/2026-10-17-phase-10-corpus-and-rxfunc.md`).

## 2. Goals

### 2.1 The 9d set

Every test with no exclusion row that fails here and passes on the oracle passes under the suite
harness, or has a
`suite-licences.txt` row citing a numbered Deviation that Moritz has ruled on. This is parent
section 3.

* **The first task re-derives `rust/corpus/suite-9d-set.txt`** from the 9c close's
  `docs/superpowers/records/2026-10-15-phase-9c/close/suite/tests.tsv`. Some rows of the current file already pass there.
* **The same task partitions the set into clusters** and commits the partition as a table that
  assigns every test to exactly one cluster. The parent's six clusters are a start. The table also
  covers the groups they miss: `SecurityManager`, `VarRef`, `TRACE`, `TRACE_TraceObject`, `CLASS`,
  `IdentityTable`, `RAISE` and the rest.
* **`samples/scclient` and `samples/sfclient`** are members like any other. Their failure at the 9c
  close is not yet measured. 9c fixed the front-end cause recorded at `phase-9a-gate.md:426-428`,
  and the crate loads C extension libraries under the harness. The cluster task measures them, and
  whatever they need follows the same rules as every other test.
* **Done means the suite's own assertion passes.** `l3_core_suite` exits 0 at the close. A run
  that exits 101 is not accepted in prose.

### 2.2 The pure-Rexx extension packages

`json`, `yaml` and `dateparser` belong to 9d (Moritz, 2026-10-17).

* **Mechanism.** `phase_10_group` (`rexx-exec/tests/support/suite_rows.rs`) makes every
  `extensions/` group Phase 10's by prefix. It gains an exception for the groups under
  `extensions/json`, `extensions/yaml` and `extensions/dateparser`. Their derived rows then leave
  `phase-9-exclusions.txt`. In `tests/suite.rs`'s fixture for the predicate, the `PASSING` row
  naming one of these groups moves to `FAILING`, and a C-extension group takes its place. These groups
  must pass under the suite harness per D1.
* **The work.** At the 9c close, json and dateparser already pass every test here. yaml fails
  `TEST35_DIRECTIVES` and `TEST47_MULTILINEFLOWCOLLECTIONS` on an `op_not_driven`-family refusal,
  which is an IR driver defect (`.superpowers/sdd/queued/2026-10-13-yaml-pass-boundary-guard.md`).
  The defect is fixed with a corpus witness. Its GUARD rows in `refusal-dispositions.tsv`, which
  call it unreachable, are corrected.
* **Which yaml source.** The harness's crate side loads the oracle's `build/bin/yaml.cls`, not the
  repository's `extensions/yaml/yaml.cls`, and the two differ. Fixes are checked against the copy
  the harness loads.
* The C extensions stay Phase 10: `rxmath`, `rxregexp`, `rxsock`, `rxunixsys`, `hostemu`,
  `ncurses` and the Windows ones.

### 2.3 Reachable refusals are covered and implemented

Moritz, 2026-10-17: "We should add test coverage for reachable sites."

* **Enumerate.** A task derives, statically and with a committed command, every class and method
  pair whose dispatch ends in `Loud::native_method` or in the generic fallthrough in `dispatch.rs`.
  The per-class method list is `rust/corpus/docs/class-methods.txt`, and the `loud` rows of
  `rust/corpus/method-bodies.txt` are the starting set. `hash.rs`'s refusal depends on the
  receiver's representation (the condition object, `.methods`), not on its class
  (`.superpowers/sdd/queued/2026-10-01-condition-object-directory-methods.md`), so those rows also
  name the receiver. The derived list is committed.
* **Cover.** Each pair a Rexx program can reach is witnessed against the oracle by a
  `corpus/lang/*.rex` program listed in `rust/corpus/phase-9.txt`. One program may witness several
  pairs. ooTest stays read-only and is
  not extended for this.
* **Implement.** Each reached pair is implemented as the oracle answers. The two sites 9a recorded
  as owed to 9d are among them (`phase-9a-gate.md:287`, `:291`): `StackFrame~executable` on a
  library routine frame, and `copy` of the REXX package.
* **Rule the rest.** A pair that stays refused needs Moritz's ruling: a numbered Deviation, or a
  later phase as owner. A pair no program can reach becomes an internal guard naming no phase,
  with a GUARD row that cites the derivation showing why it cannot be reached. No reachable
  refusal becomes a GUARD on an agent's judgement.

### 2.4 Every queued item has a disposition

Moritz, 2026-10-17: "all queued items should be inventoried."

* **The inventory.** It is a committed table with one row for each file in
  `.superpowers/sdd/queued/`, and for each item in the carried lists of the 9a, 9b, 9b1 and 9c gate
  records.
* **Dispositions.** Each row gets exactly one:
  * fixed in 9d, with its witness;
  * a numbered Deviation that Moritz has ruled;
  * owned by a later phase, named;
  * already closed, with its evidence;
  * not a divergence, such as a process or performance note, with its owner.
* **The batch ruling.** The inventory proposes a disposition for each row, and goes to Moritz
  as one batch (Moritz, 2026-10-18: "alternative is better and matches the intention"). 9d fixes
  only the rows he accepts for 9d and the rows already inside a 9d set cluster. Every other open
  row gets a named later phase as its owner.
* **Each queued file** gets a first line stating its disposition, as `2026-10-11-output-lost-on-kill.md`
  has.

### 2.5 No refusal names Phase 9

* **Remaining sites.** The three `Owner::Phase9` sites in `rexx-exec/src/lib.rs` are covered, along
  with the INTERPRET traceback row, which is the only open `OWNER: Phase 9` row in
  `phase-4-exclusions.txt`.
* **`SysFork` and named semaphores.** They stay refused under Deviations 42 and 43. Their refusals
  name the Deviation, and each refusal (`named_semaphore`, `fork_child`) gets its row in
  `refusal-dispositions.tsv`.
* **Deleting `Owner::Phase9`.** The `Owner::Phase9` variant and its `"Phase 9"` literals in
  `rexx-core/src/owner.rs` are deleted, together with every site and fixture that names them. The
  compiler enumerates the sites, and the fixtures include
  `dispatch/native/tests.rs` and `tests/internal_routines.rs`.

### 2.6 The Phase 9 close

The close meets parent section 10, all six criteria. It also makes section 7's 9d edits:
* close the INTERPRET traceback row;
* add `"Phase 9"` to `closed_phases`' `CLOSED` and remove it from `PHASES`, `OPEN` and
  `OPEN_PHASES`;
* teach `closed_phases` that `Phase 9` is not `Phase 9.1`, with a unit case for the `Phase 9.1`
  literal.

Beyond those:
* The roadmap's row 9 gets the `CLOSED` edit that rows 6 to 8 carry.
* Two records are written: `phase-9d-gate.md` for the sub-phase, and `phase-9-gate.md` for the
  phase. Parent criterion 1 names `phase-9-gate.md`, which holds the suite table and the
  oracle-difference report.
* The close waits on Moritz's rulings from 2.1, 2.3 and 2.4. The plan collects them as they arise
  rather than at the end.

## 3. Non-goals

* **The corpus of real programs, and `RxFuncAdd`, `RxFuncQuery` and `RxFuncDrop`.** These go to
  Phase 10 (`.superpowers/sdd/queued/2026-10-17-phase-10-corpus-and-rxfunc.md`). What was measured
  is in `docs/superpowers/records/2026-10-17-corpus-probe/README.md`.
* **The rxapi daemon.** It stays Phase 10, as do the `reaching_rxapi` and `Phase 10 Rxapi` rows.
* **The C extension libraries, the embedding API and a second interpreter instance.** These stay
  Phase 10, as does `Row10NativeApi`.
* **`SysFork` and named semaphores.** These are Deviations 42 and 43.
* **Windows-only groups.** They stay in `suite-not-run.txt` under `restrictOS`.
* **Extending ooTest.**
* **Performance work beyond the budget.** Row 9.1 owns it.
* **Language extensions.** These belong to a later research phase (Moritz, 2026-10-17).

## 4. Anti-goals

* **No copied oracle bugs.** Where the oracle is wrong, the crate answers sensibly, and Moritz rules
  a Deviation before the entry is written.
* **No silent licences or guards.** A test leaves the 9d set only by passing or by a licence row
  that names a ruled Deviation. A reachable refusal leaves the code only by being implemented or
  ruled.
* **No relabelling to finish.** A test moves to a Phase 10 row only when it reaches rxapi, a C
  library or the embedding API, and the row names which one.

## 5. Constraints

* **Global constraints carry over from the 9c plan.**
  * the oracle-run recipe and memcap;
  * the prose and shell rules;
  * no new `unsafe` without Moritz's grant;
  * specialization stays internal (`rust/CLAUDE.md`).
* **Performance budget.** At the close, every bench program stays within +0.5% Ir, whole process,
  of the Phase 9 pin (`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`, sha256 beginning
  `c4b4e8c4`). At the 9c close, `churn` was +0.165% and `sayloop` +0.145%.
* **Per-task perf check.** A task that changes the interpreter's run path runs
  `bench-programs/callgrind.sh -p "startup interpret parse churn sayloop"` against the pin. Its
  own delta must be at most +0.05% on each program. The cumulative figure stays under the close
  budget. This run is the one
  exception to the 5-minute per-task bound. New error and refusal construction stays `#[cold]` and
  out of line.
* **Checks.** Per task, at most about 5 minutes otherwise:
  * fmt;
  * clippy on the touched crates;
  * the touched test binaries and ooTest groups.

  Full gates, the TSan run and callgrind on every program run only at the close.
* **Deviations.** A new numbered Deviation is written only after Moritz rules on it, and the entry
  quotes his words.
* **Fix rounds.** Fix work on one cluster or one finding follows 6.1's stopping rule: three rounds,
  then the remainder goes to Moritz with a proposed owner or Deviation.
* **The close build.** `cargo test --no-run` runs under its own `memcap 16G`, before the gated
  test run under `memcap 8G`.

## 6. Decided

| Decision | By | Date |
|---|---|---|
| `json`, `yaml` and `dateparser` belong in 9d; the C extensions stay Phase 10 | Moritz | 2026-10-17 |
| The corpus and `RxFuncAdd`/`Query`/`Drop` move to Phase 10 | Moritz | 2026-10-17 |
| Reachable refusal sites get test coverage, and ooTest is not extended | Moritz; the medium is the controller's | 2026-10-17 |
| Every queued item is inventoried; dispositions go to Moritz as one batch, and 9d fixes only what he accepts or a cluster already holds | Moritz | 2026-10-17, 2026-10-18 |
| `SysFork` is not supported: Deviation 42 | Moritz | 2026-10-17 |
| Named RexxUtil semaphores are not supported: Deviation 43 | Moritz | 2026-10-17 |
| Fix rounds stop after three, and the remainder goes to Moritz | controller, from 6.1 | 2026-10-17 |

## 7. Left to the builder

* The cluster partition, and how each cluster is fixed.
* The static derivation's command, provided it is committed and re-runnable.
* The inventory's file format.
* The task order. A sensible one:
  1. re-derive and partition the 9d set, and measure `scclient` and `sfclient`;
  2. the queued inventory, because it may move items into clusters;
  3. the refusal enumeration;
  4. the extension packages;
  5. the clusters;
  6. the labels and the close.
