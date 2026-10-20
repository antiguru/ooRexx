/* MAX and MIN raise LOSTDIGITS for an argument wider than DIGITS, named
   by its number unrounded, in either form. */
call t 'max(1.5, 1111111111)'
call t 'min(1.5, 1111111111)'
call t '1.5~min(2222222222)'
call t 'max(1.5, 1, 3333333333)'
call t "max('1.5', ' 4444444444 ')"
call t "max(1.5, '44444444440E-1')"
call t "max(1.5, '-0004444444444.0')"
call t "max(1.5, '4.444444444E+3')"
call t "max(1.5, '4444444444E+20')"
call t "max(1.5, '.00004444444444')"
call t "max(1.5, '1.00000000000000000000000001E-7')"
call t 'max(1.5, 1.111111111)'
call t 'max(1.5, 111111111)'
numeric form engineering
call t "max(1.5, '44444444440E-1')"
call t "max(1.5, '4444444444E+20')"
call t "max(1.5, '1.00000000000000000000000001E-7')"
numeric form scientific

-- Under ::OPTIONS LOSTDIGITS SYNTAX, with nothing trapped, it is 98.972.
r = .routine~new('lostdigits', ('say max(1, 2)', 'say max(1.5, 1111111111)', -
  '::options lostdigits syntax'))
r~call
exit

t: procedure
  parse arg expr
  signal on lostdigits
  numeric digits 9
  interpret 'x=' expr
  say left(expr, 30) 'none' x
  return
lostdigits:
  say left(expr, 30) condition('C') '['condition('D')']' sigl
  return
