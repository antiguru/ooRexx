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

# The 4a exit gate's criterion 6: a committed list of one-line mutations to
# rexx-exec, each of which the L0 subset (rust/corpus/phase-4a.txt, run
# through tests/corpus.rs) must catch. This replaces "substituting any other
# binary reports divergences on every program", which /bin/true satisfies
# and demonstrates only that the harness notices absent output.
#
# This is the one gate criterion a `cargo test` cannot be: it edits the
# source it tests. Run it, read its output, and record that output in the
# gate document -- it does not leave a green/red mark of its own the way a
# test suite does.
#
# THE GUARD THIS SCRIPT EXISTS FOR: a mutation is applied by finding an
# EXACT, committed OLD string and replacing it with a NEW one. If OLD is not
# found in the target file EXACTLY ONCE, this is a hard failure (exit
# non-zero), not a skip. A pattern that no longer matches -- because the
# surrounding code moved, or review already changed it -- would otherwise
# report a mutation as "not caught" or, worse, silently skip it while every
# other mutation still runs, and the summary would read as full coverage
# when part of it never executed at all. This exact guard has fired in four
# separate tasks on this project before this script existed; it is not
# hypothetical here, it is the reason the script has this shape.
#
# WHAT "CAUGHT" MEANS: after applying one mutation, this script runs
# `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`, the same
# command criterion 1 runs, and reads its printed "N of M matching" line
# rather than trusting the exit code alone -- see "THE INFRASTRUCTURE-
# FAILURE GUARD" below for why the exit code alone is not enough. A
# mutation is caught only when that line names a real divergence (N < M):
# a program's output disagreeing with the oracle, or the mutated code
# panicking or failing to compile, all of which `corpus.rs` still prints
# that line for before reporting `Err`. A mutation caught only by
# `cargo test -p rexx-exec --lib` and not by the corpus itself would not
# satisfy this criterion, so this script does not run the lib suite at all
# -- see the gate report for which of the nine mutations needed a new
# corpus witness before the subset could see them.
#
# THE INFRASTRUCTURE-FAILURE GUARD, added after a branch review reproduced
# the exact defect this script exists to close, arriving from the other
# side. `run_subset`'s exit code alone cannot tell "the subset diverged"
# apart from "the run never got that far": with the oracle binary missing,
# `corpus_differential` panics before printing any "N of M matching" line
# at all, its exit code is non-zero all the same, and the first version of
# this script counted that as a catch -- nine of nine, exit 0, having
# compared nothing. Two things close it. First, a **baseline run** before
# the first mutation and another after the last restore, each required to
# report every program matching; either failing aborts the whole script
# before it draws any conclusion from a mutation. Second, `subset_status`
# below parses the captured output for the "N of M matching" line itself:
# no such line at all is `INFRA_FAILURE`, a hard, immediate abort, never a
# catch and never "not caught" either, because neither of those is true --
# the mutation was never actually tested.
#
# EQUIVALENT MUTANTS are not this script's problem to invent: two are
# already known and documented elsewhere in this project's own history
# rather than here, because they were found by earlier tasks' own review-
# round mutation testing, not by this gate script:
#   - `drop_by_name`'s Stem arm rerouted to a slot clear (Task 9's review,
#     task-9-review.md, mutation M3): survives because nothing in 4a can
#     hold a second reference to a dropped stem (needs `procedure expose`/
#     argument passing, which is 4b's) -- rebinding the slot to a fresh
#     empty stem and clearing it are observationally identical until
#     aliasing exists.
#   - `tracing_intermediates()` returning `trace_mode.all` (Task 13's
#     review, task-13-review.md): survives because every `trace_literal`/
#     `trace_variable`/etc. method in trace.rs re-checks
#     `trace_mode.intermediates` internally, so the outer gate in `eval` is
#     a redundant early-out, not the only guard.
# Neither is one of the nine mutations below, and neither needed
# rediscovering here -- they are cited, not reproduced, so this script's own
# summary does not imply they were found by it.
#
# USAGE: run from anywhere; it locates the repository root from its own
# path. No arguments. Exits 0 only if every mutation below was both applied
# and caught. Leaves the tree exactly as it found it -- every mutation is
# reverted immediately after its own check, whether the check passed or
# failed, and a trap guarantees the revert runs even if the script is
# interrupted mid-mutation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${RUST_DIR}"

