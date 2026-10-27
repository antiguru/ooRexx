/* repeated: each iteration builds the cycle inside a forward; HALT should interrupt; memory flat */
parse arg n
if n = '' then n = 1000000
signal on halt
s. = .o~new
do i = 1 to n
  r = s.~foo
end
say 'done' r
exit
halt: say 'HALT after' i; exit
::class o
::method unknown
  signal on syntax
  x. = >x.
  return x.~bar
syntax: return 'caught' condition('O')~code
