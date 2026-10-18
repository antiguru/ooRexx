#!/bin/bash
# usage: two PROG.rex -- oracle vs crate, compared per scenario
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad
P=$(readlink -f "$1")
O=$(mktemp -d $S/t65/run/two-o.XXXX); H=$(mktemp -d $S/t65/run/two-h.XXXX)
( cd $O && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 120 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$P" >$O.out 2>&1 )
( cd $H && memcap 2G timeout -k 5 120 $S/target-9d-t6/release/rexx "$P" >$H.out 2>&1 )
python3 - $O.out $H.out <<'PY'
import sys,re
o,h=[re.split(rb'\n(?=s\d+ :)', open(f,'rb').read()) for f in sys.argv[1:]]
d=[(x,z) for x,z in zip(o,h) if x!=z]
for x,z in d: print('DIFF\n  oracle:', x.decode(errors='replace')[:400], '\n  crate: ', z.decode(errors='replace')[:400])
print('scenarios',len(o),'differ',len(d), 'lengths', len(o), len(h))
PY
