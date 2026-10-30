# Task 8.8 final gate plan

This is an unexecuted controller plan for the final committed Task 8.8 source. The controller must supply the final full source commit and release pin after the repair owner finishes. No current HEAD, previous successful slice, or prerequisite enumeration is final-gate evidence. Tasks 9 and 10 remain outside this plan.

## Inputs and bounds

The retained prerequisite runner is `/tmp/codex-p9d-t88-prereq-enumeration/driver.py`, with its durable archive at `docs/superpowers/records/2026-10-18-phase-9d/t8.8/prerequisites/raw.tar.gz`. Its completed enumeration at `87357a519cdac3cbfc79810dc58026d525c0c6d9` selected 154 default test binaries, with 3,769 listed tests, and separately completed 15 doctests. Task 8.8f subsequently added the `rexx-exec` integration target `semantic_boundaries`, which is explicitly reconciled as an addition and must run in full. Final binary and test counts come from fresh metadata, Cargo artifacts, and complete listings, while the historical counts remain provenance only.

The owned draft `/tmp/codex-p9d-t88-final-gate-draft.py` reuses the old runner's Cargo JSON selection, package-directory execution, separate descriptors, bounded children, and standalone Cargo doctests. Each invocation runs one action, with an outer 535-second foreground timeout and five-second kill grace. A default target action allows 15 seconds for listing and 510 seconds for its tests, plus bounded termination. Run each generated target command as a separate foreground command, never one enclosing shell loop over the whole population.

Use this environment from `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust` after substituting the controller-supplied full SHA. The output directory below is a proposed new scratch destination, not existing evidence. The draft removes inherited `REXX_*` settings before each command and enables only the debug corpus gate and diagnostic directory. Clear inherited codegen/profile/target overrides before use, including `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_PROFILE_*`, and preexisting target runners, and record the toolchain and allowed overrides without serializing credentials.

```bash
export T88_FINAL_COMMIT=<controller-supplied-final-full-sha>
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target
export CARGO_BUILD_JOBS=4
export T88_PROFILE_COMPLETION_RECORD=/tmp/codex-p9d-t88-final-profile-completion.json
mkdir -p /tmp/codex-p9d-t88-final-gates
timeout --foreground -k 5 535 memcap 8G python3 scripts/test_preflight.py
timeout --foreground -k 5 535 rustc -Vv
timeout --foreground -k 5 535 cargo --version
timeout --foreground -k 5 535 git rev-parse HEAD
timeout --foreground -k 5 535 git status --short --untracked-files=all
timeout --foreground -k 5 535 git diff --cached --exit-code
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py freeze
```

The preflight must pass with this checkout's real `build/lib`, `ootest`, and `oodocs`, without substitutions from the oracle tree. The full suite additionally validates the actual ooTest SVN revision through its existing `suite_revision()` path. Commit source and its report placeholders before the gates, verify that HEAD equals the supplied SHA, then freeze source/test inputs and index staging until every command finishes. Retain command, cwd, allowed environment, separate stdout/stderr, exit status, timeout flag, elapsed time, and source pin for every command, including failures.

Profiling and source repairs must complete before the blanket freeze begins. The parent first settles and commits its tracked performance/report outputs, then writes `T88_PROFILE_COMPLETION_RECORD` as JSON with `source_commit` equal to the final supplied SHA, `profiling_complete: true`, and `active_profile_commands: []`. The `freeze` action copies that completion record, records an empty evidence-output allowlist, and snapshots the full `git ls-files --stage -z` identity before any broad gate. A dirty tracked evidence file blocks freeze initiation without claiming the interpreter source changed, and no concurrent tracked report/performance update or index staging is allowed during validation.

The tracked/index guards run before and after each draft action, checking fixed HEAD, no tracked diff, unchanged staging identity, and no uncommitted Rust inputs. Keep checkout-local native inputs, ooTest/reference sources, and the oracle tree read-only throughout the same interval. Raw gate outputs stay in the owned scratch directory until all commands complete, so reporting does not invalidate the frozen committed tree. The parent releases the blanket freeze before copying or staging final evidence, and the runner has no mutable-path exception framework.

## Formatting, lint, and release gates

Run the following commands separately under the environment above. `REXX_*` variables must be absent except the exact values on each command, so use a controller environment cleared in the same way as the draft. Save each command's descriptors and unpiped status under a distinct name in the proposed scratch output, then copy the final records into the proposed durable directory `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-gates/` after the freeze ends. Keep an initial failure and its diagnostics intact if a later repair requires a new run.

