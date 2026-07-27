/* extracted from Array::test_difference */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2

  self~assertEquals(ce, ce~difference(ce))

  self~assertEquals(c1, c1~difference(ce))
  self~assertEquals(c2, c2~difference(ce))

  self~assertEquals(collDir~difference1, c1~difference(c2))
  self~assertEquals(collDir~difference2, c2~difference(c1))

  self~assertEquals(ce, c1~difference(c1))
  self~assertEquals(ce, c2~difference(c2))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~put("1", "101")~~put("2", "102")~~put(o1, "103")
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")     -- expected result
  d2=c~difference(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic .Array difference .Table test
  receiverArray = .array~new
  receiverArray[1] = 'Lemon'
  receiverArray[2] = 'Orange'
  receiverArray[3] = 'Dogwood'

  argumentTable = .table~new
  argumentTable[1] = 'Dogwood'
  argumentTable[2] = 'Elm'
  argumentTable[3] = 'Rose'
  argumentTable[4] = 'Tulip'

  resultObj = receiverArray~difference(argumentTable)

  nl = '0d0a'
  self~assertTrue(resultObj~isA(.array), "After set-like operation (intersection) result object must be array")
  self~assertSame(2, resultObj~items, 'Intersection of Lemon Orange Dogwood with Dogwood Elm Rose Tulip' || nl ||                    'should have 2 items')
  self~assertTrue(resultObj~hasItem('Lemon'), 'Intersection result should have a Lemon item')
  self~assertTrue(resultObj~hasItem('Orange'), 'Intersection result should have an Orange item')


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
