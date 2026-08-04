#!/usr/bin/env bash
#------------------------------------------------------------------------------
#
# Copyright (c) 2026 Rexx Language Association. All rights reserved.
#
# This program and the accompanying materials are made available under
# the terms of the Common Public License v1.0 which accompanies this
# distribution. A copy is also available at the following address:
# https://www.oorexx.org/license.html
#
#------------------------------------------------------------------------------

# The 4b exit gate's criterion 6: a committed list of one-line mutations to the
# code Phase 4b added, each of which some instrument in this gate must catch.
#
# THE MUTATIONS ARE NOT 4a's, AND THAT IS THE POINT. `mutate-4a.sh`'s nine
# target the value model and 4a's control flow; re-running them here would
# test 4a and report a number that says nothing about procedures, conditions,
# INTERPRET or the queue. Every OLD string below is in code that did not exist
# before this sub-phase.
#
# WHAT IS REUSED IS THE GUARD, and it is reused because a branch review
# reproduced the exact defect it closes: the first `mutate-4a.sh` reported
# "9 of 9 mutations caught", exit 0, with the oracle binary absent, because
# any non-zero exit counted as a catch. Three devices carry over, unchanged in
# intent:
#
#   * `apply_mutation` requires OLD to occur EXACTLY ONCE. Zero or two is a
#     hard failure, never a skip -- a pattern that no longer matches would
#     otherwise report a mutation as uncaught, or silently drop it while the
#     summary still read as full coverage.
#   * `require_baseline_pass` runs the unmutated tree before the first
#     mutation and again after the last restore, on BOTH instruments. Either
#     failing aborts before this script draws a conclusion from anything.
#   * `subset_status`/`suite_status` classify each run into PASSED, DIVERGED
#     or INFRA_FAILURE by reading the instrument's own printed report rather
#     than its exit code alone. INFRA_FAILURE aborts immediately and is never
#     folded into either "caught" or "not caught", because neither is true --
#     the mutation was never actually exercised.
#
# TWO INSTRUMENTS, RUN FOR EVERY MUTATION, AND EACH MUTATION DECLARES WHAT IT
# EXPECTS FROM BOTH. This is the substantive difference from 4a's script,
# which ran the corpus alone and treated "the corpus did not see it" as a
# failure. That is the wrong verdict for two of 4b's own constructs:
#
#   CORPUS  `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` -- the
#           byte-for-byte differential against the C++ oracle over the union
#           of `corpus/phase-4a.txt` and `corpus/phase-4b.txt`. Classified by
#           its own "N of M matching" line.
#   SUITE   `cargo test -p rexx-exec --lib --test trace_oracle --test
#           collect_stress` -- the in-crate instruments the corpus cannot be:
#           the queue's own order tests, the pinned unnormalised indent
#           witnesses, the committed trace expectations, and the
#           collect-on-every-allocation mode. Classified by libtest's own
#           "test result:" lines, WITH A NON-ZERO RUN COUNT REQUIRED.
#
# WHY THE RUN COUNT IS ASSERTED: `cargo test <name>` exits 0 when it matches
# nothing (`0 passed; 0 failed; N filtered out`, status 0), so a harness that
# runs a test by name and reads only the status CANNOT TELL "passed" from
# "does not exist". That has already produced a STAYED GREEN report on this
# project for a test that had ceased to exist.
#
# EXPECTED SURVIVORS ARE FIRST-CLASS ROWS, NOT OMISSIONS. Two mutations below
# are expected to leave an instrument green, and for each one the survival is
# the assertion rather than a gap being tolerated:
#
#   * `queue-storage-discarded` is expected to PASS the corpus and DIVERGE on
#     the suite. That is criterion 9's evidence, executed rather than argued:
#     nothing that reads the queue back (PULL, PARSE PULL, QUEUED()) exists
#     before 4c, so no corpus program can observe what was stored -- only what
#     was written and how it traced. If this row ever starts diverging on the
#     corpus, the queue has gained a differential witness and criterion 9's
#     "ships undifferentiated" wording is stale.
#   * `i17-stem-drop-as-slot-clear` is expected to PASS BOTH. It is inherited
#     item I17, and this script runs it rather than citing it. The scoping
#     document predicted the mutant would become pinnable once 4b landed,
#     on the grounds that nothing in 4a could hold a second reference to a
#     stem. THE PREMISE IS FALSE -- measured, `b. = a.` shares the object in
#     4a on both interpreters -- and so is the conclusion, for a different
#     reason: `a.1 = 'orig'; b. = a.; drop a.; say a.1 b.1` prints `A.1 orig`
#     under both the current code and the mutant, because "the slot holds a
#     fresh empty stem" and "the slot is unset" are not distinguishable
#     through a second reference. It is genuinely equivalent, and the
#     mechanism -- not just the verdict -- is why. If this row ever starts
#     being caught, something has made the two states distinguishable and
#     I17's reclassification needs revisiting.
#
# So a mutation is a FAILURE of this script when its observed pair of statuses
# differs from its declared pair, in either direction. An unexpected catch is
# reported as loudly as an unexpected survival: both mean this file's own
# claims about which instrument sees what have gone stale.
#
# USAGE: run from anywhere; it locates the repository root from its own path.
# No arguments. Exits 0 only if every mutation below was applied and every
# observed status pair matched its declared pair. Leaves the tree exactly as
# it found it -- every mutation is reverted immediately after its own check,
# and an EXIT trap guarantees the revert runs even if the script is
# interrupted mid-mutation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${RUST_DIR}"

