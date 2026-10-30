from pathlib import Path
import hashlib, json, os, shutil, subprocess
root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
tests = root/'rust/crates/rexx-exec/tests'
records = root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1'
work = Path('/tmp/t88e1-yaml-harness')
work.mkdir(exist_ok=True)
for path in tests.iterdir():
    if path.name != 'suite.rs' and not (work/path.name).exists():
        (work/path.name).symlink_to(path, target_is_directory=path.is_dir())
text = (tests/'suite.rs').read_text()
old = 'fs::copy(&file, &to).unwrap_or_else(|e| panic!("cannot copy {}: {e}", file.display()));'
new = '''let chosen = if file.file_name().is_some_and(|name| name == "yaml.cls") {
                PathBuf::from(std::env::var_os("REXX_DIAGNOSTIC_YAML").expect("explicit yaml copy"))
            } else { file.clone() };
            fs::copy(&chosen, &to).unwrap_or_else(|e| panic!("cannot copy {}: {e}", chosen.display()));'''
assert text.count(old) == 1
text = text.replace(old, new)
(work/'suite.rs').write_text(text)
(records/'yaml-harness-overlay.json').write_text(json.dumps(dict(source=str(tests/'suite.rs'),old=old,new=new,base_sha256=hashlib.sha256((tests/'suite.rs').read_bytes()).hexdigest()),indent=2)+'\n')
deps = Path('/tmp/codex-p9d-t88-target/debug/deps')
env = os.environ.copy()
env.update(CARGO_MANIFEST_DIR=str(tests.parent), CARGO_TARGET_TMPDIR=str(work/'targettmp'),
           CARGO_BIN_EXE_rexx='/tmp/codex-p9d-t88-target/debug/rexx',
           CARGO_BIN_EXE_rexxc='/tmp/codex-p9d-t88-target/debug/rexxc')
command = ['rustc','--edition=2024','--test','-C','opt-level=3',str(work/'suite.rs'),'-L',f'dependency={deps}','-o',str(work/'yaml-suite-exec')]
for crate in ['rexx_exec','rexx_extract','rexx_parse','rexx_api','rexx_core','rustix','rayon','libc','num_bigint','num_traits']:
    found = list(deps.glob(f'lib{crate}-*.rlib'))
    if found:
        path = max(found,key=lambda path:path.stat().st_mtime)
        command += ['--extern', f'{crate}={path}']
subprocess.run(command,env=env,check=True,timeout=120)
for name, source in [('installed',Path('/home/moritz/dev/repos/ooRexx/build/bin/yaml.cls')),('repository',root/'extensions/yaml/yaml.cls')]:
    env.update(REXX_DIAGNOSTIC_YAML=str(source),REXX_SUITE_GATE='1',REXX_SUITE_GROUPS='extensions/yaml/yaml',REXX_SUITE_TABLE=str(records/f'yaml-{name}/table.tsv'))
    folder = records/f'yaml-{name}'
    folder.mkdir(exist_ok=True)
    with (folder/'harness.stdout').open('w') as out, (folder/'harness.stderr').open('w') as err:
        result = subprocess.run([str(work/'yaml-suite-exec'),'--exact','l3_core_suite','--nocapture'],cwd=root/'rust',env=env,stdout=out,stderr=err,timeout=150)
    entries = (folder/'tests.tsv').read_text()
    for test in ['TEST35_DIRECTIVES','TEST47_MULTILINEFLOWCOLLECTIONS']:
        matched = [line for line in entries.splitlines() if '\t'+test+'\t' in line]
        assert len(matched)==1 and matched[0].endswith('\tpassed\tpassed'), matched
        print(name, matched[0],flush=True)
    metadata = dict(status=result.returncode,source=str(source),sha256=hashlib.sha256(source.read_bytes()).hexdigest(),environment={key:value for key,value in env.items() if key.startswith('REXX_')})
    (folder/'descriptor.json').write_text(json.dumps(metadata,indent=2)+'\n')
    assert result.returncode == 0, (folder/'harness.stderr').read_text()
print('Both actual named yaml methods pass in the full yaml group against both class copies.')
