numeric digits 9

call time 'R'
do bench_outer = 1 to 3
do 4; leave; end
do 4; leave; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
