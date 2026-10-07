# usage: attr.py PROG N  (in prof dir): file-level s1 vs head delta per iteration, and head Ir on lines added since s1
import sys,collections
p,n=sys.argv[1],int(sys.argv[2])
def load(v):
    d={}
    for l in open(f'{p}.{v}.lines.tsv'):
        f,ln,ir=l.rstrip('\n').split('\t'); d[(f,int(ln))]=int(ir)
    return d
a,b=load('s1'),load('head')
fa,fb=collections.Counter(),collections.Counter()
for (f,_),ir in a.items(): fa[f]+=ir
for (f,_),ir in b.items(): fb[f]+=ir
tot=sum(b.values())-sum(a.values())
print(f"total delta {tot:+d} = {tot/n:+.1f}/it")
print("-- per file delta (|>=1/it|)")
for f in sorted(set(fa)|set(fb),key=lambda f:-abs(fb[f]-fa[f])):
    d=fb[f]-fa[f]
    if abs(d)/n>=1: print(f"{d/n:+8.1f}/it  {f}")
added=set()
for l in open('added.tsv'):
    f,ln=l.split('\t'); added.add((f,int(ln)))
rows=[(ir,f,ln) for (f,ln),ir in b.items() if (f,ln) in added and ir/n>=0.5]
print(f"-- head Ir on added lines: {sum(r[0] for r in rows)/n:+.1f}/it")
src='/tmp/claude-1000/p6-t26/trees/head/rust/crates/'
for ir,f,ln in sorted(rows,reverse=True)[:40]:
    try: t=open(src+f).read().split('\n')[ln-1].strip()[:80]
    except Exception: t='?'
    print(f"{ir/n:8.1f}/it  {f}:{ln}  {t}")
