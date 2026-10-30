numeric digits 9

call time 'R'
do bench_outer = 1 to 3
bench_i=0; do while bench_i<4; bench_i=bench_i+1; end
bench_i=0; do while bench_i<4; bench_i=bench_i+1; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
