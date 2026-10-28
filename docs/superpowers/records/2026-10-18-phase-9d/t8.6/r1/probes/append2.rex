b = .stream~new('b.txt'); b~open('write'); b~lineout('b1')
'echo cmd1 >> b.txt'
b~lineout('b2'); b~close
say 'write' charin('b.txt', 1, 40)~changestr('0a'x, '|')
b = .stream~new('c.txt'); b~open('both append'); b~lineout('b1')
'echo cmd1 >> c.txt'
b~lineout('b2'); b~close
say 'both append' charin('c.txt', 1, 40)~changestr('0a'x, '|')
