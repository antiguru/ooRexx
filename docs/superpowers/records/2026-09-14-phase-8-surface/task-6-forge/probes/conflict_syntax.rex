/* a SYNTAX after a write where the output target is also the input source */
call AddCmd 'xr', 'r'
s.0 = 2; s.1 = 's1'; s.2 = 's2'
signal on syntax name a
address xr 'WSYNTAX' with input stem s. output stem s.
a:
say 'stem' condition('o')~code s.0 s.1 s.2
arr = .array~of('a1', 'a2')
signal on syntax name b
address xr 'WSYNTAX' with input using (arr) output using (arr)
b:
say 'array' condition('o')~code arr~items arr[1] arr[2]
exit
::requires 'cmd' LIBRARY
