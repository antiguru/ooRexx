/* extracted from Array::testDelete */
::routine main public
  -- TODO ADD cases where an extension was performed.
  -- TODO add case where a non- occupied slot is deleted
  target = .array~of("A","B","C")
  self~assertEquals("B", target~delete(2))
  self~assertEquals(2, target~size)
  self~assertEquals(2, target~items)
  self~assertEquals(.array~of("A", "C"), target)

  target = .array~of("A","B","C")
  self~assertEquals("B", target~delete(.array~of(2)))
  self~assertEquals(2, target~size)
  self~assertEquals(2, target~items)
  self~assertEquals(.array~of("A", "C"), target)

  target = .array~of("A",,"C")
  self~assertNull(target~delete(2))
  self~assertEquals(2, target~size)
  self~assertEquals(2, target~items)
  self~assertEquals(.array~of("A", "C"), target)

  target = .array~new(0)
  self~assertNull(target~delete(5))
  self~assertEquals(0, target~size)
  self~assertEquals(0, target~items)
  self~assertEquals(.array~new(0), target)

  target = .array~of("A","B","C")
  self~assertEquals("A", target~delete(1))
  self~assertEquals(2, target~size)
  self~assertEquals(2, target~items)
  self~assertEquals(.array~of("B", "C"), target)

  target = .array~of("A","B","C")
  self~assertEquals("C", target~delete(3))
  self~assertEquals(2, target~size)
  self~assertEquals(2, target~items)
  self~assertEquals(.array~of("A", "B"), target)

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
