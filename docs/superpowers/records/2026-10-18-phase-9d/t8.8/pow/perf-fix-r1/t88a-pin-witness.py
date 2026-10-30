import hashlib
import json
import os
from pathlib import Path
import resource
import shutil
import subprocess
import sys
import tempfile

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RECORDS = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/pow/perf-fix-r1'
PIN = Path('/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx')
BASE = Path('/tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx')

def digest(path):
    return hashlib.file_digest(path.open('rb'), 'sha256').hexdigest()

def limit_oracle():
    resource.setrlimit(resource.RLIMIT_AS, (1073741824, 1073741824))

mode = sys.argv[1]
if mode == 'pin':
    expected = '470dcb41db61ab7a4d64f3276ce28e539d5686d130be245a4ec7b30e6d6725ac'
    assert digest(BASE) == expected
    assert not PIN.exists()
    shutil.copy2('/tmp/codex-p9d-t88-target/release/rexx', PIN)
    compiler = Path(subprocess.check_output(['rustup', 'which', 'rustc'], text=True).strip())
    data = {
        'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'source_tree': subprocess.check_output(['git', 'rev-parse', 'HEAD:rust'], cwd=ROOT, text=True).strip(),
        'pin': str(PIN), 'pin_sha256': digest(PIN),
        'build_binary': '/tmp/codex-p9d-t88-target/release/rexx',
        'compiler': str(compiler), 'compiler_sha256': digest(compiler),
        'rustc_version': subprocess.check_output(['rustc', '-Vv'], text=True),
        'cargo_version': subprocess.check_output(['cargo', '-V'], text=True),
        'baseline': str(BASE), 'baseline_source': '1233b81da730142057fcf810e324d10d0e55c109',
        'baseline_sha256': expected,
        'matched_all_program_comparison': 'Pending controller-owned all-program comparison and scoped review.',
    }
    (RECORDS / 'pin-provenance.json').write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps(data, indent=2))
elif mode in ['oracle', 'crate']:
    cwd = Path(tempfile.mkdtemp(prefix=f'codex-t88a-perf-{mode}-'))
    witness = ROOT / 'rust/corpus/lang/power_integral_exponent.rex'
    shutil.copy2(witness, cwd / witness.name)
    environment = os.environ.copy()
    environment['LD_LIBRARY_PATH'] = '/home/moritz/dev/repos/ooRexx/build/lib'
    command = ['timeout', '-k', '5', '20', '/home/moritz/dev/repos/ooRexx/build/bin/rexx' if mode == 'oracle' else str(PIN), witness.name]
    metadata = {'command': command, 'cwd': str(cwd), 'witness_sha256': digest(witness), 'LD_LIBRARY_PATH': environment['LD_LIBRARY_PATH']}
    if mode == 'oracle':
        metadata.update(RLIMIT_AS=1073741824, crash_list_sha256=digest(ROOT / 'rust/corpus/oracle-crashes.txt'))
    (RECORDS / f'{mode}-power.command.json').write_text(json.dumps(metadata, indent=2) + '\n')
    with (RECORDS / f'{mode}-power.stdout').open('wb') as stdout, (RECORDS / f'{mode}-power.stderr').open('wb') as stderr:
        result = subprocess.run(command, cwd=cwd, env=environment, stdout=stdout, stderr=stderr, preexec_fn=limit_oracle if mode == 'oracle' else None)
    (RECORDS / f'{mode}-power.status').write_text(f'{result.returncode}\n')
    print(json.dumps(metadata, indent=2))
    print(f'{mode}: exit {result.returncode}')
    print((RECORDS / f'{mode}-power.stdout').read_text())
    print((RECORDS / f'{mode}-power.stderr').read_text())
    sys.exit(result.returncode)
