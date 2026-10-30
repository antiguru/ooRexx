numeric digits 9
bench_items=.array~of(1,2,3,4)
call time 'R'
do bench_outer = 1 to 3
loop bench_item over bench_items; nop; end
loop bench_item over bench_items; nop; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