RUN_RS="crates/rexx-exec/src/run.rs"
EVAL_RS="crates/rexx-exec/src/eval.rs"
VALUE_RS="crates/rexx-exec/src/value.rs"

# One associative-array-free mutation record per line: NAME, FILE, then OLD
# and NEW passed as separate heredoc-fed variables below, because several of
# these span more than one line and a single-line "sed -i s/../../" cannot
# express them without fragile escaping. Python's exact-count-then-replace
# does the actual edit; nothing here depends on Python beyond straight
# string replacement, which every Python3 on this project's machines has.

TOTAL=0
CAUGHT=0
FAILURES=()

# A plain file copy, not `git checkout -- <path>`: this project's own git
# discipline forbids that command regardless of caller, and a byte-for-byte
# backup taken before the first mutation and restored with `cp` afterwards
# needs git at all only to fail loudly (`require_clean`, below) if the tree
# was not clean to begin with -- it never asks git to discard anything.
BACKUP_DIR="$(mktemp -d)"
cp "${RUN_RS}" "${BACKUP_DIR}/run.rs"
cp "${EVAL_RS}" "${BACKUP_DIR}/eval.rs"
cp "${VALUE_RS}" "${BACKUP_DIR}/value.rs"

# Restores every file this script can mutate to its backed-up, pre-mutation
# state. Used both between mutations and by the EXIT trap, so an interrupted
# run (Ctrl-C mid-mutation) cannot leave the tree with a mutation still
# applied.
restore() {
    cp "${BACKUP_DIR}/run.rs" "${RUN_RS}"
    cp "${BACKUP_DIR}/eval.rs" "${EVAL_RS}"
    cp "${BACKUP_DIR}/value.rs" "${VALUE_RS}"
}
cleanup() {
    restore
    rm -rf "${BACKUP_DIR}"
}
trap cleanup EXIT

# Refuses to run against a dirty tree: the backup above is only a safety net
# for *this script's own* mutations, and running it over uncommitted work
# would mean the backup itself captures changes that are not this script's
# to manage, and `restore` would silently paper over them being present at
# all rather than the caller seeing their own `git status`.
require_clean() {
    if ! git diff --quiet -- "${RUN_RS}" "${EVAL_RS}" "${VALUE_RS}"; then
        echo "FATAL: ${RUN_RS}/${EVAL_RS}/${VALUE_RS} have uncommitted changes" \
             "before this script touched them. Refusing to run: commit or" \
             "stash first, so this script's own backup-and-restore is not" \
             "masking work that belongs to someone else." >&2
        exit 1
    fi
}

# Applies OLD -> NEW in FILE, requiring OLD to appear EXACTLY ONCE. This is
# the guard the whole script exists for: see the header comment.
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

# Runs the L0 subset against the oracle in gate (STRICT) mode, capturing
# output for `subset_status` to classify. Called only from inside an `if`
# (or with its exit captured via the same idiom), never bare, so a non-zero
# exit does not trip `set -e` -- every call site below follows that rule.
run_subset() {
    REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus \
        > "${SUBSET_OUTPUT}" 2>&1
}

# The last "N of M matching" line `corpus.rs` printed into `${SUBSET_OUTPUT}`,
# as "N M" on stdout, or nothing if no such line exists at all -- which is
# exactly the signal that the run never reached the point of comparing
# anything (a missing oracle, a compile error, a panic before the report).
matching_line() {
    grep -oE '^[0-9]+ of [0-9]+ matching' "${SUBSET_OUTPUT}" | tail -n 1 |
        sed -E 's/^([0-9]+) of ([0-9]+) matching$/\1 \2/'
}

