numeric digits 9
bench_tail = 7
bench_stem. = 0
bench_stem.7 = 12345
call time 'R'
do bench_outer = 1 to 3
bench_result = bench_stem.bench_tail
bench_result = bench_stem.bench_tail
end
say '__BENCH_SECONDS__' time('E')
say bench_result
