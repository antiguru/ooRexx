import hashlib
import json
from pathlib import Path
import shutil
import subprocess

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation'
base = '4714f6521f54e583e86c579bd803a80067a6939d'
pure = '0d8d1b29cd33846a4d93e658afaca2d25397e745'
red = '0333f95b1ca96e576d1216986ead3803e6f06c2a'
source = '6f2c1346eb1855c4d84301dab7fed3a4c844f0bf'

def git(*args):
    return subprocess.check_output(['git', *args], cwd=root).decode().strip()

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def names(path):
    return {line for line in path.read_text().splitlines() if line.endswith(': test')}

assert git('rev-parse', 'HEAD') == source
assert not git('diff', '--name-only')
assert not git('diff', '--cached', '--name-only')
changed = git('diff', '--name-only', base, source, '--', 'rust').splitlines()
expected = [
    'rust/crates/rexx-exec/tests/suite.rs',
    'rust/crates/rexx-exec/tests/suite/diagnostics.rs',
    'rust/crates/rexx-exec/tests/suite/fixtures/temporary_directory/base/rexxutil/SysFileTree.testGroup',
    'rust/crates/rexx-exec/tests/suite/fixtures/tests.rs',
    'rust/crates/rexx-exec/tests/suite/launch/mod.rs',
]
assert sorted(changed) == sorted(expected)
before = names(records / 'suite-before.list')
assert names(records / 'checks/pure-move-list.stdout') == before
after = names(records / 'checks/final-list.stdout')
added = {
    'fixtures::tests::sysfiletree_launches_use_distinct_populated_child_temporary_directories: test',
    'fixtures::tests::sysfiletree_temporary_directory_selection_is_exact: test',
}
assert after - before == added and not before - after
original = subprocess.check_output(['git', 'show', f'{base}:rust/crates/rexx-exec/tests/suite.rs'], cwd=root)
start = original.index(b'/// `group` of `dir` on the oracle, once,')
end = original.index(b'/// One table row.', start)
extracted = subprocess.check_output(['git', 'show', f'{pure}:rust/crates/rexx-exec/tests/suite/launch/mod.rs'], cwd=root)
assert extracted.endswith(original[start:end])
protected = root / 'ootest/ooRexx/base/rexxutil/SysFileTree.testGroup'
assert sha(protected) == json.loads((records / 'pure-move.json').read_text())['protected_sha256']
frozen = Path('/tmp/codex-p9d-t88-final-gates-resume/suite-static-inputs-before.json')
entries = json.loads(frozen.read_text())['entries']
protected_files = {path: value for path, value in entries.items() if path.startswith(str(root / 'ootest') + '/') and value['kind'] == 'file'}
for path, value in protected_files.items():
    assert sha(Path(path)) == value['sha256'], path
pin = Path('/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx')
assert sha(pin) == 'd9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145'
checks = {}
for descriptor in sorted((records / 'checks').glob('*.json')):
    data = json.loads(descriptor.read_text())
    name = descriptor.stem
    status = 101 if name == 'red-child-tmpdir' else 1 if name == 'green-child-tmpdir' else 0
    assert data['status'] == status, name
    commit = pure if name.startswith('pure-move') else red if name == 'red-child-tmpdir' else source
    if name.startswith('format-'):
        commit = 'uncommitted formatting before ' + (red if name == 'format-regression' else source)
    checks[name] = {'source': commit, **data}
table = (records / 'original-group/table.tsv').read_text().splitlines()[2:]
for line in table:
    fields = line.split('\t')
    assert fields[2:5] == ['26'] * 3
    assert fields[5:9] == ['', '', '-', '-']
    assert fields[9:11] == ['1', 'yes'] and fields[12] == '26'
proof = {
    'base': base, 'pure_move': pure, 'regression': red, 'source': source,
    'changed_rust_files': changed,
    'production_cargo_corpus_licence_inputs_unchanged': True,
    'moved_bodies_byte_identical': True,
    'source_sha256': {path: sha(root / path) for path in expected},
    'test_population': {'before': len(before), 'pure_move': len(before), 'after': len(after), 'added': sorted(added), 'removed': []},
    'protected_group_sha256': sha(protected),
    'protected_ootest_files_verified': len(protected_files),
    'protected_reference_manifest': str(frozen),
    'protected_reference_manifest_sha256': sha(frozen),
    'pin': str(pin), 'pin_sha256': sha(pin),
    'original_group': {'oracle': '26/26', 'crate': '26/26', 'attempts_per_side': 1, 'failed': 0, 'blocked': 0},
    'checks': checks,
}
(records / 'final-proof.json').write_text(json.dumps(proof, indent=2) + '\n')
shutil.copyfile(__file__, records / 'proof.py')
manifest = {str(path.relative_to(records)): {'bytes': path.stat().st_size, 'sha256': sha(path)} for path in sorted(records.rglob('*')) if path.is_file() and path.name != 'manifest.json'}
(records / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps({key: proof[key] for key in ['source', 'test_population', 'protected_ootest_files_verified', 'original_group']}, indent=2))
