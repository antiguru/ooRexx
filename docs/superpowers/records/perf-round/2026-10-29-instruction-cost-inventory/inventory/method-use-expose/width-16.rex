numeric digits 9
bench_receiver=.Bench~new
call time 'R'
do bench_outer = 1 to 15598
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
bench_answer=bench_receiver~ping(7)
end
say '__BENCH_SECONDS__' time('E')
say bench_answer
exit
::class Bench
::method ping
expose bench_state
use arg bench_arg
bench_state=bench_arg
return bench_state
