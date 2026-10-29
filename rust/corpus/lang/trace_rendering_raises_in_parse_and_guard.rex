/* A condition a traced rendering raises is the clause's: in a PARSE
   pattern, a PARSE VALUE source and a GUARD WHEN expression. */
o = .bad~new
signal on syntax name s1
p = o
trace i
parse value 'xyz' with a (p) b
trace off
say 'not reached 1'
s1: trace off; say 's1' condition('o')~code condition('o')~message
signal on syntax name s2
trace i
parse value o with a
trace off
say 'not reached 2'
s2: trace off; say 's2' condition('o')~code condition('o')~message
.t~new~m(o)
say 'after'
exit
::class t
::method m
  expose v
  use arg v
  signal on syntax name s
  trace i
  guard on when v \== 1
  trace off
  say 'not reached 3'
  return
s: trace off; say 's3' condition('o')~code condition('o')~message; return
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
