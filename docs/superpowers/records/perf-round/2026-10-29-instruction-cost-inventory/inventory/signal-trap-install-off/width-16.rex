numeric digits 9

call time 'R'
do bench_outer = 1 to 91150
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
signal on syntax name bench_caught
signal off syntax
end
say '__BENCH_SECONDS__' time('E')
say 'done'
exit
bench_caught: exit 1
