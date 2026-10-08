/* setMethod with a DELEGATE method reads the delegate variable in the .nil
 * scope's pool, where it is unset, and sends to its name. */
o = .t~new
say o~go(.u~method('LENGTH'))
::class t
::method init
expose a
a = 'xyz'
::method go
use arg m
self~setMethod('length', m)
return self~length
::class u
::method length delegate a
