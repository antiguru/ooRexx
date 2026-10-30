import hashlib,json,pathlib,os
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix')
files=list((root/'red-physical').rglob('*'))+[root/'checks/red-physical-probes.stdout']
manifest=[]
for p in files:
 if not p.is_file() or p.stat().st_size<=65536:continue
 size=p.stat().st_size;h=hashlib.sha256()
 with p.open('rb') as f:
  for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
 with p.open('rb') as f:
  head=f.read(4096);f.seek(-4096,2);tail=f.read()
 sample=p.with_name(p.name+'.head-tail')
 sample.write_bytes(head+b'\n[Infrastructure-only self-reading output omitted. See cleanup manifest.]\n'+tail)
 manifest.append(dict(path=str(p.relative_to(root)),bytes=size,sha256=h.hexdigest(),sample=str(sample.relative_to(root))))
 p.unlink()
(root/'infrastructure-cleanup.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(files=len(manifest),bytes_removed=sum(x['bytes'] for x in manifest),available_tmp_bytes=os.statvfs('/tmp').f_bavail*os.statvfs('/tmp').f_frsize)))
