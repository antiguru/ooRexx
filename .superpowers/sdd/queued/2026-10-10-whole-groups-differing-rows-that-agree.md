# `whole_groups` DIFFERING rows that now agree

Found by Phase 6.1 Task 5 I2 (ledger: "DIFFERING rows that agree in both green runs"; "CALL whole now passes TEST_4 but the rest run failed it"). Queued by Phase 6.1 Task 12 (2026-10-10).

At `97cb37712` the `whole_groups` run (`REXX_CORPUS_GATE=1 ... --release --test concurrency_tests whole_groups`, exit 0) prints `listed as differing, agreeing:` for TIME whole and derived (normal and every), CALL derived (normal and every) and REPLY whole and derived (every). Its table, `.superpowers/sdd/2026-10-07-phase-6-1/task-12-evidence/whole-groups-table.tsv`, has TIME (524 and 81 assertions) and CALL (181 and 18) agreeing with the oracle in both modes; REPLY's every-mode run matches one of the oracle's racing outcomes. The harness allows a listed row to agree, so the stale `DIFFERING` entries (`concurrency_tests.rs`, the TIME rows citing the elapsed-clock defect Task 1 fixed) no longer state why they are there. CALL has no rest part since Task 5 I2, so the earlier "rest run failed TEST_4" cannot be re-run; CALL whole passes TEST_4 here.

No probe: a harness-table item. Suggested change: delete the TIME and CALL rows after two more green runs, and keep REPLY's every-mode rows, whose outcome races.
