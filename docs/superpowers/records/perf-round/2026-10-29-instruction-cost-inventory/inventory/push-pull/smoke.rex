numeric digits 9
bench_line='alpha beta'
call time 'R'
do bench_outer = 1 to 3
push bench_line
pull bench_got
push bench_line
pull bench_got
end
say '__BENCH_SECONDS__' time('E')
say bench_got
