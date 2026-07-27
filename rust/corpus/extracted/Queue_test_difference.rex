/* extracted from Queue::test_difference */
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
  c =clz~new~~append(o1)~~append("1")~~append("2")
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")     -- expected result
  d2=c~difference(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of list difference table
  receiverList = clz~of('w', 'x', 'y', 'z')
  argumentTable = .table~new
  argumentTable['w'] = 'w'
  argumentTable['y'] = 'y'
  argumentTable['z'] = 'z'

  resultObj = receiverList~difference(argumentTable)
  self~assertSame(clz, resultObj~class, "(1.) After difference operation result must be same class as receiver")
  self~assertSame(1, resultObj~items, 'w x y z difference w y z should result in 1 item')
  self~assertTrue(resultObj~hasItem('x'), 'w x y z difference w y z result should contain an item of x')


/* Test whether both collections contain the same entries.
   returns .true, if the same, .false else
*/
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
