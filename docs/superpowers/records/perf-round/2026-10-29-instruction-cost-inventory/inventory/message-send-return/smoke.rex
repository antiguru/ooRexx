numeric digits 9
bench_receiver=.Bench~new
call time 'R'
do bench_outer = 1 to 3
bench_receiver~ping
bench_receiver~ping
end
say '__BENCH_SECONDS__' time('E')
say 'done'
exit
::class Bench
::method ping
return 7
