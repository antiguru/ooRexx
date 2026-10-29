/* DEFAULTNAME raising, rendered in an error insert */
signal on syntax
o = .bad~new
o~nosuch
exit
syntax: say 'trapped' condition('o')~code condition('o')~message; exit
::class bad
::method defaultname
  say 'dn ran'
  raise syntax 98.900 array('boom')
