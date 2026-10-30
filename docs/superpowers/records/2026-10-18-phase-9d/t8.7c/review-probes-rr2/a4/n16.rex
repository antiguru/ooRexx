b = .noisy~new
q = >b
signal on syntax name s
say 5~compareTo(q)
say 'returned'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class noisy
::method defaultname
  say '    (noisy dn)'
  return 'NOISY'
