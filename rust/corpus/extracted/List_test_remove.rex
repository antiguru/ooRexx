/* extracted from List::test_remove */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem o1 o2

  ce=collDir~emptyColl~copy
  c1=collDir~coll_1~copy

  self~assertNull(ce~remove(98))

  self~assertNull(c1~remove(99))

  self~assertEquals("1v", c1~remove(c1~first))
  self~assertEquals("2v", c1~remove(c1~first))
  self~assertEquals("2v", c1~remove(c1~first))
  self~assertEquals("2v", c1~remove(c1~first))
  self~assertEquals(o1, c1~remove(c1~first))
  self~assertEquals(o1, c1~remove(c1~first))
  self~assertNull(c1~remove("0"))
  self~assertEquals(0, c1~items)

  -- now doing it in reverse order
  c1=collDir~coll_1~copy

  self~assertEquals(6, c1~items)

  idx = c1~last
  self~assertEquals(o1, c1~remove(idx))
  self~assertNull(c1~remove(idx))

  idx = c1~last
  self~assertEquals(o1, c1~remove(idx))
  self~assertNull(c1~remove(idx))

  idx = c1~last
  self~assertEquals("2v", c1~remove(idx))
  self~assertNull(c1~remove(idx))

  idx = c1~last
  self~assertEquals("2v", c1~remove(idx))
  self~assertNull(c1~remove(idx))

  idx = c1~last
  self~assertEquals("2v", c1~remove(idx))
  self~assertNull(c1~remove(idx))

  idx = c1~last
  self~assertEquals("1v", c1~remove(idx))
  self~assertNull(c1~remove(idx))

  self~assertEquals(0, c1~items)

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
