import json, pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[6]
records = pathlib.Path(__file__).parent
inventory = 'docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv'
original = subprocess.check_output(['git', 'show', 'b300b9c25:' + inventory], cwd=root).decode().splitlines()
current = (root / inventory).read_text().splitlines()
assert len(original) == len(current)
closed = dict(line.split('\t') for line in (records / 'closed.tsv').read_text().splitlines())
changed = []
for before, after in zip(original, current):
    if before == after:
        continue
    left, right = before.split('\t'), after.split('\t')
    assert left[0] in closed and left[:3] == right[:3] and left[4] == right[4]
    assert right[3] == right[5] == closed[left[0]]
    changed.append(left[0])
assert set(changed) == set(closed)
for name in closed:
    assert (root / '.superpowers/sdd/queued' / name).read_text().startswith('**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**')
for name in ['2026-10-18-sysfiletree-tree-directory-flaky.md', '2026-10-29-suite-fixture-stderr-refusal-load-flake.md']:
    assert not (root / '.superpowers/sdd/queued' / name).read_text().startswith('**CLOSED')
program = 'lang/command_redirected_target_waits_elsewhere.rex'
assert program in (root / 'rust/corpus/phase-9.txt').read_text().splitlines()
assert program not in (root / 'rust/corpus/sim-gate.tsv').read_text()
assert 'phase-9.txt' in (root / 'rust/crates/rexx-exec/tests/collect_stress.rs').read_text()
result = dict(status='passed', changed_inventory_rows=changed, preserved_columns=[1,2,3,5], unresolved=['SysFileTree', 'stderr-refusal fixture'], redirected_phase9_and_collector_inclusion=True)
(records / 'disposition-audit.json').write_text(json.dumps(result, indent=2) + '\n')
print('Selected closure inventory/header audit and redirected witness inclusion: passed')
