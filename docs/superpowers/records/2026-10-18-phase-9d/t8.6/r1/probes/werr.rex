say 'first'
r1 = lineout('STDOUT', 'a'); s1 = stream('STDOUT', 'd')
r2 = .stdout~lineout('b'); s2 = .stdout~description
say 'second'
r3 = charout(, 'c'); s3 = stream('STDOUT', 's')
r4 = .output~lineout('d'); s4 = .output~description
r5 = lineout('STDERR', 'e'); s5 = stream('STDERR', 'd')
o = .stream~new('STDOUT'); r6 = o~lineout('f'); s6 = o~description; f6 = o~flush; d6 = o~description
call lineout 'STDERR', r1 s1 '|' r2 s2 '|' r3 s3 '|' r4 s4 '|' r5 s5 '|' r6 s6 f6 d6
exit 7
