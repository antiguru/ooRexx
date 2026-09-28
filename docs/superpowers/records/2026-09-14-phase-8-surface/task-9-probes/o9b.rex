x = 'main'; y = 'main-y'; s.1 = 'one'; s.k = 'kay'; k = 'K'; d = 'dropme'; n = 1
t. = 'tdef'; t.3 = 'three'; u.1 = 'u1'; e = 'e'
say Outer(.c~new)
say 'after' x y s.1 s.k s.2 symbol('D') n
say 'stems' t.3 t.9 u.1 u.2 symbol('U.1') symbol('E')
exit
::requires 'outer9b' LIBRARY
::class c
::method run
  h = .context~identityHash
  x = 'run'; y = 'run-y'; s.1 = 'r1'; k = 'Q'; d = 'rund'; n = 7; t. = 'runt'; e = 're'
  say 'get' OGet('X') OGet('S.1') OGet('S.K') OGet('S.N') OGet('Z') IGet('X')
  say 'stem' OGet('T.') OGet('T.3') OGet('T.8')
  say OSet('Y', 'set-by-inner') OSet('S.2', 'two') OSet('S.N', 'viaN') ODrop('D')
  say OSet('T.', 'newdef') OSet('U.2', 'u2') ODrop('U.1') ODrop('E')
  a = OAll()
  say 'all' a~items a['X'] a['Y']
  say 'inner' x y s.1 symbol('D') n t.3 e
  say 'context' (h == .context~identityHash)
  return 'ret'
