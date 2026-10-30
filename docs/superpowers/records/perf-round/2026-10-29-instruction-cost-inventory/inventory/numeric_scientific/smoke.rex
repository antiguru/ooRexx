numeric digits 9

call time 'R'
do bench_outer = 1 to 3
numeric form scientific
numeric form scientific
end
say '__BENCH_SECONDS__' time('E')
say form()
