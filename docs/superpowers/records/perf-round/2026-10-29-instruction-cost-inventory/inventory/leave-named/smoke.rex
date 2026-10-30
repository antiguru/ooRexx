numeric digits 9

call time 'R'
do bench_outer = 1 to 3
do bench_i=1 to 4; do 2; leave bench_i; end; end
do bench_i=1 to 4; do 2; leave bench_i; end; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
