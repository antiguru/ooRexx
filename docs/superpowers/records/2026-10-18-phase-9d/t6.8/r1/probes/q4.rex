say a.
call t 'twiceStem', >a.
say 'a.1' a.1
drop c.
call t 'twiceStem', >c.
say 'c.1' c.1
d.1 = 'x'; drop d.1
call t 'twiceStem', >d.
call t 'twiceStem', >b.
z. = e.
call t 'twiceStem', >z.
call t 'twiceRenamed', >a., >f.
exit
t: procedure
  use arg name, r1, r2
  signal on syntax name caught
  if arg() = 2 then answer = .k~new~send(name, r1)
  else answer = .k~new~send(name, r1, r2)
  say name 'answers' answer
  return
caught:
  say name condition('o')~code condition('o')~message
  return
::class k
::method twiceStem
  use arg >abc.
  use arg >abc.
  abc.1 = 'set'
  return 'ok'
::method twiceRenamed
  use arg >abc.
  use arg , >abc.
  return 'ok'
