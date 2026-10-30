/* TRACE I: the >=> line's own DEFAULTNAME send raises (third send). */
b = .once~new
signal on syntax name s
trace i
w = b
trace off
say 'not reached'
exit
s: trace off; say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class once
::method init; expose n; n = 0
::method defaultname; expose n
  n = n + 1
  say '    (once' n')'
  if n = 3 then raise syntax 98.900 array('boom')
  return 'ONCE'
