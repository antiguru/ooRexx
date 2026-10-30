numeric digits 9

call time 'R'
do bench_outer = 1 to 15915
do bench_i=1 to 4; iterate; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
