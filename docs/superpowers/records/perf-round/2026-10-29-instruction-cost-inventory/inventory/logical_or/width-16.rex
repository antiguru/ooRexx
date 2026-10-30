numeric digits 9
bench_left = 1
bench_right = 0
call time 'R'
do bench_outer = 1 to 333333
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
bench_result = bench_left | bench_right
end
say '__BENCH_SECONDS__' time('E')
say bench_result
