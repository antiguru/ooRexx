/* extracted from CircularQueue::test_xor */
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
  c =clz~new(5)~~append(o1)~~append("1")~~append("2")
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")          -- expected result
  d2=c~xor(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of list XOR table
  receiverList = .list~of('w', 'x', 'y', 'z')
  argumentTable = .table~new
  argumentTable['w'] = 'w'
  argumentTable['y'] = 'y'
  argumentTable['z'] = 'z'

  resultObj = receiverList~xor(argumentTable)
  self~assertSame(.list, resultObj~class, "(1.) After xor operation result must be same class as receiver")
  self~assertSame(1, resultObj~items, 'w x y z xor w y z should result in 1 item')
  self~assertTrue(resultObj~hasItem('x'), 'w x y z xor w y z result should contain an item of x')

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
