numeric digits 9

call time 'R'
do bench_outer = 1 to 42402
do bench_i=1 to 4; nop; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
