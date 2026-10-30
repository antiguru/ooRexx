numeric digits 9
bench_source = 'alpha:beta:gamma'
bench_delim = ':'
call time 'R'
do bench_outer = 1 to 61871
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
parse var bench_source bench_a (bench_delim) bench_b (bench_delim) bench_c
end
say '__BENCH_SECONDS__' time('E')
say bench_a || '|' || bench_b || '|' || bench_c
