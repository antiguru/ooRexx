b = .bad~new
q = >b
signal on syntax name s
x = 'abc'; parse var x (q) y
say 'returned'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
