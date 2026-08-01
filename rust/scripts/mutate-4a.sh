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
# command criterion 1 runs. A mutation is caught if that command's exit
# code is non-zero -- either because a program's output diverges from the
# oracle (the ordinary case) or because the mutated code no longer compiles
# or panics (still a real, observable failure of the subset, not something
# this script tries to distinguish from a value mismatch). A mutation
# caught only by `cargo test -p rexx-exec --lib` and not by the corpus
# itself would not satisfy this criterion, so this script does not run the
# lib suite at all -- see the gate report for which of the nine mutations
# needed a new corpus witness before the subset could see them.
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

# Runs the L0 subset against the oracle in gate (STRICT) mode. Returns the
# command's own exit code; does not itself decide pass/fail.
run_subset() {
    REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus \
        > /tmp/mutate-4a-subset-output.txt 2>&1
}

# One mutation: apply, run the subset, report, revert. Exits the whole
# script non-zero immediately if the pattern could not be applied
# (UNAPPLIED PATTERN is fatal, never merely "skipped"); records but does not
# abort on a mutation that fails to be CAUGHT, so one uncaught mutation does
# not hide the result of the other eight.
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

    if run_subset; then
        echo "NOT CAUGHT: the subset still passes under this mutation."
        tail -n 5 /tmp/mutate-4a-subset-output.txt
        FAILURES+=("${name}")
    else
        echo "caught: the subset diverges under this mutation."
        grep -E "matching|FAILED" /tmp/mutate-4a-subset-output.txt || true
        CAUGHT=$((CAUGHT + 1))
    fi

    restore
    echo
}

require_clean

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
