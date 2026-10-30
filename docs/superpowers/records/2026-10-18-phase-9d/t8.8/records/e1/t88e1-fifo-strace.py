from pathlib import Path
import json, os, subprocess, tempfile
root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1'
work = Path(tempfile.mkdtemp(prefix='t88e1-fifo-strace-'))
fifo = work / 'pipe'
os.mkfifo(fifo)
reader = os.open(fifo, os.O_RDONLY | os.O_NONBLOCK)
program = work / 'probe.rex'
program.write_text(f"s=.Stream~new('{fifo}')\nsay 'write' s~lineout('hello')\nsay 'close' s~lineout\n")
trace = records / 'fifo-oracle.trace'
wrapper = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/diagnostics/fixed-rechecks/oracle.sh'
command = ['strace','-f','-yy','-e','trace=read,write,lseek,close','-o',str(trace),'bash',str(wrapper),str(program)]
result = subprocess.run(command, cwd=work, input=b'', stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=25)
payload = os.read(reader, 65536)
os.close(reader)
(records/'fifo-strace.source.rex').write_bytes(program.read_bytes())
(records/'fifo-strace.stdout').write_bytes(result.stdout)
(records/'fifo-strace.stderr').write_bytes(result.stderr)
(records/'fifo-strace.payload').write_bytes(payload)
(records/'fifo-strace.json').write_text(json.dumps(dict(command=command,cwd=str(work),status=result.returncode,payload_hex=payload.hex()),indent=2)+'\n')
print(result.returncode, result.stdout, result.stderr, payload)
for line in trace.read_text().splitlines():
    if str(fifo) in line:
        print(line)
