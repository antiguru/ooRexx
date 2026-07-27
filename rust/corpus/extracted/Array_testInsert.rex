/* extracted from Array::testInsert */
::routine main public
  target = .array~of(1,2,3)
  self~assertEquals(3, target~insert(4, 2))
  self~assertEquals(4, target~items)
  self~assertEquals(4, target~size)
  self~assertEquals(.array~of(1,2,4,3), target)

  target = .array~of(1,2,3)
  self~assertEquals(3, target~insert(4, .array~of(2)))
  self~assertEquals(4, target~items)
  self~assertEquals(4, target~size)
  self~assertEquals(.array~of(1,2,4,3), target)

  target = .array~of(1,2,3)
  self~assertEquals(3, target~insert(,2))
  self~assertEquals(3, target~items)
  self~assertEquals(4, target~size)
  self~assertNull(target[3])
  self~assertFalse(target~hasIndex(3))
  self~assertEquals(.array~of(1,2,,3), target)

  target = .array~new(0)
  self~assertEquals(6, target~insert(4,5))
  self~assertEquals(1, target~items)
  self~assertEquals(6, target~size)
  self~assertEquals(.array~of(,,,,,4), target)

  target = .array~new(0)
  self~assertEquals(1, target~insert(4,.nil))
  self~assertEquals(1, target~items)
  self~assertEquals(1, target~size)
  self~assertEquals(.array~of(4), target)

  target = .array~new(0)
  self~assertEquals(1, target~insert(4))
  self~assertEquals(1, target~items)
  self~assertEquals(1, target~size)
  self~assertEquals(.array~of(4), target)

  -- insert before the first item
  target = .array~of(1,2,3)
  self~assertEquals(1, target~insert(4, .nil))
  self~assertEquals(4, target~items)
  self~assertEquals(4, target~size)
  self~assertEquals(.array~of(4,1,2,3), target)

  -- insert at the end

  target = .array~of(1,2,3)
  self~assertEquals(4, target~insert(4,))
  self~assertEquals(4, target~items)
  self~assertEquals(4, target~size)
  self~assertEquals(.array~of(1,2,3,4), target)

  target = .array~new(0)
  self~assertEquals(1, target~insert(4,.nil))
  self~assertEquals(1, target~items)
  self~assertEquals(1, target~size)
  self~assertEquals(.array~of(4), target)

  target = .array~new(0)
  self~assertEquals(1, target~insert(4))
  self~assertEquals(1, target~items)
  self~assertEquals(1, target~size)
  self~assertEquals(.array~of(4), target)

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
