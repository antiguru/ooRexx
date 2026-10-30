numeric digits 9
bench_stem. = 12345
call time 'R'
do bench_outer = 1 to 42811
drop bench_stem.
bench_stem. = 12345
end
say '__BENCH_SECONDS__' time('E')
say bench_stem.
