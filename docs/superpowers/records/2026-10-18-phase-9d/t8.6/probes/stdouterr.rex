r1 = lineout('STDOUT', 'a'); s1 = stream('STDOUT', 'd')
'true'
r2 = lineout('STDOUT', 'b'); s2 = stream('STDOUT', 'd')
r3 = .stdout~charout('c'); s3 = .stdout~description
o = .stream~new('STDOUT'); r4 = o~lineout('d'); s4 = o~description
call lineout 'STDERR', r1 s1 '|' r2 s2 '|' r3 s3 '|' r4 s4
