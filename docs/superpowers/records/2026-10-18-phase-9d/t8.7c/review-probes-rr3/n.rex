o = .bad~new
q = .noisy~new
call t 'raise syntax 93.900 additional (o)'
call t 'raise syntax 93.900 description (o)'
call t 'raise user zz description (o)'
call t 'raise syntax (o)'
call t 'raise syntax 93.900 additional (q)'
call t 'raise user zz description (q)'
exit
t:
  signal on syntax name s
  call on user zz name u
  interpret arg(1)
  say '  ' arg(1) '-> returned'
  return
s: say '  ' arg(1) '-> SYNTAX' condition('o')~code condition('o')~message; return
u: say '  ' arg(1) '-> USER' condition('d'); return
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
::class noisy
::method defaultname
  say '    (noisy dn sent)'
  return 'NOISY'
