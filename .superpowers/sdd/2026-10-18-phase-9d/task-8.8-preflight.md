# Task 8.8 preflight inventory

This inventory is a static read at `0c2ff21ae1844e86c1937b58fc82f080fc43b5ab`, matching the expected revision. It covers 54 Task 8.8 entries: 35 unassigned triage rows (20 process, 13 fixed, 2 unclear), four explicit gate lines, and 15 named queued files. The entries comprise 38 queued files and 16 gate references, plus five inventory reconciliation rows listed separately. No oracle, build, interpreter, or test was run, and no existing status marker was changed.

Sources: `task-8-batches.md` header and 8.8, `task-8-triage.tsv`, `global-constraints.md`, all 38 selected queued files, `queued-inventory.tsv`, the carried gate text and Phase 9a progress notes. Counts were derived with Python csv/re: select triage rows whose status is process/unclear/fixed and whose item is absent from the batch text preceding `## 8.8`, add gate lines 456 through 459, then add the dated filenames following that heading. Inventory discrepancies were derived by checking each queued file first line for `**CLOSED` while either proposed or ruled equals `9d`. Code searches only confirm the explicitly described current anchors and are not behavioral verification.

## Sequential subgroups

* P1: Make harness verdicts safe: binary selection, process ownership, byte comparisons, empty selection, stdin deadline coverage.
* P2: Diagnose timing and environment flakes after P1; combine both redirected-target entries but retain both ledger closures.
* P3: Repair measurement instruments and mutation/rooting evidence; coordinate collect_stress work once.
* P4: Resolve comments, generated pointers, and limited carry-in records.
* P5: Recheck fixed and no-behavior items against their full stated contracts, then close only proven scope.
* P6: Resolve unclear/unconstructible cases with bounded witnesses or controller judgements.
* P7: Implement semantic residuals serially, deriving contracts and reusing existing conversion paths.
* P8: Obtain explicit semantic timing ruling and record close-only handoffs; never run Phase 9/10 close.

## Per-item inventory

Queued paths below are relative to `.superpowers/sdd/queued/`. Probe paths are relative to `docs/superpowers/records/2026-10-18-phase-9d/` unless a full repository path is given. “Fixed” means triage classification only, and requires a fresh recheck before closure. Every selected queued file remains open in inventory with proposed=ruled=`9d`; two have leading RULED markers, and none has a leading CLOSED marker.

