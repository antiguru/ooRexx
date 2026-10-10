# cgtext.py A.cg SHA_A B.cg SHA_B FUNC [N]: per-source-line self Ir inside FUNC, keyed by the
# line's text (repo files read at each sha), so moved lines still match.
import sys, collections, subprocess, functools
REPO='/home/moritz/dev/repos/ooRexx-rust-rewrite'
@functools.lru_cache(None)
def src(sha, path):
    try: return subprocess.run(['git','-C',REPO,'show',f'{sha}:{path}'],capture_output=True,text=True,check=True).stdout.split('\n')
    except Exception: return None
def key(sha, f, line):
    if '/crates/' in f:
        rel='rust/crates/'+f.split('/crates/',1)[1]
        lines=src(sha, rel)
        if lines and 0 < line <= len(lines):
            return rel.split('/src/',1)[-1]+' | '+lines[line-1].strip()
    return f.split('/')[-1]+':'+str(line)
def load(path, sha, target):
    out=collections.Counter(); fn=None; fl=None; fi=None; line=0; skip=False
    for raw in open(path):
        l=raw.rstrip('\n')
        if l.startswith('fl='): fl=l[3:]; fi=None; continue
        if l.startswith(('fi=','fe=')): fi=l[3:]; continue
        if l.startswith('fn='): fn=l[3:]; fi=None; continue
        if l.startswith(('cfn=','cfi=','cfl=','cob=','ob=')): continue
        if l.startswith('calls='): skip=True; continue
        if not l or not (l[0].isdigit() or l[0] in '+-*'): continue
        parts=l.split(); p=parts[0]
        if p.startswith('+'): line+=int(p[1:])
        elif p.startswith('-'): line-=int(p[1:])
        elif p!='*': line=int(p)
        if skip: skip=False; continue
        if fn and target in fn and len(parts)>=2:
            out[key(sha, fi or fl, line)]+=int(parts[-1])
    return out
a=load(sys.argv[1],sys.argv[2],sys.argv[5]); b=load(sys.argv[3],sys.argv[4],sys.argv[5])
d={k:b[k]-a[k] for k in set(a)|set(b) if b[k]!=a[k]}
print('total', sum(d.values()))
for k,v in sorted(d.items(), key=lambda x:-abs(x[1]))[:int(sys.argv[6]) if len(sys.argv)>6 else 20]:
    print(f'{v:+d}\t{k[:170]}')
