# usage: lines.py CGFILE -> prints "file\tline\tIr" self cost per source line (rust/crates paths only normalised)
import sys,re,os
UPD=os.environ.get('UPD')=='1'
FN0=os.environ.get('FN0')=='1'; fn=''
def norm(f):
    i=f.find('rust/crates/')
    return f[i+len('rust/crates/'):] if i>=0 else f
files={}; cost={}
fl=fi=None; last=0; skip=False
def name(tok):
    m=re.match(r'\((\d+)\)(?: (.*))?',tok)
    if m:
        if m.group(2) is not None: files[m.group(1)]=m.group(2)
        return files[m.group(1)]
    return tok
for l in open(sys.argv[1]):
    l=l.rstrip('\n')
    if not l: continue
    c=l[0]
    if c.isdigit() or c in '+-*':
        parts=l.split()
        p=parts[0]
        if p=='*': ln=last
        elif p[0] in '+-': ln=last+int(p)
        else: ln=int(p)
        last=ln
        if skip: skip=False; continue
        ir=int(parts[1]) if len(parts)>1 else 0
        k=(norm(fi or fl),ln if (ln or not FN0) else 'L0:'+fn); cost[k]=cost.get(k,0)+ir
        continue
    if l.startswith('fl='): fl=name(l[3:]); fi=None
    elif l.startswith(('fi=','fe=')): fi=name(l[3:])
    elif l.startswith('fn='): fi=None; fn=l[3:]
    elif l.startswith('calls='):
        skip=True
        t=l.split()[1]
        if UPD:
            last = last+int(t) if t[0] in '+-' else (last if t=='*' else int(t))
    elif l.startswith(('cfl=','cfi=')): name(l[4:])
    elif l.startswith('cob=') or l.startswith('ob='): pass
for (f,ln),ir in cost.items():
    if ir: print(f"{f}\t{ln}\t{ir}")
