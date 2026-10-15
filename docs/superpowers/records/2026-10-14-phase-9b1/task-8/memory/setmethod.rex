parse arg n
m = .k~method('FOO')
o = .k~new
do i = 1 to n; o~put(m); end
say o~x
::class k
::method foo
  return 1
::method put
  use arg m
  self~setMethod('X', m)
