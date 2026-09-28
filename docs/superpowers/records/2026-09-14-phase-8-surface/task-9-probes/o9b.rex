x = 'main'; y = 'main-y'; s.1 = 'one'; s.k = 'kay'; k = 'K'; d = 'dropme'; n = 1
say Outer(.c~new)
say 'after' x y s.1 s.k s.2 symbol('D') n
exit
::requires 'outer9b' LIBRARY
::class c
::method run
  x = 'run'; y = 'run-y'; s.1 = 'r1'; k = 'Q'; d = 'rund'; n = 7
  say 'get' OGet('X') OGet('S.1') OGet('S.K') OGet('S.N') OGet('Z') IGet('X')
  say OSet('Y', 'set-by-inner') OSet('S.2', 'two') OSet('S.N', 'viaN') ODrop('D')
  a = OAll()
  say 'all' a~items a['X'] a['Y']
  say 'inner' x y s.1 symbol('D') n
  return 'ret'
