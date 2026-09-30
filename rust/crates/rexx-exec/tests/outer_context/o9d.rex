x = 'main'; n = 3
say Outer(.c~new)
say 'after' newv symbol('NEWV') t.1 t.9 symbol('T.') u.7 symbol('U.1') symbol('GONE')
say 'tails' k.x k.main k.nobody symbol('K.MAIN') second
interpret 'say "interp" newv n'
call proc
say 'proc' newv w
exit
proc: procedure expose newv w
  w = 'from-proc' newv
  newv = 'proc-set'
  return
::requires 'outer9b' LIBRARY
::class c
::method run
  a = 'ra'; b = 'rb'; c. = 'rc'
  say 'get' OGet('NEWV') OGet('T.') OGet('U.1') OGet('X')
  say OSet('NEWV', 'made') OSet('T.', 'tdef') OSet('U.7', 'u7') ODrop('GONE') OSet('K.X', 'kx') OSet('K.NOBODY', 'kn')
  say 'get2' OGet('NEWV') OGet('T.1') OGet('U.7') OGet('K.main') OGet('K.X')
  say OSet('NEWV', 'again') OSet('SECOND', 'two')
  all = OAll()
  say 'all' all~items all['NEWV'] all['SECOND'] all['X']
  interpret 'late = a b'
  say 'inner' a b c.1 late symbol('NEWV') symbol('SECOND')
  return 'ret'
