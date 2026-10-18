#!/bin/bash
# usage: yamlrun.sh WORK CLS   -- lays out the yaml group's crate-side run with CLS as bin-rexx/yaml.cls and runs it
set -u
R=/home/moritz/dev/repos/ooRexx-rust-rewrite
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad
W=$1; CLS=$2
mkdir -p "$W/run/ooRexx/extensions" "$W/bin-rexx" || exit 1
cp -r "$R/ootest/framework" "$W/run/framework"
cp -r "$R/ootest/ooRexx/extensions/yaml" "$W/run/ooRexx/extensions/yaml"
for f in testOORexx.rex worker.rex ooTest.frm; do cp "$R/ootest/$f" "$W/run/$f"; done
python3 -I - "$W/run/ooTest.frm" <<'PY'
import sys
p=sys.argv[1]; lines=open(p,'rb').read().split(b'\n')
hits=[i for i,l in enumerate(lines) if b'rxfuncquery(' in l]
assert hits
drop=set()
for i in hits:
    assert lines[i-1].lstrip().startswith(b'--') and lines[i+1].lstrip().startswith(b'versions[')
    drop.update([i-1,i,i+1])
open(p,'wb').write(b'\n'.join(l for i,l in enumerate(lines) if i not in drop))
PY
cp "$R/extensions/rxregexp/rxregexp.cls" "$W/run/rxregexp.cls"
cp "$S/t4/bin/rexx-fix" "$W/bin-rexx/rexx"
cp /home/moritz/dev/repos/ooRexx/build/bin/*.cls "$W/bin-rexx/"
cp "$CLS" "$W/bin-rexx/yaml.cls"
ln -s /home/moritz/dev/repos/ooRexx/build/samples "$W/samples"
cd "$W/run" || exit 1
PATH="$W/bin-rexx:/usr/local/bin:/usr/bin:/bin" LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/.local/bin/memcap 2G timeout --foreground -s KILL 300 "$W/bin-rexx/rexx" "$W/run/testOORexx.rex" -f "$W/run/ooRexx/extensions/yaml/yaml.testGroup" -U -V 2 > "$W/stdout.txt" 2> "$W/stderr.txt"
echo "rc=$?"
