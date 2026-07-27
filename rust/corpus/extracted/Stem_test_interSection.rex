/* extracted from Stem::test_interSection */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  res=collDir~interSectionColl

  self~assertTrue(sameContent(ce, ce~interSection(ce)))
  self~assertTrue(sameContent(ce, c1~interSection(ce)))
  self~assertTrue(sameContent(ce, c2~interSection(ce)))

  self~assertTrue(sameContent(res, c1~interSection(c2)))
  self~assertTrue(sameContent(res, c2~interSection(c1)))

  self~assertTrue(sameContent(c1, c1~interSection(c1)))
  self~assertTrue(sameContent(c2, c2~interSection(c2)))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  c =clz~new~~put("1", "1")~~put("2", "2")~~put(o1, o1)~~put("a","a")
  other=.array~new
  other[104]="2"
  other[106]=o1
  other[107]="a"
                 -- expected result
  d1=clz~new~~put("a", "a")~~put(2,2)~~put(o1,o1)
  d2=c~interSection(other)

  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

   -- now test where other is a MapCollection: "makeArray" returns "allItems"
  other=.table~new
  other["2"]=2
  other[o1] =o1
  other["a"] ="a"
                 -- expected result
  d1=clz~new~~put("<2-value>", "2")~~put("<o1-value>", o1)~~put("a","a")
  d2=c~interSection(other)
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
