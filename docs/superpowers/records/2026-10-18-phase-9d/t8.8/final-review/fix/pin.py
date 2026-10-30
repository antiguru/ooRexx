import hashlib, json, pathlib, shutil, subprocess
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix'
def sha(path):
    with pathlib.Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()
source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
assert source == '2f774eab16567dbc0a3bb758f3e29ee9bade918e', source
assert not subprocess.check_output(['git', 'status', '--porcelain', '--', 'rust'], cwd=root).strip()
old = {
 '/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx': '22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb',
 '/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx': '6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e',
 '/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx': 'ecb06e984ed736233ce0e2ef779b74941488ae64fdec10333dfb2215c309198a',
 '/tmp/codex-p9d-t88-pins/8.8f-r1-rexx': 'b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be',
 '/tmp/codex-p9d-t88-pins/8.8f-rexx': '0c9fb8371ab3d6dcea780194e02c112ed7981447f7f396e606e5f418975583f0',
}
for name, expected in old.items():
    assert sha(name) == expected, name
build = pathlib.Path('/tmp/codex-p9d-t88-target/release/rexx')
pin = pathlib.Path('/tmp/codex-p9d-t88-pins/8.8-final-fix-rexx')
with build.open('rb') as src, pin.open('xb') as dest:
    shutil.copyfileobj(src, dest)
pin.chmod(build.stat().st_mode & 0o777)
assert sha(pin) == sha(build)
files = ['rust/Cargo.toml', 'rust/Cargo.lock'] + json.loads((records/'source-proof.json').read_text())['changed_runtime']
result = dict(pin=str(pin), sha256=sha(pin), source_commit=source,
 dispatch_head='4fe3e24d640702cddaaca25b87efc93012c1928f', source_worktree_clean=True,
 compiler=subprocess.check_output(['rustc', '-Vv'], text=True), build_record='checks/release.json',
 build_record_sha256=sha(records/'checks/release.json'), build_executable=str(build),
 build_sha256=sha(build), source_hashes={f:sha(root/f) for f in files}, preserved_pins=old)
(records/'release-pin.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(result, indent=2))
