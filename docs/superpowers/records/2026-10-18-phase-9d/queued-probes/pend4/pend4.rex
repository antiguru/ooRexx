o = .k~new
m = .message~new(o, 'm')
say m~send
say m~halt
call SysSleep 0.3
say 'main end'
::class k
::method m
  reply 'v'
  do i = 1 to 3; say 'rest' i; end
