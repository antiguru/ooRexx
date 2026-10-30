numeric digits 9

call time 'R'
do bench_outer = 1 to 108892
numeric fuzz 0
end
say '__BENCH_SECONDS__' time('E')
say fuzz()
