numeric digits 9
bench_result = 0
call time 'R'
do bench_outer = 1 to 3
interpret 'bench_result = 7'
interpret 'bench_result = 7'
end
say '__BENCH_SECONDS__' time('E')
say bench_result