```bash
timeout --foreground -k 5 535 memcap 8G cargo fmt --all --check
timeout --foreground -k 5 535 memcap 8G cargo clippy --workspace --all-targets --locked -j4 -- -D warnings
timeout --foreground -k 5 535 memcap 8G python3 scripts/test_preflight.py -- cargo test --release --locked -j4 -p rexx-exec --test corpus --test suite --no-run --message-format=json
REXX_CORPUS_GATE=1 timeout --foreground -k 5 535 memcap 8G cargo test --release --locked -j4 -p rexx-exec --test corpus -- --test-threads=1 --nocapture
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/tmp/codex-p9d-t88-final-gates/suite/table.tsv REXX_SUITE_DIAGNOSTICS=/tmp/codex-p9d-t88-final-gates/suite-diagnostics timeout --foreground -k 5 535 memcap 8G cargo test --release --locked -j4 -p rexx-exec --test suite l3_core_suite -- --exact --test-threads=1 --nocapture
```

The release corpus command runs the whole target with its full gate enabled. Require its completed nonzero test population, the differential's completed program denominator, zero mismatches, and status 0, rather than accepting a Cargo build or an early gate skip. The suite command selects exactly one Rust harness test but covers every ooTest group because `REXX_SUITE_GROUPS` is absent. Require the final release executable/source pin supplied by the controller to match the source under test, and retain the fresh release Cargo JSON executable identities rather than reusing the prerequisite worktree's binaries.

Task 8.8 is not a phase boundary, so the required target directory remains `/tmp/codex-p9d-t88-target` for lint. `rust/CLAUDE.md` separately requires clean-target lint at a phase boundary, which is not established by a same-directory Task 8.8 lint. Do not delete caches to manufacture that condition during another owner's work. A release build exceeding this bound is BLOCKED evidence for the controller, not authority to enlarge the timeout or reduce its target selection.

## Bounded debug workspace gate

The debug obligation is `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast`, including default tests and Cargo doctests. `[profile.test]` has `opt-level=3` while retaining debug assertions and overflow checks, unlike release. The prerequisite's measured longest binary was `collect_stress` at 313.185 seconds, the default corpus target took 151.246 seconds, and a monolithic workspace command risks exceeding 540 seconds. The draft therefore separates Cargo build, nonexecuting dispatch capture, each actual default test binary, and native Cargo doctests.

```bash
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py metadata
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py build
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py dispatch
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py target 0
```

The metadata action proves that default `test=true` targets have supported library, binary, or integration kinds, without test-enabled examples or custom harnesses. The build uses `cargo test --workspace --no-run --locked -j4 --message-format=json` and requires exact target-key equality against fresh metadata, with every historical target retained and only the explicitly reconciled `semantic_boundaries` addition. Its key is `(manifest parent directory name, target name, target kind, target crate_types)`, so the prior worktree's package URL prefix is not mistaken for a changed package. Every selected artifact must have an empty feature list, a true test profile, debug assertions, overflow checks, and a Cargo-owned executable.

The dispatch action runs explicit `--lib --bins --tests` after that parity proof, using a temporary target runner which records dispatch and exits without executing the executable. **Dispatch is NOT gate evidence, and its zero exit does not mean tests passed.** Its exact capture count and executable set must equal the fresh artifact selection, and it must capture each Cargo package cwd, complete executable argv, executable hash, and whitelisted nonsecret Cargo package/runtime-library variables. It excludes doctests by explicit target selection, reports unsupported targets instead of excluding them, and never copies binaries or reconstructs executable names from directory scans.

After successful dispatch verification, generate the remaining individual controller commands with the following preparation-only command. Execute each emitted line separately and continue independent target commands after a nonzero test result to preserve `--no-fail-fast` coverage. The draft refuses to overwrite a target record and applies the blanket tracked/index freeze around each action after recorded profiling completion. A target action lists the exact binary, verifies a completed libtest summary against that listed population, rejects filtering or timeout, and records its executable hash.

```bash
python3 - <<'PY'
import json
from pathlib import Path
out = Path('/tmp/codex-p9d-t88-final-gates')
items = json.loads((out / 'artifacts.json').read_text())
assert items
reconciled = json.loads((out / 'artifact-population-reconciliation.json').read_text())
assert len(items) == reconciled['fresh_count']
lines = [f'timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py target {index}' for index in range(len(items))]
(out / 'target-commands.txt').write_text('\n'.join(lines) + '\n')
PY
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py doc
timeout --foreground -k 5 535 memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py audit
```

If target 0 already ran, the controller starts the emitted queue at target 1. Workspace completion requires exactly one complete result per fresh artifact, the entire reconciled fresh population covered, zero target timeouts or failures, matching per-binary list/summary counts, and no unexpected filtered tests. Empty unit harnesses are legitimate and must still execute, while the aggregate nonignored test population and the named required gates below must be nonzero. Added tests are included automatically, including every test in the new `semantic_boundaries` target, while removed tests or unexplained target changes stop for explicit controller reconciliation rather than accepting a smaller population.

