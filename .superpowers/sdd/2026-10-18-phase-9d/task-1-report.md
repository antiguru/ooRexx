# Phase 9d Task 1 report

## Status

DONE. Measurement only, no source change.

## Commits

* `8120c43fb` Phase 9d Task 1: the 9d set re-derived and partitioned into clusters.
  `rust/corpus/suite-9d-set.txt`, `docs/superpowers/records/2026-10-18-phase-9d/clusters.tsv`,
  `clusters.md`, `causes/<group>/` and `probes/<name>/`.
* This report is committed separately (see `git log`).

## Re-derivation

From `docs/superpowers/records/2026-10-15-phase-9c/close/suite/tests.tsv` (9c close `25197002a`):

    awk -F'\t' '$3=="-" && $4=="passed" && $5=="failed" {print $1"\t"$2}' tests.tsv

Row count: `grep -vc '^#' rust/corpus/suite-9d-set.txt` answers 178 (193 at `328fc0e28`). Against
the previous file, `diff` of the two files' non-comment rows shows 15 rows left
(`base/bif/ARG TEST_EXTERNAL_CALLS`, `OPTIONS TEST_ALL_SYNTAX_INVALID` and `_MISSING`, and the
`Literals` `*_INVALID_WHITESPACE_*` tests) and none arrived. None of the old witness exclusions fails at
the 9c close except `utilities/rexx/rexx_command TEST_PASSINGARGS7`, which carries a `Deviation(40)`
row and so is outside the filter.

The failing set the group runs printed equals the file: the sorted `group test` pairs from every
crate failure and error block, diffed against the file's rows, give no difference.

## Method

A throwaway copy of `tests/suite.rs` (`tests/zz_t1_dump.rs`, deleted, never committed) wrote each
run's stdout, stderr and status from `record`. Each group ran alone (`REXX_SUITE_GROUPS=<group>`,
`l3_core_suite`, `--test-threads 1`), oracle then crate, so the harness laid out every run directory.
Built once: `CARGO_TARGET_DIR=<scratchpad>/target-9d-t1 memcap 8G cargo test -j 4 --release -p
rexx-exec --test zz_t1_dump --no-run`, exit 0. Every group's run exited 101 (red, as expected). The
raw outputs (904K in all, none over 200 KB per group) are kept whole under `causes/`. Probes under
`probes/` ran from fresh directories under `memcap 2G` and the oracle recipe.

## scclient and sfclient

Cluster `redirected-command-holds-baton`. The test starts the server with `Object~start`, and the
server method runs `rexx scserver` (`sfserver`) under `ADDRESS ... WITH INPUT USING ... OUTPUT STEM`.
The crate waits for a redirected command's child on the baton (`Interp::run_command`,
`rust/crates/rexx-exec/src/command.rs:728`, parks only when `io.is_none()`), so the two client
activities start only after the server child exits, and connect to a closed port:
`Connect failed: ECONNREFUSED` and `SockConnect failed: ECONNREFUSED`. The oracle releases the
kernel lock around a command handler (`NativeActivation::run`). Probe `command-redirected` shows
the main activity stalled 2 s behind a redirected `sleep 2` on the crate and not on the oracle, and
`command-plain` shows an unredirected command does not stall it.

## Clusters

Counts are `awk -F'\t' '$3=="<cluster>"' clusters.tsv | wc -l`.

