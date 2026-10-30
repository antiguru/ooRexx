numeric digits 9

call time 'R'
do bench_outer = 1 to 3
numeric digits 9
numeric digits 9
end
say '__BENCH_SECONDS__' time('E')
say digits()
