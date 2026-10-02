-- Message~halt reaching a started activity spinning in an empty-body loop
-- ends the loop with 4.1, raised in the spinning method's own activation,
-- for each kind of loop. Which clause the halt lands on is not printed: it
-- depends on where the request finds the loop (ruling P32).
do kind over .array~of('forever', 'while', 'controlled', 'until')
  g = .gate~new
  m = .message~new(.t~new, kind, 'I', g)
  m~start
  do while g~i < 5
  end
  say kind 'halt' m~halt
  m~wait
  say kind 'error' m~hasError 'result' m~result
end
::class gate
::attribute i unguarded
::method init
  expose i
  i = 0
::class t
::method forever
  use arg g
  signal on syntax name trapped
  g~i = 10
  do forever
  end
trapped:
  return 'forever' condition('c') condition('o')~code
::method while
  use arg g
  signal on syntax name trapped
  g~i = 10
  do while 1
  end
trapped:
  return 'while' condition('c') condition('o')~code
::method controlled
  use arg g
  signal on syntax name trapped
  g~i = 10
  do i = 1
  end
trapped:
  return 'controlled' condition('c') condition('o')~code
::method until
  use arg g
  signal on syntax name trapped
  g~i = 10
  do until 0
  end
trapped:
  return 'until' condition('c') condition('o')~code
