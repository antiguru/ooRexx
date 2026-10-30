import hashlib, json, pathlib, subprocess
records = pathlib.Path(__file__).parent
root = records.resolve().parents[6]
def git(*args):
    return subprocess.check_output(['git', *args], cwd=root).decode()
base, source = '9ca84d4e2', 'd7581cc4a'
changed = git('diff', '--name-only', base, source).splitlines()
assert set(changed) == {
 'rust/corpus/lang/command_redirected_target_waits_elsewhere.rex',
 'rust/crates/rexx-exec/tests/rexxutil_process.rs',
 'rust/crates/rexx-exec/tests/rexxutil_process/owned_child.py',
 'rust/crates/rexx-exec/tests/timing_witnesses.rs',
 'rust/crates/rexx-parse/tests/sourceline_oracle/command_redirected_target_waits_elsewhere.txt'}
assert not git('diff', base, source, '--', 'rust/crates/rexx-exec/src', 'rust/crates/rexx-parse/src')
inventory = 'docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv'
before = git('show', base + ':' + inventory).splitlines()
after = (root / inventory).read_text().splitlines()
expected = {'2026-10-21-redirected-target-timed-line-flake.md', '2026-10-26-redirected-target-waits-timing-flake.md', '2026-10-27-reaped-child-test-load-flake.md'}
seen = set()
for left, right in zip(before, after):
    if left == right:
        continue
    left, right = left.split('\t'), right.split('\t')
    assert left[:3] == right[:3] and left[4] == right[4]
    assert right[0] in expected and right[3] == right[5] == 'closed:' + source
    seen.add(right[0])
assert seen == expected and len(before) == len(after)
for name in expected:
    assert (root / '.superpowers/sdd/queued' / name).read_text().startswith('**CLOSED 2026-10-30 by Phase 9d Task 8.8c (' + source + ').**')
for name in ['2026-10-18-sysfiletree-tree-directory-flaky.md', '2026-10-29-suite-fixture-stderr-refusal-load-flake.md']:
    assert not (root / '.superpowers/sdd/queued' / name).read_text().startswith('**CLOSED')
pin = pathlib.Path('/tmp/codex-p9d-t88-pins/8.8c-rexx')
metadata = json.loads(pin.with_name(pin.name + '.json').read_text())
with pin.open('rb') as binary:
    digest = hashlib.file_digest(binary, 'sha256').hexdigest()
assert digest == metadata['sha256'] and metadata['source_revision'].startswith('949a7806b')
(records / 'audit.json').write_text(json.dumps(dict(status='passed',base=base,source=source,changed_source_files=changed,production_source_changed=False,updated_closures=sorted(seen),runtime_pin=str(pin),runtime_pin_sha256=digest),indent=2)+'\n')
print('Scoped files, closure metadata, open diagnoses and unchanged runtime pin: passed')
