numeric digits 9
bench_left = 12
bench_right = 3
call time 'R'
do bench_outer = 1 to 160320
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
