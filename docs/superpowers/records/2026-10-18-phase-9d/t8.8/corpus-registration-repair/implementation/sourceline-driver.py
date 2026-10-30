from pathlib import Path
import tempfile,subprocess,resource,os,json
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite');records=root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/corpus-registration-repair/implementation/sourceline';records.mkdir(exist_ok=True)
source='''parse arg f
signal on syntax name fallback
a = .Package~new(f)~source
call report a
exit 0
fallback:
lines = .array~new
do while lines(f) \\== 0
  lines~append(linein(f))
end
call stream f, 'c', 'close'
call report lines
exit 0
report: procedure
use arg a
say '%SRCG%COUNT' a~items
do i = 1 to a~items
  say '%SRCG%L' || a[i]
end
return
'''
for name in ['reference_conversion_raises_in_clause','reference_conversion_without_description']:
 cwd=Path(tempfile.mkdtemp(prefix='t88f-src-'));driver=cwd/'source.rex';driver.write_text(source)
 env=os.environ.copy();env['LD_LIBRARY_PATH']='/home/moritz/dev/repos/ooRexx/build/lib'
 def cap():resource.setrlimit(resource.RLIMIT_AS,(1073741824,1073741824))
 command=['timeout','-k','5','20','/home/moritz/dev/repos/ooRexx/build/bin/rexx',str(driver),str(root/'rust/corpus/lang'/f'{name}.rex')]
 result=subprocess.run(command,cwd=cwd,env=env,preexec_fn=cap,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 (records/f'{name}.stdout').write_bytes(result.stdout);(records/f'{name}.stderr').write_bytes(result.stderr);(records/f'{name}.json').write_text(json.dumps({'command':command,'cwd':str(cwd),'status':result.returncode}))
 assert result.returncode==0
 lines=result.stdout.splitlines();count=[x[len(b'%SRCG%COUNT '):] for x in lines if x.startswith(b'%SRCG%COUNT ')];assert len(count)==1
 output=b'count '+count[0]+b'\n'+b'\n'.join(x[len(b'%SRCG%L'):] for x in lines if x.startswith(b'%SRCG%L'))+b'\n'
 (root/'rust/crates/rexx-parse/tests/sourceline_oracle'/f'{name}.txt').write_bytes(output)
 print(name,result.returncode,count[0])
