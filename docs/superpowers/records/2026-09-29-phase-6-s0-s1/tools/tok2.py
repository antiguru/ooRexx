import json,sys,collections,re
f=sys.argv[1]
tools={}; inp_chars=collections.Counter(); text_chars=0; think=0
cat=collections.Counter(); catn=collections.Counter()
def classify(cmd):
    if 'rexx' in cmd and ('.rex' in cmd or 'printf' in cmd or 'oracle' in cmd.lower() or 'build/bin/rexx' in cmd): return 'probe/oracle run'
    if 'cargo test' in cmd: return 'cargo test'
    if 'cargo build' in cmd or 'cargo clippy' in cmd or 'cargo check' in cmd: return 'cargo build/check/clippy'
    if 'callgrind' in cmd or 'cgdiff' in cmd: return 'perf'
    if re.search(r'\b(sed -n|cat |grep|/bin/grep|rg )',cmd): return 'read/grep'
    if 'git ' in cmd: return 'git'
    return 'other'
for line in open(f):
    try: d=json.loads(line)
    except: continue
    m=d.get('message') or {}
    if d.get('type')=='assistant':
        for c in m.get('content') or []:
            if not isinstance(c,dict): continue
            if c.get('type')=='tool_use':
                s=json.dumps(c.get('input')); inp_chars[c['name']]+=len(s)
                tools[c['id']]=c
            elif c.get('type')=='text': text_chars+=len(c.get('text',''))
            elif c.get('type')=='thinking': think+=len(c.get('thinking',''))
    if d.get('type')=='user' and isinstance(m.get('content'),list):
        for c in m['content']:
            if isinstance(c,dict) and c.get('type')=='tool_result':
                t=tools.get(c.get('tool_use_id'))
                if t and t['name']=='Bash':
                    cont=c.get('content'); s=json.dumps(cont) if not isinstance(cont,str) else cont
                    k=classify(t['input'].get('command',''))
                    cat[k]+=len(s); catn[k]+=1
print('tool input chars',inp_chars.most_common()); print('text',text_chars,'thinking',think)
print('bash result chars by class'); [print(' ',k,catn[k],v) for k,v in cat.most_common()]
