signal on halt
signal on syntax
s.0 = 5; s.1 = 'old'
t = time('E')
address system 'echo first; sleep 2.13; echo late' with output stem s. error stem e.
say 'no halt' rc s.0
exit
halt: say 'HALT rc' rc 'stem' s.0 s.1 'line' sigl
exit
syntax: say 'SYNTAX' condition('o')~code 'stem' s.0 s.1 'line' sigl
exit
