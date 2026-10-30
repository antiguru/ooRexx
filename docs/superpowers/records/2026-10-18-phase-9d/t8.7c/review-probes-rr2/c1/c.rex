/* TRACE I over assignments of an object: how many DEFAULTNAME sends. */
o = .dn~new
trace i
z = o
trace r
y = o
trace off
b = .once~new
signal on syntax name s
trace r
w = b
trace off
say 'not reached'
exit
s: trace off; say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class dn
::method defaultname
  say '  (dn sent)'
  return 'DN'
::class once
::method init; expose n; n = 0
::method defaultname; expose n
  n = n + 1
  say '    (once' n')'
  if n = 2 then raise syntax 98.900 array('boom')
  return 'ONCE'
