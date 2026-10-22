o = .t~new
say 'got' o~m
call SysSleep 0.2
say 'main done'
::class t
::method m
  signal on syntax
  x = 1/0
  return 0
syntax:
  reply 1
  say 'cont propagates'
  raise propagate
