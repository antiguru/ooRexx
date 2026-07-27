/* extracted from Table::test_putAll */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  tmpColl=clz~new

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  cu=collDir~unionColl

  self~assertTrue(sameContent(ce, ce~copy~~putAll(ce)))
  self~assertTrue(sameContent(c1, c1~copy~~putAll(ce)))
  self~assertTrue(sameContent(c2, c2~copy~~putAll(ce)))

  self~assertTrue(sameContent(c1, clz~new~~putAll(c1)))
  self~assertTrue(sameContent(c2, clz~new~~putAll(c2)))

  self~assertTrue(sameContent(cu, c1~copy~~putAll(c2)))
  self~assertTrue(sameContent(cu, c2~copy~~putAll(c1)))



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
