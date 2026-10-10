call time 'r'
call SysSleep 0.05
o = .t~new
say 'main got' o~m
e = time('e')
say 'main elapsed' (e >= 0.05)
call SysSleep 0.3
say 'm after below main' (o~after < time('e'))
::class t
::attribute after get unguarded
::method m
  expose after
  after = 99
  say 'm before zero' (time('e') = 0)
  reply 7
  call SysSleep 0.01
  after = time('e')
  say 'm after' (after > 0) (after < 0.2)
