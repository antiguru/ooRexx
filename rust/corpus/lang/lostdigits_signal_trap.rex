/* An operand with more digits than NUMERIC DIGITS raises LOSTDIGITS, which a
   SIGNAL ON LOSTDIGITS or SIGNAL ON ANY trap takes. */
exprs = '1111111111+1#1+1111111111#1111111111-1#1111111111*1#1/1111111111#' ||,
  '1111111111%1#1111111111//1#1111111111**2#-1111111111#+1111111111#' ||,
  '1111111111=1#1<1111111111#1.111111111+1#111111111+1#1e10+1#0111111111+1#' ||,
  "'-1111111111'+1#' 1111111111 '+1#1111111111==1#1111111111='abc'#" ||,
  '1111111111||1#v+0#w+0#w<1#w-v'
do while exprs <> ''
  parse var exprs e '#' exprs
  call try e, 9
end
call try '111111111+1', 8
call try '1111111111+1', 10

-- Untrapped, and under CALL ON ANY, which does not take it.
numeric digits 9
say 'untrapped' 1 / 9999999999
call on any name callany
say 'call on any' 1 / 9999999999
call off any

-- SIGNAL ON ANY.
signal on any
a = 1 / 9999999999
say 'not raised under ANY'
afterany:
signal off any

-- An internal routine with no trap of its own raises to the caller's.
signal on lostdigits name incaller
call sub
say 'sub returned' result
aftersub:

-- A method's own trap, through INTERPRET.
say .probe~new~run('1111111111+1')
say .probe~new~run('111111111+1')

-- A DO header operand.
signal on lostdigits name inloop
do i = 1 to 12345678901
  say 'loop ran' i
  leave
end
afterloop:
signal off lostdigits
say 'trap off' 1 / 9999999999
exit

try: procedure
  parse arg expr, d
  v = 1111111111
  numeric digits 12
  w = 999999999 + 1
  signal on lostdigits
  numeric digits d
  interpret 'x=' expr
  say left(expr, 20) 'none' x
  return
lostdigits:
  say left(expr, 20) condition('C') '['condition('D')']' condition('I') sigl
  return

callany:
  say 'CALL ON ANY ran' condition('C')
  return

any:
  say 'ANY' condition('C') condition('D') sigl
  signal afterany

incaller:
  say 'caller trap' condition('C') condition('D') sigl
  signal aftersub

inloop:
  say 'loop header' condition('C') condition('D') sigl
  signal afterloop

sub: procedure
  z = 1 / 9999999999
  return z

::class probe
::method run
  use arg expr
  signal on lostdigits
  interpret 'x=' expr
  return 'method none' x
lostdigits:
  return 'method' condition('C') condition('D')
