import subprocess,resource,tempfile,os,pathlib,json
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite');records=root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/remaining-oracle';records.mkdir(exist_ok=True)
for source in [root/'rust/crates/rexx-exec/tests/semantic_boundaries/hash-raises.rex',root/'rust/corpus/lang/required_string_boundaries.rex',root/'rust/corpus/lang/raise_additional_untraced.rex']:
 cwd=tempfile.mkdtemp(prefix='t88f-remaining-');env=os.environ.copy();env['LD_LIBRARY_PATH']='/home/moritz/dev/repos/ooRexx/build/lib'
 def cap():resource.setrlimit(resource.RLIMIT_AS,(1073741824,1073741824))
 cmd=['timeout','-k','5','20','/home/moritz/dev/repos/ooRexx/build/bin/rexx',str(source)]
 result=subprocess.run(cmd,cwd=cwd,env=env,preexec_fn=cap,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 d=records/source.stem;d.mkdir(exist_ok=True)
 for name,data in [('source.rex',source.read_bytes()),('stdout',result.stdout),('stderr',result.stderr)]: (d/name).write_bytes(data)
 (d/'descriptor.json').write_text(json.dumps({'command':cmd,'cwd':cwd,'status':result.returncode,'as_limit':1073741824})+'\n')
 print(source.stem,result.returncode,result.stdout,result.stderr)
