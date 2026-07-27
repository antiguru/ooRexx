/* extracted from List::test_subset */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem

  ce=collDir~emptyColl
  c1=collDir~coll_1
  c2=collDir~coll_2
  cu1=collDir~unionColl1
  cu2=collDir~unionColl2

  self~assertTrue(ce~subset(ce))
  self~assertTrue(ce~subset(c1))
  self~assertTrue(ce~subset(c2))

  self~assertTrue(c1~subset(c1))
  self~assertTrue(c2~subset(c2))

  self~assertTrue(ce~subset(cu1))

  self~assertTrue(c1~subset(cu1))
  self~assertTrue(c1~subset(cu2))

  self~assertTrue(c2~subset(cu1))
  self~assertTrue(c2~subset(cu2))

  self~assertFalse(c1~subset(ce))
  self~assertFalse(c1~subset(c2))

  self~assertFalse(c2~subset(ce))
  self~assertFalse(c2~subset(c1))


   -- now test where other is an 'OrderedCollection': "makeArray" returns "allItems"
  o1=.object~new
  c =clz~new~~append(o1)~~append("1")~~append("2")
  other=.array~new
  other[104]="2"
  other[106]=o1
  d2=c~subSet(other)
  self~assertFalse(d2, "subtest14: 'other' is an 'OrderedCollection'")

  -- Simplistic test of list subset table
  receiverList = .list~of('w', 'x', 'y', 'z')
  argumentTable = .table~new
  argumentTable['w'] = 'w'
  argumentTable['y'] = 'y'
  argumentTable['z'] = 'z'

  self~assertFalse(receiverList~subset(argumentTable), 'w x y z is not a subset of w y z')
  receiverList~removeItem('x')
  receiverList~removeItem('z')
  self~assertTrue(receiverList~subset(argumentTable), 'w y is a subset of w y z')

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
