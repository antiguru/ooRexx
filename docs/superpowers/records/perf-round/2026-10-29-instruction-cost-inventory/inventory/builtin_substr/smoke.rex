numeric digits 9
bench_source = 'alpha beta gamma'
call time 'R'
do bench_outer = 1 to 3
bench_result = substr(bench_source, 4, 8)
bench_result = substr(bench_source, 4, 8)
end
say '__BENCH_SECONDS__' time('E')
say bench_result
