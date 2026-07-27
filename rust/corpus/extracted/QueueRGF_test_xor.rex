/* extracted from QueueRGF::test_xor */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  res=collDir~xorColl

  self~assertEquals(ce, ce~xor(ce))

  self~assertEquals(c1, c1~xor(ce))
  self~assertEquals(c2, c2~xor(ce))

  self~assertEquals(collDir~xorColl1, c1~xor(c2))
  self~assertEquals(collDir~xorColl2, c2~xor(c1))

  self~assertEquals(ce, c1~xor(c1))
  self~assertEquals(ce, c2~xor(c2))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~append(o1)~~append("1")~~append("2")
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")          -- expected result
  d2=c~xor(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of queue xor table
  receiverQ = .queue~new
  do i = 1 to 4
    receiverQ~push(i)
  end

  argumentTable = .table~new
  do i = 1 to 5 by 2
    argumentTable[i] = i
  end

  resultObj = receiverQ~xor(argumentTable)

  self~assertSame(.queue, resultObj~class, "(1.) result object after xor operation must be the same class as the receiver")
  self~assertSame(3, resultObj~items, "1 2 3 4 xor 1 3 5 should result in a collection of 3 items")
  self~assertTrue(resultObj~hasItem(2), "1 2 3 4 xor 1 3 5 result should have an item of 2")
  self~assertTrue(resultObj~hasItem(4), "1 2 3 4 xor 1 3 5 result should have an item of 4")
  self~assertTrue(resultObj~hasItem(5), "1 2 3 4 xor 1 3 5 result should have an item of 5")




/* ================= additional, Queue specific methods =============== */

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
