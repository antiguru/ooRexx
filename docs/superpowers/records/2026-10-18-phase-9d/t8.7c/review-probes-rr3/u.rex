/* Untraced RAISE through a variable reference: RC and DESCRIPTION. */
nz = .noisy~new
bd = .bad~new
call on user zz name u
call on error name u
signal on syntax name s
call ru 'user', >nz
call ru 'error', >nz
call ru 'user', >bd
say 'after bd user'
call ru 'error', >bd
say 'after bd error'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; exit
u: say 'handler' condition('c'); return
ru:
  use arg how, v
  if how = 'user' then raise user zz description (v) return
  raise error (v) return
::class noisy
::method defaultname
  say '    (noisy dn)'
  return 'NOISY'
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
