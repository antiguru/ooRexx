import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RECORDS = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/pow/perf-fix-r1'
COMMANDS = {
    'rexx-num-tests': ['cargo', 'test', '-j', '4', '--locked', '-p', 'rexx-num'],
    'fmt': ['cargo', 'fmt', '--all', '--check'],
    'clippy': ['cargo', 'clippy', '-j', '4', '--locked', '-p', 'rexx-num', '--all-targets', '--', '-D', 'warnings'],
    'release-build': ['cargo', 'build', '-j', '4', '--locked', '--release', '-p', 'rexx-exec', '--bin', 'rexx'],
}
name = sys.argv[1]
command = ['timeout', '-k', '5', '540', *COMMANDS[name]]
environment = os.environ.copy()
environment.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target')
RECORDS.mkdir(parents=True, exist_ok=True)
if name == 'rexx-num-tests' and (RECORDS / f'{name}.status').read_text().strip() == '1':
    for suffix in ['stdout', 'stderr', 'status']:
        source = RECORDS / f'{name}.{suffix}'
        source.rename(RECORDS / f'{name}-sandbox-failure.{suffix}')
metadata = {'command': command, 'cwd': str(ROOT / 'rust'), 'environment': {key: environment[key] for key in ['CARGO_INCREMENTAL', 'CARGO_TARGET_DIR']}, 'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()}
(RECORDS / f'{name}.command.json').write_text(json.dumps(metadata, indent=2) + '\n')
print(json.dumps(metadata), flush=True)
with (RECORDS / f'{name}.stdout').open('wb') as stdout, (RECORDS / f'{name}.stderr').open('wb') as stderr:
    result = subprocess.run(command, cwd=ROOT / 'rust', env=environment, stdout=stdout, stderr=stderr)
(RECORDS / f'{name}.status').write_text(f'{result.returncode}\n')
print(f'{name}: exit {result.returncode}', flush=True)
print((RECORDS / f'{name}.stderr').read_text()[-5000:], flush=True)
print((RECORDS / f'{name}.stdout').read_text()[-3500:], flush=True)
sys.exit(result.returncode)
