b = .bad~new
q = >b
signal on syntax name s
x = .directory~new; x[q] = 1; say x~items
say 'returned'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
