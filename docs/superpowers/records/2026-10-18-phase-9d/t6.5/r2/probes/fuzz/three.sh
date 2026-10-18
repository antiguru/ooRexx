#!/bin/bash
# usage: three PROG.rex -- oracle vs base (r1) vs head, per scenario pair of lines
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad
P=$(readlink -f "$1")
O=$(mktemp -d $S/t65/run/three-o.XXXX); B=$(mktemp -d $S/t65/run/three-b.XXXX); H=$(mktemp -d $S/t65/run/three-h.XXXX)
( cd $O && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 60 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$P" >$O.out 2>&1 )
( cd $B && memcap 2G timeout -k 5 60 $S/target-9d-t6-base/release/rexx "$P" >$B.out 2>&1 )
( cd $H && memcap 2G timeout -k 5 60 $S/target-9d-t6/release/rexx "$P" >$H.out 2>&1 )
python3 - $O.out $B.out $H.out <<'PY'
import sys
import re
o,b,h=[open(f,'rb').read() for f in sys.argv[1:]]
def pairs(x): return re.split(rb'\n(?=s\d+ :)', x)
o,b,h=pairs(o),pairs(b),pairs(h)
reg=fixed=both=0
for i,(x,y,z) in enumerate(zip(o,b,h)):
    if x==y and x!=z: reg+=1; print('REGRESSION', x.split(b' :')[0].decode())
    elif x!=y and x==z: fixed+=1
    elif x!=y and x!=z: both+=1
print('scenarios',len(o),'regressions',reg,'fixed',fixed,'still-differ',both)
PY
