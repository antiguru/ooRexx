import hashlib, json, os, pathlib, resource, subprocess, sys, tempfile
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/corpus-registration-repair/implementation'
pin = pathlib.Path(sys.argv[1])
assert hashlib.sha256(pin.read_bytes()).hexdigest() == 'd9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145'
for name in ['reference_conversion_raises_in_clause', 'reference_conversion_without_description']:
    source = root / 'rust/corpus/lang' / (name + '.rex')
    channels = {}
    for side in ['oracle', 'crate']:
        dest = records / 'probes' / name / side
        dest.mkdir(parents=True, exist_ok=True)
        cwd = tempfile.mkdtemp(prefix='t88-reg-')
        env = os.environ.copy()
        if side == 'oracle':
            env['LD_LIBRARY_PATH'] = '/home/moritz/dev/repos/ooRexx/build/lib'
            command = ['timeout', '-k', '5', '20', '/home/moritz/dev/repos/ooRexx/build/bin/rexx', str(source)]
            def cap():
                resource.setrlimit(resource.RLIMIT_AS, (1073741824, 1073741824))
        else:
            command = ['memcap', '2G', 'timeout', '-k', '5', '20', str(pin), str(source)]
            cap = None
        result = subprocess.run(command, cwd=cwd, env=env, preexec_fn=cap, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        values = dict(stdout=result.stdout, stderr=result.stderr, status=f'{result.returncode}\n'.encode())
        for channel, value in values.items():
            (dest / channel).write_bytes(value)
        (dest / 'command.json').write_text(json.dumps(dict(command=command, cwd=cwd, oracle_as=1073741824 if side == 'oracle' else None, source_sha256=hashlib.sha256(source.read_bytes()).hexdigest()), indent=2)+'\n')
        channels[side] = values
        print(name, side, result.returncode, repr(result.stdout), repr(result.stderr), flush=True)
        assert result.returncode == 0
    if name == 'reference_conversion_without_description':
        assert channels['oracle'] == channels['crate'], channels
    else:
        fixtures = root / 'rust/crates/rexx-exec/tests/semantic_boundaries'
        for channel, value in channels['crate'].items():
            assert value == (fixtures / ('reference-conversion-raises-in-clause.' + channel)).read_bytes()
        assert channels['oracle']['stdout'] != channels['crate']['stdout']
        assert channels['oracle']['stderr'] == channels['crate']['stderr'] == b''
print('companion: three exact oracle/pin channels; full ruled fixture: three exact pin channels')
