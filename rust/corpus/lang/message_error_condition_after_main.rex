/* Main's send fails untrapped; a started activity reading the message
   afterwards finds the condition object. */
m = .message~new(.t~new, 'boom')
.t~new~start('later', m)
m~send
::class t
::method later unguarded
  use arg m
  call SysSleep 0.5
  say 'later' m~hasError m~errorCondition~code m~errorCondition~traceback~items
::method boom unguarded
  return 1/0
