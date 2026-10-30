numeric digits 9

call time 'R'
do bench_outer = 1 to 19314
bench_i=0; do until bench_i=4; bench_i=bench_i+1; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
