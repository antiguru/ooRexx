import re,sys,os
MODE=os.environ.get('MODE','filefn')
def load(p):
    d={}
    for l in open(p):
        m=re.match(r'\s*([\d,]+)\s+(?:\([^)]*\)\s+)?(\S.*?)\s*$',l)
        if not m or ':' not in m.group(2): continue
        name=m.group(2)
        name=re.sub(r'\s*\[[^\]]*\]$','',name)
        name=re.sub(r'^.*?(?=rust/crates)|trees/[a-z0-9]+/','',name)
        if MODE=='fn': name=name.split(':',1)[1] if not name.startswith('<') else name
        d[name]=d.get(name,0)+int(m.group(1).replace(',',''))
    return d
a=load(sys.argv[1]); b=load(sys.argv[2]); n=int(sys.argv[3]) if len(sys.argv)>3 else 1
rows=sorted(((b.get(k,0)-a.get(k,0),a.get(k,0),b.get(k,0),k) for k in set(a)|set(b)),key=lambda r:-abs(r[0]))
for d,x,y,k in rows[:int(sys.argv[4]) if len(sys.argv)>4 else 30]:
    print(f"{d:+14d} {d/n:+9.1f}/it {x:14d} {y:14d}  {k[:150]}")
