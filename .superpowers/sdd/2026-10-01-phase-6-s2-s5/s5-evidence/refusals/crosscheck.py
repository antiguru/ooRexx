# python3 crosscheck.py > crosscheck.txt  (reads Setup.cpp and dispatch-rows.txt)
import re
setup=open('/home/moritz/dev/repos/ooRexx/interpreter/memory/Setup.cpp').read().split('\n')
want=[]
for lo,hi,cls in [(1050,1082,'Message'),(1325,1341,'EventSemaphore'),(1348,1362,'MutexSemaphore')]:
    for i in range(lo-1,hi):
        m=re.search(r'Add(Class|Unguarded|)Method\("([A-Za-z]+)"',setup[i])
        if m: want.append((cls,m.group(2).upper(),m.group(1)=='Class',i+1))
rows=[l.split('\t') for l in open('/home/moritz/dev/repos/ooRexx-rust-rewrite/.superpowers/sdd/2026-10-01-phase-6-s2-s5/s5-evidence/refusals/dispatch-rows.txt').read().strip().split('\n')]
for cls,name,iscls,ln in want:
    hits=[r[0]+' '+r[1] for r in rows if r[2]==cls and r[3]==name and (('CLASS' in r[1])==iscls)]
    print(f"Setup.cpp:{ln}\t{cls}\t{'class ' if iscls else ''}{name}\t{hits if hits else 'MISSING'}")
print('total',len(want),'missing',sum(1 for c,n,i,l in want if not [r for r in rows if r[2]==c and r[3]==n and (('CLASS' in r[1])==i)]))
