/* extracted from Array::test_interSection */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  res=collDir~interSectionColl

  self~assertEquals(ce, ce~interSection(ce))

  self~assertEquals(ce, c1~interSection(ce))
  self~assertEquals(ce, c2~interSection(ce))

  self~assertEquals(res, c1~interSection(c2))
  self~assertEquals(res, c2~interSection(c1))

  self~assertEquals(c1, c1~interSection(c1))
  self~assertEquals(c2, c2~interSection(c2))

   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~put("1", 101)~~put("2", 102)~~put(o1, 103)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("2", o1)          -- expected result
  d2=c~interSection(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of array intersect table
  receiverArray = .array~of("Tree", "Rock", "Planet", "Star")
  argumentTable = .table~new
  argumentTable[1] = "Elm"
  argumentTable[2] = "Rock"
  argumentTable[3] = "Mars"
  argumentTable[4] = "Monroe"

  resultObj = receiverArray~intersection(argumentTable)

  nl = '0d0a'x
  self~assertSame(.array, resultObj~class, "(1.) result of intersection must be same class as receiver")
  self~assertSame(1, resultObj~items, "Tree Rock Planet Star intersect Elm Rock Mars Monroe should produce 1 item")

  self~assertTrue(resultObj~hasItem("Rock"), "Result of Tree Rock Planet Star intersect Elm Rock Mars Monroe" || nl ||                    "should have Rock item")

  argumentArray = .array~of("Elm", "Rock", "Mars", "Monro")
  result1 = receiverArray~intersection(argumentArray)
  result2 = argumentArray~intersection(receiverArray)
  self~assertSame(.array, result1~class, "(2.) result of intersection must be same class as receiver")
  self~assertSame(.array, result2~class, "(3.) result of intersection must be same class as receiver")
  self~assertTrue(sameContent(result1, result2), "Array A intersect array b should be the same as array B intersect arrry A")


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
