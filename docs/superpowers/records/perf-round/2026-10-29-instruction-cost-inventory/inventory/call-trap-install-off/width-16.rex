numeric digits 9

call time 'R'
do bench_outer = 1 to 90531
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
call on error name bench_caught
call off error
end
say '__BENCH_SECONDS__' time('E')
say 'done'
exit
bench_caught: return
