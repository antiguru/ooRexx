numeric digits 9
bench_env='SYSTEM'
call time 'R'
do bench_outer = 1 to 116959
address value bench_env
end
say '__BENCH_SECONDS__' time('E')
say 'done'
