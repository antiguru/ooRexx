"""Task 8.8 draft: capture Cargo dispatch, then run one default target per call."""

import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import tomllib

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RUST = ROOT / 'rust'
OUT = Path('/tmp/codex-p9d-t88-final-gates')
TARGET = '/tmp/codex-p9d-t88-target'
BASE = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/prerequisites'
RELEASE_PIN_PROVENANCE = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix-r1/release-pin.json'
TEST_VARIANT_AUTHORIZATION = OUT / 'release-variant-diagnosis/root-authorization.json'
TEST_VARIANT_AUTHORIZATION_SHA256 = 'e2f37ea8d96b6fe16f94254034f1f30262b39424fc3bf6ec1e5973bbd9758188'
ADDED_TARGETS = {('rexx-exec', 'semantic_boundaries', ('test',), ('bin',)):
                 'Task 8.8f adds semantic boundary integration coverage after the prerequisite enumeration'}
RENAMED_TESTS = {
    ('rexx-exec', 'rexx_exec', ('lib',), ('lib',)): {
        'run::tests::an_executable_rebuilt_under_the_nil_scope_answers': 'run::tests::an_executable_retained_under_the_nil_scope_answers',
        'scheduler::tests::pool::a_spawn_failure_leaves_ready_activities_to_the_next_baton_holder': 'scheduler::tests::pool::a_disabled_pool_leaves_ready_activities_to_the_next_baton_holder',
    },
    ('rexx-exec', 'suite', ('test',), ('bin',)): {
        'fixtures::tests::the_suite_revision_is_the_recorded_one': 'fixtures::tests::the_recorded_suite_revision_is_a_revision_number',
    },
    ('rexx-exec', 'concurrency_tests', ('test',), ('bin',)): {
        'group_runs::whole_groups::sim_gate::calibration': 'group_runs::whole_groups::sim_gate::calibration::calibration',
    },
}
CONFIG_PATH = Path('/tmp/codex-p9d-t88-final-gate-config.json')
CONFIG = json.loads(CONFIG_PATH.read_text())
assert set(CONFIG) == {'source_commit', 'profile_completion_record', 'release_binary', 'release_binary_sha256'}
PIN = CONFIG['source_commit']
ENV = {k: v for k, v in os.environ.items() if not k.startswith('REXX_')}
for key_name in list(ENV):
    if key_name in {'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS', 'CARGO_BUILD_RUSTFLAGS', 'CARGO_BUILD_RUSTDOCFLAGS', 'CARGO_BUILD_TARGET'} or key_name.startswith('CARGO_PROFILE_') or (key_name.startswith('CARGO_TARGET_') and key_name.endswith(('_RUNNER', '_RUSTFLAGS', '_RUSTDOCFLAGS', '_LINKER'))):
        ENV.pop(key_name)
ENV.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR=TARGET, CARGO_BUILD_JOBS='4')
ENV.update(REXX_CORPUS_GATE='1', REXX_SUITE_DIAGNOSTICS=str(OUT / 'debug-diagnostics'))
PUBLIC_ENV = ('CARGO', 'CARGO_HOME', 'CARGO_MANIFEST_DIR', 'CARGO_MANIFEST_PATH',
              'CARGO_PKG_AUTHORS', 'CARGO_PKG_DESCRIPTION', 'CARGO_PKG_HOMEPAGE',
              'CARGO_PKG_LICENSE', 'CARGO_PKG_LICENSE_FILE', 'CARGO_PKG_NAME',
              'CARGO_PKG_README', 'CARGO_PKG_REPOSITORY', 'CARGO_PKG_RUST_VERSION',
              'CARGO_PKG_VERSION', 'CARGO_PKG_VERSION_MAJOR', 'CARGO_PKG_VERSION_MINOR',
              'CARGO_PKG_VERSION_PATCH', 'CARGO_PKG_VERSION_PRE', 'CARGO_CRATE_NAME',
              'CARGO_BIN_NAME', 'CARGO_TARGET_TMPDIR', 'OUT_DIR', 'LD_LIBRARY_PATH',
              'DYLD_FALLBACK_LIBRARY_PATH', 'CARGO_INCREMENTAL', 'CARGO_TARGET_DIR',
              'CARGO_BUILD_JOBS', 'REXX_CORPUS_GATE', 'REXX_SUITE_DIAGNOSTICS')


def git_output(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, timeout=10)


