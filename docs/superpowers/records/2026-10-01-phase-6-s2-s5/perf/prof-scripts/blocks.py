# usage: blocks.py INSTR_TSV N [LINE0ONLY] -> runs of instructions with equal exec count
import sys
N=float(sys.argv[2]); only0=len(sys.argv)>3
rows=[]
for l in open(sys.argv[1]):
    a,ln,f,ir=l.rstrip('\n').split('\t'); rows.append((int(a,16),int(ln),f,int(ir)/N))
runs=[]; cur=None
for a,ln,f,c in rows:
    if c<0.5: cur=None; continue
    if cur and a-cur['end']<=16 and abs(c-cur['c'])<0.01:
        cur['end']=a; cur['n']+=1; cur['z']+= (ln==0)
    else:
        cur={'start':a,'end':a,'n':1,'c':c,'z':int(ln==0)}; runs.append(cur)
runs=[r for r in runs if not only0 or r['z']]
for r in sorted(runs,key=lambda r:-(r['z']*r['c']))[:30]:
    print(f"{r['start']:x}..{r['end']:x} n={r['n']} line0={r['z']} exec/it={r['c']:.2f} line0Ir/it={r['z']*r['c']:.1f}")
