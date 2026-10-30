numeric digits 9

call time 'R'
do bench_outer = 1 to 24444
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
call bench_external 7
end
say '__BENCH_SECONDS__' time('E')
say result
exit
::routine bench_external
use arg bench_arg
return bench_arg
