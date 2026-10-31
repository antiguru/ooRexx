import hashlib, json, pathlib, re, subprocess
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/corpus-registration-repair/implementation'
base = '5091eba39531c661c407c0d6a985a99b60d281a1'
def git(*args):
    return subprocess.check_output(['git', *args], cwd=root)
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
expected = [
 'rust/corpus/lang/reference_conversion_raises_in_clause.rex',
 'rust/corpus/lang/reference_conversion_without_description.rex',
 'rust/corpus/phase-9.txt', 'rust/corpus/unfiled.txt',
 'rust/crates/rexx-exec/tests/semantic_boundaries.rs',
 *['rust/crates/rexx-exec/tests/semantic_boundaries/reference-conversion-raises-in-clause.' + ext for ext in ['rex','stdout','stderr','status']],
 'rust/crates/rexx-parse/tests/sourceline_oracle/reference_conversion_raises_in_clause.txt',
 'rust/crates/rexx-parse/tests/sourceline_oracle/reference_conversion_without_description.txt',
]
changed = git('diff','--name-only',base,'HEAD','--','rust').decode().splitlines()
assert sorted(changed) == sorted(expected), changed
assert not git('status','--porcelain','--','rust').strip()
def tree(revision):
    answer = {}
    for entry in git('ls-tree','-r',revision,'--','rust').decode().splitlines():
        meta, name = entry.split('\t')
        if '/src/' in name or pathlib.Path(name).name in ['Cargo.toml','Cargo.lock','build.rs'] or '/.cargo/' in name:
            answer[name] = meta
    return answer
before = tree(base); after = tree('HEAD'); assert before == after
production = {name: dict(git=meta, sha256=sha(root/name)) for name,meta in after.items()}
(records/'production-manifest.json').write_text(json.dumps(production,indent=2)+'\n')
program = 'rust/corpus/lang/reference_conversion_raises_in_clause.rex'
assert git('show',base+':'+program).splitlines()[2:] == (root/program).read_bytes().splitlines()[2:]
harness = 'rust/crates/rexx-exec/tests/semantic_boundaries.rs'
names = lambda source: re.findall(rb'#\[test\]\s*fn (\w+)\(',source,re.M)
assert names(git('show',base+':'+harness)) == names((root/harness).read_bytes())
corpus = root/'rust/corpus'
runner = (root/'rust/crates/rexx-exec/tests/corpus.rs').read_text()
subsets = re.findall(r'"(phase-[^"\n]+\.txt)"',runner.split('const SUBSET_FILES:')[1].split('];')[0])
filed = set()
for name in subsets:
    filed.update(line.strip() for line in (corpus/name).read_text().splitlines() if line.strip() and not line.lstrip().startswith('#'))
unfiled = [line.split('\t')[0] for line in (corpus/'unfiled.txt').read_text().splitlines() if line.strip() and not line.startswith('#')]
assert len(filed)==1242, len(filed)
assert len(unfiled)==4, unfiled
assert not filed.intersection(unfiled)
fixtures = list((root/'rust/crates/rexx-exec/tests/semantic_boundaries').glob('*.rex')); assert len(fixtures)==42
result = dict(base=base, source_commit=git('rev-parse','HEAD').decode().strip(), changed_rust=changed,
 all_production_and_cargo_inputs_unchanged=True, production_manifest_files=len(production), production_manifest_sha256=sha(records/'production-manifest.json'),
 original_executable_lines_preserved=True, semantic_test_names_unchanged=[n.decode() for n in names((root/harness).read_bytes())],
 differential_programs=len(filed), unfiled_programs=unfiled, semantic_fixtures=len(fixtures), source_hashes={name:sha(root/name) for name in changed})
(records/'source-proof.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k!='source_hashes'},indent=2))
