/* extracted from QueueRGF::test_remove */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem o1 o2

  ce=collDir~emptyColl~copy
  c1=collDir~coll_1~copy

  self~assertNull(ce~remove(98))

  self~assertNull(c1~remove(99))

   -- if removing an item, all successive items move up to the top
  self~assertEquals("1v", c1~remove("1"))
  self~assertEquals("2v", c1~remove("1"))
  self~assertEquals("2v", c1~remove("1"))
  self~assertEquals("2v", c1~remove("1"))
  self~assertEquals(o1, c1~remove("1"))
  self~assertEquals(o1, c1~remove("1"))
  self~assertNull(c1~remove("1"))
  self~assertEquals(0, c1~items)

  -- now doing it in reverse order
  c1=collDir~coll_1~copy

  self~assertEquals(6, c1~items)

  self~assertEquals(o1, c1~remove("6"))
  self~assertNull(c1~remove("6"))

  self~assertEquals(o1, c1~remove("5"))
  self~assertNull(c1~remove("5"))

  self~assertEquals("2v", c1~remove("4"))
  self~assertNull(c1~remove("4"))

  self~assertEquals("2v", c1~remove("3"))
  self~assertNull(c1~remove("3"))

  self~assertEquals("2v", c1~remove("2"))
  self~assertNull(c1~remove("2"))

  self~assertEquals("1v", c1~remove("1"))
  self~assertNull(c1~remove("1"))

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
