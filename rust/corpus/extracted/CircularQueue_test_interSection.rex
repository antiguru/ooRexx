/* extracted from CircularQueue::test_interSection */
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
  c =clz~new(5)~~append(o1)~~append("1")~~append("2")
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("2", o1)          -- expected result
  d2=c~interSection(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of list intersection table
  receiverList = .queue~of('w', 'x', 'y', 'z')
  argumentTable = .table~new
  argumentTable['w'] = 'w'
  argumentTable['y'] = 'y'
  argumentTable['z'] = 'z'

  resultObj = receiverList~intersection(argumentTable)
  self~assertSame(.queue, resultObj~class, "(1.) After intersection operation result must be same class as receiver")
  self~assertSame(3, resultObj~items, 'w x y z intersect w y z should result in 3 items')
  self~assertTrue(resultObj~hasItem('w'), 'w x y z intersect w y z result should contain an item of w')

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
