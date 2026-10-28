/* A stream opened APPEND writes at the end of its file as the file stands
   when a held write reaches it, so what a command or another stream appended
   meanwhile is kept. A plain WRITE and an implicit open write at their own
   position. */
b = .stream~new('b.txt'); b~open('write append'); b~lineout('b1')
'echo cmd1 >> b.txt'
b~lineout('b2'); b~close
say 'write append:' charin('b.txt', 1, 40)~changestr('0a'x, '|')
c = .stream~new('c.txt'); c~open('both append'); c~lineout('c1')
'echo cmd2 >> c.txt'
c~lineout('c2'); c~close
say 'both append:' charin('c.txt', 1, 40)~changestr('0a'x, '|')
d = .stream~new('d.txt'); d~open('write append'); d~lineout('d1')
e = .stream~new('d.txt'); e~open('write append'); e~lineout('e1'); e~close
d~lineout('d2'); d~close
say 'two appends:' charin('d.txt', 1, 40)~changestr('0a'x, '|')
f = .stream~new('f.txt'); f~open('write'); f~lineout('f1')
'echo cmd3 >> f.txt'
f~lineout('f2'); f~close
say 'write:' charin('f.txt', 1, 40)~changestr('0a'x, '|')
call lineout 'g.txt', 'g1'
'echo cmd4 >> g.txt'
call lineout 'g.txt', 'g2'; call lineout 'g.txt'
say 'implicit:' charin('g.txt', 1, 40)~changestr('0a'x, '|')
h = .stream~new('h.txt'); h~open('write append'); h~charout(copies('h', 5000))
'echo cmd5 >> h.txt'
h~lineout('h2'); h~close
say 'past the buffer:' chars('h.txt') charin('h.txt', 5001, 40)~changestr('0a'x, '|')
