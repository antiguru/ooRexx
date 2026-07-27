/* extracted from Stem::test_makeArray */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  tmp=collDir~coll_1~makeArray
  self~assertTrue(tmp~isInstanceOf(.array))
  if bUserDefinedIndex then
     self~assertTrue(sameContent(.bag~new~union(tmp                ),                                               .bag~new~union(collDir~allIndexes1)))
  else
     self~assertTrue(sameContent(.bag~new~union(tmp                ),                                               .bag~new~union(collDir~allItems1)))


  tmp=collDir~coll_2~makeArray
  self~assertTrue(tmp~isInstanceOf(.array))
  if bUserDefinedIndex then
     self~assertTrue(sameContent(.bag~new~union(tmp                ),                                               .bag~new~union(collDir~allIndexes2)))
  else
     self~assertTrue(sameContent(.bag~new~union(tmp                ),                                               .bag~new~union(collDir~allItems2)))


  tmp=collDir~emptyColl~makeArray
  self~assertTrue(tmp~isInstanceOf(.array))
  self~assertEquals(0, tmp~items)



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
