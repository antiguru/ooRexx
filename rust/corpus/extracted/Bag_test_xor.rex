/* extracted from Bag::test_xor */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  res=collDir~xorColl

  self~assertTrue(sameContent(ce, ce~xor(ce)))

  self~assertTrue(sameContent(c1, c1~xor(ce)))
  self~assertTrue(sameContent(c2, c2~xor(ce)))

  self~assertTrue(sameContent(res, c1~xor(c2)))
  self~assertTrue(sameContent(res, c2~xor(c1)))

  self~assertTrue(sameContent(ce, c1~xor(c1)))
  self~assertTrue(sameContent(ce, c2~xor(c2)))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~put("1")~~put("2")~~put("2")~~put(o1)~~put(o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1", "2", o1)          -- expected result
  d2=c~xor(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of XOR where the argument is a table
  obj = .Bag.testGroup~new
  lionStr = "Lion"
  tigerStr = "Tiger"
  dogStr = "Dog"
  receiverBag = .bag~of(lionStr, tigerStr, dogStr, obj, tigerStr, tigerStr)

  argumentTable = .table~new
  elmStr = "Elm"
  mapleStr = "Maple"
  argumentTable[ElmStr] = ElmStr
  argumentTable[MapleStr] = MapleStr
  argumentTable[DogStr] = DogStr
  argumentTable[obj] = obj

  expectedBag = .bag~of(LionStr, TigerStr, TigerStr, TigerStr, ElmStr, MapleStr)
  resultBag = receiverBag~xor(argumentTable)

  nl = '0d0a'x
  self~assertTrue(resultBag~isA(.bag), "Result object after union must be a bag")

  self~assertSame(6, resultBag~items, "XOR of Lion Tiger Dog 'obj' Tiger Tiger with Elm Maple Dog 'obj' should" || nl ||                    "contain 6 items")

  self~assertTrue(sameContent(expectedBag, resultBag), "Using Bag, XOR of Lion Tiger Dog 'obj' Tiger Tiger with Elm Maple Dog 'obj'" || nl ||                    "should be Lion Tiger Tiger Tiger Elm Maple")

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
