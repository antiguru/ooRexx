signal on halt
call time 'r'
call go
say 'after' result
exit
halt: say 'HALT handler line' sigl 'fast' (time('e') < 1.2); exit 0
go: call SysSleep 3; return 'done'