RUN_RS="crates/rexx-exec/src/run.rs"
QUEUE_RS="crates/rexx-exec/src/queue.rs"

TOTAL=0
AS_DECLARED=0
FAILURES=()

# A plain file copy, not `git checkout -- <path>`: this project's git
# discipline forbids that command regardless of caller, and restoring from a
# backup taken before the first mutation can never discard someone else's
# uncommitted work the way `git checkout --` silently has.
BACKUP_DIR="$(mktemp -d)"
cp "${RUN_RS}" "${BACKUP_DIR}/run.rs"
cp "${QUEUE_RS}" "${BACKUP_DIR}/queue.rs"

restore() {
    cp "${BACKUP_DIR}/run.rs" "${RUN_RS}"
    cp "${BACKUP_DIR}/queue.rs" "${QUEUE_RS}"
}
cleanup() {
    restore
    rm -rf "${BACKUP_DIR}"
}
trap cleanup EXIT

# Refuses to run against a dirty tree: the backup above is a safety net for
# *this script's own* mutations only. Over uncommitted work the backup would
# capture changes that are not this script's to manage, and `restore` would
# paper over their presence rather than the caller seeing their own
# `git status`.
require_clean() {
    if ! git diff --quiet -- "${RUN_RS}" "${QUEUE_RS}"; then
        echo "FATAL: ${RUN_RS}/${QUEUE_RS} have uncommitted changes before this" \
             "script touched them. Refusing to run: commit or stash first, so" \
             "this script's own backup-and-restore is not masking work that" \
             "belongs to someone else." >&2
        exit 1
    fi
}

# Applies OLD -> NEW in FILE, requiring OLD to appear EXACTLY ONCE.
apply_mutation() {
    local file="$1" old="$2" new="$3"
    python3 - "${file}" "${old}" "${new}" <<'PYEOF'
import sys
path, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
with open(path) as f:
    content = f.read()
count = content.count(old)
if count != 1:
    print(
        f"UNAPPLIED PATTERN: found {count} times (need exactly 1) in {path}.\n"
        f"pattern:\n{old}",
        file=sys.stderr,
    )
    sys.exit(1)
with open(path, "w") as f:
    f.write(content.replace(old, new, 1))
PYEOF
}

# ---------------------------------------------------------------------------
# Instrument 1: the differential corpus, in gate (STRICT) mode.
# ---------------------------------------------------------------------------

