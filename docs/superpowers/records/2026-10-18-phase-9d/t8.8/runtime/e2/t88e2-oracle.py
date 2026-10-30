from pathlib import Path
import subprocess, tempfile, json, os, hashlib
repo=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records=repo/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2'
root=Path(tempfile.mkdtemp(prefix='t88e2-oracle-'))
wrapper=root/'oracle.sh'
wrapper.write_text('#!/bin/bash\nulimit -v 1048576\nexport LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib\nexec timeout -k 2s 20s /home/moritz/dev/repos/ooRexx/build/bin/rexx "$@"\n')
srclines=root/'srclines.rex'
srclines.write_text('parse arg f\nsignal on syntax name fallback\na = .Package~new(f)~source\ncall report a\nexit 0\nfallback:\nlines = .array~new\ndo while lines(f) \\== 0\nlines~append(linein(f))\nend\ncall stream f, "c", "close"\ncall report lines\nexit 0\nreport: procedure\nuse arg a\nsay "%SRCG%COUNT" a~items\ndo i = 1 to a~items\nsay "%SRCG%L" || a[i]\nend\nreturn\n')
for name in ['executing_method_after_redefinition', 'enhanced_metaclass_method_identity']:
 source=repo/f'rust/corpus/lang/{name}.rex'
 for mode in ['run','sourceline']:
  cwd=root/f'{name}-{mode}';cwd.mkdir()
  cmd=['bash',str(wrapper),str(source)] if mode=='run' else ['bash',str(wrapper),str(srclines),str(source)]
  p=subprocess.run(cmd,cwd=cwd,input=b'',stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=25)
  base=records/f'{name}-oracle-{mode}'
  base.with_suffix('.stdout').write_bytes(p.stdout);base.with_suffix('.stderr').write_bytes(p.stderr)
  base.with_suffix('.json').write_text(json.dumps(dict(command=cmd,cwd=str(cwd),status=p.returncode,source_sha256=hashlib.sha256(source.read_bytes()).hexdigest()),indent=2)+'\n')
  print(name,mode,p.returncode,p.stdout.decode() if mode=='run' else 'captured',p.stderr.decode())
  assert p.returncode==0
  if mode=='sourceline':
   lines=p.stdout.splitlines(); count=next(l.removeprefix(b'%SRCG%COUNT ') for l in lines if l.startswith(b'%SRCG%COUNT ')); contents=b'count '+count+b'\n'+b'\n'.join(l.removeprefix(b'%SRCG%L') for l in lines if l.startswith(b'%SRCG%L'))+b'\n'
   (root/f'{name}.txt').write_bytes(contents)
print(root)
