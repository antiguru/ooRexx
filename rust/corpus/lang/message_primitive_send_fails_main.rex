/* Main's send to a primitive method fails untrapped; a started reader finds
   the condition object with main's POSITION, PROGRAM and PACKAGE. */
m = .message~new('abc', 'left', 'I', -1)
.t~new~start('later', m)
m~send
::class t
::method later unguarded
  use arg m
  call SysSleep 0.5
  c = m~errorCondition
  say 'later' m~hasError c~code c~position '['c~program']' (c~package == .nil)
