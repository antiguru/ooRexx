.local['R'] = .Routine~new('r', 'use arg x; .local~remove("R"); do 30; z = copies("x",100000); end;' -
  'say "exe" .context~executable~class~id .context~executable~source[1]~left(9); return x + 1')
say 'call' .local['R']~call(1)
.local['R'] = .Routine~new('r', 'use arg x; .local~remove("R"); do 30; z = copies("x",100000); end; return x * 2')
r = .local['R']
say 'callvar' r~call(5)
r = .nil
.local['R'] = .Routine~new('r', 'use arg x; .local~remove("R"); do 30; z = copies("x",100000); end; return x - 1')
say 'callwith' .local['R']~callWith(.array~of(10))
