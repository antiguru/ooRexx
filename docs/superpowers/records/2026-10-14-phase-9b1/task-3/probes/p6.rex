signal on syntax name s1
p = .context~package
p~addRoutine('SQ2', p~findRoutine('RXCALCSQRT')); say 'added' sq2(16) (p~findRoutine('sq2') == p~findRoutine('RXCALCSQRT'))
s1b:
signal on syntax name s2
c = p~findRoutine('RXCALCSQRT')~copy
p~addRoutine('SQ3', c); say 'copy added' sq3(25) (p~findRoutine('sq3') == c)
s2b:
signal on syntax name s3
t = .Routine~loadExternalRoutine('x', 'LIBRARY orxfunction TestIntArg')
p~addRoutine('T1', t~copy); say 'lib copy added' t1(7)
exit
s1: say 's1' condition('o')~code; signal s1b
s2: say 's2' condition('o')~code; signal s2b
s3: say 's3' condition('o')~code; exit
::requires 'rxmath' LIBRARY
