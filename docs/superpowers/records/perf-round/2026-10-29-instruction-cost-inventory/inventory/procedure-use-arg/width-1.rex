numeric digits 9

call time 'R'
do bench_outer = 1 to 19680
call bench_sub 7,8
end
say '__BENCH_SECONDS__' time('E')
say result
exit
bench_sub: procedure
use arg bench_a,bench_b
return bench_a+bench_b
