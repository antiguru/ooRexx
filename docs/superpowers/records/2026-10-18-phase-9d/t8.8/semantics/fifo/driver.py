from pathlib import Path
import ctypes, hashlib, json, os, resource, signal, subprocess, sys, time

ROOT = Path('/tmp/t88f-fifo')
ORACLE = '/home/moritz/dev/repos/ooRexx/build/bin/rexx'
LIB = '/home/moritz/dev/repos/ooRexx/build/lib'
libc = ctypes.CDLL(None, use_errno=True)
libc.tgkill.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_int]
libc.tgkill.restype = ctypes.c_int

def limited():
    resource.setrlimit(resource.RLIMIT_AS, (1073741824, 1073741824))

def identity(pid):
    stat = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
    return {'pid': pid, 'starttime': stat[19], 'exe': os.readlink(f'/proc/{pid}/exe')}

def descendants(pid):
    result = []
    try:
        children = Path(f'/proc/{pid}/task/{pid}/children').read_text().split()
    except FileNotFoundError:
        return result
    for child in children:
        child = int(child)
        result.append(child)
        result.extend(descendants(child))
    return result

def traces(directory):
    return '\n'.join(p.read_text(errors='replace') for p in directory.glob('trace.*'))

env = os.environ.copy()
env.update(LD_LIBRARY_PATH=LIB, TZ='UTC')
version = subprocess.run([ORACLE, '-v'], cwd=ROOT, env=env, preexec_fn=limited,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10)
(ROOT/'oracle-version.stdout').write_bytes(version.stdout)
(ROOT/'oracle-version.stderr').write_bytes(version.stderr)
(ROOT/'oracle-version.json').write_text(json.dumps({'command': [ORACLE, '-v'], 'status': version.returncode,
    'rlimit_as': 1073741824, 'LD_LIBRARY_PATH': LIB}, indent=2)+'\n')
print('oracle version', version.returncode, version.stdout.decode(), version.stderr.decode(), flush=True)

args = sys.argv[1:]
partial = '--partial' in args
if partial:
    args.remove('--partial')
