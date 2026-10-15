call value 'P6X', 'main', 'ENVIRONMENT'
o = .t~new
say 'got' o~m
call SysSleep 0.3
say 'main endlocal' endlocal()
say 'value' value('P6X',, 'ENVIRONMENT')
::class t
::method m
  say 'm setlocal' setlocal()
  call value 'P6X', 'm', 'ENVIRONMENT'
  reply 1
  say 'cont endlocal' endlocal()
