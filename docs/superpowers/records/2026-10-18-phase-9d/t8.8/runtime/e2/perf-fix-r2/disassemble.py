from pathlib import Path
import subprocess, json, hashlib, sys
pin, name = sys.argv[1:]
records = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2')
nm = ['nm', '-S', '-C', pin]
symbols = subprocess.check_output(nm, text=True)
row = next(line.split(maxsplit=3) for line in symbols.splitlines() if line.endswith(' <rexx_exec::Interp>::begin_invoke'))
address, size = int(row[0], 16), int(row[1], 16)
command = ['objdump', '-d', '-C', '-l', '--no-show-raw-insn', f'--start-address={address:#x}', f'--stop-address={address+size:#x}', pin]
result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
(records / f'{name}.asm').write_bytes(result.stdout)
(records / f'{name}.stderr').write_bytes(result.stderr)
(records / f'{name}.json').write_text(json.dumps(dict(pin=pin,sha256=hashlib.sha256(Path(pin).read_bytes()).hexdigest(),nm_command=nm,command=command,status=result.returncode,address=hex(address),size=size),indent=2)+'\n')
print(name, hex(address), size)
print('\n'.join(line for line in result.stdout.decode().splitlines() if any(word in line for word in ['call   ', 'memcpy@']) and any(word in line for word in ['cold_mut', 'push_activation', 'memcpy@'])))
