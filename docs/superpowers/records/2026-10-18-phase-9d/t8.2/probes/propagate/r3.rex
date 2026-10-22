o = .t~new
say "got" o~m
call SysSleep 0.2
say "main done"
::class t
::method m
  reply 1
  signal on syntax
  x = 1/0
syntax:
  say "cont propagates"
  raise propagate
