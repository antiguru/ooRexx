/* Main's send to a Rexx method fails untrapped; the condition object a
   started reader finds has main's level among its stack frames. */
m = .message~new(.t~new, 'boom')
.t~new~start('later', m)
m~send
::class t
::method later unguarded
  use arg m
  call SysSleep 0.5
  c = m~errorCondition
  say 'later' m~hasError c~code c~position '['c~program']' (c~package == .nil) c~traceback~items c~stackframes~items
::method boom unguarded
  x = 1/0
