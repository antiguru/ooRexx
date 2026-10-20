signal on halt
call time 'r'
x = .M~new~go
say 'after' x
exit
halt: say 'HALT handler line' sigl 'fast' (time('e') < 1.2); exit 0
::class M
::method go
  call SysSleep 3
  return 'done'
