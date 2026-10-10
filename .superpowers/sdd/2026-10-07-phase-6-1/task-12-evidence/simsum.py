import sys, collections, re
t=open(sys.argv[1]).read()
head=t.split('\n',1)[0]
print(head)
sec=lambda name: t.split('## '+name,1)[1].split('\n## ',1)[0].strip('\n') if '## '+name in t else ''
diff=sec('Differences from the oracle sets')
c=collections.Counter()
for l in diff.split('\n'):
    if not l.strip(): continue
    g,p=l.split(':')[0:2]
    c[(g,p)]+=1
print('differences', sum(c.values()))
for (g,p),n in sorted(c.items()): print(f'  {g} {p}: {n}')
for name in ['Wall-clock reruns (ruling P48)','Reds','Exempted reds']:
    body=sec(name)
    lines=[l for l in body.split('\n') if l.strip()]
    print(name, len(lines))
    for l in lines[:20]: print('   ', l[:200])