The doc action invokes `cargo test --workspace --doc --locked -j4 --no-fail-fast -- --test-threads=1`, retaining Cargo/rustdoc ownership of compilation and execution. Require all 15 semantic doctest identities from the retained `doctest-results.json`, allowing line-number movement only. Their modes remain four executed examples, ten compile-fail examples, and one compile-only example, with no ignored example or unvisited doc target. Status 0, exact identities/modes, and completed doctest summaries are separate requirements from the binary tests.

## Coverage mapping and source checks

Each following obligation is satisfied by its single unfiltered default-profile target execution above. Keep its package, target key, artifact hash, listed test names, actual count, and result in a final coverage table. Do not rerun a successful binary solely to give it another gate label. Release corpus and release suite remain distinct release obligations, while collect stress and debug corpus must retain debug assertions.

| Obligation | Default-profile target |
| --- | --- |
| Source-line behavior and oracle expectations | `rexx-parse`: `sourceline`, `sourceline_oracle` |
| Admitted keyword/directive and witness coverage | `rexx-exec`: `coverage` |
| Refusal tables and source-site derivation | `rexx-exec`: `refusal_dispositions`, `refusal_sites` |
| Ownership and closed-phase policy | `rexx-exec`: `owners`, `closed_phases`, `phase_ten_reasons` |
| Representation policy with unchanged `ALLOWED` | `rexx-exec`: `representation_refusals` |
| Granted unsafe-site inventory | `rexx-core`: `unsafe_sites` |
| Collection on allocation with debug tripwires | `rexx-exec`: `collect_stress` |
| Full differential with debug tripwires | `rexx-exec`: `corpus`, `REXX_CORPUS_GATE=1` |
| Stderr-refusal fixture and suite instrument tests | `rexx-exec`: `suite`, full default target, diagnostics enabled |
| Task 8.8f semantic boundary regressions | `rexx-exec`: `semantic_boundaries`, full newly added target |
| Other default unit/integration regressions | Every remaining fresh Cargo artifact |

No oracle refresh or refusal-table refresh variable is enabled during these checks. In particular `REXX_REFUSAL_SITES_REFRESH`, oracle expectation regeneration, docs-less modes, suite selectors, calibration settings, and simulation overrides remain absent. Compare the final tracked/index freeze identities before and after, confirm `ALLOWED` did not change relative to Task 8.8's supplied source base, and inspect `unsafe_sites` output rather than inferring no new site from compilation. The source owner supplies any missing final/base pin, and all profiling/report updates must precede the blanket freeze.

## Whole-suite join and diagnostics

The exact prior full record is `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r4/suite/tests.tsv`. A read of that file and its companion tables found revision 13178, `# groups all`, 24,523 unique `(group,test)` keys, and 402 groups. Its schema is `group<TAB>test<TAB>row<TAB>oracle<TAB>crate`, while `table.tsv` has one `(group,side)` row per side and `verdicts.tsv` has one group row. The newly proposed `/tmp/codex-p9d-t88-final-gates/suite/tests.tsv` is emitted beside `REXX_SUITE_TABLE` before the harness's green assertion, so a failing final assertion can still have complete comparison evidence.

Perform a full outer join by the exact case-sensitive `(group,test)` pair, retaining `row`, `oracle`, and `crate` separately on both sides. Emit every change in any of those fields, plus every added or removed key, rather than comparing only aggregate passing counts. Reject duplicate keys, wrong headers, missing revision/full-groups markers, missing output companions, or unequal full-group populations before calling the record complete. Reconcile each added/removed key and each changed verdict explicitly, including improvements, oracle changes, owner changes, and regressions.

The controller can use the following exact join after the suite command completes. It writes the delta before enforcing population completeness, so a population failure still retains the discrepant keys. The final status and per-group verdicts remain evidence to assess separately from the join. Existing phase failures or SysFileTree history do not authorize a licence or a retry-until-green policy.

