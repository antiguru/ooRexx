from pathlib import Path
import importlib.util, json, os, subprocess, tempfile, time, tty
from unittest.mock import patch

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/fifo-close'
records.mkdir(parents=True, exist_ok=True)
work = Path(tempfile.mkdtemp(prefix='t88e1-probes-'))
crate = Path('/tmp/t88f-fifo/rexx-frozen')
oracle = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/diagnostics/fixed-rechecks/oracle.sh'
rows = []

def run(name, shape, terminal=False):
    for side in ('crate', 'oracle'):
        directory = work / f'{name}-{side}'
        directory.mkdir()
        master = slave = reader = None
        if terminal:
            master, slave = os.openpty()
            tty.setraw(slave)
            path = os.ttyname(slave)
            os.write(master, b'abc')
        else:
            path = str(directory / 'pipe')
            os.mkfifo(path)
            reader = os.open(path, os.O_RDONLY | os.O_NONBLOCK)
        source = shape.replace('SUBJECT', path)
        program = directory / 'probe.rex'
        program.write_text(source)
        command = ['memcap','2G','timeout','-k','2s','20s',str(crate),str(program)] if side == 'crate' else ['bash',str(oracle),str(program)]
        env = os.environ.copy()
        env.update(TZ='UTC', REXX_PATH=str(directory))
        started = time.monotonic()
        completed = subprocess.run(command, cwd=directory, env=env, input=b'', stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=25)
        descriptor = master if terminal else reader
        os.set_blocking(descriptor, False)
        payload = b''
        while True:
            try:
                chunk = os.read(descriptor, 65536)
            except BlockingIOError:
                break
            if not chunk:
                break
            payload += chunk
        if terminal:
            os.close(slave)
        os.close(descriptor)
        entry = dict(name=name, side=side, command=command, cwd=str(directory), source=source,
                     status=completed.returncode, stdout_hex=completed.stdout.hex(), stderr_hex=completed.stderr.hex(),
                     payload_hex=payload.hex(), elapsed=time.monotonic()-started,
                     terminal_raw=terminal, input_hex='616263' if terminal else '',
                     crate_artifact=str(crate))
        rows.append(entry)
        for label, data in [('stdout',completed.stdout), ('stderr',completed.stderr), ('payload',payload), ('source.rex',source.encode())]:
            (records / f'{name}-{side}.{label}').write_bytes(data)
        (records / f'{name}-{side}.json').write_text(json.dumps(entry, indent=2)+'\n')
        print(name, side, completed.returncode, completed.stdout, completed.stderr, payload, flush=True)

run('terminal', "say 'open' stream('SUBJECT','C','OPEN BOTH')\nsay 'read' '['charin('SUBJECT',,3)']'\nsay 'write' lineout('SUBJECT','xyz')\nsay 'close' stream('SUBJECT','C','CLOSE')\n", terminal=True)
run('fifo-close-only', "say 'write' lineout('SUBJECT','hello')\nsay 'close' lineout('SUBJECT')\n")
run('fifo-state-close', "say 'write' lineout('SUBJECT','hello')\nsay 'state' stream('SUBJECT','S')\nsay 'close' lineout('SUBJECT')\n")
run('fifo-method-close', "s=.Stream~new('SUBJECT')\nsay 'write' s~lineout('hello')\nsay 'close' s~lineout\n")
(records/'additional-results.json').write_text(json.dumps(rows, indent=2)+'\n')

