call time 'r'
call SysSleep 0.2
o = .t~new
say 'main got' o~m
call SysSleep 0.3
say 'main after >= 0.5:' (time('e') >= 0.5)
exit
::class t
::method m
  say 'm before fresh:' (time('e') < 0.1)
  call SysSleep 0.05
  reply 7
  call SysSleep 0.05
  say 'm after continues own clock:' (time('e') >= 0.1) (time('e') < 0.2)
