numeric digits 9
bench_source = 'Alpha Beta Gamma'
call time 'R'
do bench_outer = 1 to 66188
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
parse lower var bench_source bench_a bench_b bench_c
end
say '__BENCH_SECONDS__' time('E')
say bench_a || '|' || bench_b || '|' || bench_c
