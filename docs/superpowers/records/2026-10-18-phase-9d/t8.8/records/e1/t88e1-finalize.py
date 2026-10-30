from pathlib import Path
import hashlib, json, shutil, subprocess
root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1'
pin = Path('/tmp/codex-p9d-t88-pins')
pin.mkdir(exist_ok=True)
artifacts = []
for name in ['rexx','rexxc']:
    source = Path('/tmp/codex-p9d-t88-target/release')/name
    target = pin/f'8.8e1-{name}'
    shutil.copy2(source,target)
    artifacts.append(dict(path=str(target),sha256=hashlib.sha256(target.read_bytes()).hexdigest(),bytes=target.stat().st_size))
assert not subprocess.check_output(['git','diff','7c6e12e84','1d8b03220','--','rust/crates/rexx-exec/src','rust/crates/rexx-parse/src','rust/crates/rexx-api/src'],cwd=root)
provenance = dict(base='dedfab2ea41aa419d993aca1d78bc1749a623620',built_revision='7c6e12e84',final_source_revision='1d8b03220',runtime_source_equivalence='git diff 7c6e12e84 1d8b03220 -- rust/crates/{rexx-exec,rexx-parse,rexx-api}/src is empty',artifacts=artifacts)
for path in [Path('/tmp/codex-p9d-t88-target/debug/rexx'),Path('/tmp/p9d-t88-fixed-20261030-a/rexx-frozen')]:
    provenance.setdefault('diagnostic_binaries',[]).append(dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
(records/'release-pin.json').write_text(json.dumps(provenance,indent=2)+'\n')
paths = [root/'docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv',records/'closed.tsv',records/'inventory-file-state.json',records/'t88e1-dispositions.py']
paths += [p for p in (root/'.superpowers/sdd/queued').iterdir() if p.is_file() and 'Task 8.8e1' in p.read_text()]
for path in paths:
    path.write_text(path.read_text().replace('7c6e12e84','1d8b03220'))
baseline = subprocess.check_output(['git','show','dedfab2ea:docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv'],cwd=root,text=True)
before = {r[0]:r for r in [line.split('\t') for line in baseline.splitlines()]}
after = {r[0]:r for r in [line.split('\t') for line in (root/'docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv').read_text().splitlines()]}
assert before.keys()==after.keys()
for key in before:
    assert all(before[key][index]==after[key][index] for index in [0,1,2,4]),key
(records/'inventory-reasons-preserved.json').write_text(json.dumps(dict(base='dedfab2ea',identity_kind_probe_and_reason_unchanged=True),indent=2)+'\n')
shutil.copy2('/tmp/t88e1-finalize.py',records/'t88e1-finalize.py')
print(json.dumps(provenance,indent=2))
