o = .t~new
say 'got' o~m
call SysSleep 0.2
say 'main endlocal' endlocal()
::class t
::method m
  call setlocal
  call value 'P6X', 'yes', 'ENVIRONMENT'
  reply 1
  say 'cont endlocal' endlocal()
