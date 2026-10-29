/* DEFAULTNAME raising inside an error insert, no trap */
o = .bad~new
o~nosuch
say 'not reached'
::class bad
::method defaultname
  say 'dn ran'
  raise syntax 98.900 array('boom')
