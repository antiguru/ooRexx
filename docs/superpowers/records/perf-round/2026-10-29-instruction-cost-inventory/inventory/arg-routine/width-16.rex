numeric digits 9

call time 'R'
do bench_outer = 1 to 13923
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
call bench_sub abc,def
end
say '__BENCH_SECONDS__' time('E')
say result
exit
bench_sub: procedure
arg bench_a,bench_b
return bench_a||bench_b
