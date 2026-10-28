call lineout 'f1.txt', 'one'
call charout 'f2.txt', 'two'
s = .stream~new('f3.txt'); s~lineout('three')
say 'end'
