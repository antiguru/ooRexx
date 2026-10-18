signal on any
x = .o~new~m
say 'not propagated'
exit
any: say 'caller' condition('c') condition('d')
::class o
::method m
  signal on any
  a = 1 / 9999999999
  return 'no raise'
  any: say 'method' condition('c'); raise propagate
