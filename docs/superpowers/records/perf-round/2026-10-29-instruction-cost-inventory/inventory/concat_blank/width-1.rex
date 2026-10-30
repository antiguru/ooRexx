numeric digits 9
bench_left = 'abcdefgh'
bench_right = 'ijklmnop'
call time 'R'
do bench_outer = 1 to 98806
bench_result = bench_left bench_right
end
say '__BENCH_SECONDS__' time('E')
say bench_result
