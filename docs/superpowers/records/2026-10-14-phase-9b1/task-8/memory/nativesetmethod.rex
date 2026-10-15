parse arg n
m = .object~method('STRING')
o = .k~new
do i = 1 to n; o~put(m); end
say o~x
::class k
::method put
  use arg m
  self~setMethod('X', m)
