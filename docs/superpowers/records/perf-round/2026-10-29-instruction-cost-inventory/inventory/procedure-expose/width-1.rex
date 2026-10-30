numeric digits 9
bench_state=7
call time 'R'
do bench_outer = 1 to 21047
call bench_sub
end
say '__BENCH_SECONDS__' time('E')
say bench_state
exit
bench_sub: procedure expose bench_state
bench_state=7
return
