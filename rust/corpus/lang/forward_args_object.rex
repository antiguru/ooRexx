/* FORWARD ARGUMENTS takes requestArray's array: a directory's MAKEARRAY and
   an instance's own, and 98.946 where there is none or it is not
   single-dimensional. */
say .t~new~go(.local)
say .t~new~go(.u~new)
say .t~new~go(.t~new)
say .t~new~go(.v~new)
say .t~new~go(.WeakReference~new(.nil))
::class t
::method go
signal on syntax
forward message 'M' arguments (arg(1))
syntax: return 'trapped' rc condition('O')~code
::method m
return arg() arg(1)
::class u
::method makearray
return .array~of(2, 1)
::class v
::method makearray
return .array~new(2, 2)
