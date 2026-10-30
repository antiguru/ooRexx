numeric digits 9
bench_line='abcdefgh'
call time 'R'
do bench_outer = 1 to 6685
say bench_line
end
.stderr~lineout('__BENCH_SECONDS__' time('E'))
say 'done'
