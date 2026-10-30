import hashlib
import json
import pathlib
import re
import subprocess

root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m6'
rows = []
for stage in ('baseline', 'mutant', 'restored'):
    for variant in ('reply', 'return'):
        name = f'{stage}-{variant}'
        descriptor = json.loads((records / f'{name}.json').read_text())
        stdout = (records / f'{name}.stdout').read_text()
        stderr = (records / f'{name}.stderr').read_text()
        selected = int(re.search(r'running (\d+) test', stdout)[1])
        expected = 101 if name == 'mutant-reply' else 0
        assert descriptor['status'] == expected and selected == 1
        steps, contended, switches = map(int, re.search(r'steps: (\d+), contended: (\d+), switches: (\d+)', stderr).groups())
        assert steps == 9 and contended == 6 and switches == (5 if variant == 'reply' else 4)
        deferrals = re.search(r'deferred_slices: (\{.*?\}), pinned_yields:', stderr)[1]
        expected_deferrals = '{[Native("SORTWITH"), SortComparator]: 1}' if expected else '{}'
        assert deferrals == expected_deferrals
        assert 'pinned_yields: {}' in stderr and 'unbalanced: 0' in stderr
        if expected:
            assert 'the incoming sort must not inherit the outgoing request' in stderr
            assert 'left: 1' in stderr and 'right: 0' in stderr
        rows.append(dict(action=name, status=descriptor['status'], selected=selected,
                         steps=steps, contended=contended, switches=switches,
                         sorting_deferrals=1 if expected else 0,
                         total_deferrals=1 if expected else 0, pinned_yields=0, unbalanced=0))
for action, status in [('focused', 0), ('fmt', 1), ('fmt-final', 0), ('focused-final', 0), ('clippy', 0)]:
    descriptor = json.loads((records / f'{action}.json').read_text())
    assert descriptor['status'] == status
    if action.startswith('focused'):
        stdout = (records / f'{action}.stdout').read_text()
        assert 'running 7 tests' in stdout and '7 passed; 0 failed' in stdout

source = 'rust/crates/rexx-exec/src/scheduler.rs'
baseline = subprocess.check_output(['git', 'show', '7662a51a3:' + source], cwd=root)
assert (root / source).read_bytes() == baseline
delta = subprocess.check_output(['git', 'diff', 'a9bf23ed4', 'HEAD', '--', 'rust'], cwd=root)
(records / 'final-source.diff').write_bytes(delta)
paths = subprocess.check_output(['git', 'diff', '--name-only', 'a9bf23ed4', 'HEAD', '--', 'rust'], cwd=root, text=True).splitlines()
assert paths == ['rust/crates/rexx-exec/src/scheduler/tests/mutants.rs']
inventory = 'docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv'
before = subprocess.check_output(['git', 'show', 'a9bf23ed4:' + inventory], cwd=root, text=True).splitlines()
after = (root / inventory).read_text().splitlines()
assert len(before) == len(after)
changed = []
for old, new in zip(before, after):
    if old != new:
        old_fields, new_fields = old.split('\t'), new.split('\t')
        assert old_fields[0] == '2026-10-07-unjudged-scheduler-mutants.md'
        assert [i for i, pair in enumerate(zip(old_fields, new_fields)) if pair[0] != pair[1]] == [3, 5]
        assert new_fields[3] == new_fields[5] == 'closed:3c76dbeeb'
        changed.append(old_fields[0])
assert len(changed) == 1
summary = dict(named_runs=rows, focused_runs_passed=7, final_fmt_status=0, clippy_status=0,
               initial_fmt_status=1, scheduler_matches_pre_mutation=True,
               scheduler_sha256=hashlib.sha256(baseline).hexdigest(),
               retained_rust_paths=paths, inventory_only_columns_changed=[4, 6],
               source_table_refresh='Unnecessary: only cfg(test) module lines changed; no production sites moved.')
(records / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))
