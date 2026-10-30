numeric digits 9
bench_receiver=.Bench~new
call time 'R'
do bench_outer = 1 to 6991
bench_answer=bench_receiver~ping
end
say '__BENCH_SECONDS__' time('E')
say bench_answer
exit
::class Bench
::method ping
forward message('pong')
::method pong
return 7
