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

# The 4c exit gate's criterion 6: a committed list of one-line mutations to the
# code Phase 4c added, **each of which declares in advance what each of two
# instruments should say about it, and is measured against that declaration**.
# An unexpected survival and an unexpected catch are both failures.
#
# THE MUTATIONS ARE 4c's. `mutate-4a.sh` targets the value model, `mutate-4b.sh`
# activations and conditions; re-running either here would report a number that
# says nothing about builtins, `PARSE`, `ADDRESS` or `::ROUTINE` dispatch. Every
# OLD string below is in code that did not exist before this sub-phase, with one
# deliberate exception noted at its own row.
#
# WHAT IS REUSED IS THE GUARD, and it is reused because a branch review
# reproduced the exact defect it closes: the first `mutate-4a.sh` reported
# "9 of 9 mutations caught", exit 0, with the oracle binary absent, because any
# non-zero exit counted as a catch. Four devices carry over:
#
#   * `apply_mutation` requires OLD to occur EXACTLY ONCE. Zero or two is a hard
#     failure, never a skip.
#   * `require_baseline_pass` runs the unmutated tree before the first mutation
#     and again after the last restore, on BOTH instruments.
#   * `corpus_status`/`suite_status` classify each run into PASSED, DIVERGED or
#     INFRA_FAILURE by reading the instrument's own printed report rather than
#     its exit code alone. INFRA_FAILURE aborts immediately and is never folded
#     into either bucket, because neither is true -- the mutation was never
#     actually exercised.
#   * Every run asserts a non-zero test count, because `cargo test <name>` exits
#     0 when it matches nothing (`0 passed; 0 failed; N filtered out`, status 0),
#     so a harness reading only the status cannot tell "passed" from "does not
#     exist".
#
# ---------------------------------------------------------------------------
# THREE THINGS THIS SCRIPT ADDS THAT 4b's DOES NOT HAVE, each of which is a
# measured defect in this phase rather than a precaution.
# ---------------------------------------------------------------------------
#
# 1. `--no-fail-fast` IS MANDATORY, AND ITS ABSENCE HAS ALREADY PRODUCED FALSE
#    COVERAGE CLAIMS HERE. `cargo test --workspace` stops after the first test
#    binary that fails, so under a mutation the run is TRUNCATED AT THE FIRST
#    CATCHER and every later binary silently never executes. That is invisible
#    in a green baseline -- the truncation only happens when a mutation bites --
#    so the flag looks unnecessary right up to the moment it matters. Measured
#    at 4c's Task 9: three of six "caught by this test and nothing else" claims
#    were false, and re-running with the flag found `corpus_differential`
#    catching two of them and `keyword_assertions::
#    the_exempt_set_matches_the_current_failures` catching a third.
#
# 2. THE BINARY COUNT IS ASSERTED, BASELINE AGAINST MUTATED, AND IT IS MEASURED
#    RATHER THAN WRITTEN DOWN. `--no-fail-fast` is what makes the later binaries
#    run; asserting the count is what proves they did. The guard counts
#    `Running`/`Doc-tests` HEADER lines and requires the mutated run to produce
#    the same number as the baseline this script performed moments earlier, so a
#    truncated run is an INFRA_FAILURE rather than a survivor or a clean catch.
#
#    HEADER LINES, NOT `test result:` LINES, AND THE TWO GENUINELY DIFFER.
#    `Doc-tests rexx_exec` prints TWO `test result:` blocks from ONE process --
#    a normal doctest run and a `compile_fail` one reported separately -- so
#    `test result:` double-counts that process and reads as off-by-one forever.
#    The header line is the process count and is what a truncation guard must
#    compare. Measured at 4c's Task 9's re-review, a green run gave 72 headers
#    and 73 `test result:` lines; re-measured at Task 15 it gave 75 and 76.
#    BOTH PAIRS ARE STALE THE MOMENT A TASK ADDS A TEST BINARY, which is why the
#    number is derived from this script's own baseline in the same run and is
#    never a constant here.
#
#    THE TWO COUNTS LIVE ON DIFFERENT DESCRIPTORS. Cargo writes the `Running`
#    headers to STDERR and libtest writes `test result:` to STDOUT, so the two
#    are captured separately below. A `2>&1` capture would interleave them
#    undefinedly and the header count would depend on buffering.
#
#    AND A MEASURED ZERO IS FATAL, BECAUSE THIS GUARD CAN OTHERWISE TURN ITSELF
#    OFF. The header count comes from a regex over cargo's own banner. If cargo
#    changes that banner the regex matches nothing and the baseline measures 0
#    binaries -- a real measurement, not a missing one -- while `test result:`
#    on the other descriptor is untouched, so every row still classifies PASSED
#    or DIVERGED and the script reports 9 of 9 at exit 0 with device (e) silently
#    disabled. `require_baseline_pass` aborts on a zero, and the "not measured
#    yet" state is the empty string rather than 0 so the two cannot be confused.
#
# 3. EVERY MUTATED RUN IS UNDER A WALL-CLOCK TIMEOUT, AND A TIMEOUT IS AN
#    INFRA_FAILURE. A mutation can HANG rather than fail: measured at 4c's
#    Task 14, a mutation to the executor left `keyword_assertions_differential`
#    running for over nine minutes on `DO::test_DO_standardTest2P`, whose
#    extracted body loops until a condition the mutation had made unreachable.
#    `cargo test` has no per-test timeout, so the whole guard sits there and no
#    bucket is ever assigned. A timeout is classified INFRA_FAILURE -- never
#    DIVERGED, which would credit the mutation with a catch the suite did not
#    actually make, and never PASSED.
#
# ---------------------------------------------------------------------------
# TWO INSTRUMENTS, RUN FOR EVERY MUTATION.
# ---------------------------------------------------------------------------
#
#   CORPUS  `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` -- the
#           byte-for-byte differential against the C++ oracle over the union of
#           `corpus/phase-4a.txt`, `phase-4b.txt` and `phase-4c.txt`. Classified
#           by its own "N of M matching" line.
#   SUITE   `cargo test --workspace --no-fail-fast` -- the whole workspace,
#           because the question every row below asks is "what else sees this",
#           and a narrower target list answers a narrower question. Classified
#           by libtest's own `test result:` lines together with the binary-count
#           guard above.
#
# A PASSED/PASSED DECLARATION NEEDS A WRITTEN JUSTIFICATION AT ITS OWN ROW,
# naming the instrument that *should* have caught it and why it cannot. Without
# that rule, declaring a mutation a survivor because nothing happens to test it
# scores "as declared" and reports coverage that does not exist.
#
# USAGE: run from anywhere; it locates the repository root from its own path. No
# arguments. Exits 0 only if every mutation below was applied and every observed
# status pair matched its declared pair. Leaves the tree exactly as it found it
# -- every mutation is reverted immediately after its own check, and an EXIT trap
# guarantees the revert runs even if the script is interrupted mid-mutation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${RUST_DIR}"

