import pathlib,subprocess,tempfile,resource,os,sys,json
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
base=root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix'
label=sys.argv[1]; binary=sys.argv[2]
for probe in sorted((base/'probes').glob('*.rex')):
 if len(sys.argv)>3 and probe.stem not in sys.argv[3:]:continue
 for side in ['oracle','baseline','crate']:
  dest=base/label/probe.stem/side;dest.mkdir(parents=True,exist_ok=True)
  cwd=tempfile.mkdtemp(prefix='t88f-')
  env=os.environ.copy()
  if side=='oracle':
   env['LD_LIBRARY_PATH']='/home/moritz/dev/repos/ooRexx/build/lib';cmd=['timeout','-k','5','20','/home/moritz/dev/repos/ooRexx/build/bin/rexx',str(probe)]
   def cap():resource.setrlimit(resource.RLIMIT_AS,(1073741824,1073741824))
  else:cmd=['memcap','2G','timeout','-k','5','20','/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx' if side=='baseline' else binary,str(probe)];cap=None
  with (dest/'stdout').open('wb') as out,(dest/'stderr').open('wb') as err:
   result=subprocess.run(cmd,cwd=cwd,env=env,stdout=out,stderr=err,preexec_fn=cap,timeout=30)
  (dest/'status').write_text(str(result.returncode)+'\n')
  (dest/'command.json').write_text(json.dumps({'cmd':cmd,'cwd':cwd,'oracle_as':1073741824 if side=='oracle' else None}))
  query=pathlib.Path(cwd)/'queries'
  if query.exists():(dest/'queries').write_bytes(query.read_bytes())
  print(label,probe.stem,side,result.returncode,(dest/'stdout').read_bytes(),(dest/'stderr').read_bytes(),flush=True)
