from pathlib import Path
import subprocess, tempfile, json, os, hashlib
repo = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = repo / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2'
root = Path(tempfile.mkdtemp(prefix='t88e2-r2-options-'))
wrapper = root / 'oracle.sh'
wrapper.write_text('#!/bin/bash\nulimit -v 1048576\nexport LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib\nexec timeout -k 2s 20s /home/moritz/dev/repos/ooRexx/build/bin/rexx "$@"\n')
pin = '/tmp/codex-p9d-t88-target/release/rexx'
results = []
for name in ['directive_options', 'directive_options_numeric_inherit', 'directive_options_trace_method']:
    source = repo / f'rust/corpus/lang/{name}.rex'
    answers = []
    for engine in ['oracle', 'rust']:
        cwd = root / f'{name}-{engine}'
        cwd.mkdir()
        command = ['bash', str(wrapper), str(source)] if engine == 'oracle' else ['timeout', '-k', '2s', '20s', pin, str(source)]
        result = subprocess.run(command, cwd=cwd, input=b'', stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=25)
        base = records / f'{name}-{engine}'
        base.with_suffix('.stdout').write_bytes(result.stdout)
        base.with_suffix('.stderr').write_bytes(result.stderr)
        descriptor = dict(command=command, cwd=str(cwd), status=result.returncode, source_sha256=hashlib.sha256(source.read_bytes()).hexdigest())
        base.with_suffix('.json').write_text(json.dumps(descriptor, indent=2)+'\n')
        assert result.returncode == 0, descriptor
        answers.append((result.returncode, result.stdout, result.stderr))
    assert answers[0] == answers[1], name
    results.append(dict(program=name, status=answers[1][0], stdout_bytes=len(answers[1][1]), stderr_bytes=len(answers[1][2]), stdout_equal=True, stderr_equal=True, status_equal=True))
(records / 'options-agreement.json').write_text(json.dumps(results, indent=2)+'\n')
print(json.dumps(results, indent=2))
