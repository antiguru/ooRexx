/* RAISE SYNTAX ADDITIONAL takes requestArray's array, which the condition
   object carries; 98.939 where there is none or it is not
   single-dimensional. Under USER nothing converts. */
call t '.environment', .environment
call t '.array', .array
call t 'instance', .t~new
call t 'makearray', .u~new
call t 'matrix', .v~new
signal on user zork name z
call ru
z: say 'user' condition('O')~additional~id
raise syntax 93.900 additional (.array)
t: procedure
signal on syntax
use arg name, value
call r value
syntax: say name condition('C') rc condition('O')~code condition('O')~additional~class~id
say condition('O')~message
return
::routine ru
raise user zork additional (.array) return
::routine r
raise syntax 40.1 additional (arg(1)) return
::class t
::class u
::method makearray
return .array~of('MYR', 3)
::class v
::method makearray
return .array~new(2, 2)
