numeric digits 9
bench_i=0
call time 'R'
do bench_outer = 1 to 277136
bench_i=bench_i+1
end
say '__BENCH_SECONDS__' time('E')
say bench_i
