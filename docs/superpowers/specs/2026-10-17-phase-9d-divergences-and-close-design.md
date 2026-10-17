# Phase 9d: divergences, the pure-Rexx extensions, and the Phase 9 close

Parent spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md` (sections 3, 7, 8, 9
and 10). This document narrows and extends the parent's 9d row. Where the two disagree, this one
holds. Phase state at writing: 9c closed at `0e27a36ca` and was pushed at `71ab69cfe`
(`docs/superpowers/plans/phase-9c-gate.md`).

## 1. Intent

Phase 9 ends when the interpreter is conformant enough to move on to Phase 10. Moritz,
2026-10-17: "all but rxapi and some C interfaces implemented, deviations documented. And tested on
a larger corpus to gain coverage." The larger corpus goes to Phase 10 (Moritz, 2026-10-17: "move
the corpus passes to phase 10, which integrates with rxapi and external libraries. Let's not load
too much into 9d"). The real programs probed so far, PPWizard and the Rexx Parser, both stop at
`RxFuncAdd` and load C libraries, which is Phase 10's ground.

9d therefore has four parts:
* the parent's divergence work: the 9d set, the exclusion files and the labels;
* the pure-Rexx extension packages `json`, `yaml` and `dateparser`;
* the refusals that still name Phase 9;
* the Phase 9 close.

## 2. Goals

1. **The 9d set is empty.** Every test in `rust/corpus/suite-9d-set.txt` passes under the suite
   harness, or has a `suite-licences.txt` row citing a numbered Deviation that Moritz has ruled
   on. This is parent section 3, and its clusters stand as written there.
   * The file is stale: some of its tests already pass at the 9c close
     (`docs/superpowers/records/2026-10-15-phase-9c/close/suite/tests.tsv`). The first task
     re-derives it from that table.
   * `samples/scclient` and `samples/sfclient` need the `rxsock` C library
     (`extensions/rxsock/socket.cls:40`). They leave the set as `Phase 10 Extension` rows in
     `phase-9-exclusions.txt`. Parent criterion 2 allows Phase 10 rows there.
2. **`json`, `yaml` and `dateparser` pass** (Moritz, 2026-10-17). These are the ooTest groups
   under `extensions/json`, `extensions/yaml` and `extensions/dateparser`, which
   `phase-9-exclusions.txt` holds as `Phase 10 Extension` today. Their rows leave that file. Each
   group passes under the suite harness per D1, or a test carries a licence row citing a ruled
   Deviation. These packages are Rexx source, so they load through `::REQUIRES` like any program.
   A test that reaches a C library or rxapi from inside them keeps a Phase 10 row naming that
   library. The C extensions stay Phase 10: `rxmath`, `rxregexp`, `rxsock`, `rxunixsys`,
   `hostemu`, `ncurses` and the Windows ones.
3. **No refusal names Phase 9.**
   * **The three `Owner::Phase9` sites** in `rexx-exec/src/lib.rs`: the named-semaphore refusal,
     the `SysFork` refusal and `Loud::native_method`.
   * **Every OWNER row** in `docs/superpowers/plans/phase-4-exclusions.txt` that names Phase 9.
     Only the INTERPRET traceback row is open.
   * **`SysFork` and the named semaphores** stay refused under Deviations 42 and 43 (Moritz,
     2026-10-17). Their refusals stop naming a phase and name the Deviation.
   * **Each `Loud::native_method` caller with a fixed subject.** These are `object_protocol.rs`'s
     `OBJECTNAME=` and `COPY`, `hash.rs`'s, and `context.rs`'s `EXECUTABLE`. Each is either
     implemented as the oracle answers, where a probe reaches it, or becomes a refusal naming no
     phase. In the second case it gets a GUARD row in `refusal-dispositions.tsv` naming the probes
     tried (parent section 3, last bullet).
   * **The generic fallthrough in `dispatch.rs`**, reached by any method that no body table holds.
     It is accepted when no ooTest test reaches it at the close, and its GUARD row cites that run.
     (Controller's call.)
   * **The `Owner::Phase9` variant** and its `"Phase 9"` literals in `rexx-core/src/owner.rs` are
     deleted, with the match arms that name them. `closed_phases` scans `rexx-core`, so they cannot
     stay.
4. **The Phase 9 close.** Parent section 10, all six criteria, plus section 7's 9d edits:
   * close the INTERPRET traceback row;
   * add `"Phase 9"` to `closed_phases`' `CLOSED` and remove it from `PHASES`, `OPEN` and
     `OPEN_PHASES`;
   * teach `closed_phases` that `Phase 9` is not `Phase 9.1`.

## 3. Non-goals

* **The corpus of real programs** goes to Phase 10: PPWizard (patched minimally, its output
  matching the patched oracle), the Rexx Parser's own suites, and any further source. The 9d close
  writes Phase 10's handover note with what was measured on 2026-10-17. That covers the sources,
  their licences, PPWizard's two-line patch in `docs/superpowers/plans/ppwizard-gate.md`, the
  Rexx Parser's standalone suite invocation and the blockers seen. It also states the corpus rule:
  sources are fetched by script and never committed.
* **`RxFuncAdd`, `RxFuncQuery` and `RxFuncDrop`,** in-process or not. Moritz, 2026-10-17: "It
  doesn't make sense to have an in-process version that we immediately replace." They stay
  refused as Phase 10 Rxapi and land with the daemon.
* **The rxapi daemon,** which covers cross-process function registration, external queues and the
  macrospace. It stays Phase 10, as do the `reaching_rxapi` and `Phase 10 Rxapi` rows of
  `phase-9-exclusions.txt`.
* **The C extension libraries** (section 2, goal 2).
* **The embedding API and a second interpreter instance** (`Phase 10 Embedding`), and the classic
  native API rows (`Phase 10 Row10NativeApi`).
* **`SysFork` and named semaphores** (Deviations 42 and 43).
* **Windows-only groups.** They stay in `suite-not-run.txt` under `restrictOS`.
* **Performance work beyond the budget.** Row 9.1 owns it.
* **Language extensions.** Typing, `.Boolean` and the like belong to a later research phase
  (Moritz, 2026-10-17).

## 4. Anti-goals

* **No copied oracle bugs.** Where the oracle is wrong, the crate answers sensibly, and Moritz
  rules a Deviation before the entry is written.
* **No silent licences.** A test leaves the 9d set only by passing or by a licence row that names
  a ruled Deviation. Nothing is licensed on an agent's own judgement.
* **No relabelling to finish.** A test moves to a Phase 10 row only when it reaches rxapi, a C
  library or the embedding API, and the row names which.

## 5. Constraints

* **Global constraints** carry over from the 9c plan:
  * the oracle-run recipe and memcap;
  * the prose and shell rules;
  * no new `unsafe` without Moritz's grant;
  * specialization stays internal (`rust/CLAUDE.md`).
* **Performance.** Every bench program stays within +0.5% Ir, whole process, of the Phase 9 pin
  (`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`, sha256 beginning `c4b4e8c4`), measured at the
  close (parent section 8). The 9c close's worst program is `churn` at +0.165%. Each task keeps its
  own delta on `startup`, `interpret` and `parse` at most +0.05%. New error and refusal
  construction stays `#[cold]` and out of line.
