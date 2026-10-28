r1 = lineout('STDOUT', 'a')
s1 = stream('STDOUT', 'd')
r2 = .stdout~lineout('b')
s2 = .stdout~description
r3 = charout('STDOUT', 'c')
s3 = stream('STDOUT','d')
h = .stream~new('HANDLE:1')
r4 = h~lineout('d')
s4 = h~description
o = .stream~new('STDOUT')
r5 = o~lineout('e')
s5 = o~description
say 'x'
call lineout 'STDERR', r1 s1 '|' r2 s2 '|' r3 s3 '|' r4 s4 '|' r5 s5 '| rc' result
