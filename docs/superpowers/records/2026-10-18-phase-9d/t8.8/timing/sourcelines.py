import json, pathlib, subprocess, tempfile, time
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/sourceline'
records.mkdir(parents=True, exist_ok=True)
driver = 'parse arg f\nsignal on syntax name fallback\na = .Package~new(f)~source\ncall report a\nexit 0\n\nfallback:\n  lines = .array~new\n  do while lines(f) \\== 0\n    lines~append(linein(f))\n  end\n  call stream f, "c", "close"\n  call report lines\n  exit 0\n\nreport: procedure\n  use arg a\n  say "%SRCG%COUNT" a~items\n  do i = 1 to a~items\n    say "%SRCG%L" || a[i]\n  end\n  return\n'
for name in ['rexxutil_file_date_time', 'command_redirected_target_waits_elsewhere']:
    directory = pathlib.Path(tempfile.mkdtemp(prefix='t88c-sourceline-'))
    script = directory / 'srclines.rex'
    script.write_text(driver)
    command = ['bash', '/tmp/p9d-t88-dst-20261030-a/oracle.sh', str(script), str(root / 'rust/corpus/lang' / (name + '.rex'))]
    began = time.monotonic()
    result = subprocess.run(command, cwd=directory, stdin=subprocess.DEVNULL, capture_output=True, timeout=20)
    (records / (name + '.stdout')).write_bytes(result.stdout)
    (records / (name + '.stderr')).write_bytes(result.stderr)
    (records / (name + '.json')).write_text(json.dumps(dict(command=command,cwd=str(directory),status=result.returncode,elapsed=time.monotonic()-began), indent=2)+'\n')
    assert result.returncode == 0, result.stderr
    lines = result.stdout.splitlines()
    count = next(line.removeprefix(b'%SRCG%COUNT ') for line in lines if line.startswith(b'%SRCG%COUNT '))
    source = [line.removeprefix(b'%SRCG%L') for line in lines if line.startswith(b'%SRCG%L')]
    assert len(source) == int(count)
    (root / 'rust/crates/rexx-parse/tests/sourceline_oracle' / (name+'.txt')).write_bytes(b'count '+count+b'\n'+b'\n'.join(source)+b'\n')
    print(name, result.returncode, count.decode())
(records / 'srclines.rex').write_text(driver)
