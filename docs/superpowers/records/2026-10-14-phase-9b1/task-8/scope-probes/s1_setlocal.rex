call value 'P6X', 'main', 'ENVIRONMENT'
o = .t~new
o~m
say 'main endlocal' endlocal()
say 'value' value('P6X',, 'ENVIRONMENT')
::class t
::method m
  say 'm setlocal' setlocal()
  call value 'P6X', 'm', 'ENVIRONMENT'
