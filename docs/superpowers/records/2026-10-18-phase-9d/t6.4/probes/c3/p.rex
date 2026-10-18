say '6:' f()
call on error name ce
say '7:' f()
call off error
signal on error name se
say '8:' f()
exit
ce: o = condition('O'); say 'ce' condition('C') o~propagated o~result o~rc; return
se: o = condition('O'); say 'se' condition('C') o~propagated o~result o~rc; exit
f: procedure
  signal on error name fh
  'exit 3'
  return 'never'
fh: raise propagate