| Path | Current state | Affected code/tests or known probe | Ruling needed / acceptance | Group |
| --- | --- | --- | --- | --- |
| `2026-09-16-worktree-build-precondition.md` | OPEN; process triage | dispatch/library/tests.rs, run/tests/routines.rs, rexx-api/tests/{load,invoke,context}.rs; fresh-checkout enumeration required | Choose actionable precondition versus automatic build; enumerate by running, never silently skip | P1 |
| `2026-09-17-oracle-deadline-is-wall-clock.md` | OPEN; process triage | tests/support/oracle.rs:53 still 10s; gate_tables/mod.rs; recorded high-load failures | Choose CPU bound, load reporting, or one reported retry; genuine hang must stay red | P2 |
| `2026-09-24-arity-harness-ignores-target-dir.md` | OPEN; process triage | tests/support/arity.rs:399 still ../../target/release/rexx; group_runner uses CARGO_BIN_EXE_rexx | Reuse Cargo binary path; resolve requested stale-binary safeguard | P1 |
| `2026-09-21-flaky-unstable-row-gate.md` | OPEN; process triage | tests/support/arity.rs:599 stable_rows_marked_unstable still compares two runs | Choose property or sampling policy; state justified residual false-failure rate | P2 |
| `2026-09-24-mutate-4c-stale-anchor.md` | OPEN; process triage | scripts/mutate-4c.sh mutation 5; parse_template.rs; git log -S requested | Re-anchor and show caught; bounded mutation run, no unrelated script rewrite | P3 |
| `2026-09-24-refusal-scanner-test-files.md` | OPEN; process triage | tests/refusal_sites.rs code_lines and surface; planted file-module refusal and constructor move probes | Cover both file-level cfg(test) exclusion and impl ownership; deferred Loud move is separate scope | P3 |
| `2026-09-25-pos-window-doc-pointers.md` | OPEN; process triage | builtin/string.rs find_forward; builtin/string/tests.rs; rexx-parse/block/references.rs; git show ac15d1042^ | Comment restoration only; preserve read-only oracle-crashes.txt | P4 |
| `2026-09-25-table-c-derivation-literals.md` | OPEN; process triage | tests/table_c/{probes,checks}.rs; corpus/gate-tables/{classes,hierarchy,methods}; README.md:95 | Reword source and rederive probes together | P4 |
| `2026-10-05-stdin-chars-after-drain.md` | OPEN; fixed triage | queued-probes/stdindrain/stdindrain.rex plus stdin.txt; triage stdindrain, stdinchars* shapes | Recheck piped shapes; queued OPEN claim conflicts with triage fixed | P5 |
| `2026-10-05-stream-fifo-terminal-seek.md` | OPEN; fixed triage | queued-probes/fifo/fifo.rex; triage fifo.rex/fifow.rex; terminals not probed | Recheck FIFO read/write with writer fds redirected; decide terminal coverage, preserve EINTR rule | P5 |
| `2026-10-07-unjudged-scheduler-mutants.md` | OPEN; process triage | Phase 6 final-review.md; switch_to M6/M7, cancel_wait M9/M10/M11, object_roots M12 | Each needs seeded witness or explicit unobservable judgement | P3 |
| `2026-10-10-failed-spawn-runs-inline.md` | OPEN; unclear triage | Phase 6.1 task-11b-review.md M2; native pool spawn fallback; no recorded runnable probe | Find bounded fault-injection witness or controller ruling; suggested 3GB address-space setup conflicts with triage cap concern | P6 |
| `2026-10-10-rooting-harness-blind-to-dropped-pushes.md` | OPEN; process triage | tests/collect_stress.rs; truth judgement and condition_value pushes; Phase 6.1 task-4a-review.md | Witness actual collection while only root holds value; keep reqstr positive control | P3 |
| `2026-10-10-trace-c-command-echo.md` | OPEN; fixed triage | queued-probes/cc2/cc2.rex; command trace ADDRESS clause | Recheck all three streams/status; OPEN claim conflicts with triage fixed | P5 |
| `2026-10-13-named-semaphores-design.md` | OPEN; fixed triage | triage p03; Deviation 43 refusal; phase-4-exclusions.txt | Already ruled unsupported; verify label, never implement sem_open | P5 |
| `2026-10-13-phase-9b-carry-in.md` | OPEN; process triage | triage p27/p28; .NAME miss memo m1, objname-table README m3, metaclass pin m4 | Only outstanding minors here; N1 belongs 9.1; Phase 9 close budget outside authorization | P4 |
| `2026-10-13-sysfork-design.md` | OPEN; fixed triage | triage p04; Deviation 42 refusal; phase-4-exclusions.txt | Already ruled unsupported; verify label, never implement fork | P5 |
| `2026-10-13-yaml-pass-boundary-guard.md` | OPEN; fixed triage | triage p21/I4 repro with oracle yaml.cls; Task 4 records | Recheck, including repo yaml copy; triage fixed is not full yaml-group proof | P5 |
| `2026-10-15-executable-flags-and-result.md` | OPEN; fixed triage | triage p14; define shapes isGuarded/isPrivate/isProtected | Recheck flags; RESULT reachability is licensed GC timing, not new semantics | P5 |
| `gate:phase-9a-gate.md:460` | process triage | Phase 9a progress.md Task 3 minors 3/5/6; owner equality, source_test_names_in, fixtures/mutants | Evidence/fixture judgement, not runtime change; record dispositions for each minor | P3 |
| `gate:phase-9a-gate.md:461` | process triage | refusal row at load; triage says no such row exists | Recheck absence; close only with evidence, do not invent a row | P5 |
| `gate:phase-9a-gate.md:462` | process triage | ffi.rs two Phase 10 refusal owner texts; owners/phase_ten_reasons tests | Pin both owners safely outside extern panic-abort paths | P3 |
| `gate:phase-9a-gate.md:466` | process triage | condition-object build, refuse_routines_write scan, eager annotations, cumulative perf | Phase 9 close callgrind owner explicit in triage; record handoff without executing Phase 9 close | P8 |
| `gate:phase-9a-gate.md:472` | process triage | rexx-bench/src/bin/rexx-bench-suite.rs:642 every_declared_runnability_still_holds; rust_binary_candidates | Honor target dir explicitly; cross-package Cargo binary availability differs from arity tests | P1 |
| `gate:phase-9a-gate.md:474` | fixed triage | triage p24, instanceMethods shadowed entries: nil 32/32; subclass 66/57 distinct | Recheck agreement; close fixed, no dedup semantics change | P5 |
| `gate:phase-9a-gate.md:476` | fixed triage | triage p26; REXX package copy; native routine frame executable requires RxFuncAdd | Recheck reachable copy shape; native frame subpart remains ruled Phase 10 | P5 |
| `gate:phase-9a-gate.md:481` | unclear triage | method_object_for class_owns on non-class scope; no known .nil-scoped class-side entry | Find constructible native/Rexx witness or controller judgement of unreachable path | P6 |
| `gate:phase-9a-gate.md:482` | process triage | sim gate debug_ms and MutexSemaphore/STREAM k calibration | Re-derive tooling values; preserve retained REPLY every-mode rows | P3 |
| `gate:phase-9a-gate.md:483` | fixed triage | suite-licences.txt TEST_SLEEP_DURATION; concurrency_tests S2 licence path | Verify S2 path as well as suite row; triage records only suite licence existence | P5 |
| `gate:phase-9a-gate.md:488` | fixed triage | triage p21/I4; yaml TEST35_DIRECTIVES and TEST47_MULTILINEFLOWCOLLECTIONS | Deduplicate yaml repro with queued guard item; full yaml group required for broader claim | P5 |
| `gate:phase-9b1-gate.md:121` | fixed triage | triage p03/p04, SysFork and named semaphore refusals | Share two fixed-item probes; respect Deviations 42/43 | P5 |
| `2026-10-18-class-methods-no-argument.md` | RULED 9d; fixed triage | queued-probes q07/methods_count.rex; refusals-probes/seeds/methods_count.rex | Respect RULED 9d; triage counts agree but name sets were not compared | P5 |
| `2026-10-18-sysfiletree-tree-directory-flaky.md` | RULED 9d; process triage | t6.2/suite-sysfiletree-{1,2,3}; base/rexxutil/SysFileTree TEST_TREE_DIRECTORY | Find cause before fix; new Flaky licence requires Moritz | P2 |
| `2026-10-21-redirected-target-timed-line-flake.md` | OPEN; process triage | corpus/lang/command_redirected_target_waits_elsewhere.rex; triage q14; sim-gate.tsv | One combined diagnosis with 2026-10-26 redirected-target row; eliminate timing threshold or justified sim | P2 |
| `2026-10-21-quick-native-calls-wall-bound-flake.md` | OPEN; process triage | scheduler/tests/native.rs:349; quick_native_calls_at_the_smallest_bound_run_alike | Make independent of elapsed wall time or deterministic sim | P2 |
| `gate:phase-9a-gate.md:456` | open triage | dispatch/library/tests.rs:177 still says phase-8.txt last | Correct stale explanation against current collect_stress behavior, preserve useful comment | P4 |
| `gate:phase-9a-gate.md:457` | open triage | tests/support/group_runner.rs:823 skip/renamed name conversion uses from_utf8_lossy | Compare method-name bytes without invalid-UTF8 collisions | P1 |
| `gate:phase-9a-gate.md:458` | open triage | tests/support/group_runner.rs:309 contains uses windows(needle.len()) | Define empty-needle contract, then guard windows(0) | P1 |
| `gate:phase-9a-gate.md:459` | open triage | tests/suite.rs:831 selected still accepts empty REXX_SUITE_GROUPS; progress.md Task 2 minors 5/7/9/10 | Recheck all listed minors: bound truncation, CapKind label constants, svn dependency, Ended doc; empty selection must fail visibly | P1 |
| `2026-10-22-collect-stress-zero-collection-drift.md` | OPEN; named 8.8 | tests/collect_stress.rs NO_ALLOCATION_PROGRAMS; t8.3/probes/collections-base-head.txt | Trace dropped programs and +4 collections in call_trace_nested/trace_output/trace_results/mutation_controlled_order; classify regression versus accepted allocation | P3 |
| `2026-10-26-file-date-time-dst-dependent.md` | OPEN; named 8.8 | corpus/lang/rexxutil_file_date_time.rex line 9, 2020-10-25 02:30 Europe/Zurich | Derive ambiguous-time semantics before deciding TZ pin versus interpreter fix | P2 |
| `2026-10-26-redirected-target-waits-timing-flake.md` | OPEN; named 8.8 | collect_stress the_l0_subset; command_redirected_target_waits_elsewhere time(e)>0.9 | Same root as 2026-10-21 row; prove wait without wall threshold | P2 |
| `2026-10-27-reaped-child-test-load-flake.md` | OPEN; named 8.8 | tests/rexxutil_process.rs:97 a_command_whose_child_was_reaped_elsewhere_fails_with_127; t8.4/close-workspace-tests.txt | Synchronize actual reap race or prove wrong runtime behavior; isolated reruns are not closure | P2 |
| `2026-10-29-suite-fixture-stderr-refusal-load-flake.md` | OPEN; named 8.8 | tests/suite.rs fixtures::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally; t8.7c/fix-r2/close-workspace-tests.txt | Find cause before known-flake entry; re-review could not reproduce | P2 |
| `2026-10-29-oracle-harness-stdin-deadlock.md` | OPEN; named 8.8 | tests/support/oracle.rs:217 synchronous write_all before readers/deadline; other write_all sites 311/355 need extent check | Concurrent stdin plus readers/deadline; pipe-sized never-reader witness must terminate red | P1 |
| `2026-10-29-pow-integral-exponent-overflow.md` | OPEN; named 8.8 | rexx-num/src/pow.rs; Number::whole_value in lib.rs:712; codex-review/pow-integral-exponent.rex | Derive exponent integer/digit ceiling from rexxref; reuse converter only if contract fits | P7 |
| `2026-10-29-hash-collection-messages-sent.md` | OPEN; named 8.8 | t8.7c/probes/primframe/f4.rex; Table/Set removals and Bag/Relation queries | Preserve ruled index == send (Keep rexxref, Deviation); rule additional sends by language, not blind oracle match | P7 |
| `2026-10-29-trace-assignment-rendered-once.md` | OPEN; named 8.8 | t8.7c/probes/dn/dn7.rex; review-probes-rr2/c1/d.rex raising third DEFAULTNAME | Derive per-line sends; preserve raising conversion; traced perf exemption wording needs controller scope | P7 |
| `2026-10-29-stdout-query-size-position-file.md` | OPEN; named 8.8 | t8.7c/probes/stdinq/q5.rex; stream QUERY SIZE/POSITION regular-file stdout | Derive iostrms.xml contract with Deviation 40 buffering; redirected-file witness | P7 |
| `2026-10-29-stem-index-string-values.md` | OPEN; named 8.8 | t8.7c/review-probes-rr1/r1a/a.rex, r1b/c.rex, r1c/d.rex; Stem [] | Required string value, makeString/NOSTRING, reference and stem index shapes | P7 |
| `2026-10-29-grandchild-sleeps-process-wide.md` | OPEN; named 8.8 | tests/suite.rs:1978 grandchild_sleeps scans /proc; fixture at 2002 | Scope process check to own run tree; coordinate same-file fixture work sequentially | P1 |
| `2026-10-30-raise-description-unsent.md` | OPEN; named 8.8 | t8.7c/review-probes-rr2/n2b/q9.rex, n2c/u.rex; instrc.xml:2866-2874 | REQUIRED: rule conversion timing, rexxref at RAISE versus oracle at condition(d); stop before new Deviation | P8 |
| `2026-10-30-number-max-object-argument.md` | OPEN; named 8.8 | t8.7c/review-probes-rr2/ld/l20.rex; 5~max(k), numeric digits 5 | Use argument string value; preserve numeric rulings/Deviation 44 LOSTDIGITS behavior | P7 |
| `2026-10-30-parse-pattern-conversion-raise.md` | OPEN; named 8.8 | t8.7c/review-probes-rr2/c2/onceparse.rex; PARSE variable pattern conversion | Required conversion raising must raise in clause; reuse I1 conversion rule | P7 |

