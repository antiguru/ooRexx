import hashlib
import json
from pathlib import Path
import shutil
import subprocess

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-test-repair'
base = '47c5a5ea4fbc9518b41ed640d25c3baa86212269'
pure = '5f281e8266cc5613382be9f54cee27e92682aa26'
source = '794dbc6297a3a4471cc2a50e56c33ac65c2a514a'

def git(*args):
    return subprocess.check_output(['git', *args], cwd=root).decode().strip()

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def names(path):
    return sorted(line for line in path.read_text().splitlines() if line.endswith(': test'))

assert git('rev-parse', 'HEAD') == source
assert not git('diff', '--name-only')
assert not git('diff', '--cached', '--name-only')
rust_changed = git('diff', '--name-only', base, source, '--', 'rust').splitlines()
expected = sorted([
    'rust/crates/rexx-exec/tests/concurrency_tests.rs',
    'rust/crates/rexx-exec/tests/dispatch_seam.rs',
    'rust/crates/rexx-exec/tests/group_runs/mod.rs',
    'rust/crates/rexx-exec/tests/group_runs/whole_groups.rs',
])
assert sorted(rust_changed) == expected
before = names(records / 'concurrency-before.list')
after = names(records / 'checks/final-list.stdout')
assert before == after and len(after) == 63
trace = root / 'ootest/ooRexx/base/keyword/TRACE_TraceObject.testGroup'
pristine = root / 'ootest/.svn/pristine/5a/5a1f4c21ab6e271b9617eec15b0754fb10066a7a.svn-base'
assert trace.read_bytes() == pristine.read_bytes()
assert digest(trace) == '98e4999d07fca459f1d2a13793a5e5d15b5b2dba82460c615e410d0fdcd0d9b3'
pin = Path('/tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx')
assert digest(pin) == 'd9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145'
calibration = 'rust/crates/rexx-exec/tests/group_runs/whole_groups/sim_gate/calibration.rs'
assert (root / calibration).read_bytes() == subprocess.check_output(['git', 'show', f'{base}:{calibration}'], cwd=root)
checks = {}
for descriptor in sorted((records / 'checks').glob('*.json')):
    value = json.loads(descriptor.read_text())
    assert value['status'] == 0
    checks[descriptor.stem] = {
        'source': pure if descriptor.stem == 'pure-move-list' else source,
        'status': value['status'],
        'elapsed': value['elapsed'],
        'sha256': {suffix: digest(descriptor.with_suffix(suffix)) for suffix in ['.json', '.stdout', '.stderr']},
    }
checks['format-edit']['source'] = 'uncommitted assertion edits over ' + pure
red = {}
for stem in ['debug-118-dispatch_seam', 'debug-154-concurrency_tests']:
    for suffix in ['.json', '.stdout', '.stderr']:
        path = Path('/tmp/codex-p9d-t88-final-gates-resume') / (stem + suffix)
        red[str(path)] = {'bytes': path.stat().st_size, 'sha256': digest(path)}
for name in ['dispatch-seam', 'traceobject-population']:
    shutil.copyfile(f'/tmp/codex-p9d-t88-{name}-diagnosis.md', records / f'{name}-diagnosis.md')
proof = {
    'base': base,
    'pure_move': pure,
    'source': source,
    'rust_changed_exactly': rust_changed,
    'production_src_cargo_and_other_rust_inputs_unchanged': True,
    'source_sha256': {path: digest(root / path) for path in expected},
    'concurrency_test_names_equal': True,
    'concurrency_tests': len(after),
    'unchanged_calibration_sha256': digest(root / calibration),
    'protected_fixture_sha256': digest(trace),
    'protected_fixture_equals_svn_pristine': True,
    'immutable_normal_pin': str(pin),
    'immutable_normal_pin_sha256': digest(pin),
    'checks': checks,
    'retained_red': red,
}
(records / 'final-proof.json').write_text(json.dumps(proof, indent=2) + '\n')
shutil.copyfile(__file__, records / 'proof.py')
print(json.dumps({key: proof[key] for key in ['source', 'rust_changed_exactly', 'concurrency_tests', 'immutable_normal_pin_sha256']}, indent=2))
