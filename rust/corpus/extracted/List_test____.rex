/* extracted from List::test_[]= */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  tmpColl=clz~new
  dummy="test"
  self~assertEquals(0, tmpColl~items)

  item="1"
  items=1
  self~assertFalse(tmpColl~hasindex(0))
  self~assertFalse(tmpColl~hasitem(item))
  tmpColl~insert(dummy)
  idx = tmpColl~first
  self~assertTrue(tmpColl~hasitem(dummy))
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertEquals(1, tmpColl~~"[]="(item,idx)~items)
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertTrue(tmpColl~hasitem(item))
  self~assertFalse(tmpColl~hasitem(dummy))

  idx=1
  item="2"
  items=2
  self~assertFalse(tmpColl~hasindex(idx))
  self~assertFalse(tmpColl~hasitem(item))
  tmpColl~insert(dummy)
  idx = tmpColl~last
  self~assertTrue(tmpColl~hasitem(dummy))
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertEquals(2, tmpColl~~"[]="(item,idx)~items)
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertTrue(tmpColl~hasitem(item))
  self~assertFalse(tmpColl~hasitem(dummy))

  idx=tmpColl~last
  item="3"
  items=2
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertEquals(2, tmpColl~~"[]="(item,idx)~items)
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertTrue(tmpColl~hasitem(item))
  self~assertFalse(tmpColl~hasitem(dummy))

  idx=tmpColl~last
  item="2"
  items=2
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertEquals(2, tmpColl~~"[]="(item,idx)~items)
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertTrue(tmpColl~hasitem(item))
  self~assertFalse(tmpColl~hasitem(dummy))

  idx=2
  ind=6
  item="3"
  items=3
  self~assertFalse(tmpColl~hasindex(idx))
  self~assertFalse(tmpColl~hasitem(item))
  tmpColl~insert(dummy)
  idx=tmpColl~last
  self~assertTrue(tmpColl~hasitem(dummy))
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertEquals(3, tmpColl~~"[]="(item,idx)~items)
  self~assertTrue(tmpColl~hasindex(idx))
  self~assertTrue(tmpColl~hasitem(item))
  self~assertFalse(tmpColl~hasitem(dummy))

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
