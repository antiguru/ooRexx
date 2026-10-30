import json
import os
from pathlib import Path
import statistics
import subprocess
import time

BASE=Path(__file__).parent
OUT=BASE/'inventory'/'process-exit'
OUT.mkdir(exist_ok=True)
(OUT/'exit.rex').write_text('exit 0\n')
env=dict(os.environ,LD_LIBRARY_PATH='/home/moritz/dev/repos/ooRexx-rust-rewrite/build/lib')
env.pop('REXX_ENGINE',None)
binaries={'oracle':'/home/moritz/dev/repos/ooRexx-rust-rewrite/build/bin/rexx','rust':str(BASE/'rust-rexx')}
rows=[]
for pair in range(-2,7):
    for side in (['oracle','rust'] if pair%2==0 else ['rust','oracle']):
        start=time.perf_counter()
        p=subprocess.run(['taskset','-c','13',binaries[side],str(OUT/'exit.rex')],cwd=OUT,env=env,stdin=subprocess.DEVNULL,capture_output=True,timeout=20)
        rows.append(dict(side=side,pair=pair,wall_seconds=time.perf_counter()-start,exit_code=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode()))
        assert p.returncode==0 and not p.stdout and not p.stderr
summary={s:dict(median_seconds=statistics.median(r['wall_seconds'] for r in rows if r['side']==s and r['pair']>=0),min_seconds=min(r['wall_seconds'] for r in rows if r['side']==s and r['pair']>=0),max_seconds=max(r['wall_seconds'] for r in rows if r['side']==s and r['pair']>=0)) for s in binaries}
(OUT/'runs.json').write_text(json.dumps(rows,indent=2))
(OUT/'summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary,indent=2))
