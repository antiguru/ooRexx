    17 *-* say 'seek size+1' s2~seek('='(size + 1) 'write char') s2~state
Error 43 running /tmp/claude-1000/t87c-rr2-big/b.rex line 17:  Routine not found.
Error 43.1:  Could not find routine "=".
open READY: size 4294967296
seek =1 0 NOTREADY
seek =4294967295 0 NOTREADY
seek =4294967296 4294967296 READY
seek =4294967295 0 NOTREADY
seek =4294967296 4294967296 READY
seek =4294967297 4294967297 READY
seek =4294967298 4294967298 READY
seek <0 4294967297 READY
seek <1 4294967296 READY
seek -1 0 NOTREADY
charout at size 0 READY
charout at size+1 0 READY
size 4294967298
reopen READY: size 4294967298
rc 213
