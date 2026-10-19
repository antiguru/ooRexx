o = .k~new
n = 'AB' || 'C.'
call try o, 'pair', .stem~new(n), >b.
call try o, 'pair', .stem~new('abc.'), >b.
call try o, 'pair', .stem~new, >b.
s = .stem~new('ABC.'); s[1] = 1; s~remove(1)
call try o, 'pair', s, >b.
call try o, 'other', .stem~new('ABC.'), >b.
exit
try: procedure
  use arg o, message, first, second
  signal on syntax name caught
  say message 'answers' o~send(message, first, second)
  return
caught:
  say message condition('o')~code condition('o')~message
  return
::class k
::method pair
  use arg abc., >abc.
  return 'bound'
::method other
  use arg abd., >abd.
  return 'bound'
