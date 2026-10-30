from pathlib import Path
import json, subprocess, shutil, hashlib, tempfile, os
repo=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records=repo/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2'
source=Path('/tmp/codex-p9d-t88-target/release/rexx');pin=Path('/tmp/codex-p9d-t88-pins/8.8e2-rexx')
assert not pin.exists();shutil.copy2(source,pin)
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
base=Path('/tmp/codex-p9d-t88-pins/8.8e1-rexx')
assert sha(base)=='434d6567458248fdb804eebe9edb715a54ddb9ded650fcca994eed6c4b691e06'
git=lambda *args:subprocess.check_output(['git',*args],cwd=repo,text=True).strip()
manifest=dict(base='a7db381c765fafd81fe775245902d6476aefe072',built_revision=git('rev-parse','HEAD'),runtime_implementation='dbbe93639',source_tree=git('rev-parse','HEAD:rust/crates/rexx-exec/src'),build_record='release-build.json',compiler=subprocess.check_output(['rustc','-vV'],text=True),artifact=dict(path=str(pin),sha256=sha(pin),bytes=pin.stat().st_size),baseline=dict(path=str(base),sha256=sha(base)),performance='All-program own-BASE comparison remains controller-owned; lookup and cold-allocation costs are not assumed free.')
(records/'release-pin.json').write_text(json.dumps(manifest,indent=2)+'\n')
root=Path(tempfile.mkdtemp(prefix='t88e2-release-probes-'))
probes=[]
for name in ['executing_method_after_redefinition','enhanced_metaclass_method_identity']:
 probes.append((name,repo/f'rust/corpus/lang/{name}.rex',records/f'{name}-oracle-run.stdout',records/f'{name}-oracle-run.stderr',0))
for name in ['flags-identity-after-only','flags-identity-prefetch-installed','flags-identity-ask-before']:
 original=records/'original-identity'
 probes.append((name,original/(name+'.rex'),original/(name+'-oracle/stdout'),original/(name+'-oracle/stderr'),int((original/(name+'-oracle/status')).read_text())))
for name,source,out,err,status in probes:
 cwd=root/name;cwd.mkdir();env=os.environ.copy();env['REXX_PATH']=str(cwd);env['TZ']='UTC';env.pop('REXX_SWITCH_MODE',None)
 cmd=['timeout','-k','2s','20s',str(pin),str(source)]
 p=subprocess.run(cmd,cwd=cwd,env=env,input=b'',stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=25)
 prefix=records/(name+'-crate-final')
 prefix.with_suffix('.stdout').write_bytes(p.stdout);prefix.with_suffix('.stderr').write_bytes(p.stderr)
 comparison=dict(stdout_equal=p.stdout==out.read_bytes(),stderr_equal=p.stderr==err.read_bytes(),status_equal=p.returncode==status)
 prefix.with_suffix('.json').write_text(json.dumps(dict(command=cmd,cwd=str(cwd),status=p.returncode,source=str(source),source_sha256=sha(source),binary_sha256=sha(pin),resident_cap='inherited memcap 2G',TZ='UTC',REXX_PATH=str(cwd),comparison=comparison),indent=2)+'\n')
 print(name,p.returncode,comparison,p.stdout.decode())
 assert all(comparison.values())
print(json.dumps(manifest))
