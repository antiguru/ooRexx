/* extracted from EventSemaphore::test_wait_simple */
::routine main public
  -- semaphores are created in the non-posted state
  sem = .EventSemaphore~new

  -- a wait with zero timeout returns immediately
  self~assertFalse(sem~wait(0))
  self~assertFalse(sem~isPosted)

  -- wait with 1 ms timeout
  self~assertFalse(sem~wait(0.001))
  self~assertFalse(sem~isPosted)

  -- wait with TimeSpan timeout
  self~assertFalse(sem~wait(.TimeSpan~fromMicroseconds(5000)))
  self~assertFalse(sem~isPosted)

  -- now, post the semaphore
  sem~post

  -- wait without timeout
  self~assertTrue(sem~wait)
  self~assertTrue(sem~isPosted)

  -- wait with indefinite timeout
  self~assertTrue(sem~wait(-10))
  self~assertTrue(sem~isPosted)

  -- a wait with zero timeout returns immediately
  self~assertTrue(sem~wait(0))
  self~assertTrue(sem~isPosted)

  -- wait with 1 ms timeout
  self~assertTrue(sem~wait(0.001))
  self~assertTrue(sem~isPosted)

  sem~reset
  self~assertFalse(sem~isPosted)

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