def check_source():
    assert re.fullmatch('[0-9a-f]{40}', PIN), 'Supply the final full commit SHA'
    assert git_output('rev-parse', 'HEAD').decode().strip() == PIN
    changed = [os.fsdecode(path) for path in git_output('diff', '--name-only', '-z', PIN).split(b'\0') if path]
    assert not changed, f'Tracked inputs/evidence changed during blanket freeze: {changed}'
    assert not git_output('ls-files', '--others', '--exclude-standard', '-z', '--', 'rust'), 'Uncommitted Rust inputs'


def guard():
    OUT.mkdir(exist_ok=True)
    frozen = json.loads((OUT / 'input-freeze.json').read_text())
    assert frozen['source_commit'] == PIN
    assert frozen['config_sha256'] == digest(CONFIG_PATH), 'Final gate config changed'
    assert frozen['authorized_evidence_outputs'] == []
    check_source()
    assert hashlib.sha256(git_output('ls-files', '--stage', '-z')).hexdigest() == frozen['index_staging_sha256'], 'Index staging changed'
    assert digest(CONFIG['release_binary']) == CONFIG['release_binary_sha256'], 'Final release pin changed'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def release_provenance():
    provenance = json.loads(RELEASE_PIN_PROVENANCE.read_text())
    assert provenance['pin'] == CONFIG['release_binary']
    assert provenance['sha256'] == provenance['build_sha256'] == CONFIG['release_binary_sha256']
    runtime_tree = git_output('rev-parse', provenance['source_commit'] + ':rust').decode().strip()
    assert runtime_tree == git_output('rev-parse', PIN + ':rust').decode().strip(), 'Pinned runtime Rust tree differs from gate source'
    for path, expected in provenance['source_hashes'].items():
        assert digest(ROOT / path) == expected, f'Pinned source hash changed: {path}'
    build_record = RELEASE_PIN_PROVENANCE.parent / provenance['build_record']
    assert digest(build_record) == provenance['build_record_sha256'], 'Pin build provenance changed'
    return {'path': str(RELEASE_PIN_PROVENANCE), 'sha256': digest(RELEASE_PIN_PROVENANCE),
            'runtime_source_commit': provenance['source_commit'], 'rust_tree': runtime_tree}


def release_test_variant():
    assert digest(TEST_VARIANT_AUTHORIZATION) == TEST_VARIANT_AUTHORIZATION_SHA256, 'Test variant authorization changed'
    authorization = json.loads(TEST_VARIANT_AUTHORIZATION.read_text())
    assert authorization['source_commit'] == PIN and authorization['source_index_freeze_unchanged'] is True
    normal = release_provenance()
    assert authorization['runtime_source_commit'] == normal['runtime_source_commit']
    assert authorization['normal_pin_sha256'] == CONFIG['release_binary_sha256'] == digest(CONFIG['release_binary'])
    proof_path = Path(authorization['one_variable_proof'])
    assert proof_path == OUT / 'release-variant-diagnosis/one-variable-result.json'
    assert digest(proof_path) == authorization['one_variable_proof_sha256'], 'One-variable proof changed'
    proof = json.loads(proof_path.read_text())
    assert proof['source_commit'] == PIN and proof['command_status'] == 0
    assert proof['matches_test_variant'] is True and proof['immutable_normal_unchanged'] is True
    assert proof['immutable_normal_sha256'] == authorization['normal_pin_sha256']
    assert proof['actual_cli_sha256'] == proof['test_built_sha256'] == authorization['test_cli_sha256']
    assert proof['rustix_artifact']['features'] == authorization['test_rustix_features']
    assert proof['cli_artifact']['features'] == [] and not proof['cli_artifact']['profile']['test']
    return {'source_commit': PIN, 'runtime_source_commit': normal['runtime_source_commit'], 'rust_tree': normal['rust_tree'],
            'authorization_path': str(TEST_VARIANT_AUTHORIZATION), 'authorization_sha256': TEST_VARIANT_AUTHORIZATION_SHA256,
            'proof_path': str(proof_path), 'proof_sha256': authorization['one_variable_proof_sha256'],
            'test_cli_sha256': authorization['test_cli_sha256'], 'normal_pin_sha256': authorization['normal_pin_sha256'],
            'cli_profile': proof['cli_artifact']['profile'], 'rustix_features': authorization['test_rustix_features'],
            'rustix_package_id': proof['rustix_artifact']['package_id'],
            'rustix_manifest_path': proof['rustix_artifact']['manifest_path'], 'rustix_profile': proof['rustix_artifact']['profile']}


