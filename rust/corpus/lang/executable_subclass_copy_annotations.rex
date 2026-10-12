/* A copy of a Method or Routine subclass instance has an annotation table of
   its own: writes to either side's table leave the other's unchanged. */
m = .MyMethod~new('m', 'return 1'); m~annotations['A'] = 'one'
c = m~copy; c~annotations['A'] = 'two'; c~annotations['B'] = 'bee'
say m~annotations['A'] m~annotations~items c~annotations['A'] c~annotations~items
m~annotations~remove('A')
say m~annotations~items c~annotations~items c~annotations['A']
r = .MyRoutine~new('r', 'return 2'); r~annotations['X'] = 'x'
d = r~copy; d~annotations~empty
say r~annotations['X'] r~annotations~items d~annotations~items
b = .Method~new('b', 'return 3'); b~annotations['A'] = 'one'
e = b~copy; e~annotations['A'] = 'two'
say b~annotations['A'] e~annotations['A']
::class MyMethod subclass Method
::class MyRoutine subclass Routine
