"""Retain the exact Task 8.7c to Task 8.8 suite-row delta."""

import csv
import hashlib
import json
from pathlib import Path

old_root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r4/suite')
new_root = Path('/tmp/codex-p9d-t88-final-gates-final/suite')


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
_, tables = load(new_root / 'table.tsv', ['group', 'side', 'expected', 'started', 'passed', 'failed', 'blocked', 'blocked at load', 'ended before a test', 'iterations', 'ended normally', 'cap', 'tests ran'])
_, verdicts = load(new_root / 'verdicts.tsv', ['group', 'verdict', 'crate', 'oracle denominator'])
load(new_root / 'oracle-differences.tsv', ['group', 'test', 'oracle', 'crate'])
groups = {group for group, _ in after}
assert len({(r['group'], r['side']) for r in tables}) == len(tables)
assert {(r['group'], r['side']) for r in tables} == {(g, s) for g in groups for s in ['oracle', 'crate']}
assert len({r['group'] for r in verdicts}) == len(verdicts) == len(groups)
assert {r['group'] for r in verdicts} == groups
(new_root / 'delta-summary.json').write_text(json.dumps({'old_tests': len(before), 'new_tests': len(after), 'groups': len(groups), 'changed_rows': len(changes), 'added': len(after.keys() - before.keys()), 'removed': len(before.keys() - after.keys())}, indent=2) + '\n')
assert before.keys() == after.keys(), 'Population changed: reconcile retained delta'
assert groups == {group for group, _ in before} and len(groups) == 402

# A separate fixed join preserves the earlier resumed attempt's licences and outcomes.
prior_root = Path('/tmp/codex-p9d-t88-final-gates-resume/suite')
prior_lines, prior_rows = load(prior_root / 'tests.tsv', fields)
assert '# groups all' in prior_lines
prior = {(row['group'], row['test']): row for row in prior_rows}
assert prior_rows and len(prior) == len(prior_rows)
prior_changes = []
for group, test in sorted(prior.keys() | after.keys()):
    a, b = prior.get((group, test)), after.get((group, test))
    if a != b:
        prior_changes.append([group, test, 'added' if a is None else 'removed' if b is None else 'changed', *[(a or {}).get(k, '<absent>') for k in fields[2:]], *[(b or {}).get(k, '<absent>') for k in fields[2:]]])
with (new_root / 'delta-vs-attempt-2.tsv').open('x') as output:
    writer = csv.writer(output, delimiter='\t', lineterminator='\n')
    writer.writerow(['group', 'test', 'change', 'old row', 'old oracle', 'old crate', 'new row', 'new oracle', 'new crate'])
    writer.writerows(prior_changes)
(new_root / 'attempt-2-delta-summary.json').write_text(json.dumps({'old_tests': len(prior), 'new_tests': len(after), 'groups': len(groups), 'changed_rows': len(prior_changes), 'added': len(after.keys() - prior.keys()), 'removed': len(prior.keys() - after.keys())}, indent=2) + '\n')
assert prior.keys() == after.keys(), 'Population changed against retained attempt 2'
paths = [old_root / 'tests.tsv', prior_root / 'tests.tsv', new_root / 'tests.tsv', new_root / 'table.tsv', new_root / 'verdicts.tsv', new_root / 'oracle-differences.tsv']
(new_root / 'delta-input-sha256.json').write_text(json.dumps({str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}, indent=2) + '\n')
