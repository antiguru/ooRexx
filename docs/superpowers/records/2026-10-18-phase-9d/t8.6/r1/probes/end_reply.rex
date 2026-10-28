call lineout 'f1.txt', 'one'
s = .stream~new('f3.txt'); s~lineout('three')
t = .t~new; t~go; exit 0; ::class t; ::method go; reply; call lineout 'f4.txt', 'reply'; call syssleep 0.2
