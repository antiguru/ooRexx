nz = .noisy~new; bd = .bad~new
signal on syntax name s; signal on error name e; signal on failure name e; signal on user zz name e
v = .bad~new
raise error 1 description (v)
say 'returned'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; say 'd' condition('d'); exit
e: say 'COND' condition('c'); say 'rc' condition('o')~rc; say 'd' condition('d'); say 'dc' condition('o')~description~class; exit
::class noisy
::method defaultname
  say '    (noisy dn)'
  return 'NOISY'
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
