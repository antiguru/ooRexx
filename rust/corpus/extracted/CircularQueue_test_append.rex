/* extracted from CircularQueue::test_append */
::routine main public

  a = .circularqueue~new(3)

  self~assertEquals(1, a~append(1))
  self~assertEquals(3, a~size)
  self~assertEquals(1, a~items)
  self~assertSame(1, a~firstitem)
  self~assertSame(1, a~lastitem)

  self~assertEquals(2, a~append(2))
  self~assertEquals(3, a~size)
  self~assertEquals(2, a~items)
  self~assertSame(1, a~firstitem)
  self~assertSame(2, a~lastitem)

  self~assertEquals(3, a~append(3))
  self~assertEquals(3, a~size)
  self~assertEquals(3, a~items)
  self~assertSame(1, a~firstitem)
  self~assertSame(3, a~lastitem)

  self~assertEquals(3, a~append(4))
  self~assertEquals(3, a~size)
  self~assertEquals(3, a~items)
  self~assertSame(2, a~firstitem)
  self~assertSame(4, a~lastitem)

  a~appendAll(.array~of(5, 6))
  self~assertEquals(3, a~size)
  self~assertEquals(3, a~items)
  self~assertSame(4, a~firstitem)
  self~assertSame(6, a~lastitem)

  a~appendAll(.array~of(7, 8, 9, 10))
  self~assertEquals(3, a~size)
  self~assertEquals(3, a~items)
  self~assertSame(8, a~firstitem)
  self~assertSame(10, a~lastitem)


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
