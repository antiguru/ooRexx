numeric digits 9
bench_operand = 0
call time 'R'
do bench_outer = 1 to 421052
bench_result = \bench_operand
end
say '__BENCH_SECONDS__' time('E')
say bench_result
