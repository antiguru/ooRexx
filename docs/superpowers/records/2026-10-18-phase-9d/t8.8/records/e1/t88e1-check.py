import json, os, pathlib, shlex, signal, subprocess, sys, time
name, bound, cwd, *command = sys.argv[1:]
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1'
records.mkdir(parents=True, exist_ok=True)
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target')
while command and command[0].startswith('--env='):
    key, value = command.pop(0)[6:].split('=', 1)
    env[key] = value
start = time.monotonic()
with (records / (name + '.stdout')).open('w') as out, (records / (name + '.stderr')).open('w') as err:
    process = subprocess.Popen(command, cwd=cwd, env=env, stdout=out, stderr=err, start_new_session=True)
    try:
        status = process.wait(timeout=float(bound))
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        status = 124
descriptor = dict(command=command, cwd=cwd, status=status, elapsed=time.monotonic()-start,
                  bound=float(bound), environment={k:v for k,v in env.items() if k.startswith(('CARGO_', 'REXX_'))})
(records / (name + '.json')).write_text(json.dumps(descriptor, indent=2)+'\n')
print(json.dumps(descriptor))
print((records / (name + '.stdout')).read_text()[-3500:])
print((records / (name + '.stderr')).read_text()[-3500:])
sys.exit(status)
