/* extracted from RexxQueue::test_pull_nil */
::routine main public
  -- both pull and lineIn return .nil for an invalid queue
  do bad over "", "*", "a"~copies(300)
    self~assertSame(.nil, .RexxQueue~new(bad)~pull)
    self~assertSame(.nil, .RexxQueue~new(bad)~lineIn)
  end

  -- pull won't wait if a valid queue is empty (lineIn will)
  q = .RexxQueue~new("test_pull_nil")~~empty
  self~assertSame(.nil, q~pull)
  self~assertSame(.nil, q~pull)

  -- when a valid queue is deleted, both pull and lineIn return .nil
  self~assertSame(0, q~delete)
  self~assertSame(.nil, q~pull)
  self~assertSame(.nil, q~lineIn)

-- makeArray

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
