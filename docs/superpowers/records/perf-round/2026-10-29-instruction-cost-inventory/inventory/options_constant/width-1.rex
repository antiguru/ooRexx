numeric digits 9

call time 'R'
do bench_outer = 1 to 70629
options 'NOCOMMANDS'
end
say '__BENCH_SECONDS__' time('E')
say 'options-ok'
