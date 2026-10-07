# usage: fmap.py PROG N FILE [MIN]: per-line s1 vs head Ir for FILE, lines aligned by difflib
import sys,difflib
p,n,f=sys.argv[1],int(sys.argv[2]),sys.argv[3]; mn=float(sys.argv[4]) if len(sys.argv)>4 else 2
T='/tmp/claude-1000/p6-t26/trees/%s/rust/crates/'+f
A=open(T%'s1').read().split('\n'); B=open(T%'head').read().split('\n')
def load(v):
    d={}
    for l in open(f'{p}.{v}.lines.tsv'):
        g,ln,ir=l.rstrip('\n').split('\t')
        if g==f: d[int(ln)]=int(ir)
    return d
a,b=load('s1'),load('head')
m={}
for blk in difflib.SequenceMatcher(None,A,B,autojunk=False).get_matching_blocks():
    for i in range(blk.size): m[blk.a+i+1]=blk.b+i+1
rows=[]
used=set()
for la,ir in a.items():
    lb=m.get(la); irb=b.get(lb,0) if lb else 0
    if lb: used.add(lb)
    rows.append((irb-ir,la,lb,ir,irb))
for lb,ir in b.items():
    if lb not in used: rows.append((ir,None,lb,0,ir))
print(f"{f}: total {(sum(b.values())-sum(a.values()))/n:+.1f}/it")
for d,la,lb,x,y in sorted(rows,key=lambda r:-abs(r[0])):
    if abs(d)/n<mn: break
    t=(B[lb-1] if lb else (A[la-1] if la else "<line 0>")).strip()[:85]
    print(f"{d/n:+8.1f}/it  s1:{la} head:{lb}  {x/n:6.1f}->{y/n:6.1f}  {t}")
