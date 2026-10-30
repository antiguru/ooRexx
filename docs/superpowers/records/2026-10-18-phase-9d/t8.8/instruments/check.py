import json, os, pathlib, shlex, signal, subprocess, sys, time, re
name, bound, *command = sys.argv[1:]
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
record = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments'
record.mkdir(parents=True, exist_ok=True)
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target', REXX_CORPUS_GATE='1')
while command and command[0].startswith('--env='):
    key, value = command.pop(0)[6:].split('=', 1)
    env[key] = value
environment = {key: value for key, value in env.items() if key.startswith(('CARGO_', 'REXX_'))}
started = time.monotonic()
with (record / (name + '.txt')).open('w') as output, (record / (name + '.stderr')).open('w') as error:
    output.write(shlex.join([key + '=' + value for key, value in sorted(environment.items())]) + ' ' + shlex.join(command) + '\n')
    output.flush()
    process = subprocess.Popen(command, cwd=root / 'rust', env=env, stdout=output, stderr=error, start_new_session=True)
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
report = (record / (name + '.txt')).read_text()
if 'test' in command and '--no-run' not in command and status in (0, 101):
    counts = re.findall(r'test result: .*? (\d+) passed; (\d+) failed;', report)
    if not counts or sum(int(a) + int(b) for a, b in counts) == 0:
        status = 125
        print('INFRA_FAILURE: no completed nonzero test count')
print(report[-6000:])
print((record / (name + '.stderr')).read_text()[-3000:])
with (record / 'commands.jsonl').open('a') as output:
    output.write(json.dumps(dict(name=name, command=command, environment=environment, bound=int(bound), status=status, elapsed=elapsed)) + '\n')
sys.exit(status)
