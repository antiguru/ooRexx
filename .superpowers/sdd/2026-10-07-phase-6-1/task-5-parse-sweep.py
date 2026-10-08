# usage: python3 -I sweep.py TSV WORKDIR OUT -- runs every translation row through both interpreters
import os, sys, subprocess, resource, shutil
tsv, work, out = sys.argv[1:4]
ORACLE='/home/moritz/dev/repos/ooRexx/build/bin/rexx'
OURS='/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run'
def unescape(f):
    o=bytearray(); i=0; b=f.encode()
    while i<len(b):
        if b[i]!=0x5c: o.append(b[i]); i+=1; continue
        n=chr(b[i+1])
        if n=='\\': o.append(0x5c)
        elif n=='t': o.append(9)
        elif n=='n': o.append(10)
        elif n=='r': o.append(13)
        elif n=='x': o.append(int(b[i+2:i+4],16)); i+=2
        i+=2
    return bytes(o)
def lim(): resource.setrlimit(resource.RLIMIT_AS,(1<<30,1<<30))
def lim4(): resource.setrlimit(resource.RLIMIT_AS,(4<<30,4<<30))
def run(cmd, d, pre, env):
    r=subprocess.run(cmd,cwd=d,capture_output=True,timeout=20,preexec_fn=pre,env=env,stdin=subprocess.DEVNULL)
    return r.returncode, r.stdout, r.stderr
def key(rc, err):
    lines=err.split(b'\n')
    # traceback lines and the major line; the sub line carries inserts (R3)
    kept=[l for l in lines if not (l.startswith(b'Error ') and b' running ' not in l)]
    return rc, kept
res=open(out,'w'); n=same=0
for idx,line in enumerate(open(tsv,encoding='utf-8')):
    if line.startswith('#') or line.startswith('class\t'): continue
    f=line.rstrip('\n').split('\t',3)
    if f[0]!='translation': continue
    prog=unescape(f[3]); n+=1
    d=os.path.join(work,'r%d'%(idx+1))
    if os.path.exists(d): shutil.rmtree(d)
    os.makedirs(d); p=os.path.join(d,'p.rex'); open(p,'wb').write(prog)
    env=dict(os.environ); env['LD_LIBRARY_PATH']='/home/moritz/dev/repos/ooRexx/build/lib'
    try:
        a=run([ORACLE,p],d,lim,env)
        b=run([OURS,p],d,lim4,dict(os.environ))
    except subprocess.TimeoutExpired:
        res.write('TIMEOUT row %d\n'%(idx+1)); continue
    ka=key(a[0],a[2]); kb=key(b[0],b[2])
    if ka==kb and a[1]==b[1]: same+=1
    else: res.write('DIFF row %d %s\n  oracle %r\n  ours   %r\n'%(idx+1,f[1],ka,kb))
    shutil.rmtree(d)
res.write('rows %d same %d\n'%(n,same)); res.close()
print('rows',n,'same',same)