run_corpus() {
    REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus \
        > "${CORPUS_OUTPUT}" 2>&1
}

# The last "N of M matching" line `corpus.rs` printed, as "N M", or nothing at
# all -- which is exactly the signal that the run never reached the point of
# comparing anything (a missing oracle, a compile error, a panic before the
# report).
corpus_matching_line() {
    grep -oE '^[0-9]+ of [0-9]+ matching' "${CORPUS_OUTPUT}" | tail -n 1 |
        sed -E 's/^([0-9]+) of ([0-9]+) matching$/\1 \2/'
}

corpus_status() {
    local exit_code="$1" line matched total
    line="$(corpus_matching_line)"
    if [ -z "${line}" ]; then
        echo "INFRA_FAILURE"
        return
    fi
    read -r matched total <<<"${line}"
    if [ "${exit_code}" -eq 0 ]; then
        if [ "${matched}" -eq "${total}" ]; then echo "PASSED"; else echo "INFRA_FAILURE"; fi
    else
        if [ "${matched}" -lt "${total}" ]; then echo "DIVERGED"; else echo "INFRA_FAILURE"; fi
    fi
}

# ---------------------------------------------------------------------------
# Instrument 2: the in-crate suite the corpus cannot be.
#
# `--lib` carries `queue.rs`'s order tests and `run.rs`'s pinned unnormalised
# indent witnesses; `--test trace_oracle` carries the committed byte-for-byte
# trace expectations; `--test collect_stress` carries the
# collect-on-every-allocation mode, which is the only instrument that can see
# a dropped GC root.
# ---------------------------------------------------------------------------

run_suite() {
    cargo test --offline -p rexx-exec --lib --test trace_oracle --test collect_stress \
        > "${SUITE_OUTPUT}" 2>&1
}

# Total tests actually RUN (passed + failed) across every `test result:` line,
# as "PASSED FAILED", or nothing if libtest printed no such line at all.
# The run count is what makes "passed" distinguishable from "matched no
# tests" -- see the header.
suite_counts() {
    grep -oE '^test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed' "${SUITE_OUTPUT}" |
        sed -E 's/^test result: [a-zA-Z]+\. ([0-9]+) passed; ([0-9]+) failed$/\1 \2/' |
        awk '{p += $1; f += $2} END {if (NR == 0) exit 0; print p, f}'
}

suite_status() {
    local exit_code="$1" counts passed failed
    counts="$(suite_counts)"
    if [ -z "${counts}" ]; then
        echo "INFRA_FAILURE"
        return
    fi
    read -r passed failed <<<"${counts}"
    # A run that executed nothing is an infrastructure failure, never a pass:
    # this is the `0 passed; 0 failed; N filtered out`, status 0 case.
    if [ "$((passed + failed))" -eq 0 ]; then
        echo "INFRA_FAILURE"
        return
    fi
    if [ "${exit_code}" -eq 0 ]; then
        if [ "${failed}" -eq 0 ]; then echo "PASSED"; else echo "INFRA_FAILURE"; fi
    else
        if [ "${failed}" -gt 0 ]; then echo "DIVERGED"; else echo "INFRA_FAILURE"; fi
    fi
}

