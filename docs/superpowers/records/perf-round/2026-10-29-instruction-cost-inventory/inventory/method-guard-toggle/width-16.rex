numeric digits 9
bench_receiver=.Bench~new
call time 'R'
do bench_outer = 1 to 21126
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
bench_receiver~ping
end
say '__BENCH_SECONDS__' time('E')
say 'done'
exit
::class Bench
::method ping
guard off
guard on
return
