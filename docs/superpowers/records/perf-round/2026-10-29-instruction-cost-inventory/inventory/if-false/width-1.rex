numeric digits 9
bench_flag=0
call time 'R'
do bench_outer = 1 to 323450
if bench_flag then nop; else nop
end
say '__BENCH_SECONDS__' time('E')
say 'done'