# Both instruments must report a clean unmutated tree before the first
# mutation and after the last restore. A compile failure, a missing oracle or
# an interrupted revert is caught here rather than being mistaken for a
# mutation result.
require_baseline_pass() {
    local label="$1" exit_code status
    echo "=== baseline (${label}): the unmutated tree must pass both instruments ==="

    if run_corpus; then exit_code=0; else exit_code=$?; fi
    status="$(corpus_status "${exit_code}")"
    if [ "${status}" != "PASSED" ]; then
        echo "FATAL: the unmutated CORPUS does not pass (${label}, status=${status})." \
             "Nothing below this point can be trusted as a mutation result --" \
             "any row could be an environment failure wearing a catch's" \
             "clothing. Full output:" >&2
        cat "${CORPUS_OUTPUT}" >&2
        exit 1
    fi
    read -r matched total <<<"$(corpus_matching_line)"
    echo "baseline corpus ok (${label}): ${matched} of ${total} matching"

    if run_suite; then exit_code=0; else exit_code=$?; fi
    status="$(suite_status "${exit_code}")"
    if [ "${status}" != "PASSED" ]; then
        echo "FATAL: the unmutated SUITE does not pass (${label}, status=${status})." \
             "Full output:" >&2
        cat "${SUITE_OUTPUT}" >&2
        exit 1
    fi
    read -r passed failed <<<"$(suite_counts)"
    echo "baseline suite ok (${label}): ${passed} passed, ${failed} failed"
    echo
}

# One mutation: apply, run BOTH instruments, compare each observed status
# against the declared one, revert.
#
# Exits the whole script non-zero immediately if the pattern could not be
# applied (UNAPPLIED PATTERN is fatal, never merely "skipped") or if either
# instrument reports INFRA_FAILURE. Otherwise records but does not abort on a
# status that differs from its declaration, so one surprising row does not
# hide the result of the others.
run_one() {
    local name="$1" file="$2" expect_corpus="$3" expect_suite="$4" old="$5" new="$6"
    TOTAL=$((TOTAL + 1))
    echo "=== ${name} ==="
    echo "declared: corpus=${expect_corpus} suite=${expect_suite}"

    if ! apply_mutation "${file}" "${old}" "${new}"; then
        echo "FATAL: mutation '${name}' could not be applied -- see UNAPPLIED" \
             "PATTERN above. This is the exact stale-pattern failure mode this" \
             "script's guard exists to catch; it is not safe to report coverage" \
             "while it stands." >&2
        exit 1
    fi

    local exit_code corpus_actual suite_actual
    if run_corpus; then exit_code=0; else exit_code=$?; fi
    corpus_actual="$(corpus_status "${exit_code}")"
    if [ "${corpus_actual}" = "INFRA_FAILURE" ]; then
        echo "FATAL: mutation '${name}' could not be assessed on the CORPUS --" \
             "the run failed for an infrastructure reason, not an observed" \
             "divergence. Reporting it as caught or not caught would both be" \
             "wrong. Full output:" >&2
        cat "${CORPUS_OUTPUT}" >&2
        restore
        exit 1
    fi

    if run_suite; then exit_code=0; else exit_code=$?; fi
    suite_actual="$(suite_status "${exit_code}")"
    if [ "${suite_actual}" = "INFRA_FAILURE" ]; then
        echo "FATAL: mutation '${name}' could not be assessed on the SUITE --" \
             "the run failed for an infrastructure reason (a compile error, or" \
             "a run that executed no tests at all), not an observed failure." \
             "Full output:" >&2
        cat "${SUITE_OUTPUT}" >&2
        restore
        exit 1
    fi

    echo "observed: corpus=${corpus_actual} suite=${suite_actual}"
    if [ "${corpus_actual}" = "DIVERGED" ]; then
        grep -E "matching|FAILED" "${CORPUS_OUTPUT}" || true
    fi
    if [ "${suite_actual}" = "DIVERGED" ]; then
        grep -E "^test .* FAILED|^failures:$" -A 20 "${SUITE_OUTPUT}" | grep -E "^    [a-z_0-9:]+$" | head -n 10 || true
    fi

    if [ "${corpus_actual}" = "${expect_corpus}" ] && [ "${suite_actual}" = "${expect_suite}" ]; then
        echo "as declared."
        AS_DECLARED=$((AS_DECLARED + 1))
    else
        echo "NOT AS DECLARED: expected corpus=${expect_corpus} suite=${expect_suite}," \
             "observed corpus=${corpus_actual} suite=${suite_actual}."
        FAILURES+=("${name} (expected ${expect_corpus}/${expect_suite}, got ${corpus_actual}/${suite_actual})")
    fi

    restore
    echo
}

