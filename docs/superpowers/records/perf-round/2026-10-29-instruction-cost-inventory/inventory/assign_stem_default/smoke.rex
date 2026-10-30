numeric digits 9
bench_stem. = 0
call time 'R'
do bench_outer = 1 to 3
bench_stem. = 12345
bench_stem. = 12345
end
say '__BENCH_SECONDS__' time('E')
say bench_stem.
