b = .stream~new('b.txt'); b~open('write append'); b~lineout('b1')
'echo cmd1 >> b.txt'
b~lineout('b2'); b~close
say 'explicit append' charin('b.txt', 1, 40)~changestr('0a'x, '|')
call lineout 'c.txt', 'c1'
'echo cmd2 >> c.txt'
call lineout 'c.txt', 'c2'; call lineout 'c.txt'
say 'implicit' charin('c.txt', 1, 40)~changestr('0a'x, '|')
d = .stream~new('d.txt'); d~open('write append'); d~lineout('d1')
e = .stream~new('d.txt'); e~open('write append'); e~lineout('e1'); e~close
d~lineout('d2'); d~close
say 'two appends' charin('d.txt', 1, 40)~changestr('0a'x, '|')
