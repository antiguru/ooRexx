b = .bad~new
q = >b
signal on syntax name s
signal on nostring name ns
numeric digits (q)
say 'returned'
exit
s: say 'SYNTAX' condition('o')~code condition('o')~message; exit
ns: say 'NOSTRING' condition('d'); exit
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
