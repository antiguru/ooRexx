numeric digits 9
bench_left = 12 + 0
bench_right = 3 + 0
call time 'R'
do bench_outer = 1 to 149346
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
bench_result = bench_left ** bench_right
end
say '__BENCH_SECONDS__' time('E')
say bench_result
