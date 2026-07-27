/* extracted from Bag::test_union */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  res=collDir~unionColl

  self~assertTrue(sameContent(ce, ce~union(ce)))

  self~assertTrue(sameContent(c1, c1~union(ce)))
  self~assertTrue(sameContent(c2, c2~union(ce)))

  self~assertTrue(sameContent(c1, ce~union(c1)))
  self~assertTrue(sameContent(c2, ce~union(c2)))

  self~assertTrue(sameContent(res, c1~union(c2)))
  self~assertTrue(sameContent(res, c2~union(c1)))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~put("1")~~put("2")~~put("2")~~put(o1)~~put(o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1", "2", "2", "2", o1, o1, o1)  -- expected result
  d2=c~union(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of union where the argument collection is a table.
  obj = .Bag.testGroup~new
  lionStr = "Lion"
  tigerStr = "Tiger"
  dogStr = "Dog"
  receiverBag = .bag~of(lionStr, tigerStr, dogStr, obj)

  argumentTable = .table~new
  elmStr = "Elm"
  mapleStr = "Maple"
  argumentTable[ElmStr] = ElmStr
  argumentTable[MapleStr] = MapleStr
  argumentTable[DogStr] = DogStr
  argumentTable[obj] = obj

  expectedBag = .bag~of(LionStr, TigerStr, DogStr, obj, ElmStr, MapleStr, DogStr, obj)
  resultBag = receiverBag~union(argumentTable)

  nl = '0d0a'x
  self~assertTrue(resultBag~isA(.bag), "Result object after union must be a bag")

  -- Documentation from Object Rexx:
  -- Set-Like Operations on Collections with Duplicates
  -- UNION operation
  --  All elements of A and B are united:
  --
  -- Using that definition, the new bag should have 8 items and contain every
  -- item in receiverBag and argumentTable.
  self~assertSame(8, resultBag~items, "Union of Lion Tiger Dog 'obj' with Elm Maple Dog 'obj' should" || nl ||                    "contain all 8 items")

  self~assertTrue(sameContent(expectedBag, resultBag), "Using Bag, union of Lion Tiger Dog 'obj' with Elm Maple Dog 'obj'" || nl ||                    "should be Lion Tiger Dog 'obj' Elm Maple Dog 'obj'")





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
