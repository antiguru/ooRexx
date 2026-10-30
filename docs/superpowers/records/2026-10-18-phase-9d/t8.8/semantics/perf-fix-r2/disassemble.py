import pathlib, subprocess, json, re, hashlib
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
out = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/perf-fix-r2/disassembly'
out.mkdir(exist_ok=True)
result = {}
for label, pin in [('a', '/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx'), ('p', '/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx'), ('q', '/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx')]:
    symbol_command = ['nm', '-S', '-C', pin]
    symbols = subprocess.check_output(symbol_command, text=True)
    rows = [line for line in symbols.splitlines() if line.endswith(' <rexx_exec::Interp>::exec_parse')]
    assert len(rows) == 1, rows
    fields = rows[0].split()
    start, size = int(fields[0], 16), int(fields[1], 16)
    command = ['objdump', '-d', '-l', '-C', f'--start-address={start}', f'--stop-address={start+size}', pin]
    asm = subprocess.check_output(command, text=True)
    (out / f'{label}.parse.asm').write_text(asm)
    stack = re.search(r'sub\s+\$(0x[0-9a-f]+),%rsp', asm).group(1)
    with open(pin, 'rb') as binary:
        digest = hashlib.file_digest(binary, 'sha256').hexdigest()
    result[label] = dict(pin=pin, sha256=digest, symbol_command=symbol_command, disassembly_command=command,
                         start=hex(start), size=size, stack_reservation=stack)
(out / 'symbols.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(result, indent=2))
