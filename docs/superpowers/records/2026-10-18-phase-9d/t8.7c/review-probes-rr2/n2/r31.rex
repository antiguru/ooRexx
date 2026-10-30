mk = .km~new; nz = .noisy~new; bd = .bad~new
signal on syntax name s
v = .km~new
trace i; raise syntax (v)
trace off; say 'returned'
exit
s: trace off; say 'SYNTAX' condition('o')~code condition('o')~message; say 'd' condition('d'); exit
::class km
::method makestring; say '    (mk)'; return '93.901'
::method defaultname; say '    (km dn)'; return 'KMDN'
::class kn
::method makestring; say '    (kn mk)'; return .nil
::method defaultname; say '    (kn dn)'; return 'KNDN'
::class noisy
::method defaultname
  say '    (noisy dn)'
  return 'NOISY'
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
