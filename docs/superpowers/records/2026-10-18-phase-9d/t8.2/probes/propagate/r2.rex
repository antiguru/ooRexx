o = .t~new
say "got" o~m
call SysSleep 0.2
say "main done"
::class t
::method m
  reply 1
  say "cont"
  x = 1/0