def release_artifacts(log_path, required_tests):
    selected = {}
    rustix = {}
    for line in log_path.read_text(errors='replace').splitlines():
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(item, dict) or item.get('reason') != 'compiler-artifact':
            continue
        if item['target']['name'] == 'rustix' and item['target']['kind'] == ['lib']:
            rustix[tuple(item['filenames'])] = item
        if not item.get('executable'):
            continue
        if Path(item['manifest_path']) != RUST / 'crates/rexx-exec/Cargo.toml':
            continue
        selected[item['executable']] = item
    cli = [item for item in selected.values() if item['target']['kind'] == ['bin'] and item['target']['name'] == 'rexx' and not item['profile']['test']]
    assert len(cli) == 1, 'Fresh Cargo CLI artifact missing or ambiguous'
    cli = cli[0]
    assert Path(cli['executable']).resolve() == (Path(TARGET) / 'release/rexx').resolve()
    assert Path(cli['target']['src_path']) == RUST / 'crates/rexx-exec/src/bin/rexx.rs'
    assert not cli['features'] and not cli['profile']['debug_assertions'] and not cli['profile']['overflow_checks']
    variant = release_test_variant()
    assert cli['profile'] == variant['cli_profile'], 'Cargo-test CLI profile changed'
    cli_sha256 = digest(cli['executable'])
    assert cli_sha256 == variant['test_cli_sha256'], 'Fresh Cargo CLI differs from authorized test variant'
    assert len(rustix) == 1, 'Fresh resolved rustix artifact missing or ambiguous'
    rustix = next(iter(rustix.values()))
    assert rustix['package_id'] == variant['rustix_package_id'] and rustix['manifest_path'] == variant['rustix_manifest_path']
    assert rustix['features'] == variant['rustix_features'] and rustix['profile'] == variant['rustix_profile'], 'Resolved rustix test features/profile changed'
    tests = {item['target']['name']: item for item in selected.values() if item['target']['kind'] == ['test'] and item['profile']['test']}
    assert set(tests) == set(required_tests), 'Release test artifact population differs'
    for item in tests.values():
        assert not item['features'] and not item['profile']['debug_assertions'] and not item['profile']['overflow_checks']
        assert item['profile'] == {**variant['cli_profile'], 'test': True}, 'Release harness profile changed'
        assert Path(item['target']['src_path']) == RUST / 'crates/rexx-exec/tests' / (item['target']['name'] + '.rs')
    return {'source_commit': PIN, 'pin_provenance': release_provenance(), 'test_variant': variant,
            'normal_pin': {'path': CONFIG['release_binary'], 'sha256': digest(CONFIG['release_binary'])},
            'cli': {'executable': cli['executable'], 'sha256': cli_sha256, 'profile': cli['profile']},
            'rustix': {'package_id': rustix['package_id'], 'features': rustix['features'], 'profile': rustix['profile'], 'filenames': rustix['filenames']},
            'tests': {name: {'executable': item['executable'], 'sha256': digest(item['executable']), 'profile': item['profile']} for name, item in tests.items()}}


ACTIVE = None


def deadline(signum, frame):
    if ACTIVE is not None:
        proc, name, record, started = ACTIVE
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        record.update(timed_out=True, action_deadline=True, elapsed_seconds=time.monotonic() - started)
        try:
            record['returncode'] = proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            record['returncode'] = None
        (OUT / (name + '.json')).write_text(json.dumps(record, indent=2) + '\n')
    else:
        (OUT / ('action-' + sys.argv[1] + '-timeout.json')).write_text(json.dumps({'source_commit': PIN, 'timed_out': True}) + '\n')
    os._exit(124)