| cluster | tests |
| --- | --- |
| requires-from-dynamic-code | 25 |
| rexxinteger-min-representation | 19 |
| lostdigits-not-raised | 13 |
| stream-notready-names | 12 |
| label-in-if-select-branch | 9 |
| stream-binary-records | 6 |
| stream-open-write-only | 6 |
| stream-command-position | 6 |
| identity-collection-equality | 6 |
| variable-reference-list | 5 |
| use-arg-reference-conflict | 5 |
| traceobject-probe-option | 5 |
| raise-propagate-from-method | 5 |
| collection-set-operations | 5 |
| dynamic-package-name | 3 |
| collection-argument-error-numbers | 3 |
| datatype-whole-wide-numbers | 3 |
| security-manager-method-event | 3 |
| raise-result-entry | 3 |
| call-on-trap-argument | 3 |
| array-array-index | 3 |
| stream-eof-state | 2 |
| stream-argument-error-numbers | 2 |
| redirected-command-holds-baton | 2 |
| method-object-identity | 2 |
| interactive-trace-setting | 2 |
| attribute-body-error-number, bag-hasitem-nil, class-activate-order, constant-expression-novalue-option, context-rs, hostemu-halt, method-package-prolog, native-stack-frames, nil-comparison, prolog-trace-lines, queue-removeitem, reply-thread-error-report-line-ends, security-manager-environment-result, setmethod-instancemethod-kind, signal-trap-missing-label, superclass-not-a-class, trace-label-before-procedure, trace-reply-continuation, traceoutput-invalid-destination, uninit-define-on-superclass | 1 each |

`requires-from-dynamic-code` is the parent's "`::REQUIRES` of a file written at run time", renamed:
probe `requires-from-loaded-package` fails the same way with a file that exists before the run. The
search base for code built by `.Routine~new` inside a package loaded from another directory is the
mechanism.

## Deviation rulings needed

* `rexxinteger-min-representation`: the group detects the oracle's internal `RexxInteger` through
  `RexxInteger::Min`, which `IntegerClass.cpp:1634` orders that way on purpose so the class is
  observable. That conflicts with "representation is never observable".
* `bag-hasitem-nil`: `BagClass::hasItemRexx` (`BagClass.cpp:173`) returns `.nil` against its own
  doc comment, and the test asserts `.nil`.
* `stream-binary-records`, partly: `LINEOUT TEST_BINARY` fails first at line 220, which the oracle
  passes correctly, but its line 233 asserts output the test's own comment marks `@@wrong`.

Not a Deviation but an owner question: `hostemu-halt` (`SIGNAL TEST_SIGNAL_ANY_CONDITION_HALT`) needs
the `hostemu` C extension's subcommand environment, which is Phase 10's. On the crate the command
raises FAILURE rc 30.

## Review Focus 3 and 4

* Focus 3 (byte names on stream paths): `stream-notready-names`.
* Focus 4 (INTERPRET, REPLY, propagated): `raise-propagate-from-method` (propagated),
  `lostdigits-not-raised` (its group evaluates through `INTERPRET`), `trace-reply-continuation`,
  `reply-thread-error-report-line-ends` and `traceobject-probe-option`'s
  `TEST_CALLER_STACK_FRAME_REPLY_START` (REPLY).
* No test of the set fails on an INTERPRET traceback line.

## Causes not fully determined

Every test has its first divergence recorded. The mechanism behind it was not probed for:
* `stream-command-position`: several stream-command behaviours grouped by area, may split;
* `collection-set-operations`: the failure prints `a Queue` on both sides, so the differing content
  is not in the output;
* `traceobject-probe-option`'s `TEST_CALLER_STACK_FRAME` and `TEST_OBJECT_AND_SCOPE`: assigned by the
  probe showing option `P` collects nothing, not checked one by one;
* the single-test clusters without "Probed" in `clusters.md`, named from the test source and the
  failure text.

`SIGNAL TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS` sits in `lostdigits-not-raised` by its first
divergence and also needs `raise-propagate-from-method`.

## Concerns

* `extensions/yaml/yaml` has two blocked tests at the 9c close (`TEST35_DIRECTIVES`: "a compiled a
  pass boundary with no loop open op has no driver arm", `TEST47_MULTILINEFLOWCOLLECTIONS`: "a
  compiled Store op has no driver arm") under `Phase10(Extension)` rows. Moritz ruled yaml into 9d,
  so the rows exclude them from this set's filter. Task 4 should know.
* The target dir `target-9d-t1` was deleted after the probes.