# Fresh temp files per run rather than fixed paths, so two concurrent
# invocations cannot interleave their output.
CORPUS_OUTPUT="$(mktemp)"
SUITE_OUTPUT="$(mktemp)"

require_clean
require_baseline_pass "before any mutation"

# ---------------------------------------------------------------------------
# The mutations. Each targets code Phase 4b added, and each declares what both
# instruments should say about it.
# ---------------------------------------------------------------------------

# --- PROCEDURE and PROCEDURE EXPOSE (Task 5) -------------------------------

# `PROCEDURE EXPOSE` installs no aliases at all, so the callee gets a pool
# isolated from the caller's with nothing shared back. `corpus/lang/
# call_procedure_expose.rex` reads exposed names in the callee and reads
# callee-written values back in the caller, and its own header states which
# wrong answer each block prints.
run_one "1. PROCEDURE EXPOSE aliases nothing" "${RUN_RS}" DIVERGED DIVERGED \
'        for (_, slot, target) in &bindings {
            self.roots.alias_slot(inner, *slot, *target);
        }' \
'        for (_, slot, target) in bindings.iter().take(0) {
            self.roots.alias_slot(inner, *slot, *target);
        }'

# --- trap inheritance across an activation (Task 7) ------------------------

# A called routine inherits an EMPTY trap table rather than its caller's, so a
# condition raised inside a callee is never trapped by a `SIGNAL ON`/`CALL ON`
# armed in the caller. Inheritance is measured and one-way (`Activation::
# traps`' own doc comment carries the three probes): the table is cloned in
# and never written back.
#
# A NOTE ON WHY THIS ROW REPLACED AN EARLIER ONE, kept because the earlier
# one's failure is the guard working rather than a mutation being unlucky. The
# first attempt at a `PROCEDURE`-shaped mutation pointed the callee's
# activation back at the caller's frame while a fresh frame was already
# pushed, which trips a `roots.rs` invariant (`grow_slots on a frame that is
# not the top one`) and panics `corpus_differential` BEFORE it prints its
# "N of M matching" line. `corpus_status` correctly classified that
# INFRA_FAILURE and the script aborted rather than scoring it: a mutation that
# breaks the harness has not been tested, and calling it "caught" would have
# been exactly the defect this guard exists to prevent.
# THE DECLARED PAIR IS ASYMMETRIC, AND FINDING THAT OUT IS WHY THIS ROW EARNS
# ITS PLACE. It was declared DIVERGED/DIVERGED and measured PASSED/DIVERGED:
# **trap inheritance into a callee has no differential witness at all.** The
# suite catches it through `run::tests::a_trap_is_inherited_by_a_callee_and_
# fires_in_the_callees_own_activation`, whose shape is a raise INSIDE a callee
# under a caller-armed trap, with the handler observably running in the
# callee's own activation. No corpus program has that shape: every raise a
# corpus program makes inside a routine is a `RAISE ... RETURN`, which unwinds
# the routine like a `RETURN` before the condition is delivered, so the trap
# that matches is the CALLER's own live table and never the inherited copy.
# `lang/call_on_trap_rearms.rex`'s own header says exactly this about its own
# `raiser`. So the corpus staying green here is a true statement about the
# corpus, and the declaration is written to record it rather than to hide it.
run_one "2. a callee does not inherit its caller's condition traps" "${RUN_RS}" PASSED DIVERGED \
'        let traps = caller.traps.clone();' \
'        let traps = Default::default();'

# --- USE ARG (Task 5) ------------------------------------------------------

# Positional binding off by one: every `USE ARG` target takes the argument to
# its right. The omitted-position rule is what makes this observable in more
# than one way -- `call sub 1,,3` into three targets gives `[1] [Q] [3]`, and
# a shifted read moves both the values and the hole.
run_one "3. USE ARG binds each target to the next argument" "${RUN_RS}" DIVERGED DIVERGED \
'            let argument = self.call_context.arguments.get(index).cloned().flatten();' \
'            let argument = self.call_context.arguments.get(index + 1).cloned().flatten();'

