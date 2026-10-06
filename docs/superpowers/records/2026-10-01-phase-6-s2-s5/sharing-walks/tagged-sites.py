#!/usr/bin/env python3
# Every call of a tagged heap accessor (`get`, `get_mut`, `body_text`,
# `is_class`) in rexx-exec, rexx-classes and rexx-api source, with the function
# it sits in and `test` for test code, as `file:line<TAB>fn<TAB>test or empty`.
# Run from rust/.
import re,subprocess,sys
out=subprocess.run(['/bin/grep','-a','-rnE',r'heap(\(\))?\.(get|get_mut|body_text|is_class)\(','crates/rexx-exec/src','crates/rexx-classes/src','crates/rexx-api/src','--include=*.rs'],capture_output=True,text=True).stdout
for line in out.splitlines():
    f,n,_=line.split(':',2); n=int(n)
    src=open(f).read().split('\n')
    fn='?'
    for i in range(n-1,-1,-1):
        m=re.search(r'\bfn\s+(\w+)',src[i])
        if m: fn=m.group(1); break
    test = '/tests' in f or f.endswith('tests.rs')
    print(f"{f}:{n}\t{fn}\t{'test' if test else ''}")
