o = .k~new
call try o, 'pair', a., >b.
call try o, 'pair', abc., >b.
call try o, 'pair', .stem~new('ABC.'), >b.
call try o, 'pair', .stem~new('XYZ.'), >b.
call try o, 'pairDropped', a., >b.
call try o, 'single', >b.
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
::method pairDropped
  use arg abc.
  drop abc.
  use arg , >abc.
  return 'bound after drop'
::method single
  use arg >abc.
  return 'bound'