RUN_RS="crates/rexx-exec/src/run.rs"
PARSE_RS="crates/rexx-exec/src/parse_template.rs"
BUILTIN_RS="crates/rexx-exec/src/builtin/mod.rs"
DATATYPE_RS="crates/rexx-exec/src/builtin/datatype.rs"
DATETIME_RS="crates/rexx-exec/src/builtin/datetime.rs"
ACTIVATION_RS="crates/rexx-exec/src/activation.rs"
MUTATED_FILES=("${RUN_RS}" "${PARSE_RS}" "${BUILTIN_RS}" "${DATATYPE_RS}" \
               "${DATETIME_RS}" "${ACTIVATION_RS}")

# How long one mutated instrument run may take before it is called an
# infrastructure failure. Generous against the slowest observed clean run and
# still far below the nine-minute hang device 3 above describes.
RUN_TIMEOUT=900

TOTAL=0
AS_DECLARED=0
FAILURES=()

# Measured by `require_baseline_pass` before the first mutation, and required of
# every mutated SUITE run thereafter. Never a constant: see device 2 above.
#
# The empty string is "not measured yet", which no measurement can produce. It
# used to be 0, and 0 is a value `suite_binaries` really can return -- if cargo
# changes its per-target banner the regex stops matching and the count is a
# genuine zero. That zero would then be stored here as the expectation, and
# `suite_status` reads a zero expectation as "no expectation", so every row
# below would skip the truncation check with nothing red anywhere.
BASELINE_BINARIES=""