# --- CALL and RETURN (Task 3) ----------------------------------------------

# A bare `RETURN` leaves `RESULT` holding whatever the caller had, rather than
# dropping it. Measured: after `return 42` the caller reads `42`; after a bare
# `return` it reads the derived name `RESULT`, which is what an unset variable
# renders as -- so this mutation replaces an unset read with stale data.
run_one "4. a bare RETURN does not drop RESULT" "${RUN_RS}" DIVERGED DIVERGED \
'            None => self.roots.clear_slot(frame, slot),' \
'            None => {}'

# --- INTERPRET (Tasks 1 and 2) ---------------------------------------------

# A fragment's clause echoes resolve their own line rather than inheriting the
# enclosing `INTERPRET` clause's. Measured, a two-line file whose line 2 is
# `interpret "say 2 & 1"` echoes BOTH the fragment's clause and the INTERPRET
# at line 2; a fragment that resolves its own line reports line 1 for the
# first of them. `corpus/lang/interpret_error_echo.rex` is the witness.
#
# Declared DIVERGED/PASSED and measured DIVERGED/DIVERGED: the suite catches
# it too, through ten `run.rs`/`trace_oracle` tests including
# `sigl_is_set_at_every_control_transfer` and
# `a_raise_inside_a_fragment_reports_both_clauses`. The declaration is
# corrected to what was measured rather than left standing as the guess.
run_one "5. an INTERPRET fragment's echo resolves its own line" "${RUN_RS}" DIVERGED DIVERGED \
'                let saved_line = std::mem::replace(&mut self.clause_line_override, base_line);' \
'                let saved_line = std::mem::take(&mut self.clause_line_override);'

# --- SIGNAL ON / conditions / SIGL (Task 7) --------------------------------

# `SIGL` off by one at the point a `SIGNAL ON` trap fires. It must be the
# RAISING clause's line, not the `SIGNAL ON` clause's and not the handler's;
# `corpus/lang/condition_traps.rex` prints `sigl` into an accumulated witness
# string at four separate traps, so this moves four numbers at once.
run_one "6. SIGL is one line past the raising clause" "${RUN_RS}" DIVERGED DIVERGED \
'        let site = self.failure_site.take();
        let sites = std::mem::take(&mut self.failure_sites);
        self.set_sigl(self.clause_state.line());' \
'        let site = self.failure_site.take();
        let sites = std::mem::take(&mut self.failure_sites);
        self.set_sigl(self.clause_state.line() + 1);'

# A `CALL ON` trap is NOT put back after its handler returns, so it behaves
# like a `SIGNAL ON` trap and disarms permanently on its first firing.
# `corpus/lang/call_on_trap_rearms.rex` (Task 10's one surviving combination
# witness) exists for exactly this: it raises the same `CALL ON`-trapped
# condition twice with a `SIGNAL` to a label between them.
run_one "7. a CALL ON trap disarms permanently when it fires" "${RUN_RS}" DIVERGED DIVERGED \
'        if let Some(trap) = removed {
            self.activation_mut().traps.insert(key, trap);
        }' \
'        if let Some(trap) = removed {
            let _ = (key, trap);
        }'

# --- the argument trace prefix (Task 9) ------------------------------------

# An omitted argument position traces NOTHING rather than an empty `>A>` line.
# Measured on the oracle (`trace i`): `call sub 1,,3` traces `>A>   "1"`,
# `>A>   ""`, `>A>   "3"`, in that order -- `traceArgument(GlobalNames::
# NULLSTRING)`, `RexxInstruction.cpp:161`. The committed witness is
# `tests/trace_oracle/call_arguments.rex`, which is a suite instrument and not
# a corpus program, so the declared pair is the asymmetric one.
run_one "8. an omitted argument position traces no >A> line" "${RUN_RS}" PASSED DIVERGED \
'                    self.trace_argument(self.clause_state.current_value_indent, b"");' \
'                    let _ = self.clause_state.current_value_indent;'

