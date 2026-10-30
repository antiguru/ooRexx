numeric digits 9
bench_flag=1
call time 'R'
do bench_outer = 1 to 3
select; when bench_flag=0 then nop; otherwise nop; end
select; when bench_flag=0 then nop; otherwise nop; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
