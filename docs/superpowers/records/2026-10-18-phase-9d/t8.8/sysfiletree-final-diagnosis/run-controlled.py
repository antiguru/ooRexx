import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

root = Path('/tmp/codex-p9d-t88-sysfiletree-final-diagnosis')
output = root / 'controlled'
output.mkdir(exist_ok=False)
source = root / 'controlled-witness.rex'
pin = Path('/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx')
oracle = Path('/home/moritz/dev/repos/ooRexx/build/bin/rexx')
expected_pin = 'd9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
executables = {'crate': {'path': str(pin), 'sha256': sha(pin)},
               'oracle': {'path': str(oracle), 'sha256': sha(oracle)}}
assert executables['crate']['sha256'] == expected_pin
(output / 'executables.json').write_text(json.dumps(executables, indent=2) + '\n')
records = []
for side in ['oracle', 'crate']:
    for mode in ['stable', 'create']:
        run = output / (side + '-' + mode)
        run.mkdir()
        cwd = run / 'cwd'
        tmpdir = run / 'tmpdir'
        cwd.mkdir()
        tmpdir.mkdir()
        (tmpdir / 'seed.txt').write_bytes(b'seed\n')
        env = os.environ.copy()
        env['TMPDIR'] = str(tmpdir)
        if side == 'oracle':
            env['LD_LIBRARY_PATH'] = '/home/moritz/dev/repos/ooRexx/build/lib'
            command = ['sh', '-c', 'ulimit -v 1048576; exec "$@"', 'oracle-wrapper',
                       'timeout', '-k', '5', '20', str(oracle), str(source), mode]
        else:
            command = [shutil.which('memcap'), '2G', 'timeout', '-k', '5', '20',
                       str(pin), str(source), mode]
            assert command[0]
        metadata = {'side': side, 'mode': mode, 'command': command, 'cwd': str(cwd),
                    'TMPDIR': str(tmpdir), 'LD_LIBRARY_PATH': env.get('LD_LIBRARY_PATH'),
                    'source_sha256': sha(source), 'executable': executables[side],
                    'seed_sha256': sha(tmpdir / 'seed.txt'), 'timeout_seconds': 20}
        (run / 'command.json').write_text(json.dumps(metadata, indent=2) + '\n')
        started = time.monotonic()
        result = subprocess.run(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        for name, data in [('stdout', result.stdout), ('stderr', result.stderr),
                           ('status', f'{result.returncode}\n'.encode())]:
            (run / name).write_bytes(data)
        metadata['elapsed_seconds'] = time.monotonic() - started
        metadata['status'] = result.returncode
        metadata['channels'] = {name: sha(run / name) for name in ['stdout', 'stderr', 'status']}
        (run / 'result.json').write_text(json.dumps(metadata, indent=2) + '\n')
        print(side, mode, result.returncode, repr(result.stdout), repr(result.stderr), flush=True)
        assert result.returncode == 0 and result.stderr == b''
        lines = result.stdout.decode().splitlines()
        before = [line.removeprefix('before ') for line in lines if line.startswith('before ')]
        after = [line.removeprefix('after ') for line in lines if line.startswith('after ')]
        seed = str(tmpdir / 'seed.txt')
        injected = str(tmpdir / 'injected.txt')
        assert before == [seed]
        assert after == sorted([seed, injected] if mode == 'create' else [seed])
        assert 'before-count 1' in lines and 'tree-rc 0' in lines
        assert f'after-count {2 if mode == "create" else 1}' in lines
        assert f'counts-equal {0 if mode == "create" else 1}' in lines
        assert ('injected ' + (injected if mode == 'create' else 'none')) in lines
        records.append({'side': side, 'mode': mode, 'before': before, 'after': after,
                        'added': sorted(set(after) - set(before)), 'removed': sorted(set(before) - set(after)),
                        'counts_equal': before == after, 'status': result.returncode})
assert sha(pin) == expected_pin
assert sha(oracle) == executables['oracle']['sha256']
(output / 'verdict.json').write_text(json.dumps({'runs': records,
    'mechanism_confirmed_on_both': True, 'stable_control_equal_on_both': True,
    'executable_hashes_unchanged': True}, indent=2) + '\n')
manifest = {str(path.relative_to(root)): {'bytes': path.stat().st_size, 'sha256': sha(path)}
            for path in sorted(output.rglob('*')) if path.is_file()}
manifest['controlled-witness.rex'] = {'bytes': source.stat().st_size, 'sha256': sha(source)}
driver = root / 'run-controlled.py'
manifest['run-controlled.py'] = {'bytes': driver.stat().st_size, 'sha256': sha(driver)}
(root / 'controlled-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print('Four bounded runs complete; mechanism and stable controls verified; executable hashes unchanged.')