def run(name, args, cwd, env, bound):
    global ACTIVE
    guard()
    record = {'source_commit': PIN, 'command': args, 'cwd': str(cwd),
              'bound_seconds': bound, 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'env': {k: env[k] for k in PUBLIC_ENV if k in env}}
    start = time.monotonic()
    with (OUT / (name + '.stdout')).open('xb') as stdout, (OUT / (name + '.stderr')).open('xb') as stderr:
        proc = subprocess.Popen(args, cwd=cwd, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
        ACTIVE = (proc, name, record, start)
        record['pid'] = proc.pid
        try:
            record['returncode'] = proc.wait(timeout=bound)
            record['timed_out'] = False
        except subprocess.TimeoutExpired:
            record['timed_out'] = True
            try:
                os.killpg(proc.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                record['returncode'] = proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                record['returncode'] = proc.wait(timeout=2)
        ACTIVE = None
    record['elapsed_seconds'] = time.monotonic() - start
    (OUT / (name + '.json')).write_text(json.dumps(record, indent=2) + '\n')
    try:
        guard()
    except Exception as error:
        record['post_run_freeze_failure'] = str(error)
        (OUT / (name + '.json')).write_text(json.dumps(record, indent=2) + '\n')
        raise
    print(json.dumps(record), flush=True)
    return record


def key(item):
    manifest = Path(item['manifest_path'])
    return (manifest.parent.name, item['target']['name'], tuple(item['target']['kind']), tuple(item['target']['crate_types']))


def artifacts():
    return json.loads((OUT / 'artifacts.json').read_text())


def reconcile(items):
    old_keys = {key(item) for item in json.loads((BASE / 'targets.json').read_text())}
    new_keys = {key(item) for item in items}
    assert items and len(new_keys) == len(items), 'Empty or duplicate default target population'
    report = {'historical_count': len(old_keys), 'fresh_count': len(new_keys),
              'added': [{'key': item, 'ruling': ADDED_TARGETS.get(item)} for item in sorted(new_keys - old_keys)],
              'removed': sorted(old_keys - new_keys)}
    assert not report['removed'], f'Historical default targets removed: {report["removed"]}'
    assert new_keys - old_keys == ADDED_TARGETS.keys(), 'Added targets differ from explicit reconciliation'
    return report


def capture_path(executable):
    return OUT / 'dispatch' / (hashlib.sha256(os.fsencode(executable)).hexdigest() + '.json')


mode = sys.argv[1]
signal.signal(signal.SIGALRM, deadline)
signal.alarm(535)
if mode == 'freeze':
    OUT.mkdir(exist_ok=True)
    completion_path = Path(CONFIG['profile_completion_record'])
    completion = json.loads(completion_path.read_text())
    assert completion['source_commit'] == PIN and completion['profiling_complete'] is True
    assert completion['active_profile_commands'] == []
    check_source()
    assert not git_output('diff', '--cached', '--name-only', '-z'), 'Index must begin unstaged'
    assert digest(CONFIG['release_binary']) == CONFIG['release_binary_sha256'], 'Final release pin mismatch'
    provenance = release_provenance()
    with (OUT / 'input-freeze.json').open('x') as output:
        json.dump({'source_commit': PIN, 'authorized_evidence_outputs': [],
                   'config_sha256': digest(CONFIG_PATH),
                   'release_pin_provenance': provenance,
                   'profiling_completion_record': str(completion_path), 'profiling_completion': completion,
                   'index_staging_sha256': hashlib.sha256(git_output('ls-files', '--stage', '-z')).hexdigest(),
                   }, output, indent=2)
    sys.exit(0)
guard()
if mode == 'reconcile-release-build':
    original_path = OUT / 'release-build.json'
    original = json.loads(original_path.read_text())
    failure_path = OUT / 'release-build-artifact-mismatch.json'
    failure = json.loads(failure_path.read_text())
    assert original['source_commit'] == PIN and original['returncode'] == 0 and not original['timed_out']
    assert failure['source_commit'] == PIN and failure['cargo_child_status'] == 0 and failure['driver_status'] == 1
    assert original['command'] == ['python3', 'scripts/test_preflight.py', '--', 'cargo', 'test', '--release', '--locked', '-j4', '-p', 'rexx-exec', '--test', 'corpus', '--test', 'suite', '--no-run', '--message-format=json']
    verified = release_artifacts(OUT / 'release-build.stdout', {'corpus', 'suite'})
    verified['reconciliation'] = {'metadata_only': True, 'original_cargo_record': str(original_path),
                                  'original_cargo_record_sha256': digest(original_path),
                                  'original_cargo_stdout_sha256': digest(OUT / 'release-build.stdout'),
                                  'original_cargo_stderr_sha256': digest(OUT / 'release-build.stderr'),
                                  'first_failure_record': str(failure_path), 'first_failure_sha256': digest(failure_path),
                                  'original_cargo_status': 0, 'original_driver_status': 1}
    guard()
    with (OUT / 'release-artifacts-verified.json').open('x') as output:
        json.dump(verified, output, indent=2)
        output.write('\n')
    record = {'source_commit': PIN, 'action': mode, 'metadata_only': True, 'returncode': 0,
              'command': ['memcap', '8G', 'python3', '/tmp/codex-p9d-t88-final-gate-draft.py', mode],
              'no_cargo_command_executed': True, 'normal_pin_sha256': verified['normal_pin']['sha256'],
              'test_cli_sha256': verified['cli']['sha256'], 'verified_marker_sha256': digest(OUT / 'release-artifacts-verified.json'),
              'authorization_sha256': verified['test_variant']['authorization_sha256'], 'proof_sha256': verified['test_variant']['proof_sha256']}
    with (OUT / 'release-build-reconciliation.json').open('x') as output:
        json.dump(record, output, indent=2)
        output.write('\n')
    with (OUT / 'release-build-reconciliation.stdout').open('x') as output:
        output.write(json.dumps(record) + '\n')
    (OUT / 'release-build-reconciliation.stderr').open('x').close()
    guard()
    print(json.dumps(record), flush=True)
elif mode in {'preflight', 'fmt', 'clippy', 'release-build', 'release-corpus', 'release-suite', 'suite-delta', 'sysfiletree-diagnostic', 'pinning'}:
    env = {k: v for k, v in ENV.items() if not k.startswith('REXX_')}
    commands = {
        'preflight': ['python3', 'scripts/test_preflight.py'],
        'fmt': ['cargo', 'fmt', '--all', '--check'],
        'clippy': ['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '-j4', '--', '-D', 'warnings'],
        'release-build': ['python3', 'scripts/test_preflight.py', '--', 'cargo', 'test', '--release', '--locked', '-j4', '-p', 'rexx-exec', '--test', 'corpus', '--test', 'suite', '--no-run', '--message-format=json'],
        'release-corpus': ['cargo', 'test', '--release', '--locked', '-j4', '-p', 'rexx-exec', '--test', 'corpus', '--message-format=json', '--', '--test-threads=1', '--nocapture'],
        'release-suite': ['cargo', 'test', '--release', '--locked', '-j4', '-p', 'rexx-exec', '--test', 'suite', 'l3_core_suite', '--message-format=json', '--', '--exact', '--test-threads=1', '--nocapture'],
        'suite-delta': ['python3', '/tmp/codex-p9d-t88-final-suite-delta.py'],
        'sysfiletree-diagnostic': ['cargo', 'test', '--release', '--locked', '-j4', '-p', 'rexx-exec', '--test', 'suite', 'fixtures::tests::sysfiletree_group_retains_assertion_and_metadata_context', '--', '--exact', '--ignored', '--test-threads=1', '--nocapture'],
        'pinning': ['cargo', 'test', '--locked', '-j4', '-p', 'rexx-exec', '--lib', '--features', 'pinning', 'scheduler::tests::mutants::', '--', '--test-threads=1', '--nocapture'],
    }
    if mode == 'release-corpus':
        env['REXX_CORPUS_GATE'] = '1'
    elif mode == 'release-suite':
        env.update(REXX_SUITE_GATE='1', REXX_SUITE_TABLE=str(OUT / 'suite/table.tsv'), REXX_SUITE_DIAGNOSTICS=str(OUT / 'suite-diagnostics'))
    elif mode == 'sysfiletree-diagnostic':
        env['REXX_SUITE_DIAGNOSTICS'] = str(OUT / 'sysfiletree-diagnostic')
    if mode in {'release-corpus', 'release-suite'}:
        prior = json.loads((OUT / 'release-artifacts-verified.json').read_text())
        assert prior['source_commit'] == PIN and prior['pin_provenance'] == release_provenance()
        assert prior['test_variant'] == release_test_variant()
        assert prior['normal_pin']['sha256'] == CONFIG['release_binary_sha256'] == digest(CONFIG['release_binary'])
        assert digest(prior['cli']['executable']) == prior['cli']['sha256'] == prior['test_variant']['test_cli_sha256']
        selected_test = mode.removeprefix('release-')
        assert digest(prior['tests'][selected_test]['executable']) == prior['tests'][selected_test]['sha256']
    result = run(mode, commands[mode], RUST, env, 525)
    if mode == 'release-build':
        assert result['returncode'] == 0 and not result['timed_out'], result
        verified = release_artifacts(OUT / 'release-build.stdout', {'corpus', 'suite'})
        (OUT / 'release-artifacts-verified.json').write_text(json.dumps(verified, indent=2) + '\n')
    elif mode in {'release-corpus', 'release-suite'}:
        fresh = release_artifacts(OUT / (mode + '.stdout'), {selected_test})
        assert fresh['cli'] == prior['cli'] and fresh['tests'][selected_test] == prior['tests'][selected_test], 'Actual release command artifacts changed'
        assert fresh['test_variant'] == prior['test_variant'] and fresh['rustix'] == prior['rustix'], 'Actual release test dependency variant changed'
        (OUT / (mode + '-artifacts.json')).write_text(json.dumps(fresh, indent=2) + '\n')
    if mode == 'pinning':
        text = (OUT / 'pinning.stdout').read_text(errors='replace')
        expected = {'scheduler::tests::mutants::' + name for name in re.findall(r'#\[test\]\s*fn\s+(\w+)\(', (RUST / 'crates/rexx-exec/src/scheduler/tests/mutants.rs').read_text())}
        actual = dict(re.findall(r'^test (scheduler::tests::mutants::\S+) \.\.\. (ok|FAILED|ignored)', text, re.M))
        assert expected and set(actual) == expected and all(status == 'ok' for status in actual.values()), 'Incomplete pinning mutant module'
        assert result['returncode'] == 0 and not result['timed_out'], result
        (OUT / 'pinning-verified.json').write_text(json.dumps({'source_commit': PIN, 'tests': sorted(actual)}, indent=2) + '\n')
    sys.exit(0 if result['returncode'] == 0 and not result['timed_out'] else 1)
elif mode == 'provenance':
    for name, args in [('rustc-version', ['rustc', '-Vv']), ('cargo-version', ['cargo', '--version']), ('git-revision', ['git', 'rev-parse', 'HEAD']), ('git-status', ['git', 'status', '--short', '--untracked-files=all'])]:
        result = run(name, args, RUST, ENV, 30)
        assert result['returncode'] == 0 and not result['timed_out'], result
elif mode == 'queue':
    items = artifacts()
    reconciliation = json.loads((OUT / 'artifact-population-reconciliation.json').read_text())
    assert items and len(items) == reconciliation['fresh_count']
    lines = [f'memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py target {index}' for index in range(len(items))]
    (OUT / 'target-commands.txt').write_text('\n'.join(lines) + '\n')
elif mode == 'capture':
    executable = sys.argv[2]
    assert Path(executable).is_file()
    (OUT / 'dispatch').mkdir(exist_ok=True)
    values = {k: os.environ[k] for k in PUBLIC_ENV if k in os.environ}
    with capture_path(executable).open('x') as output:
        json.dump({'executable': executable, 'argv': sys.argv[2:], 'cwd': os.getcwd(),
                   'cargo_env': values, 'sha256': digest(executable), 'source_commit': PIN}, output, indent=2)
elif mode == 'metadata':
    result = run('metadata', ['cargo', 'metadata', '--no-deps', '--locked', '--format-version=1'], RUST, ENV, 30)
    assert result['returncode'] == 0 and not result['timed_out'], result
    metadata = json.loads((OUT / 'metadata.stdout').read_text())
    expected = []
    for package in metadata['packages']:
        if package['id'] not in metadata['workspace_members']:
            continue
        manifest = tomllib.loads(Path(package['manifest_path']).read_text())
        for target in package['targets']:
            if not target['test']:
                continue
            assert target['kind'] in [['lib'], ['bin'], ['test']], 'Unsupported default test target'
            kind = target['kind'][0]
            entries = [manifest.get('lib', {})] if kind == 'lib' else manifest.get(kind, [])
            assert all(entry.get('harness', True) for entry in entries if kind == 'lib' or entry.get('name') == target['name']), 'Custom harness needs separate proof'
            expected.append({'manifest_path': package['manifest_path'], 'target': target})
    report = reconcile(expected)
    (OUT / 'metadata-population-reconciliation.json').write_text(json.dumps(report, indent=2) + '\n')
    (OUT / 'metadata-default-targets.json').write_text(json.dumps(expected, indent=2) + '\n')
elif mode == 'build':
    result = run('debug-build', ['cargo', 'test', '--workspace', '--no-run', '--locked', '-j4', '--message-format=json'], RUST, ENV, 525)
    assert result['returncode'] == 0 and not result['timed_out'], result
    selected = {}
    for line in (OUT / 'debug-build.stdout').read_text().splitlines():
        item = json.loads(line)
        if item.get('reason') == 'compiler-artifact' and item.get('profile', {}).get('test') and item.get('executable'):
            assert item['target']['test'] and item['target']['kind'] in [['test'], ['lib'], ['bin']]
            assert not item['features'], 'Feature population changed'
            assert item['profile']['debug_assertions'] and item['profile']['overflow_checks']
            selected[item['executable']] = item
    metadata_default = json.loads((OUT / 'metadata-default-targets.json').read_text())
    report = reconcile(list(selected.values()))
    (OUT / 'artifact-population-reconciliation.json').write_text(json.dumps(report, indent=2) + '\n')
    assert sorted(map(key, selected.values())) == sorted(map(key, metadata_default)), 'Metadata default population differs'
    (OUT / 'artifacts.json').write_text(json.dumps(list(selected.values()), indent=2) + '\n')
elif mode == 'dispatch':
    host = subprocess.check_output(['rustc', '-vV'], cwd=RUST, text=True, timeout=10)
    triple = next(line.removeprefix('host: ') for line in host.splitlines() if line.startswith('host: '))
    env = ENV.copy()
    env['CARGO_TARGET_' + triple.upper().replace('-', '_') + '_RUNNER'] = 'python3 /tmp/codex-p9d-t88-final-gate-draft.py capture'
    result = run('cargo-dispatch-only', ['cargo', 'test', '--workspace', '--lib', '--bins', '--tests', '--locked', '-j4', '--no-fail-fast', '--', '--test-threads=1', '--nocapture'], RUST, env, 525)
    assert result['returncode'] == 0 and not result['timed_out'], result
    items = artifacts()
    captured = list((OUT / 'dispatch').glob('*.json'))
    assert len(captured) == len(items) and items
    assert {json.loads(p.read_text())['executable'] for p in captured} == {i['executable'] for i in items}
    for item in items:
        dispatch = json.loads(capture_path(item['executable']).read_text())
        assert dispatch['cwd'] == str(Path(item['manifest_path']).parent)
        assert dispatch['argv'] == [item['executable'], '--test-threads=1', '--nocapture']
        assert dispatch['sha256'] == digest(item['executable'])
    (OUT / 'dispatch-verified.json').write_text(json.dumps({'source_commit': PIN, 'count': len(captured), 'tests_executed': False}) + '\n')
elif mode == 'target':
    assert json.loads((OUT / 'dispatch-verified.json').read_text())['source_commit'] == PIN
    index = int(sys.argv[2])
    item = artifacts()[index]
    dispatch = json.loads(capture_path(item['executable']).read_text())
    assert dispatch['source_commit'] == PIN and dispatch['sha256'] == digest(item['executable'])
    env = ENV.copy()
    env.update(dispatch['cargo_env'])
    name = f'debug-{index:03}-{item["target"]["name"]}'
    listed = run(name + '-list', [item['executable'], '--list', '--format', 'terse'], dispatch['cwd'], env, 15)
    assert listed['returncode'] == 0 and not listed['timed_out'], listed
    names = [line[:-6] for line in (OUT / (name + '-list.stdout')).read_text().splitlines() if line.endswith(': test')]
    assert len(names) == len(set(names))
    old_target = next((old for old in json.loads((BASE / 'targets.json').read_text()) if key(old) == key(item)), None)
    assert old_target is not None or key(item) in ADDED_TARGETS
    old_tests = [old for old in json.loads((BASE / 'tests.json').read_text()) if old_target is not None and old['target_record'] == old_target['name']]
    old_names = {old['test_name'] for old in old_tests}
    renames = RENAMED_TESTS.get(key(item), {})
    assert renames.keys() <= old_names, 'Rename has no historical source test'
    assert not renames.keys() & set(names), 'Historical renamed name still present'
    assert set(renames.values()) <= set(names), 'Reconciled replacement test missing'
    expected_names = {renames.get(name, name) for name in old_names}
    population = {'added': sorted(set(names) - expected_names), 'removed': sorted(expected_names - set(names)), 'reconciled_renames': renames}
    (OUT / (name + '-population.json')).write_text(json.dumps(population, indent=2) + '\n')
    assert not population['removed'], 'Removed default tests need controller reconciliation'
    result = run(name, dispatch['argv'], dispatch['cwd'], env, 510)
    stdout = (OUT / (name + '.stdout')).read_text(errors='replace')
    summaries = re.findall(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', stdout, re.M)
    assert len(summaries) == 1 and sum(map(int, summaries[0])) == len(names), 'Incomplete default population'
    assert int(summaries[0][4]) == 0 and not result['timed_out'], 'Filtered or timed-out run'
    result.update(listed_tests=len(names), executable_sha256=dispatch['sha256'], summary=list(map(int, summaries[0])))
    (OUT / (name + '.json')).write_text(json.dumps(result, indent=2) + '\n')
    sys.exit(0 if result['returncode'] == 0 else 1)
elif mode == 'doc':
    result = run('doctests', ['cargo', 'test', '--workspace', '--doc', '--locked', '-j4', '--no-fail-fast', '--', '--test-threads=1'], RUST, ENV, 525)
    text = (OUT / 'doctests.stdout').read_text()
    rows = re.findall(r'^test (.*?) \.\.\. (ok|FAILED|ignored)[^\n]*$', text, re.M)
    previous = json.loads((BASE / 'doctest-results.json').read_text())
    normalize = lambda name: re.sub(r'\(line \d+\)', '(line N)', name)
    assert sorted(normalize(name) for name, _ in rows) == sorted(normalize(row['test_name']) for row in previous)
    assert len(rows) == 15 and all(status == 'ok' for _, status in rows)
    assert result['returncode'] == 0 and not result['timed_out'], result
    (OUT / 'doctests-verified.json').write_text(json.dumps({'source_commit': PIN, 'identities': rows}, indent=2) + '\n')
elif mode == 'audit':
    items = artifacts()
    prior_targets = {key(old): old for old in json.loads((BASE / 'targets.json').read_text())}
    prior_tests = json.loads((BASE / 'tests.json').read_text())
    rows = []
    for index, item in enumerate(items):
        name = f'debug-{index:03}-{item["target"]["name"]}'
        record = json.loads((OUT / (name + '.json')).read_text())
        assert record['source_commit'] == PIN and record['returncode'] == 0 and not record['timed_out']
        assert record['summary'][1] == 0 and record['summary'][3:] == [0, 0]
        assert sum(record['summary']) == record['listed_tests']
        old_target = prior_targets.get(key(item))
        assert old_target is not None or key(item) in ADDED_TARGETS
        expected_ignored = {old['test_name'] for old in prior_tests if old_target is not None and old['target_record'] == old_target['name'] and old['status'] == 'ignored'}
        text = (OUT / (name + '.stdout')).read_text(errors='replace')
        actual_ignored = set(re.findall(r'^test (.*?) \.\.\. ignored[^\n]*$', text, re.M))
        assert actual_ignored == expected_ignored, 'Ignored population changed'
        assert record['summary'][2] == len(actual_ignored)
        rows.append({'key': key(item), 'listed_tests': record['listed_tests'], 'summary': record['summary'], 'executable_sha256': record['executable_sha256']})
    required = {('rexx-parse', 'sourceline'), ('rexx-parse', 'sourceline_oracle'), ('rexx-core', 'unsafe_sites')}
    required.update(('rexx-exec', name) for name in ['coverage', 'refusal_dispositions', 'refusal_sites', 'owners', 'closed_phases', 'phase_ten_reasons', 'representation_refusals', 'collect_stress', 'corpus', 'suite'])
    required.add(('rexx-exec', 'semantic_boundaries'))
    assert required <= {(r['key'][0], r['key'][1]) for r in rows if r['listed_tests'] > 0}
    assert len(rows) == len(items) and sum(row['summary'][0] for row in rows) > 0
    reconciliation = reconcile(items)
    docs = json.loads((OUT / 'doctests.json').read_text())
    assert docs['source_commit'] == PIN and docs['returncode'] == 0 and not docs['timed_out']
    assert json.loads((OUT / 'doctests-verified.json').read_text())['source_commit'] == PIN
    assert json.loads((OUT / 'pinning-verified.json').read_text())['source_commit'] == PIN
    assert len(re.findall(r'^test .*? \.\.\. ok[^\n]*$', (OUT / 'doctests.stdout').read_text(), re.M)) == 15
    (OUT / 'debug-completeness.json').write_text(json.dumps({'source_commit': PIN, 'population': reconciliation, 'targets': rows}, indent=2) + '\n')
else:
    raise ValueError(mode)