## Closed-file reconciliation

These five files are already CLOSED and must stay closed. Only the inventory proposed/ruled columns need reconciliation, leaving reason text unchanged. The first three first lines omit a SHA, so the literal instruction to copy a SHA from that line cannot be followed. File history identifies `fcb358fd7` as their closure-record commit, which the controller must confirm before using it.

| Queued path | Current state | Evidence / value available | Ruling needed | Group |
| --- | --- | --- | --- | --- |
| `2026-10-02-trace-entry-after-expose.md` | CLOSED; inventory 9d/9d | Task 6.10 first line has no SHA; `t6.10/queued/`; history `fcb358fd7` | Confirm closure SHA rather than inventing one | P4 |
| `2026-10-02-traced-init-routed-sink-silent-exit.md` | CLOSED; inventory 9d/9d | Task 6.10 first line has no SHA; `t6.10/queued/`; history `fcb358fd7` | Confirm closure SHA rather than inventing one | P4 |
| `2026-10-02-triggered-exit-trace-line-unrouted.md` | CLOSED; inventory 9d/9d | Task 6.10 first line has no SHA; `t6.10/queued/`; history `fcb358fd7` | Confirm closure SHA rather than inventing one | P4 |
| `2026-10-18-max-integer-unrounded.md` | CLOSED; inventory 9d/9d | `009fd1978` in first line | Set both to `closed:009fd1978` | P4 |
| `2026-10-18-fuzz-comparison-lostdigits.md` | CLOSED; inventory 9d/9d | `009fd1978` in first line | Set both to `closed:009fd1978` | P4 |

