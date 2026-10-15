o = .t~new
say 'main got' o~m
call time 'r'
call SysSleep 0.4
say 'main done'
::class t
::method m
  reply 7
  call SysSleep 0.7
  say 'cont untouched by main reset:' (time('e') < 0.1)
