call lineout 'f.txt', '1234567890'
call lineout 'f.txt'
s = .stream~new('f.txt')
call show s~charin(9,6), s
call show s~charin(, 2), s
call show s~charin(30), s
call show s~charin(11), s
call show s~charin(12), s
call show s~charin(1, 0), s
call show s~charin(5), s
call show s~charin(1.0E1, 1), s
call show s~linein(1.0), s
call show s~charin(' 2 ', 1), s
s~close
signal on syntax name e1
call show s~charin(0), s
e1: say 'e1' condition('O')~code condition('O')~message
signal on syntax name e2
call show s~linein(-3), s
e2: say 'e2' condition('O')~code condition('O')~message
signal on syntax name e3
call show s~lineout('x', -3), s
e3: say 'e3' condition('O')~code condition('O')~message
signal on syntax name e4
call show s~lineIn(.nil), s
e4: say 'e4' condition('O')~code condition('O')~message
signal on syntax name e5
call show s~lineIn('abc'), s
e5: say 'e5' condition('O')~code condition('O')~message
signal on syntax name e6
call show s~charout('abc', 'x'), s
e6: say 'e6' condition('O')~code condition('O')~message
signal on syntax name e7
call show s~lineout('abc', 1.5), s
e7: say 'e7' condition('O')~code condition('O')~message
signal on syntax name e8
call show .stream~new('/')~charin(0), s
e8: say 'e8' condition('O')~code condition('O')~message
signal on syntax name e9
call show s~charin(99999999999999999999), s
e9: say 'e9' condition('O')~code condition('O')~message
exit
show: say '['arg(1)']' arg(2)~description; return
