# usage: instr.py CG FNSUBSTR -> "addr\tline\tfile\tIr" self cost per instruction in matching fn (positions: instr line, no compression)
import sys
fl=fi=fn=None; skip=False; C={}
for l in open(sys.argv[1]):
    l=l.rstrip('\n')
    if not l: continue
    if l.startswith('0x'):
        if skip: skip=False; continue
        p=l.split()
        if fn and sys.argv[2] in fn:
            k=(int(p[0],16),int(p[1]),fi or fl); C[k]=C.get(k,0)+int(p[2])
        continue
    if l.startswith('fl='): fl=l[3:]; fi=None
    elif l.startswith(('fi=','fe=')): fi=l[3:]
    elif l.startswith('fn='): fn=l[3:]; fi=None
    elif l.startswith('calls='): skip=True
for (a,ln,f),ir in sorted(C.items()):
    print(f"{a:x}\t{ln}\t{f.split('rust/crates/')[-1].split('library/')[-1]}\t{ir}")
