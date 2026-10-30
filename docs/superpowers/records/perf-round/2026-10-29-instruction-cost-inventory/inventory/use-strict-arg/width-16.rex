numeric digits 9

call time 'R'
do bench_outer = 1 to 21255
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
call bench_sub 7
end
say '__BENCH_SECONDS__' time('E')
say result
exit
bench_sub: procedure
use strict arg bench_a
return bench_a