# Classifies the `run_subset` call that just finished, given its exit code.
# Three outcomes, and `INFRA_FAILURE` is the one the old version of this
# script could not name -- see the header comment's "INFRASTRUCTURE-FAILURE
# GUARD" for why that mattered.
#   PASSED        -- exit 0, every program matching.
#   DIVERGED       -- non-zero exit, and the report names N < M matching:
#                     a real, observed divergence from the oracle.
#   INFRA_FAILURE -- no "N of M matching" line at all (whatever the exit
#                     code), or an exit/line combination that contradicts
#                     itself. Never a catch, never "not caught" -- the
#                     mutation (or, at the baseline, nothing) was never
#                     actually exercised.
subset_status() {
    local exit_code="$1" line matched total
    line="$(matching_line)"
    if [ -z "${line}" ]; then
        echo "INFRA_FAILURE"
        return
    fi
    read -r matched total <<<"${line}"
    if [ "${exit_code}" -eq 0 ]; then
        if [ "${matched}" -eq "${total}" ]; then
            echo "PASSED"
        else
            echo "INFRA_FAILURE"
        fi
    else
        if [ "${matched}" -lt "${total}" ]; then
            echo "DIVERGED"
        else
            echo "INFRA_FAILURE"
        fi
    fi
}

# Runs the subset and requires `PASSED`, aborting the whole script loudly
# otherwise. Called once before the first mutation and once after the last
# restore, so neither a broken environment nor an interrupted revert can be
# mistaken for a clean run of nine real mutations.
require_baseline_pass() {
    local label="$1" exit_code
    echo "=== baseline (${label}): the unmutated subset must pass ==="
    if run_subset; then
        exit_code=0
    else
        exit_code=$?
    fi
    local status
    status="$(subset_status "${exit_code}")"
    if [ "${status}" != "PASSED" ]; then
        echo "FATAL: the unmutated subset does not pass (${label}, status=${status})." \
             "Nothing below this point can be trusted as a mutation result --" \
             "any of the nine could be an environment failure wearing a" \
             "catch's clothing. Full output:" >&2
        cat "${SUBSET_OUTPUT}" >&2
        exit 1
    fi
    read -r matched total <<<"$(matching_line)"
    echo "baseline ok (${label}): ${matched} of ${total} matching"
    echo
}

# One mutation: apply, run the subset, report, revert. Exits the whole
# script non-zero immediately if the pattern could not be applied
# (UNAPPLIED PATTERN is fatal, never merely "skipped"), or if the subset run
# itself failed for a reason that is not a divergence (`INFRA_FAILURE`,
# never silently folded into either "caught" or "not caught"). Otherwise
# records but does not abort on a mutation that fails to be CAUGHT, so one
# uncaught mutation does not hide the result of the other eight.
run_one() {
    local name="$1" file="$2" old="$3" new="$4"
    TOTAL=$((TOTAL + 1))
    echo "=== ${name} ==="

    if ! apply_mutation "${file}" "${old}" "${new}"; then
        echo "FATAL: mutation '${name}' could not be applied -- see UNAPPLIED" \
             "PATTERN above. This is the exact stale-pattern failure mode this" \
             "script's guard exists to catch; it is not safe to report" \
             "coverage while it stands." >&2
        exit 1
    fi

    local exit_code
    if run_subset; then
        exit_code=0
    else
        exit_code=$?
    fi
    local status
    status="$(subset_status "${exit_code}")"

    case "${status}" in
    DIVERGED)
        echo "caught: the subset diverges under this mutation."
        grep -E "matching|FAILED" "${SUBSET_OUTPUT}" || true
        CAUGHT=$((CAUGHT + 1))
        ;;
    PASSED)
        echo "NOT CAUGHT: the subset still passes under this mutation."
        tail -n 5 "${SUBSET_OUTPUT}"
        FAILURES+=("${name}")
        ;;
    INFRA_FAILURE)
        echo "FATAL: mutation '${name}' could not be assessed -- the subset run" \
             "failed for an infrastructure reason, not an observed divergence" \
             "(no \"N of M matching\" line in its output). Reporting this as" \
             "caught or not caught would both be wrong; something about the" \
             "environment broke instead. Full output:" >&2
        cat "${SUBSET_OUTPUT}" >&2
        restore
        exit 1
        ;;
    esac

    restore
    echo
}