# --- the collect-on-every-allocation negative control (criterion 4) --------

# THE ACTIVATION-SHAPED NEGATIVE CONTROL, and the reason it is here rather
# than being 4a's control re-run. 4a's control deletes `eval_arithmetic`'s
# `push_temp(left_value)`; that tests a root an EXPRESSION holds, and 4a's
# criterion 4 passed having exercised zero call frames. This deletes a root a
# CALL holds: the argument list's, between an argument's evaluation and the
# callee's own `USE ARG`, which is a window that only exists because 4b built
# activations. The corpus cannot see it -- a dropped root is invisible until
# something collects -- so the declared pair is asymmetric and the suite's
# `collect_stress` is the instrument that sees it.
run_one "9. the argument list's root is dropped (activation-shaped control)" "${RUN_RS}" PASSED DIVERGED \
'                    let argument = self.eval_argument(code, expr)?;
                    self.roots.push_temp(argument.value());' \
'                    let argument = self.eval_argument(code, expr)?;'

# --- PUSH and QUEUE (Task 8) -----------------------------------------------

# `PUSH` inserts at the tail, collapsing it into `QUEUE`. The queue's order is
# oracle-measured (`push "a"`, `queue "b"`, `push "c"` leaves `c`, `a`, `b`)
# and pinned by `queue.rs`'s own unit tests. No corpus program can see it.
run_one "10. PUSH inserts at the tail, like QUEUE" "${QUEUE_RS}" PASSED DIVERGED \
'        self.lines.push_front(line);' \
'        self.lines.push_back(line);'

# CRITERION 9's EVIDENCE, EXECUTED RATHER THAN ARGUED. The expression is still
# evaluated and still traced; only the rendered line is thrown away instead of
# being stored. The corpus is EXPECTED to stay green, because nothing that
# reads the queue back exists before 4c -- this row is what makes "PUSH/QUEUE
# ships undifferentiated" a measured statement rather than a claim, and it
# goes red the day 4c gives the queue a differential witness.
run_one "11. PUSH/QUEUE evaluate and trace but store nothing" "${RUN_RS}" PASSED DIVERGED \
'                if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    self.queue.push(line);
                } else {
                    self.queue.queue(line);
                }' \
'                if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    let _ = &line;
                } else {
                    let _ = &line;
                }'

# --- the equivalent mutant, run rather than cited (I17) --------------------

# INHERITED ITEM I17, AND ITS RECLASSIFICATION. `drop_by_name`'s Stem arm
# rerouted to a plain slot clear. The scoping document expected this to become
# pinnable once 4b landed, because nothing in 4a could hold a second reference
# to a stem. Both halves of that are wrong: `b. = a.` already shares the object
# in 4a on both interpreters, AND the mutant is equivalent anyway, because
# "the slot holds a fresh empty stem" and "the slot is unset" are not
# distinguishable through a second reference -- `a.1 = 'orig'; b. = a.;
# drop a.; say a.1 b.1` prints `A.1 orig` either way. Declared to survive both
# instruments; if it ever stops surviving, something has made the two states
# distinguishable and I17 needs revisiting rather than this row being edited
# to match.
run_one "12. I17: DROP of a stem as a plain slot clear (equivalent mutant)" "${RUN_RS}" PASSED PASSED \
'            NameShape::Stem => self.stem_drop(name),' \
'            NameShape::Stem => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.roots.clear_slot(frame, slot);
            }'

require_baseline_pass "after the last restore"

echo "==============================================================================
${AS_DECLARED} of ${TOTAL} mutations behaved exactly as declared
=============================================================================="

if [ "${#FAILURES[@]}" -gt 0 ]; then
    echo "NOT AS DECLARED:"
    for f in "${FAILURES[@]}"; do
        echo "  - ${f}"
    done
    exit 1
fi

exit 0
