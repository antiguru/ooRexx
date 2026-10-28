call lineout 'f1.txt', 'one'
s = .stream~new('f3.txt'); s~lineout('three')
call syssleep 5
