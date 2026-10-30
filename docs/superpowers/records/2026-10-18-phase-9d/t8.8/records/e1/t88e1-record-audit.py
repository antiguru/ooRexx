from pathlib import Path
import hashlib, json, subprocess
root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1'
directory = root/'docs/superpowers/records/2026-10-11-phase-9a/deviation-probes'
mapping = (directory/'README.md').read_text()
table = (directory/'objname-table.rex').read_text()
other = (directory/'objname4.rex').read_text().splitlines()
stdout = (directory/'objname4.oracle.out').read_text().splitlines()
assert 'internal routine (`r: procedure`)' in mapping
assert 'r: procedure' in table
assert other[9]=='say other()' and other[24]=='::routine other'
assert stdout[7]=='long'
commit = subprocess.check_output(['git','log','-1','--format=%H %s','--',str(directory/'README.md')],cwd=root,text=True).strip()
(records/'carry-in-m3.json').write_text(json.dumps(dict(mapping_commit=commit,source_call_line=10,source_routine_lines=[25,26],oracle_output_line=8,oracle_value=stdout[7],files=[dict(path=str(path.relative_to(root)),sha256=hashlib.sha256(path.read_bytes()).hexdigest()) for path in [directory/'README.md',directory/'objname-table.rex',directory/'objname4.rex',directory/'objname4.oracle.out']]),indent=2)+'\n')
concurrency = (root/'rust/crates/rexx-exec/tests/concurrency_tests.rs').read_text()
licences = (root/'rust/corpus/suite-licences.txt').read_text()
assert '"base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION"' in concurrency
assert 'if (!judged.1 || !hung.is_empty()) && WALL_CLOCK.contains(&row.as_str())' in concurrency
assert 'base/rexxutil/SysSleep\tTEST_SLEEP_DURATION\tFlaky\t' in licences
(records/'gate-483-current-source.txt').write_text('\n'.join(line for line in licences.splitlines() if 'SysSleep' in line)+'\n\n'+concurrency[concurrency.index('fn row_in_both_modes('):concurrency.index('fn row_in_both_modes(')+6000]+'\n')
print('m3 source/output mapping and provenance verified; current gate483 licence and P48 handling verified.')
