/* extracted from Stem::test_subset */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  cu=collDir~unionColl

  self~assertTrue(ce~subset(ce))
  self~assertTrue(ce~subset(c1))
  self~assertTrue(ce~subset(c2))

  self~assertTrue(c1~subset(c1))
  self~assertTrue(c2~subset(c2))

  self~assertTrue(ce~subset(cu))
  self~assertTrue(c1~subset(cu))
  self~assertTrue(c2~subset(cu))

  self~assertFalse(c1~subset(ce))
  self~assertFalse(c1~subset(c2))

  self~assertFalse(c2~subset(ce))
  self~assertFalse(c2~subset(c1))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
      -- use a relation which is bag-like
  c =clz~new~~put("1", "1")~~put("2", "2")~~put(o1, o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d2=c~subSet(other)
  self~assertFalse(d2, "subtest14: 'other' is an 'OrderedCollection'")

   -- now test where other is a 'MapCollection': "makeArray" returns "allItems"
  other=.table~new   -- use a table which is set-like
  other["2"]="2"
  other[o1] =o1
  d2=c~subSet(other)

  self~assertFalse(d2, "subtest15: 'other' is a 'MapCollection'")



   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  other=.array~new
  other[101]="1"
  other[102]="2"
  other[103]="3"
  other[104]="2"
  other[106]=o1
  other[107]=o1
  d2=c~subSet(other)
  self~assertTrue(d2, "subtest16: 'other' is an 'OrderedCollection'")

   -- now test where other is a 'MapCollection': "makeArray" returns "allItems"
  other=.table~new
  other["1"]="1"
  other["2"]="2"
  other[o1] =o1
  other["3"] ="3"
  d2=c~subSet(other)

  self~assertTrue(d2, "subtest17: 'other' is a 'MapCollection'")



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