# A plain file copy, not `git checkout -- <path>`: this project's git discipline
# forbids that command regardless of caller, and restoring from a backup taken
# before the first mutation can never discard someone else's uncommitted work
# the way `git checkout --` silently has.
BACKUP_DIR="$(mktemp -d)"
for f in "${MUTATED_FILES[@]}"; do
    cp "${f}" "${BACKUP_DIR}/$(echo "${f}" | tr / _)"
done

restore() {
    for f in "${MUTATED_FILES[@]}"; do
        cp "${BACKUP_DIR}/$(echo "${f}" | tr / _)" "${f}"
    done
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
    if ! git diff --quiet -- "${MUTATED_FILES[@]}"; then
        echo "FATAL: one of ${MUTATED_FILES[*]} has uncommitted changes before" \
             "this script touched it. Refusing to run: commit or stash first, so" \
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

# Exit status 124 is `timeout`(1)'s own, and it is returned here rather than
# swallowed so the caller can tell a hang from a failure.
run_corpus() {
    REXX_CORPUS_GATE=1 timeout "${RUN_TIMEOUT}" \
        cargo test --offline -p rexx-exec --test corpus \
        > "${CORPUS_STDOUT}" 2> "${CORPUS_STDERR}"
}

# The last "N of M matching" line `corpus.rs` printed, as "N M", or nothing at
# all -- which is exactly the signal that the run never reached the point of
# comparing anything (a missing oracle, a compile error, a panic before the
# report). The report goes to the process's real stderr, so both descriptors are
# searched rather than guessing which one carried it.
corpus_matching_line() {
    grep -hoE '^[0-9]+ of [0-9]+ matching' "${CORPUS_STDOUT}" "${CORPUS_STDERR}" |
        tail -n 1 | sed -E 's/^([0-9]+) of ([0-9]+) matching$/\1 \2/'
}

corpus_status() {
    local exit_code="$1" line matched total
    if [ "${exit_code}" -eq 124 ]; then
        echo "INFRA_FAILURE"
        return
    fi
    line="$(corpus_matching_line)"
    if [ -z "${line}" ]; then
        echo "INFRA_FAILURE"
        return
    fi
    read -r matched total <<<"${line}"
    # "0 of 0 matching" at exit 0 is an EMPTY SUBSET, not a pass, and it is the
    # corpus's own version of `cargo test` matching no tests.
    if [ "${total}" -eq 0 ]; then
        echo "INFRA_FAILURE"
        return
    fi
    if [ "${exit_code}" -eq 0 ]; then
        if [ "${matched}" -eq "${total}" ]; then echo "PASSED"; else echo "INFRA_FAILURE"; fi
    else
        if [ "${matched}" -lt "${total}" ]; then echo "DIVERGED"; else echo "INFRA_FAILURE"; fi
    fi
}

# ---------------------------------------------------------------------------
# Instrument 2: the whole workspace.
# ---------------------------------------------------------------------------

# STDOUT AND STDERR ARE SEPARATE FILES, never `2>&1`: the binary-count guard
# reads cargo's `Running` headers off stderr and the pass/fail counts off
# libtest's stdout, and interleaving them makes the header count depend on
# buffering. See device 2 in the header.
run_suite() {
    timeout "${RUN_TIMEOUT}" \
        cargo test --offline --workspace --no-fail-fast \
        > "${SUITE_STDOUT}" 2> "${SUITE_STDERR}"
}

# How many test binaries cargo actually started, from its own per-target banner
# on stderr. `Doc-tests <crate>` is one process and is counted once, which is
# the whole reason this is not counted from `test result:` lines.
suite_binaries() {
    grep -cE '^ +(Running|Doc-tests) ' "${SUITE_STDERR}" || true
}

# One "PASSED FAILED" pair per `test result:` line, in order -- NOT a sum.
suite_counts() {
    grep -oE '^test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed' "${SUITE_STDOUT}" |
        sed -E 's/^test result: [a-zA-Z]+\. ([0-9]+) passed; ([0-9]+) failed$/\1 \2/'
}

# Classifies a `run_suite` call. `expected_binaries` is 0 for the baseline call,
# which is the run that establishes the figure, and `BASELINE_BINARIES`
# thereafter.
#
# Five ways to be INFRA_FAILURE before the pass/fail question is even asked:
#
#   * the run timed out -- see device 3 in the header;
#   * no `test result:` lines at all -- nothing compiled or ran;
#   * fewer binaries than the baseline started -- the run was TRUNCATED, which
#     is what `--no-fail-fast` exists to prevent and what this guard proves;
#   * more binaries than the baseline started -- the tree gained a test binary
#     mid-run, so the comparison is not between the same two things;
#   * every `test result:` line reporting zero tests -- `cargo test` exits 0
#     when it matches nothing, which is the defect this whole guard exists for.
suite_status() {
    local exit_code="$1" expected_binaries="$2"
    local counts binaries total_failed=0 passed failed total_run=0
    if [ "${exit_code}" -eq 124 ]; then
        echo "INFRA_FAILURE"
        return
    fi
    counts="$(suite_counts)"
    if [ -z "${counts}" ]; then
        echo "INFRA_FAILURE"
        return
    fi
    binaries="$(suite_binaries)"
    if [ "${expected_binaries}" -ne 0 ] && [ "${binaries}" -ne "${expected_binaries}" ]; then
        echo "INFRA_FAILURE"
        return
    fi
    while read -r passed failed; do
        total_run=$((total_run + passed + failed))
        total_failed=$((total_failed + failed))
    done <<<"${counts}"
    if [ "${total_run}" -eq 0 ]; then
        echo "INFRA_FAILURE"
        return
    fi
    if [ "${exit_code}" -eq 0 ]; then
        if [ "${total_failed}" -eq 0 ]; then echo "PASSED"; else echo "INFRA_FAILURE"; fi
    else
        if [ "${total_failed}" -gt 0 ]; then echo "DIVERGED"; else echo "INFRA_FAILURE"; fi
    fi
}

# The aggregate, for progress lines only -- never for a decision.
suite_totals() {
    suite_counts | awk '{p += $1; f += $2} END {if (NR == 0) exit 0; print p, f}'
}

# WHICH TARGET BINARY failed, by name. Cargo prints a `Running <target> (...)`
# banner before each binary's own report -- on stderr -- and libtest prints the
# report on stdout, so the two cannot be correlated by adjacency in one stream.
# What can be correlated is the `failures:` block libtest prints, which names the
# tests themselves; that is what is reported, since a test name identifies its
# instrument as well as its binary does and does not depend on interleaving.
suite_failing_tests() {
    awk '/^failures:$/ { grab = 1; next }
         /^test result:/ { grab = 0 }
         grab && /^    [A-Za-z_0-9:]+$/ { print $1 }' "${SUITE_STDOUT}" |
        sort -u | head -n 12 | tr '\n' ' '
}

# Both instruments must report a clean unmutated tree before the first mutation
# and after the last restore. A compile failure, a missing oracle or an
# interrupted revert is caught here rather than being mistaken for a mutation
# result.
require_baseline_pass() {
    local label="$1" exit_code status matched total passed failed
    echo "=== baseline (${label}): the unmutated tree must pass both instruments ==="

    if run_corpus; then exit_code=0; else exit_code=$?; fi
    status="$(corpus_status "${exit_code}")"
    if [ "${status}" != "PASSED" ]; then
        echo "FATAL: the unmutated CORPUS does not pass (${label}, status=${status})." \
             "Nothing below this point can be trusted as a mutation result -- any" \
             "row could be an environment failure wearing a catch's clothing." >&2
        cat "${CORPUS_STDOUT}" "${CORPUS_STDERR}" >&2
        exit 1
    fi
    read -r matched total <<<"$(corpus_matching_line)"
    echo "baseline corpus ok (${label}): ${matched} of ${total} matching"

    if run_suite; then exit_code=0; else exit_code=$?; fi
    # The baseline is the run that ESTABLISHES the binary count, so it is
    # classified against 0 (no expectation) and the count is read out of it
    # afterwards. Comparing the baseline against a figure it has not produced
    # yet is the circularity this argument exists to avoid.
    status="$(suite_status "${exit_code}" 0)"
    if [ "${status}" != "PASSED" ]; then
        echo "FATAL: the unmutated SUITE does not pass (${label}, status=${status})." >&2
        cat "${SUITE_STDOUT}" "${SUITE_STDERR}" >&2
        exit 1
    fi
    local binaries
    binaries="$(suite_binaries)"
    # A suite that just passed started at least one binary, so a zero here is
    # the regex in `suite_binaries` no longer matching cargo's output rather
    # than a fact about the tree. Fatal rather than stored: stored, it would
    # disarm the truncation check for every row while all nine still scored
    # normally from the `test result:` lines on the other descriptor.
    if [ "${binaries}" -eq 0 ]; then
        echo "FATAL: the ${label} baseline started 0 test binaries, which a passing" \
             "suite cannot do. suite_binaries' Running/Doc-tests pattern no longer" \
             "matches cargo's stderr, so device (e), the truncation guard, has" \
             "nothing to compare against and every mutation below would skip it." >&2
        exit 1
    fi
    if [ -z "${BASELINE_BINARIES}" ]; then
        BASELINE_BINARIES="${binaries}"
    elif [ "${binaries}" -ne "${BASELINE_BINARIES}" ]; then
        echo "FATAL: the tree started ${binaries} test binaries at the ${label}" \
             "baseline and ${BASELINE_BINARIES} at the first one. Every mutation" \
             "result above was compared against a figure that has since moved." >&2
        exit 1
    fi
    read -r passed failed <<<"$(suite_totals)"
    echo "baseline suite ok (${label}): ${binaries} binaries, ${passed} passed, ${failed} failed"
    echo
}

# One mutation: apply, run BOTH instruments, compare each observed status
# against the declared one, revert.
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
        echo "FATAL: mutation '${name}' could not be assessed on the CORPUS -- the" \
             "run failed for an infrastructure reason (exit ${exit_code}; 124 is a" \
             "timeout), not an observed divergence. Reporting it as caught or not" \
             "caught would both be wrong." >&2
        cat "${CORPUS_STDOUT}" "${CORPUS_STDERR}" >&2
        restore
        exit 1
    fi

    if run_suite; then exit_code=0; else exit_code=$?; fi
    suite_actual="$(suite_status "${exit_code}" "${BASELINE_BINARIES}")"
    if [ "${suite_actual}" = "INFRA_FAILURE" ]; then
        echo "FATAL: mutation '${name}' could not be assessed on the SUITE -- the run" \
             "failed for an infrastructure reason (exit ${exit_code}; 124 is a" \
             "timeout), started $(suite_binaries) of ${BASELINE_BINARIES} test" \
             "binaries, or executed no tests at all." >&2
        tail -n 40 "${SUITE_STDOUT}" >&2
        tail -n 40 "${SUITE_STDERR}" >&2
        restore
        exit 1
    fi

    echo "observed: corpus=${corpus_actual} suite=${suite_actual}"
    if [ "${corpus_actual}" = "DIVERGED" ]; then
        grep -hE 'matching' "${CORPUS_STDOUT}" "${CORPUS_STDERR}" | tail -n 1 || true
    fi
    if [ "${suite_actual}" = "DIVERGED" ]; then
        echo "  failing test(s): $(suite_failing_tests)"
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
CORPUS_STDOUT="$(mktemp)"
CORPUS_STDERR="$(mktemp)"
SUITE_STDOUT="$(mktemp)"
SUITE_STDERR="$(mktemp)"

require_clean
require_baseline_pass "before any mutation"

# ---------------------------------------------------------------------------
# The mutations.
# ---------------------------------------------------------------------------

# --- the builtin dispatch and its arity table (Task 2) ---------------------

# Every optional argument reads as omitted, so every builtin falls back to its
# own default whatever the call supplied. `optional_string` is the one accessor
# the optional-typed helpers share, and `required_string` is deliberately NOT
# touched: a mutation that broke required arguments too would be caught by
# everything and would say nothing about which instrument sees an optional one.
run_one "1. a builtin's optional argument reads as omitted" "${BUILTIN_RS}" DIVERGED DIVERGED \
'    let value = arg(args, position)?;
    Some(interp.to_text(value).into_owned())' \
'    let value = arg(args, position).filter(|_| false)?;
    Some(interp.to_text(value).into_owned())'

# The minimum-argument bound off by one, so a call one argument short of the
# minimum is accepted and reaches the builtin body instead of raising 40.3.
run_one "2. the minimum arity bound is off by one" "${BUILTIN_RS}" PASSED DIVERGED \
'    if args.len() < builtin.min {' \
'    if args.len() + 1 < builtin.min {'

# --- the PARSE template engine (Task 7) ------------------------------------

# An absolute positional trigger measures from origin zero rather than origin
# one, so `=5` lands one character late. `corpus/lang/parse_triggers.rex`'s own
# header states, per block, which wrong answer this prints.
run_one "3. an absolute PARSE trigger is off by one" "${PARSE_RS}" DIVERGED DIVERGED \
'        let offset = column.saturating_sub(1);' \
'        let offset = column;'

# The `.` placeholder discards its SLOT rather than its FIELD: it consumes
# nothing, so the target after it receives the field the placeholder should have
# eaten, and no `>.>` line is emitted. This is the shape the criterion asks for
# -- a placeholder that fails to discard -- expressed as the one thing the
# `None` arm can be made to do wrongly, since there is no target for it to
# assign to.
run_one "4. the . placeholder consumes no field" "${PARSE_RS}" DIVERGED DIVERGED \
'        for (index, target) in trigger.targets.iter().enumerate() {
            let piece = if index == last {' \
'        for (index, target) in trigger.targets.iter().enumerate() {
            if target.is_none() {
                continue;
            }
            let piece = if index == last {'

# The comma fence stops advancing to the next parse string, so every template
# after the first re-reads the first source. `PARSE ARG a, b` then binds `b`
# from argument one rather than argument two.
run_one "5. the comma fence does not advance to the next source" "${PARSE_RS}" DIVERGED DIVERGED \
'                cursor = self.next_template(&mut strings, parse, indent);
                continue;' \
'                continue;'

# --- ADDRESS (Task 9) ------------------------------------------------------

# The bare `ADDRESS` toggle keeps the current name and overwrites the alternate
# with it, so a swap becomes a no-op after the first. `corpus/lang/
# address_env.rex`'s block D is the swap four toggles deep, read back through
# `ADDRESS()`, which is the only shape that can tell a real swap from a
# one-way copy.
run_one "6. the bare ADDRESS toggle keeps the current name" "${ACTIVATION_RS}" DIVERGED DIVERGED \
'        std::mem::swap(&mut self.current, &mut self.alternate);' \
'        self.alternate = self.current.clone();'

# --- ::ROUTINE dispatch (Task 13) ------------------------------------------

# A same-file `::ROUTINE` is consulted BEFORE the builtin table, so a routine
# named for a builtin shadows it. Measured on the oracle in the other
# direction: `call max 1, 9` with a `::routine max` present reports 9 and the
# routine never runs, so this mutation makes the crate answer the routine.
#
# DECLARED PASSED/DIVERGED AND MEASURED DIVERGED/DIVERGED, and the declaration
# is corrected to what was measured rather than left standing as the guess.
# `corpus/lang/routine_dispatch.rex` defines `::routine length` and
# `::routine 'max'` for exactly this reason -- the resolution order is what
# that program was written to pin, and it does. The guess was that the order
# had only an in-crate witness; it has both, and the corpus one is the
# stronger of the two because it compares against the oracle rather than
# against this crate's own expectation.
run_one "7. a ::ROUTINE is resolved before the builtin table" "${RUN_RS}" DIVERGED DIVERGED \
'            None if builtin::is_builtin(name) => Resolved::Builtin,' \
'            None if !self.routines.contains_key(&name.to_ascii_uppercase()[..])
                && builtin::is_builtin(name) =>
            {
                Resolved::Builtin
            }'

# --- TIME('R') (Task 12) -------------------------------------------------

# The elapsed-time anchor is never moved, so `TIME('R')` reports the time since
# the first reading forever instead of resetting. THE CORPUS CANNOT SEE THIS AT
# ALL, and not because no program happens to exercise it: decision D11 bars
# `RANDOM`, `DATE` and `TIME` from every differential corpus program, because
# their answers differ between two runs of the same interpreter. Task 12's own
# unit tests in `crates/rexx-exec/src/builtin/datetime.rs` are the whole gate
# this construct has.
#
# WHICH of them, because the obvious name is the wrong one.
# `two_time_r_reads_in_one_clause_answer_identically` survives this mutation:
# it asserts that two reads inside one clause agree, and they still do when the
# anchor never moves. What fires is
# `time_r_resets_relative_to_the_last_reset_not_program_start` and
# `a_callees_own_time_r_leaks_into_the_caller_after_it_returns`, both of which
# need a SECOND reset to be distinguishable from the first.
run_one "8. TIME('R') never moves the elapsed anchor" "${DATETIME_RS}" PASSED DIVERGED \
'            interp.elapsed_anchor = Some(stale);' \
'            let _ = stale;'

# --- the collect-on-every-allocation negative control (criterion 4) --------

# THE BUILTIN-SHAPED NEGATIVE CONTROL, and finding it took establishing that
# the obvious one does not exist. Builtins reuse `resolve_and_run_call`'s
# argument evaluation, so the obvious root in that window is
# `self.roots.push_temp(argument.value())` -- which is verbatim `mutate-4b.sh`
# row 9, and re-running it would re-test 4b.
#
# A builtin's own RESULT holds no root: `resolve_and_run_call`'s builtin arm
# hands it straight back, rooted by whichever caller receives it exactly as a
# callee's `RETURN` value is. So there is no root there to delete. What there
# IS, and what this row deletes, is a root a builtin holds over an argument of
# its own across an allocation it makes itself: `VALUE(name, newvalue)` reads
# the old value out of the stem and must keep it alive across `stem_set`,
# because on a never-touched stem both `stem_get` and `stem_set` allocate on
# the same branch. That is a builtin's own root, in a builtin's own window,
# and it is not 4b's.
run_one "9. VALUE's old stem value is unrooted across stem_set (builtin-shaped control)" "${DATATYPE_RS}" PASSED DIVERGED \
'            interp.roots.push_temp(old);' \
'            let _ = &old;'

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
