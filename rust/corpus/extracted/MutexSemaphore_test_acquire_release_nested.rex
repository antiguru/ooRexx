/* extracted from MutexSemaphore::test_acquire_release_nested */
::routine main public
  -- mutexes are created in the released state
  sem = .MutexSemaphore~new
  self~assertFalse(sem~release)

  -- acquire twice, release twice
  self~assertTrue(sem~acquire)
  self~assertTrue(sem~acquire(0.001))
  self~assertTrue(sem~release)
  self~assertTrue(sem~release)
  self~assertFalse(sem~release)

  -- acquire three times, and release
  self~assertTrue(sem~acquire(0.001))
  self~assertTrue(sem~acquire(-1))
  self~assertTrue(sem~acquire(.TimeSpan~fromSeconds(2)))
  self~assertTrue(sem~release)
  self~assertTrue(sem~release)
  self~assertTrue(sem~release)
  self~assertFalse(sem~release)

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
