say '1:' stream('/', 'c', 'open read')
say '2:' stream('zz_missing.dat', 'C', 'OPEN READ')
say '3:' .stream~new('/')~command('open read')
call on notready name cn
say '4:' stream('/', 'c', 'open read')
call off notready
signal on notready name sn
say '5:' stream('/', 'c', 'open read')
say 'not here'
sn:
o = condition('O')
say '5h:' condition('C') condition('D') o~propagated o~result o~additional~class~id
say '6:' f()
say '7:' g()
exit
cn:
o = condition('O')
say '4h:' condition('C') condition('D') o~propagated o~result o~additional~class~id
return
f: procedure
  signal on user foo name fh
  raise user foo description 'dd' return 'rv'
  return 'never'
fh: raise propagate
g: procedure
  call h
  return 'g-ret'
h:
  signal on user foo name hh
  raise user foo description 'dd' return 'rv2'
  return 'never'
hh: raise propagate
