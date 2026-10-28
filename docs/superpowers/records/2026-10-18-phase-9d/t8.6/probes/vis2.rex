s = .stream~new('a.txt')
s~lineout('obj')
call lineout 'b.txt', 'byname'
address system 'cat a.txt b.txt; echo cmd-end'
say 'lines b' lines('b.txt') 'chars a' chars('a.txt')
say 'query size a' stream('a.txt','c','query size') s~query('size')
say 'lines a via obj' s~lines
address system 'cat a.txt; echo cmd2-end'
t = .stream~new('c.txt'); t~open('write replace nobuffer'); t~lineout('nb')
say 'c' lines('c.txt')
u = .stream~new('d.txt'); u~lineout('noclose')
v = .stream~new('e.txt'); v~lineout('flushed'); v~flush; say 'e' lines('e.txt')
w = .stream~new('f.txt'); w~charout(copies('x',5000)); say 'f' chars('f.txt')
