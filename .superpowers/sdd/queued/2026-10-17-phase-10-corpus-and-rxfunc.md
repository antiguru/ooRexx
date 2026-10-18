# Phase 10: a real-program corpus, and RxFuncAdd with the daemon

Moritz, 2026-10-17, during the Phase 9d scoping. These are Phase 10's, not 9d's:

* **The corpus of real programs.** "move the corpus passes to phase 10, which integrates with
  rxapi and external libraries. Let's not load too much into 9d". The sources named so far are
  PPWizard and Josep Maria Blasco's Rexx Parser, with its own suites. Possible further sources
  are Rosetta Code's ooRexx tasks and Regina's `trip` tests.
* **PPWizard's bar.** "Patch it minimally, output must match the patched oracle." The two-line
  patch is in `docs/superpowers/plans/ppwizard-gate.md`.
* **Corpus sources** are fetched by a script with a pinned version and a sha256, and are never
  committed. This follows Moritz's ruling for the 2026-10-16 rx2cpp probe ("Don't include any of
  it in our repo"). PPWizard's licence forbids distributing `ppwizard.rex` alone, and Rosetta
  Code is GFDL.
* **`RxFuncAdd`, `RxFuncQuery` and `RxFuncDrop`** land with the daemon, with no in-process
  version: "It doesn't make sense to have an in-process version that we immediately replace."
  An open question goes with them. On the oracle a registration outlives the process in the
  daemon, so `RxFuncQuery` answers 0 on a later run (`rexx-exec/src/directives.rs:71`).

What was measured is in `docs/superpowers/records/2026-10-17-corpus-probe/README.md`.
