s.0 = 7; s.1 = 'old'; e. = 'E'; e.0 = 3; a = .array~of('x','y'); q = .array~of('in1','in2')
j = .J~new
m = j~start('run', s., e., a, q)
call SysSleep 0.5
say 'mid out' s.0 s.1 'err' e.0 e.1 'arr' a~items 'same' q~items q[1]
m~result
say 'end out' s.0 s.1 s.2 'err' e.0 e.1 'arr' a~items a[1] 'same' q~items q[1] q[3]
exit
::class J
::method run
  use arg s., e., a, q
  address system 'echo one; echo eone >&2; sleep 1; echo two' with output stem s. error stem e.
  address system 'echo three; sleep 1' with output using (a)
  address system 'cat; echo four; sleep 0.3' with input using (q) output using (q)
