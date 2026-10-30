numeric digits 9
bench_flag=1
call time 'R'
do bench_outer = 1 to 3
if bench_flag then nop; else nop
if bench_flag then nop; else nop
end
say '__BENCH_SECONDS__' time('E')
say 'done'
