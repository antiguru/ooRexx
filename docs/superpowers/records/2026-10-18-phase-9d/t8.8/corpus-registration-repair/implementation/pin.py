import hashlib, json, pathlib, shutil, subprocess
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/corpus-registration-repair/implementation'
pins = pathlib.Path('/tmp/codex-p9d-t88-pins')
expected = 'd9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145'
def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()
source = subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
assert source == 'f65066b04fe1f2355ce64d8be581fa9e79641673'
assert not subprocess.check_output(['git','status','--porcelain','--','rust'],cwd=root).strip()
proof = json.loads((records/'source-proof.json').read_text()); assert proof['source_commit']==source
artifact = [json.loads(line) for line in (records/'checks/release.stdout').read_text().splitlines() if line.startswith('{')]
cli = [a for a in artifact if a.get('reason')=='compiler-artifact' and a.get('executable') and a['target']['name']=='rexx']
assert len(cli)==1 and cli[0]['features']==[] and not cli[0]['profile']['test']
build = pathlib.Path(cli[0]['executable'])
old = {str(path):sha(path) for path in sorted(pins.iterdir()) if path.is_file()}
assert old[str(pins/'8.8-final-fix-r2-rexx')]==expected
assert sha(build)==expected, ('STOP: production artifact differs',sha(build))
pin = pins/'8.8-corpus-registration-rexx'
with build.open('rb') as src, pin.open('xb') as dest:
    shutil.copyfileobj(src,dest)
pin.chmod(build.stat().st_mode & 0o777)
assert sha(pin)==expected
for name,value in old.items():
    assert sha(pathlib.Path(name))==value, name
result = dict(source_commit=source, base=proof['base'], pin=str(pin), sha256=sha(pin),
    measured_normal_pin=str(pins/'8.8-final-fix-r2-rexx'), measured_normal_sha256=expected,
    byte_identical_to_measured_normal=True, build_executable=str(build), build_sha256=sha(build),
    compiler=subprocess.check_output(['rustc','-Vv'],text=True), cargo_artifact=cli[0],
    build_record='checks/release.json', build_record_sha256=sha(records/'checks/release.json'),
    build_output_sha256=sha(records/'checks/release.stdout'), source_proof_sha256=sha(records/'source-proof.json'),
    production_manifest_sha256=sha(records/'production-manifest.json'), preserved_pins=old)
(records/'release-pin.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['preserved_pins','cargo_artifact']},indent=2))
print(f'Preserved and rehashed {len(old)} older pin files')
