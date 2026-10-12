/* Package~options(name, value): each option name the setting switch takes,
   the previous value it answers, the value read back, and the effect on an
   activation of this package's code started afterwards. ~options('I') keeps
   the settings as they stood before the first write, R restores them, S
   replaces them from an ::OPTIONS directive and leaves X alone. */
p = .context~package
say 'start [' || p~options || ']'
say 'D' p~options('D', 12) p~options('D') 'here' digits()
call show
say 'FO' p~options('FO', 'eng') p~options('form')
say 'FU' p~options('fu', ' 3 ') p~options('FUZZ')
call show
say 'NU' p~options('NU', 'inherit') p~options('numeric')
numeric digits 20
call show
say 'NU' p~options('nu', 'N') p~options('nu')
call show
say 'E' p~options('E', 'syntax') p~options('error')
say 'FA' p~options('fa', 's') p~options('failure')
say 'L' p~options('L', 'S') p~options('lostdigits')
say 'NOS' p~options('nos', 's') p~options('nostring')
say 'NOT' p~options('not', 's') p~options('notready')
say 'NOV' p~options('nov', 'S') p~options('novalue')
call try 'unset()'
call try 'lost()'
say 'NOV' p~options('nov', 'c') p~options('novalue')
say 'A' p~options('A', 'C')
call try 'unset()'
call try 'lost()'
say 'A' p~options('all', 'syntax')
say 'P' p~options('P', 'noprolog') p~options('prolog')
say 'T' p~options('T', 'labels') p~options('trace') p~trace
call show
say 'T' p~options('trace', 'O') p~options('T') p~trace
call show
say 'now [' || p~options || ']'
say 'I [' || p~options('I') || ']'
say 'X [' || p~options('X') || ']'
say 'R [' || p~options('R') || ']'
say 'after R [' || p~options || ']'
call show
say 'S [' || p~options('S', '  ::options digits 7 form engineering trace e') || ']'
say 'after S [' || p~options || ']'
say 'X after S [' || p~options('X') || ']'
call show
say 'S of no directive [' || p~options('S', '::routine nothing') || ']'
say 'after it [' || p~options || ']'
say 'D 1.0' p~options('D', 1.0) p~options('D')
say 'D back' p~options('D', 9) p~options('D')
np = .routine~new('np', ('return', '::options noprolog prolog'))~package
say 'NOPROLOG then PROLOG names [' || np~options('X') || ']' np~options('P')
pn = .routine~new('pn', ('return', '::options prolog noprolog'))~package
say 'PROLOG then NOPROLOG names [' || pn~options('X') || ']' pn~options('P')

rexx = .object~package
say 'rexx D' rexx~options('D', 15) rexx~options('D') rexx~digits
say 'rexx I [' || rexx~options('I') || ']'
say 'rexx [' || rexx~options || ']'

call try "p~options('D', -1)"
call try "p~options('D', 0)"
call try "p~options('D', 'abc')"
call try "p~options('FU', -1)"
call try "p~options('FU', 'x')"
call try "p~options('FU', 9)"
call try "p~options('FU', 5) p~options('D', 5)"
call try "p~options('FU', 0)"
call try "p~options('FO', 'wrong')"
call try "p~options('NU', 'x')"
call try "p~options('E', 'wrong')"
call try "p~options('A', 'wrong')"
call try "p~options('P', 'wrong')"
call try "p~options('T', 'wrong')"
call try "p~options('I', 'x')"
call try "p~options('R', 'x')"
call try "p~options('X', 'x')"
call try "p~options('D', '')"
call try "p~options('ZZZ', 'x')"
call try "p~options('S', 'digits 5')"
call try "p~options('S', '::options digits 5; say 1')"
call try "p~options('S', '::options digits 5' || '0a'x)"
call try "p~options('S', '::options digits 5' || '0d'x)"
call try "p~options('S', '::options digits 3 fuzz 5')"
call try "p~options('S', .nil)"
say 'unchanged [' || p~options || ']'
exit

show:
  say '  routine' calc()
  return

try:
  signal on syntax name refused
  interpret 'say' arg(1)
  return
refused:
  c = condition('o')
  say '  ' arg(1) '->' c~code c~errortext c~message
  return

::routine calc
  return digits() form() fuzz() trace()

::routine unset
  return zzunset

::routine lost
  numeric digits 5
  return 1234567 + 1