for side in (args or ('crate', 'oracle')):
    directory = ROOT/('partial-'+side+'-r1' if partial else side+'-r4')
    directory.mkdir()
    fifo = directory/'pipe'
    os.mkfifo(fifo)
    writer = os.open(fifo, os.O_RDWR | os.O_NONBLOCK)
    os.write(writer, b'abc')
    program = directory/'probe.rex'
    program.write_text(f'''f = '{fifo}'
log = '{directory / 'events'}'
call on halt name h
call on notready name nr
say 'warm ['charin(f,,3)']'
call lineout '{directory / 'ready'}', 'READY'
call stream '{directory / 'ready'}', 'c', 'close'
x = charin(f,,{2 if partial else 1})
say 'read [' || x || ']'
say 'description' stream(f,'d')
exit 0
nr:
say 'NOTREADY' condition('d') stream(f,'d')
call lineout log, 'NOTREADY 'stream(f,'d')
call stream log, 'c', 'close'
return
h:
say 'HALT' stream(f,'d')
call lineout log, 'HALT 'stream(f,'d')
call stream log, 'c', 'close'
return
''')
    executable = str(ROOT/'rexx-frozen') if side == 'crate' else ORACLE
    target = ['memcap', '2G', executable, str(program)] if side == 'crate' else [executable, str(program)]
    command = ['strace', '-ff', '-tt', '-T', '-yy', '-e',
        'trace=read,write,openat,close,lseek,rt_sigaction,rt_sigprocmask,rt_sigreturn',
        '-o', str(directory/'trace')] + target
    stdout = (directory/'stdout').open('wb')
    stderr = (directory/'stderr').open('wb')
    started = time.monotonic()
    proc = subprocess.Popen(command, cwd=directory, env=env, stdin=subprocess.DEVNULL,
                            stdout=stdout, stderr=stderr, preexec_fn=limited if side == 'oracle' else None)
    descriptor = {'command': command, 'cwd': str(directory), 'tracer': identity(proc.pid),
                  'source_sha256': hashlib.sha256(program.read_bytes()).hexdigest(), 'events': [],
                  'rlimit_as': 1073741824 if side == 'oracle' else None, 'LD_LIBRARY_PATH': LIB}
    target_identity = None
    target_pidfd = None
    try:
        deadline = started + 15
        while time.monotonic() < deadline:
            for pid in descendants(proc.pid):
                try:
                    candidate = identity(pid)
                    if candidate['exe'] == executable:
                        target_identity = candidate
                        break
                except (FileNotFoundError, ProcessLookupError):
                    pass
            if target_identity and (directory/'ready').exists() and (directory/'ready').read_bytes() == b'READY\n':
                break
            if proc.poll() is not None:
                raise RuntimeError('exited before ready')
            time.sleep(.01)
        assert target_identity and (directory/'ready').read_bytes() == b'READY\n', 'no readiness'
        tgid = target_identity['pid']
        target_pidfd = os.pidfd_open(tgid)
        if partial:
            os.write(writer, b'P')
            descriptor['events'].append({'event': 'partial writer P', 'elapsed': time.monotonic()-started})
        blocked = None
        while time.monotonic() < deadline:
            for task in Path(f'/proc/{tgid}/task').iterdir():
                try:
                    syscall = (task/'syscall').read_text().strip()
                    fields = syscall.split()
                    if len(fields) > 1 and fields[0] == '0':
                        fd = int(fields[1], 16)
                        link = os.readlink(f'/proc/{tgid}/fd/{fd}')
                        trace = traces(directory)
                        partial_seen = any(str(fifo) in line and '"P"' in line and '= 1' in line
                                           for line in trace.splitlines())
                        if link == str(fifo) and (not partial or partial_seen):
                            status = (task/'status').read_text()
                            assert f'Tgid:\t{tgid}\n' in status
                            blocked = {'tgid': tgid, 'tid': int(task.name), 'fd': fd,
                                       'fd_link': link, 'syscall': syscall, 'status': status,
                                       'comm': (task/'comm').read_text().strip()}
                            break
                except (FileNotFoundError, ProcessLookupError):
                    pass
            if blocked:
                break
            time.sleep(.01)
        assert blocked, 'no exact named-stream read found'
        assert identity(tgid) == target_identity
        assert tgid in descendants(proc.pid)
        descriptor['target'] = target_identity
        descriptor['blocked_before_signal'] = blocked
        descriptor['ready_hex'] = (directory/'ready').read_bytes().hex()
        result = libc.tgkill(tgid, blocked['tid'], signal.SIGINT)
        error = ctypes.get_errno()
        assert result == 0, (result, error)
        descriptor['events'].append({'event': 'tgkill SIGINT', 'elapsed': time.monotonic()-started,
                                     'tgid': tgid, 'tid': blocked['tid'], 'result': result})
        # A post-EINTR blocked read proves retry without requiring a scheduling delay.
        observation = None
        while time.monotonic() < deadline:
            trace = traces(directory)
            if proc.poll() is not None:
                observation = 'exited after signal without writer unblock'
                break
            task = Path(f'/proc/{tgid}/task/{blocked["tid"]}')
            if 'EINTR' in trace:
                try:
                    syscall = (task/'syscall').read_text().strip()
                except FileNotFoundError:
                    observation = 'exited after signal without writer unblock'
                    break
                fields = syscall.split()
                if len(fields) > 1 and fields[0] == '0' and int(fields[1],16) == blocked['fd']:
                    descriptor['blocked_after_eintr'] = {'syscall': syscall,
                        'events_file_exists': (directory/'events').exists()}
                    assert not (directory/'events').exists(), 'handler ran before unblock'
                    observation = 'read blocked again after EINTR'
                    break
            time.sleep(.01)
        assert observation, 'no post-signal result within bound'
        descriptor['observation'] = observation
        if observation == 'read blocked again after EINTR':
            assert identity(tgid) == target_identity
            descriptor['events'].append({'event': 'controlled writer unblock Z',
                                        'elapsed': time.monotonic()-started})
            os.write(writer, b'Z')
        proc.wait(timeout=max(.1, deadline-time.monotonic()))
        descriptor['status'] = proc.returncode
    except Exception as exc:
        descriptor['failure'] = repr(exc)
        if target_pidfd is not None:
            try:
                signal.pidfd_send_signal(target_pidfd, signal.SIGKILL)
                descriptor['cleanup_target_killed'] = True
            except ProcessLookupError:
                pass
        elif target_identity:
            try:
                if identity(target_identity['pid']) == target_identity:
                    os.kill(target_identity['pid'], signal.SIGKILL)
            except (FileNotFoundError, ProcessLookupError):
                pass
        if proc.poll() is None:
            proc.kill()
        descriptor['status'] = proc.wait(timeout=3)
    finally:
        if target_pidfd is not None:
            os.close(target_pidfd)
        os.close(writer)
        stdout.close()
        stderr.close()
    descriptor['elapsed'] = time.monotonic()-started
    (directory/'descriptor.json').write_text(json.dumps(descriptor, indent=2)+'\n')
    print(side, json.dumps(descriptor, indent=2), flush=True)
    print('stdout', (directory/'stdout').read_bytes(), 'stderr', (directory/'stderr').read_bytes(), flush=True)
    for line in traces(directory).splitlines():
        if str(fifo) in line or 'EINTR' in line or 'SIGINT' in line:
            print(line, flush=True)
    if 'failure' in descriptor:
        raise RuntimeError(descriptor['failure'])
