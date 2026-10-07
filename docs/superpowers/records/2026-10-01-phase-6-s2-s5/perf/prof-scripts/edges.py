# usage: edges.py CG -> "caller\tcallee\tcalls\tinclIr"
import sys,re
fn=None; cfn=None; pend=None; E={}
for l in open(sys.argv[1]):
    l=l.rstrip('\n')
    if pend is not None:
        ir=int(l.split()[1]) if len(l.split())>1 else 0
        k=(fn,cfn); c,i=E.get(k,(0,0)); E[k]=(c+pend,i+ir); pend=None; continue
    if l.startswith('fn='): fn=l[3:]
    elif l.startswith('cfn='): cfn=l[4:]
    elif l.startswith('calls='): pend=int(l[6:].split()[0])
for (a,b),(c,i) in E.items(): print(f"{a}\t{b}\t{c}\t{i}")