## Scope and requirement conflicts

* The gate 466 item explicitly belongs to Phase 9 close callgrind, and carry-in N1 explicitly belongs to Phase 9.1. Keep accountable handoffs without doing either close or expanding 8.8 into performance optimization.
* RAISE DESCRIPTION needs Moritz’s ruling on conversion timing: documentation says convert at RAISE, the oracle converts at condition(d). New oracle departures stop at the controller before a new Deviation is written; existing RULED/Deviation markers remain authoritative.
* SysFork and named semaphores are unsupported under Deviations 42/43. Their old design text is not authorization to implement them; recheck the refusal labels and close that bounded item.
* The failed-spawn triage says its suggested setup is outside the 2G cap, while the original description is a virtual-address-space limit. Those limits are different quantities; establish a compliant resource envelope or fault-injection witness instead of assuming a cap exception.
* Fixed triage conflicts with stale OPEN behavior descriptions for stdin drain, FIFO, command echo, yaml and flags. Recheck rather than blindly copying either claim; terminals, method name sets, S2 licensing, repo yaml copy and native frame subparts exceed the narrow evidence currently recorded.
* The header’s later controller perf rule supersedes the global five-program rule: every program, no -p, against the instance BASE, own delta at most +0.05% libc-subtracted; report scheduler SPREAD flags. Global checks have an aggregate ten-minute bound, so a large 8.8 implementation needs controller-managed sequential instances rather than one giant run that silently exceeds it.
* The trace assignment item says traced performance is no target, but this does not explicitly waive the broader run-path perf rule. Ask the controller to settle any intended exemption; do not omit mandatory performance proof by inference.
* Comments/docs and process-only items cannot always provide a red oracle witness. Use the relevant mutation, fresh-checkout failure, static stale-pointer proof or structural test, and explicitly record non-behavioral dispositions instead of manufacturing a semantic difference.
* A new Flaky licence requires a cause and Moritz’s ruling where the queued item says so. Repeated isolated passes cannot close the SysFileTree or suite-fixture failures.
* Closure notation differs: verified fixed items use `closed:probe agrees` plus a durable probe path, ordinary implemented items use `closed:<sha>`, and gate items also append `gate:<file>:<line>	closed:<sha>` to the instance closed.tsv. Reconcile the three SHA-less historical CLOSED markers before applying their inventory update.

## Minimal ordering and stop condition

Run P1 before collecting new process-flake evidence so pipe deadlocks, stale executables and cross-run process checks do not corrupt the result. Run P2 and P3 next, grouping shared files and duplicated probes while retaining separate item dispositions. P4 and P5 can then retire evidence-backed bookkeeping and fixed rows, followed by bounded P6 judgements and serial P7 semantic work. Resolve the P8 timing ruling before that implementation and leave close-only performance work explicitly handed off, with all remaining 8.8 entries accounted for.

Implementation must retain the global memory caps, foreground bounds, durable record paths, shared-path ownership, three-round limit, required suite/corpus/refusal checks, and per-run command/status records. A fresh witness goes in phase-9.txt with its generated sourceline oracle fixture, and collection output is compared as a multiset. No new unsafe site, representation refusal or numeric-id side map is permitted. Task 8.8 completion does not authorize Phase 9d Tasks 9/10 or Phase 9/10 closure.
