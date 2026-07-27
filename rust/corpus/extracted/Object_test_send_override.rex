/* extracted from Object::test_send_override */
::routine main public
  obj = .sendTester~new
  -- non-array version of the test, just as a baseline
  self~assertEquals(123, obj~testSend(obj, 'TestTarget'))
  self~assertEquals(123, obj~testSendWith(obj, 'TestTarget', (1,2)))

  -- now with a superclass override
  self~assertEquals(456, obj~testSend(obj, ('TestTarget', .sendTester2)))
  self~assertEquals(456, obj~testSendWith(obj, ('TestTarget', .sendTester2), (1,2)))

  -- this should still work even starting from the top class
  self~assertEquals(123, obj~testSend(obj, ('TestTarget', .sendTester)))
  self~assertEquals(123, obj~testSendWith(obj, ('TestTarget', .sendTester), (1,2)))

-- issued from a different class context
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
