numeric digits 9

call time 'R'
do bench_outer = 1 to 3
numeric fuzz 2
numeric fuzz 2
end
say '__BENCH_SECONDS__' time('E')
say fuzz()
