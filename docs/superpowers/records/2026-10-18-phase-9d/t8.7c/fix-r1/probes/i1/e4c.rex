/* DEFAULTNAME raising inside an error insert, CALL ON ANY no; SIGNAL ON SYNTAX in a caller routine */
signal on syntax
call r
exit
r:
  o = .bad~new
  o~nosuch
  say 'not reached'
  return
syntax: say 'trapped' condition('o')~code condition('o')~message; exit
::class bad
::method defaultname
  say 'dn ran'
  raise syntax 98.900 array('boom')
