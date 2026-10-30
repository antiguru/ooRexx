    17 *-* say 'seek size+1' s2~seek('='(size + 1) 'write char') s2~state
Error 43 running /tmp/claude-1000/t87c-rr2-big/b.rex line 17:  Routine not found.
Error 43.1:  Could not find routine "=".
open READY: size 5368709120
seek =1 0 NOTREADY
seek =4294967295 0 NOTREADY
seek =4294967296 4294967296 READY
seek =5368709119 5368709119 READY
seek =5368709120 5368709120 READY
seek =5368709121 5368709121 READY
seek =5368709122 5368709122 READY
seek <0 5368709121 READY
seek <1 5368709120 READY
seek -1 5368709119 READY
charout at size 0 READY
charout at size+1 0 READY
size 5368709122
reopen READY: size 5368709122
rc 213
