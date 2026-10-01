import json,sys,collections
f=sys.argv[1]
turns=0; last=None; outs=0; res=collections.Counter(); rescnt=collections.Counter(); big=[]
tools={}
for line in open(f):
    try: d=json.loads(line)
    except: continue
    m=d.get('message') or {}
    if d.get('type')=='assistant':
        u=m.get('usage') or {}
        if u:
            turns+=1; outs+=u.get('output_tokens',0)
            last=u
        for c in m.get('content') or []:
            if isinstance(c,dict) and c.get('type')=='tool_use':
                inp=c.get('input',{})
                desc=inp.get('command') or inp.get('file_path') or inp.get('pattern') or ''
                tools[c['id']]=(c['name'],str(desc)[:110], inp.get('offset'), inp.get('limit'))
    if d.get('type')=='user':
        for c in m.get('content') or [] if isinstance(m.get('content'),list) else []:
            if isinstance(c,dict) and c.get('type')=='tool_result':
                cont=c.get('content'); s=json.dumps(cont) if not isinstance(cont,str) else cont
                n=len(s); t=tools.get(c.get('tool_use_id'),('?','',None,None))
                res[t[0]]+=n; rescnt[t[0]]+=1; big.append((n,t))
print('turns',turns,'output_tokens',outs,'last usage',last)
print('result chars by tool',res.most_common(), rescnt.most_common())
tot=sum(res.values()); print('total result chars',tot)
for n,t in sorted(big,reverse=True)[:25]: print(n,t)
reads=collections.Counter((t[1]) for n,t in big if t[0]=='Read')
print('reads by file', reads.most_common(15))
