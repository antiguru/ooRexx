numeric digits 9
bench_drop = 12345
call time 'R'
do bench_outer = 1 to 150848
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
drop bench_drop
bench_drop = 12345
end
say '__BENCH_SECONDS__' time('E')
say bench_drop
