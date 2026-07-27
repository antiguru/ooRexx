/* extracted from Bag::test_interSection */
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
  o1=.object~new
  c =clz~new~~put("1")~~put("2")~~put("2")~~put(o1)~~put(o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("2", o1)          -- expected result
  d2=c~interSection(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic intersection test where the argument collection is a table.
  obj = .Bag.testGroup~new
  receiverBag = .bag~of("Lion", "Tiger", "Dog", obj)

  argumentTable = .table~new
  argumentTable["Elm"] = "Elm"
  argumentTable["Maple"] = "Maple"
  argumentTable["Dog"] = "Dog"
  argumentTable[obj] = obj

  expectedBag = .bag~of("Dog", obj)
  resultBag = receiverBag~intersection(argumentTable)

  self~assertTrue(resultBag~isA(.bag), "Result object after intersection must be a bag")
  self~assertTrue(sameContent(expectedBag, resultBag), "Intersection of bag and table should be Dog and obj")



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
