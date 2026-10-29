/* A condition a traced rendering raises in a GUARD WHEN expression is the
   GUARD's. */
.t~new~m(.bad~new)
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
  say 'not reached'
  return
s: trace off; say 's' condition('o')~code condition('o')~message; return
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
