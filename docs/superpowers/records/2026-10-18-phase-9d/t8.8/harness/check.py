import json, os, pathlib, shlex, signal, subprocess, sys, time
name, bound, *command = sys.argv[1:]
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
record = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/harness'
record.mkdir(parents=True, exist_ok=True)
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target', REXX_CORPUS_GATE='1')
started = time.monotonic()
with (record / (name + '.txt')).open('w') as output:
    output.write('CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target REXX_CORPUS_GATE=1 ' + shlex.join(command) + '\n')
    output.flush()
    process = subprocess.Popen(command, cwd=root / 'rust', env=env, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        status = process.wait(timeout=float(bound))
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        status = 124
    elapsed = time.monotonic() - started
    output.write(f'\nexit={status} elapsed={elapsed:.3f}s\n')
print((record / (name + '.txt')).read_text()[-16000:])
with (record / 'commands.jsonl').open('a') as output:
    output.write(json.dumps(dict(name=name, command=command, bound=int(bound), status=status, elapsed=elapsed)) + '\n')
sys.exit(status)
