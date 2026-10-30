numeric digits 9

call time 'R'
do bench_outer = 1 to 21982
call bench_sub
end
say '__BENCH_SECONDS__' time('E')
say result
exit
bench_sub: procedure
signal bench_target
bench_target:
return 7
