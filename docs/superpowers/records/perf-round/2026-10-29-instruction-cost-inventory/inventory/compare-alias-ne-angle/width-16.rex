numeric digits 9
bench_left='123.450'
bench_right='123.46'
call time 'R'
do bench_outer = 1 to 97402
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
bench_result=bench_left <> bench_right
end
say '__BENCH_SECONDS__' time('E')
say bench_result
