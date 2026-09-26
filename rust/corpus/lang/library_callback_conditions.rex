/* The thread table's raise members through orxmethod's methods, each of
   which goes on running after the raise: the condition a SIGNAL ON SYNTAX
   sees with its substitutions, and RaiseCondition's RESULT answered where
   no trap takes it, handed to CALL ON and to SIGNAL ON. */
t = .T~new
call try t, 'r0', 40001
call try t, 'r0', 88917
call try t, 'r1', 88917, 'Conversion error'
call try t, 'r1', 40001, .array~of(1,2)
call try t, 'r2', 93903, 'x', 'y'
call try t, 'ra', 93903, .array~of('p', 'q', 'r')
call try t, 'ra', 40001, .array~new
say 'untrapped' t~rc('USER FOO', , .array~of(1,2), 'res') t~continue
t~rc('USER FOO'); say 'nores' var('result')
t~rc('ERROR'); say 'error' var('result')
call on user bar name handler
say 'callon' t~rc('USER BAR', , , 'r2')
say 'after callon'
signal on user baz
say 'signalon' t~rc('USER BAZ', , .array~of('a'), 'r3')
say 'not here'
exit
baz:
  say 'caught' condition('c') '['condition('d')']' condition('i') condition('a')~items condition('o')~result
  exit
handler:
  say 'handler' condition('c') '['condition('d')']' condition('i') condition('o')~result
  return 'fromhandler'
try: procedure
  use arg t, m, n, a, b
  t~continue = 0
  signal on syntax
  select
    when m == 'r0' then t~r0(n)
    when m == 'r1' then t~r1(n, a)
    when m == 'r2' then t~r2(n, a, b)
    otherwise t~ra(n, a)
  end
  say 'no raise'
  return
syntax:
  o = condition('o')
  say m n '->' o~code o~rc '|' o~errortext '|' o~message '|' o~additional~items '|' t~continue
  return
::class T
::attribute continue
::method r0 external "LIBRARY orxmethod TestRaiseException0"
::method r1 external "LIBRARY orxmethod TestRaiseException1"
::method r2 external "LIBRARY orxmethod TestRaiseException2"
::method ra external "LIBRARY orxmethod TestRaiseException"
::method rc external "LIBRARY orxmethod TestRaiseCondition"