* **Checks.** Each task's checks take about 5 minutes at most: fmt, clippy on the touched crates,
  the touched test binaries and the touched ooTest groups. Full gates, the TSan run and callgrind
  run only at the 9d close (parent section 9).
* **Deviations.** A new numbered Deviation is written only after Moritz rules on it, and the entry
  quotes his words.
* **Fix rounds.** Fix work on one failing cluster follows 6.1's stopping rule: three rounds, then
  the remainder goes to Moritz with a proposed owner or Deviation. (Controller's call.)
* **The close build.** `cargo test --no-run` runs under its own `memcap 16G` before the gated test
  run under `memcap 8G`. A release test build on a fresh target directory peaks past 8G (9c close,
  first run).

## 6. Decided

| Decision | By | Date |
|---|---|---|
| `json`, `yaml` and `dateparser` belong in 9d; C extensions stay Phase 10 | Moritz | 2026-10-17 |
| The corpus of real programs moves to Phase 10 | Moritz | 2026-10-17 |
| `RxFuncAdd`/`RxFuncQuery`/`RxFuncDrop` move to Phase 10 with the daemon; no in-process version | Moritz | 2026-10-17 |
| `SysFork` is not supported: Deviation 42 | Moritz | 2026-10-17 |
| Named RexxUtil semaphores are not supported: Deviation 43 | Moritz | 2026-10-17 |
| PPWizard (for Phase 10) is patched minimally, and its output must match the patched oracle | Moritz | 2026-10-17 |
| Corpus sources are fetched by script and never committed | controller, following Moritz's ruling for the rx2cpp probe | 2026-10-17 |

## 7. Left to the builder

* Task order. A sensible order:
  1. re-derive the 9d set;
  2. the extension packages, because they may expose language gaps the 9d set also hits;
  3. the 9d set clusters;
  4. the labels;
  5. the close.
* How each cluster is fixed, within the constraints.
