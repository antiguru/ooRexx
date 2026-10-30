numeric digits 9
bench_stem. = 0
bench_stem.7 = 12345
call time 'R'
do bench_outer = 1 to 62079
drop bench_stem.7
bench_stem.7 = 12345
end
say '__BENCH_SECONDS__' time('E')
say bench_stem.7
