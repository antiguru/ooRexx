/* extracted from RexxQueue::test_push_et_al */
::routine main public
  q = .RexxQueue~new("test_push")
  q~empty -- just to make sure
  do method over "push", "queue", "lineOut", "say"
    -- omitted argument
    self~assertSame(0, q~send(method), method)
    self~assertSame(1, q~queued, method)
    self~assertSame("", q~lineIn, method)

    -- arguments null string, NUL char, full xrange, and 10.000 blanks
    do arg over "", '00'x, xrange('00'x, 'ff'x), " "~copies(10000)
      self~assertSame(0, q~send(method, arg ), method arg~length "bytes")
      self~assertSame(1, q~queued, method arg~length "bytes")
      self~assertSame(arg, q~pull, method arg~length "bytes")
    end
  end
  q~delete -- clean up

::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
