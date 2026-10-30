numeric digits 9
bench_stem. = 12345
call time 'R'
do bench_outer = 1 to 426287
bench_result = bench_stem.
end
say '__BENCH_SECONDS__' time('E')
say bench_result
