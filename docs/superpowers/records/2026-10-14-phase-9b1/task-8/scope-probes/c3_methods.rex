o = .t~new
call time 'r'
call SysSleep 0.2
say 'method starts its own clock:' (o~e < 0.1)
o~r
call SysSleep 0.2
say 'main clock untouched by a method reset:' (time('e') >= 0.5)
say 'another method starts its own clock:' (o~e < 0.1)
::class t
::method e
  return time('e')
::method r
  call SysSleep 0.2
  call time 'r'
