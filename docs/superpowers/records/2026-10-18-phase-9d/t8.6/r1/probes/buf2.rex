call lineout 'a.txt', 'one'
say 'size by name open' stream('a.txt', 'c', 'query size')
call lineout 'a.txt', 'two'
say 'linein same' linein('a.txt')
call charout 'a.txt', 'xyz'
say 'pos' stream('a.txt', 'c', 'query position write') stream('a.txt', 'c', 'query position read') stream('a.txt', 'c', 'query position sys')
s = .stream~new('a.txt'); say 'other chars' s~chars s~query('size'); s~close
call charout 'a.txt', 'Q', 2
say 'after positioned' charin('a.txt', 1, 20)
b = .stream~new('b.txt'); b~open('write append'); b~lineout('b1')
c = .stream~new('b.txt'); c~open('write append'); c~lineout('c1')
b~close; c~close
say 'appends' charin('b.txt', 1, 20)~changestr('0a'x, '|')
d = .stream~new('d.txt'); d~open('write replace'); d~charout('abc'); say 'seek' d~seek('=1') d~query('position write'); d~charout('Z'); d~close
say 'd' charin('d.txt', 1, 10)
e = .stream~new('e.txt'); e~charout('12345'); say 'e qs' e~query('size') e~query('position write'); e~charout('6', 10); say e~query('size'); e~close
say 'e' c2x(charin('e.txt', 1, 20))
f = .stream~new('f.txt'); f~open('write'); f~lineout('f1'); say 'lines by obj' f~lines; say 'f dt' (f~query('datetime') \== ''); f~close
g = .stream~new('g.txt'); g~lineout('g1'); g~lineout('g2'); say 'g lines via arrayin' g~arrayin~items
h = .stream~new('h.txt'); h~lineout('h1'); h~lineout('h2'); say 'h linein 1' h~linein(1); say h~linein
