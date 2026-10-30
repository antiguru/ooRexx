numeric digits 9

call time 'R'
do bench_outer = 1 to 66061
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
loop 4; nop; end
end
say '__BENCH_SECONDS__' time('E')
say 'done'
