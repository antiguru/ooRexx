call on halt name h
s.0 = 5; s.1 = 'old'
say 'a'
address system 'echo first; sleep 3; echo late' with output stem s.
say 'b' rc s.0 s.1
exit
h: say 'halted' sigl; return
