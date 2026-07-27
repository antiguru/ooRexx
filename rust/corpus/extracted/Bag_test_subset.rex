/* extracted from Bag::test_subset */
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
  o1=.object~new
  c =clz~new~~put("1")~~put("2")~~put("2")~~put(o1)~~put(o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d2=c~subSet(other)
  self~assertFalse(d2, "subtest14: 'other' is an 'OrderedCollection'")

  -- Simplistic test of subset where the argument is a table
  obj = .Bag.testGroup~new
  receiverBag = .bag~of("Lion", "Tiger", "Dog", obj)

  argumentTable = .table~new
  argumentTable["Dog"] = "Dog"
  argumentTable[obj] = obj
  self~assertFalse(receiverBag~subset(argumentTable), "Lion Tiger Dog 'obj' is not a subset of Dog 'obj'")

  argumentTable = .table~new
  argumentTable["Elm"] = "Elm"
  argumentTable["Maple"] = "Maple"
  argumentTable["Dog"] = "Dog"
  argumentTable[obj] = obj

  receiverBag~remove("Lion")
  receiverBag~remove("Tiger")
  self~assertTrue(receiverBag~subset(argumentTable), "Dog 'obj' is a subset of Elm Maple Dog 'obj'")

  receiverBag~remove("Dog")
  receiverBag~remove(obj)
  self~assertTrue(receiverBag~subset(argumentTable), "Empty bag is a subset of Elm Maple Dog 'obj'")

  emptyTable = .table~new
  self~assertTrue(receiverBag~subset(emptyTable), "Empty bag is a subset table")

   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~put("1")~~put("2")~~put("2")~~put(o1)~~put(o1)
  other=.array~new
  other[101]="1"
  other[102]="2"
  other[103]="3"
  other[104]="2"
  other[106]=o1
  other[107]=o1
  d2=c~subSet(other)
  self~assertTrue(d2, "subtest16: 'other' is an 'OrderedCollection'")



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
