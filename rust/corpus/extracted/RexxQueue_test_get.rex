/* extracted from RexxQueue::test_get */
::routine main public
  self~assertSame("SESSION", .RexxQueue~new~get)

  -- SESSION or invalid queues should still return same get/string
  do name over "session", "", "*", "a"~copies(300)
    self~assertSame(name~upper, .RexxQueue~new(name)~get)
    self~assertSame(name~upper, .RexxQueue~new(name)~string)
  end

  name = "test_get"
  qLower = .RexxQueue~new(name)
  qLower~empty -- just to make sure
  self~assertSame(name~upper, qLower~get)
  self~assertSame(qLower~get, qLower~string)

  qUpper = .RexxQueue~new(name~upper)
  self~assertSame(name~upper, qUpper~get)
  self~assertSame(qUpper~get, qUpper~string)

  -- both qLower and qUpper refer to the same external data queue
  self~assertSame(0, qLower~queued)
  self~assertSame(0, qUpper~queued)
  qLower~lineOut("line")
  self~assertSame(1, qLower~queued)
  self~assertSame(1, qUpper~queued)
  self~assertSame(0, .RexxQueue~delete(name)) -- clean up


-- set

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
