/* Position and count arguments of the stream builtins and methods. The
   builtins LINEIN, CHARIN, LINEOUT and CHAROUT take each as a whole number in
   the 64-bit range, or raise 40.12 naming the builtin and the argument. The
   methods take a position the same way, or raise 88.907 naming the argument,
   and a count as an unsigned one. A position below 1 is 93.907, which names
   argument 1 whichever argument carried it, and is checked only after the
   implicit open. */
call lineout 'r.txt', 'abc'
call lineout 'r.txt'
values = .array~of('line', '1.5', ' 2 ', '1E1', '-1', 0, .nil, 99999999999999999999, '1.0')
do v over values
  call t "lineout('w.txt', , v)"
  call t "linein('r.txt', v)"
  call t "linein('r.txt', 1, v)"
  call t "charin('r.txt', v)"
  call t "charin('r.txt', 1, v)"
  call t "charout('w.txt', , v)"
  call t "charout('w.txt', 'q', v)"
  call t ".Stream~new('r.txt')~lineIn(v)"
  call t ".Stream~new('r.txt')~charIn(1, v)"
  call t ".Stream~new('w.txt')~lineOut('q', v)"
  call t ".Stream~new('w.txt')~charOut('q', v)"
end
call t ".Stream~new('/')~charIn(0)"
call t ".Stream~new('/')~lineIn(-1)"
call t ".Stream~new('nodir/x')~lineOut('q', 0)"
exit

t:
  signal on syntax name failed
  call on notready name ignored
  interpret 'r =' arg(1)
  say arg(1) '[' || v || '] [' || r || ']'
  return
failed:
  -- A count other than 0 or 1 is the bare 93, whose message is not shown.
  code = condition('O')~code
  if code == '93.0' then say arg(1) '[' || v || '] error' code
  else say arg(1) '[' || v || '] error' code condition('O')~message
  return
ignored:
  return
