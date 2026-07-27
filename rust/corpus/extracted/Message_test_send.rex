/* extracted from Message::test_send */
::routine main public
  m = .message~new('abc', 'length')
  self~assertEquals('abc', m~target)
  self~assertEquals('LENGTH', m~messageName)
  self~assertTrue(m~arguments~isEmpty)
  self~assertFalse(m~hasResult)
  self~assertFalse(m~hasError)
  self~assertFalse(m~completed)
  self~assertEquals(3, m~send)
  self~assertEquals(3, m~result)
  self~assertTrue(m~hasResult)
  self~assertFalse(m~hasError)
  self~assertTrue(m~completed)
  -- verify this can be invoked a second time
  self~assertEquals(3, m~send)
  -- specify a different receiver
  self~assertEquals(5, m~send("aeiou"))
  -- the message object updates
  self~assertEquals('aeiou', m~target)

  -- new method that takes arguments
  m = .message~new('abc', 'subchar', 'individual', 3)
  self~assertTrue(m~arguments~equivalent(.array~of(3)))
  self~assertEquals('c', m~send)
  self~assertEquals('a', m~send(,1))
  -- arguments update
  self~assertTrue(m~arguments~equivalent(.array~of(1)))
  self~assertEquals('e', m~send('def',2))
  self~assertEquals('def', m~target)
  self~assertTrue(m~arguments~equivalent(.array~of(2)))
  self~assertEquals('i', m~sendWith('ghi',.array~of(3)))
  self~assertEquals('ghi', m~target)
  self~assertTrue(m~arguments~equivalent(.array~of(3)))

  self~assertEquals('g', m~sendWith(,.array~of(1)))
  self~assertEquals('ghi', m~target)
  self~assertTrue(m~arguments~equivalent(.array~of(1)))

-- simple start tests
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
