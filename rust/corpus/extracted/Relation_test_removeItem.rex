/* extracted from Relation::test_removeItem */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem o1 o2

  ce=collDir~emptyColl~copy
  c1=collDir~coll_1~copy

  self~assertNull(ce~removeItem("1", "1"))
  self~assertNull(ce~removeItem(.nil, .nil))

  self~assertNull(c1~removeItem("99", "99"))

  self~assertEquals("1", c1~removeItem("1", "1"))
  self~assertNull(c1~removeItem("1", "1"))

  self~assertEquals("2", c1~removeItem("2","2"))
  self~assertEquals("2", c1~removeItem("2","2"))
  self~assertNull(c1~removeItem("2","2"))

  self~assertEquals(o1, c1~removeItem(o1,o1))
  self~assertEquals(o1, c1~removeItem(o1,o1))
  self~assertNull(c1~removeItem(o1,o1))

  self~assertEquals(0, c1~items)



-- TODO: determine whether INDEX part may truly be left out, otherwise change
--       testcase to expect the appropriate error message
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
