q = .array~of('in1','in2')
j = .J~new
m = j~start('run', q)
call SysSleep 0.4
say 'mid same' q~items q[1]
m~result
say 'end same' q~items q[1] q[3]
exit
::class J
::method run
  use arg q
  address system 'cat; echo four; sleep 1' with input using (q) output using (q)
