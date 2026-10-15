o = .t~new
o~put(.StackFrame~method('EXECUTABLE'))
say o~ex
::class t
::method put
use arg m
self~setMethod('EX', m)
