numeric digits 9
bench_a=12348
bench_b=7
call time 'R'
do bench_outer = 1 to 207253
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
bench_answer=bench_a/bench_b
end
say '__BENCH_SECONDS__' time('E')
say bench_answer
