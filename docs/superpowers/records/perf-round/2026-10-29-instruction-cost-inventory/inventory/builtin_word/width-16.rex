numeric digits 9
bench_source = 'alpha beta gamma'
call time 'R'
do bench_outer = 1 to 102389
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
bench_result = word(bench_source, 2)
end
say '__BENCH_SECONDS__' time('E')
say bench_result
