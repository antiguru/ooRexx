say 'close-create' lineout('newfile.txt') stream('newfile.txt', 'c', 'query exists')~length > 0
say 'charclose-create' charout('newfile2.txt') stream('newfile2.txt', 'c', 'query exists')~length > 0
call lineout 'ro.txt', 'abc'
call lineout 'ro.txt'
say stream('ro.txt', 'c', 'open read')
call on notready name h
say 'charout-ro' charout('ro.txt', 'xyz') stream('ro.txt', 'D')
say 'lineout-ro' lineout('ro.txt', 'xyz') stream('ro.txt', 'D')
say 'linein-after' linein('ro.txt', 1) stream('ro.txt', 'D')
say 'close-ro' lineout('ro.txt') stream('ro.txt', 'D')
s = .stream~new('ro.txt')
say s~open('read')
say 'm-lineout-pos' s~lineout(, 1) s~description
s~close
call off notready
do v over .array~of('line', '1.5', ' 2 ', '1E1', '-1', 0, .nil, 99999999999999999999, '1.0')
  call t "lineout('w.txt', , v)"
  call t "linein('ro.txt', v)"
  call t "linein('ro.txt', 1, v)"
  call t "charin('ro.txt', v)"
  call t "charin('ro.txt', 1, v)"
  call t "charout('w.txt', , v)"
  call t "charout('w.txt', 'q', v)"
end
exit
t:
  signal on syntax name ts
  interpret 'r =' arg(1)
  say arg(1) '['v']' '->' '['r']'
  return
ts:
  say arg(1) '['v']' '-> E' condition('O')~code condition('O')~message
  return
h: say '  notready' condition('D') condition('O')~result; return
