s. = .stem~new; s.1 = 'a'; s.0 = 1
j = .J~new
m = j~start('run', s.)
call SysSleep 0.4
say 'mid same' s.0 s.1 s.2
r = m~result
say 'end same' s.0 s.1 s.2 s.3
exit
::class J
::method run
  use arg s.
  address system 'cat; echo four; sleep 1' with input stem s. output stem s.
