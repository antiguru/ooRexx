numeric digits 9
bench_source = 'abcdefghijklmnop'
call time 'R'
do bench_outer = 1 to 3
parse var bench_source bench_a 5 bench_b 10 bench_c
parse var bench_source bench_a 5 bench_b 10 bench_c
end
say '__BENCH_SECONDS__' time('E')
say bench_a || '|' || bench_b || '|' || bench_c
