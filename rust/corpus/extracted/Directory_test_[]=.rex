/* extracted from Directory::test_[]= */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  tmpColl=clz~new
  self~assertEquals(0, tmpColl              ~items)

  self~assertEquals(1, tmpColl~~"[]="("1","1")~items)
  self~assertTrue(tmpColl~hasindex("1"))
  self~assertTrue(tmpColl~hasitem("1"))

  self~assertEquals(2, tmpColl~~"[]="("2","2")~items)
  self~assertTrue(tmpColl~hasindex("2"))
  self~assertTrue(tmpColl~hasitem("2"))

  self~assertEquals(2, tmpColl~~"[]="("3","2")~items)
  self~assertTrue(tmpColl~hasindex("2"))
  self~assertTrue(tmpColl~hasitem("3"))

  self~assertEquals(2, tmpColl~~"[]="("2","2")~items)
  self~assertTrue(tmpColl~hasindex("2"))
  self~assertTrue(tmpColl~hasitem("2"))

  self~assertEquals(3, tmpColl~~"[]="("3","3")~items)
  self~assertTrue(tmpColl~hasindex("3"))
  self~assertTrue(tmpColl~hasitem("3"))

  self~assertTrue(sameContent(.bag~of("1","2","3"), tmpColl))



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
