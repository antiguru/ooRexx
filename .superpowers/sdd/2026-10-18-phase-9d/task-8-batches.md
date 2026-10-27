# Task 8 instances (controller, 2026-10-21)

From the triage (task-8-triage.tsv, at 43690322b). Ruling: one instance per area batch, run one after
another, each with one implementer and a task review; the instance's items are listed below by queued
file name (under .superpowers/sdd/queued/) or gate line (docs/superpowers/plans/<file>:<line>). Each
item follows the Task 8 template (task-8-brief.md): witness from its probe (the triage probe beside
task-8-triage.tsv is a start), shown red; fix, or stop and report where the oracle is wrong; witness
matches; the item's file gets `**CLOSED <date> by Phase 9d Task 8.<n> (<sha>).**` as its first line
and its inventory row (docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv) `proposed` and `ruled` columns (4 and 6)
`closed:<sha>`, leaving the reason column as it is. A gate-line item is closed by a line in the instance's report and a row
`gate:<file>:<line>	closed:<sha>` appended to t8.<n>/closed.tsv. Semantics follow plan Ruled 13
(derive from rexxref, follow what you can model); a new oracle departure goes to the controller, never
into a Deviation. Cost if the batching is wrong: larger reviews.

Perf check (controller ruling 2026-10-22, after Task 8.3): run `callgrind.sh` on every program (no
`-p`) against the instance's BASE, not the five-program set; the five missed a +2.83% call-path
regression in 8.3. Own delta at most +0.05% libc subtracted on each program; schedule-dependent
SPREAD flags (pingguard, pingsem) are reported, not failed.

## 8.1 Crashes and activation lifetime
2026-10-14-routine-start-call-panic.md (rc 101 panic; first), gate:phase-9b1-gate.md:115,
2026-10-20-collect-stress-raise-additional-array.md, 2026-10-19-sigterm-in-trace-delivery.md,
2026-10-19-trace-destination-recursion-oom.md, 2026-10-10-uninit-drain-signal-interpret-and-trace.md,
2026-10-19-guarded-trace-destination-spin.md (large: fix if the pinned-guard-wait limit is tractable,
else report with a proposed owner, Phase 10).

## 8.2 Conditions and tracebacks
2026-10-01-message-start-result-traceback.md, 2026-10-20-call-on-delayed-reraise.md,
2026-10-20-halt-after-signal-on-syntax.md, 2026-10-10-reply-propagated-condition-traceback-line.md,
2026-10-10-leave-iterate-error-not-trappable.md, 2026-10-10-propagated-condition-extra-traceback-line.md,
2026-10-10-raise-novalue-return-into-call-on-any.md, 2026-10-10-directory-setmethod-unknown-traceback-line.md,
2026-10-02-while-condition-failure-traceback.md, gate:phase-9a-gate.md:471,
2026-10-21-lineout-nil-names-file.md, 2026-10-19-condition-deviation-candidates.md (Moritz's three
rulings are at its top: witness the two the crate already follows, implement RAISE RESULT, and give
the controller a Deviation candidate block for each of the three).

## 8.3 Call sites, frames, routines, context
2026-10-21-call-site-reads-kept-before-arguments.md (both shapes, incl. the Task 7.1 regression),
2026-10-21-raise-in-routine-keeps-call-site.md, 2026-10-01-routine-call-context-name.md,
2026-10-21-routine-new-call-frames.md, gate:phase-9c-gate.md:143, 2026-10-17-routine-call-frame-name.md
(large, .rxo format), 2026-10-21-copied-package-routine-executable.md,
2026-10-21-main-program-executable-annotations.md, 2026-10-19-context-name-during-install.md,
2026-10-15-9b1-task-8b-review-minors.md, 2026-10-15-isguarded-copy-and-additional.md.

## 8.4 Install order, directives, scanner
2026-10-18-duplicate-member-checked-at-install.md, 2026-10-18-install-error-names-caller-file.md,
2026-10-17-library-bind-before-options.md, gate:phase-9c-gate.md:142, gate:phase-9c-gate.md:145,
2026-10-17-scanner-reports-later-clause-error.md, 2026-10-18-options-trace-constant-expression.md, 2026-10-22-constant-frame-and-executable.md.

## 8.5 Variables
2026-09-30-drop-unset-stem-symbol.md, 2026-10-19-local-novalue-hook.md, 2026-10-19-variable-reference-gaps.md,
2026-10-10-variable-reference-concatenation.md, 2026-10-15-routines-table-silent-none.md,
gate:phase-9c-gate.md:138, 2026-10-19-variable-reference-deviation-candidates.md (Moritz's rulings at
its top: witness each and give the controller Deviation candidate blocks; the stem-identity one is a
sentence for Deviation 4).

## 8.6 Streams, paths, commands
2026-09-27-stream-write-visibility.md, 2026-10-13-stdin-stream-object-queries.md,
2026-10-13-stdout-write-errors-swallowed.md, 2026-10-13-lossy-path-sites.md, gate:phase-9c-gate.md:146,
2026-10-18-command-waits-for-background-child.md.

## 8.7 Builtins, collections, dispatch, security, native API
2026-10-10-beep-wrong-type-message.md, 2026-10-10-native-routine-arguments-not-strings.md,
2026-10-14-filespec-string-arg.md, 2026-10-14-class-side-defaultname.md, 2026-10-10-string-translate-empty-tables.md,
2026-10-10-array-sort-after-remove.md, 2026-10-19-sortwith-on-foreign-class.md (the shape now segfaults
the oracle: add it to rust/corpus/oracle-crashes.txt), 2026-10-19-index-argument-string-conversion.md,
2026-10-13-class-named-drops-raised-condition.md, 2026-10-10-security-manager-command-rc-requires.md,
2026-10-13-security-manager-and-library-key.md, 2026-10-17-newmethod-parent-package.md and
gate:phase-9c-gate.md:144 (unclear: native NewMethod; witness through the C API test harness or report), 2026-10-22-live-frames-omit-nested-primitives.md,
2026-10-22-executable-at-quadratic-depth.md,
2026-10-26-main-program-level-during-requires-install.md, 2026-10-26-compile-failure-condition-names-caller.md,
2026-10-26-library-rxmath-error-number.md, 2026-10-27-deadlock-report-shape.md,
2026-10-27-varref-operator-traceback-line.md.

## 8.8 Process items, unclear items, and items triage found fixed
Every `process` and `unclear` row of task-8-triage.tsv not named above, the gate lines
phase-9a-gate.md:456-459, and the `fixed` rows (re-check each, then close with `closed:probe agrees`
and the probe path), 2026-10-22-collect-stress-zero-collection-drift.md,
2026-10-26-file-date-time-dst-dependent.md, 2026-10-26-redirected-target-waits-timing-flake.md and 2026-10-27-reaped-child-test-load-flake.md.
