numeric digits 9

call time 'R'
do bench_outer = 1 to 505
call bench_sub
end
say '__BENCH_SECONDS__' time('E')
say result
exit
bench_sub: procedure
signal on user bench_condition name bench_caught
call bench_raise
bench_caught:
return 7
bench_raise:
raise user bench_condition return
