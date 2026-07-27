/* extracted from Set::test_xor */
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
  c =clz~new~~put("1")~~put("2")~~put(o1)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")          -- expected result
  d2=c~xor(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic XOR test where the argument collection is a table.
  receiverSet = .set~of("Tom", "Frank", "Mary")

  argumentTable = .table~new
  argumentTable["Tom"]   = "Tom"
  argumentTable["Frank"] = "Frank"
  argumentTable["Sue"]   = "Sue"

  expectedSet = .set~of("Mary", "Sue")

  nl = '0d0a'x
  resultObj = receiverSet~xor(argumentTable)
  self~assertTrue(resultObj~isA(.set), "Result object after xor must be same class as receiver")
  self~assertTrue(sameContent(resultObj, expectedSet), "Expected XOR of Tom Frank Mary with Tom Frank Sue is:" || nl ||                    "Mary Sue")

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
