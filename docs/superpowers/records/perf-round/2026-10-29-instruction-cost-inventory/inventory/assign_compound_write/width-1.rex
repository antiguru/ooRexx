numeric digits 9
bench_tail = 7
bench_stem. = 0
call time 'R'
do bench_outer = 1 to 169014
bench_stem.bench_tail = 12345
end
say '__BENCH_SECONDS__' time('E')
say bench_stem.bench_tail
