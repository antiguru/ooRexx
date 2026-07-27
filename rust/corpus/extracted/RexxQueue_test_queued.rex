/* extracted from RexxQueue::test_queued */
::routine main public
  -- invalid queue names return 0
  do name over "", "*", "a"~copies(300)
    self~assertSame(0, .RexxQueue~new(name)~queued)
  end

  -- SESSION and valid named queue
  do name over "session", "test_queued"
    q = .RexxQueue~new(name)
    q~empty
    self~assertSame(0, q~queued)
    q~push
    self~assertSame(1, q~queued)
    q~pull
    self~assertSame(0, q~queued)
    q~queue
    q~lineout("line")
    self~assertSame(2, q~queued)
    do i = 1 to 99
      q~say(i)
    end
    self~assertSame(2 + 99, q~queued)
    self~assertSame(0, q~empty)
    self~assertSame(0, q~queued)
  end
  self~assertSame(0, q~delete) -- clean up


-- push, queue, and queue aliases lineOut, say

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