# A fresh temp file per run rather than a fixed path, so two concurrent
# invocations of this script cannot interleave their output.
SUBSET_OUTPUT="$(mktemp)"

require_clean
require_baseline_pass "before any mutation"

# ---------------------------------------------------------------------------
# The nine mutations, mapped onto the design spec's own list (4a exit gate,
# criterion 6). Each OLD string was checked by hand against the committed
# source before this script was written; each was independently verified
# (applied for real, subset run, reverted) to be caught by the current
# subset, three of them only after Task 16 added a witness for exactly this
# purpose -- see the gate report for which three and why.
# ---------------------------------------------------------------------------

run_one "1. off-by-one on If::false_target" "${RUN_RS}" \
'                    Ok(Flow::Goto(false_target))' \
'                    Ok(Flow::Goto(false_target + 1))'

run_one "2. off-by-one on When::exit" "${RUN_RS}" \
'                            holds.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))' \
'                            holds.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len) + 1))'

run_one "3. off-by-one on Loop::end" "${RUN_RS}" \
'        let resume = end_index + 1;' \
'        let resume = end_index;'

run_one "4. Controlled::order evaluated in fixed To/By/For order" "${RUN_RS}" \
'        for entry in &ctrl.order {' \
'        for entry in ctrl.order.iter().rev() {'

run_one "5. Abuttal treated as Blank" "${EVAL_RS}" \
'                let separator: &[u8] = if *op == Operator::Blank { b" " } else { b"" };' \
'                let separator: &[u8] = if *op == Operator::Blank || *op == Operator::Abuttal { b" " } else { b"" };'

run_one "6. = treated as ==" "${EVAL_RS}" \
'        Equal => CompareOp::Equal,' \
'        Equal => CompareOp::StrictEqual,'

run_one "7. LEAVE unwinding one block too few" "${RUN_RS}" \
'            Flow::Leave(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };' \
'            Flow::Leave(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(_) => true,
                };'

run_one "8. formatting with the current digits, not the created digits" "${VALUE_RS}" \
'        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => {
                let rendered = text.get_or_insert_with(|| {
                    number
                        .format_form(u64::from(*created_digits), *created_form)
                        .into_bytes()
                });
                Cow::Borrowed(rendered.as_slice())
            }' \
'        let current_digits = self.activation().settings.digits();
        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits: _,
                created_form,
                text,
            } => {
                let rendered = text.get_or_insert_with(|| {
                    number
                        .format_form(u64::from(current_digits), *created_form)
                        .into_bytes()
                });
                Cow::Borrowed(rendered.as_slice())
            }'

run_one "9. formatting with the current form, not the created form" "${VALUE_RS}" \
'        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => {
                let rendered = text.get_or_insert_with(|| {
                    number
                        .format_form(u64::from(*created_digits), *created_form)
                        .into_bytes()
                });
                Cow::Borrowed(rendered.as_slice())
            }' \
'        let current_form = self.activation().settings.form();
        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits,
                created_form: _,
                text,
            } => {
                let rendered = text.get_or_insert_with(|| {
                    number
                        .format_form(u64::from(*created_digits), current_form)
                        .into_bytes()
                });
                Cow::Borrowed(rendered.as_slice())
            }'

require_baseline_pass "after the last restore"

echo "==============================================================================
${CAUGHT} of ${TOTAL} mutations caught by rust/corpus/phase-4a.txt
=============================================================================="

if [ "${#FAILURES[@]}" -gt 0 ]; then
    echo "NOT CAUGHT:"
    for f in "${FAILURES[@]}"; do
        echo "  - ${f}"
    done
    exit 1
fi

exit 0
