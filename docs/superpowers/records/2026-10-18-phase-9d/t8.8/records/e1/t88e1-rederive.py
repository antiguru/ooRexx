import pathlib, subprocess
deps = pathlib.Path('/tmp/codex-p9d-t88-target/debug/deps')
library = max(deps.glob('librexx_extract-*.rlib'), key=lambda p:p.stat().st_mtime)
command = ['rustc','--edition=2024','-A','dead_code','/tmp/t88e1-rederive.rs','-L',f'dependency={deps}','--extern',f'rexx_extract={library}','-o','/tmp/t88e1-rederive']
subprocess.run(command, check=True, timeout=60)
subprocess.run(['/tmp/t88e1-rederive'], check=True, timeout=30)
