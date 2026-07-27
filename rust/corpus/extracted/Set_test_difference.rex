/* extracted from Set::test_difference */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2

  self~assertTrue(sameContent(ce, ce~difference(ce)))

  self~assertTrue(sameContent(c1, c1~difference(ce)))
  self~assertTrue(sameContent(c2, c2~difference(ce)))

  self~assertTrue(sameContent(collDir~difference1, c1~difference(c2)))
  self~assertTrue(sameContent(collDir~difference2, c2~difference(c1)))

  self~assertTrue(sameContent(ce, c1~difference(c1)))
  self~assertTrue(sameContent(ce, c2~difference(c2)))

   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~put("1")~~put("2")~~put(o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")     -- expected result
  d2=c~difference(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

   -- now test where other is a MapCollection: "makeArray" returns "allItems"
  other=.table~new
  other["2"]='item mapped by index "2"'
  other[o1] ='item mapped by index "o1"'
  d2=c~difference(other)

  self~assertTrue(sameContent(d1, d2), "subtest9: 'other' is a 'MapCollection'")



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
