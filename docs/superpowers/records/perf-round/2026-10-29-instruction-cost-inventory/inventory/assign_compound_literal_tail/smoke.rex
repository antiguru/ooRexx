numeric digits 9
bench_stem. = 0
bench_stem.1 = 12345
call time 'R'
do bench_outer = 1 to 3
bench_result = bench_stem.1
bench_result = bench_stem.1
end
say '__BENCH_SECONDS__' time('E')
say bench_result
