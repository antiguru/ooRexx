/* extracted from METHOD::testSetGetMutableBuffer */
::routine main public
  tester = .METHODtester~new

  buffer = .mutablebuffer~new("1234567890", 10)
  tester~TestSetMutableBufferValue(buffer, "abcdef")
  self~assertSame(10, buffer~getBufferSize)
  self~assertSame(6, buffer~length)
  self~assertSame("abcdef", tester~TestGetMutableBufferValue(buffer))

  -- now force the buffer to expand the size
  tester~TestSetMutableBufferValue(buffer, "12345678901234567890")
  self~assertSame(20, buffer~getBufferSize)
  self~assertSame(20, buffer~length)
  self~assertSame("12345678901234567890", buffer~string)
  self~assertSame("12345678901234567890", tester~TestGetMutableBufferValue(buffer))


-- test new APIs for [feature-requests:#634] Add a per-object storage management API
-- AllocateObjectMemory, FreeObjectMemory, ReallocateObjectMemory
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
