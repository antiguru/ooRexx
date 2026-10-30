numeric digits 9

call time 'R'
do bench_outer = 1 to 3
call bench_sub
call bench_sub
end
say '__BENCH_SECONDS__' time('E')
say 'done'
exit
bench_sub:
return