```bash
python3 - <<'PY'
import csv
import json
from pathlib import Path
old_root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r4/suite')
new_root = Path('/tmp/codex-p9d-t88-final-gates/suite')
def load(path, fields):
    lines = path.read_text().splitlines()
    assert '# ootest revision 13178' in lines, path
    reader = csv.DictReader((line for line in lines if not line.startswith('#')), delimiter='\t')
    assert reader.fieldnames == fields, (path, reader.fieldnames)
    return lines, list(reader)
fields = ['group', 'test', 'row', 'oracle', 'crate']
records = []
for root in [old_root, new_root]:
    lines, rows = load(root / 'tests.tsv', fields)
    assert '# groups all' in lines, root
    keyed = {(r['group'], r['test']): r for r in rows}
    assert len(keyed) == len(rows) and rows, root
    records.append(keyed)
before, after = records
changes = []
for group, test in sorted(before.keys() | after.keys()):
    a, b = before.get((group, test)), after.get((group, test))
    if a != b:
        changes.append([group, test, 'added' if a is None else 'removed' if b is None else 'changed', *[(a or {}).get(k, '<absent>') for k in fields[2:]], *[(b or {}).get(k, '<absent>') for k in fields[2:]]])
with (new_root / 'delta-vs-t8.7c-fix-r4.tsv').open('x') as output:
    writer = csv.writer(output, delimiter='\t', lineterminator='\n')
    writer.writerow(['group', 'test', 'change', 'old row', 'old oracle', 'old crate', 'new row', 'new oracle', 'new crate'])
    writer.writerows(changes)
tables = list(csv.DictReader((line for line in (new_root / 'table.tsv').read_text().splitlines() if not line.startswith('#')), delimiter='\t'))
verdicts = list(csv.DictReader((line for line in (new_root / 'verdicts.tsv').read_text().splitlines() if not line.startswith('#')), delimiter='\t'))
groups = {group for group, _ in after}
assert len({(r['group'], r['side']) for r in tables}) == len(tables)
assert {(r['group'], r['side']) for r in tables} == {(g, s) for g in groups for s in ['oracle', 'crate']}
assert len({r['group'] for r in verdicts}) == len(verdicts) == len(groups)
assert {r['group'] for r in verdicts} == groups
assert (new_root / 'oracle-differences.tsv').is_file()
(new_root / 'delta-summary.json').write_text(json.dumps({'old_tests': len(before), 'new_tests': len(after), 'groups': len(groups), 'changed_rows': len(changes), 'added': len(after.keys() - before.keys()), 'removed': len(before.keys() - after.keys())}, indent=2) + '\n')
assert before.keys() == after.keys(), 'Population changed: reconcile retained delta'
assert groups == {group for group, _ in before} and len(groups) == 402
PY
```

Enable `REXX_SUITE_DIAGNOSTICS` on both the release whole suite and debug workspace suite target before their first run. The existing hook retains each run's stdout, stderr, status/end reason, elapsed time, GroupRun fields, marker, generated `child.rex` where present, and run/group/interpreter path metadata before cleanup. Preserve these records for SysFileTree `TEST_TREE_DIRECTORY` and `fixtures::tests::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally`, including the first failure. The prior non-reproductions in `t8.8/timing/sysfiletree/`, `sysfiletree-trace/`, and `stderr/` are context, not final evidence or a Flaky licence.

If a fresh SysFileTree failure needs field-level reduction, retain its failing full-suite records first, then run the existing explicit diagnostic once under a distinct diagnostic directory. This is an additional diagnosis, not a substitute for the failed whole suite, and it does not alter the real ooTest source. A stderr-fixture failure already has its default target output and generated-child diagnostics, so do not rerun the whole target merely to obtain a green result. Bring the retained failing field, command, source pin, and proposed next action to the controller without adding a licence.

```bash
REXX_SUITE_DIAGNOSTICS=/tmp/codex-p9d-t88-final-gates/sysfiletree-diagnostic timeout --foreground -k 5 535 memcap 8G cargo test --release --locked -j4 -p rexx-exec --test suite fixtures::tests::sysfiletree_group_retains_assertion_and_metadata_context -- --exact --ignored --test-threads=1 --nocapture
```

## Completion and unresolved concerns

The draft has been written but has not been executed or certified against a final source pin. In particular its Cargo target-runner interception must satisfy the dispatch parity guard before any direct binary execution is credited, and metadata/custom-harness mismatches must stop rather than omit coverage. The controller still needs the final committed source/base hashes, release executable pin, recorded profiling completion and settled tracked outputs, fresh target counts, and measured statuses, and must review the bounded release build/suite duration on that source. This plan makes no claim that final formatting, lint, corpus, suite, stress, workspace tests, or doctests have passed.

The artifact population guards retain historical targets plus the known Task 8.8f addition, include all freshly listed tests, and stop on unexplained removals or additions. Each target writes its exact added/removed-name record, and the final `audit` action checks all fresh artifact results, derived counts, unchanged ignored-test identities, nonzero named obligations including `semantic_boundaries`, and completed doctests. The doctest action compares semantic identities with line-number normalization, while the suite join retains every changed row and population discrepancy. Add an audit result only after reading those actual outputs, then release the source/input and index-staging freeze and copy the raw runner, commands, diagnostics, and summaries into the durable final record.
