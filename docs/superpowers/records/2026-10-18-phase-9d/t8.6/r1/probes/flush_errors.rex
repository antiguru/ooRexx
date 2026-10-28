o = .stream~new('STDOUT')
f0 = o~flush; d0 = o~description
r1 = o~lineout('f'); f1 = o~flush; d1 = o~description
f2 = o~flush; d2 = o~description
r3 = o~lineout('g'); d3 = o~description
f4 = .stdout~flush; d4 = .stdout~description
e = .stream~new('STDERR'); r5 = e~lineout('h'); f5 = e~flush
call lineout 'STDERR', f0 d0 '|' r1 f1 d1 '|' f2 d2 '|' r3 d3 '|' f4 d4 '|' r5 f5
