numeric digits 9

call time 'R'
do bench_outer = 1 to 32935
bench_answer=bench_fn()
end
say '__BENCH_SECONDS__' time('E')
say bench_answer
exit
bench_fn:
return 7
