import hashlib
import json
import os
import pathlib
import signal
import subprocess
import sys
import time

root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m6'
records.mkdir(parents=True, exist_ok=True)
action = sys.argv[1]
primary = 'scheduler::tests::mutants::a_terminal_reply_slice_is_not_attributed_to_the_incoming_sort'
twin = 'scheduler::tests::mutants::a_terminal_return_leaves_no_slice_for_the_incoming_sort'
test = ['cargo', 'test', '-j4', '-p', 'rexx-exec', '--lib', '--features', 'pinning']
if action in ('fmt', 'fmt-final'):
    command = ['cargo', 'fmt', '--all', '--check']
elif action == 'clippy':
    command = ['cargo', 'clippy', '-j4', '-p', 'rexx-exec', '--all-targets', '--features', 'pinning', '--', '-D', 'warnings']
elif action in ('focused', 'focused-final'):
    command = test + ['scheduler::tests::mutants::', '--', '--nocapture']
else:
    stage, variant = action.split('-')
    assert stage in ('baseline', 'mutant', 'restored')
    assert variant in ('reply', 'return')
    command = test + [primary if variant == 'reply' else twin, '--', '--exact', '--nocapture']
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
scheduler = root / 'rust/crates/rexx-exec/src/scheduler.rs'
source_sha256 = hashlib.sha256(scheduler.read_bytes()).hexdigest()
start = time.monotonic()
with (records / (action + '.stdout')).open('w') as out, (records / (action + '.stderr')).open('w') as err:
    process = subprocess.Popen(command, cwd=root / 'rust', env=env, stdout=out, stderr=err, start_new_session=True)
    try:
        status = process.wait(timeout=540)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        status = 124
descriptor = dict(action=action, command=command, cwd=str(root / 'rust'), status=status,
                  head=head, scheduler_sha256=source_sha256, elapsed=time.monotonic()-start,
                  bound=540, launcher=['memcap', '8G', 'python3', '/tmp/t88g-check.py', action],
                  environment={k:v for k,v in env.items() if k.startswith(('CARGO_', 'REXX_'))})
(records / (action + '.json')).write_text(json.dumps(descriptor, indent=2) + '\n')
print(json.dumps(descriptor), flush=True)
print((records / (action + '.stdout')).read_text()[-5000:], flush=True)
print((records / (action + '.stderr')).read_text()[-5000:], flush=True)
sys.exit(status)
