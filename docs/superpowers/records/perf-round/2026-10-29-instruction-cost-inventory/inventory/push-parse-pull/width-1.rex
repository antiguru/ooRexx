numeric digits 9
bench_line='alpha beta'
call time 'R'
do bench_outer = 1 to 1750
push bench_line
parse pull bench_got
end
say '__BENCH_SECONDS__' time('E')
say bench_got
