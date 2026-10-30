/* A DEFAULTNAME that raises on its first send only, through a reference. */
b = .once~new
q = >b
signal on syntax name s
x = "abc"; parse var x (q) y
say 'returned'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class once
::method init; expose n; n = 0
::method defaultname; expose n
  n = n + 1
  say '    (dn' n')'
  if n = 1 then raise syntax 98.900 array('boom')
  return 'ONCE'
