/* extracted from Array::test_xor */
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
  c =clz~new~~put("1", 101)~~put("2", 102)~~put(o1, 103)
  other=.array~new
  other[104]="2"
  other[106]=o1
  d1=clz~of("1")          -- expected result
  d2=c~xor(other)
  self~assertTrue(sameContent(d1, d2), "subtest8: 'other' is an 'OrderedCollection'")

  -- Simplistic test of array XOR table
  receiverArray = .array~new
  receiverArray[10] = "Elm"
  receiverArray[13] = "Cadillac"
  receiverArray[15] = "Morning Glory"

  argumentTable = .table~new
  argumentTable["a"] = "School"
  argumentTable["b"] = "Office"
  argumentTable["c"] = "Church"
  argumentTable["d"] = "Apartment"

  nl = '0d0a'x
  resultObj = receiverArray~XOR(argumentTable)
  self~assertSame(.array, resultObj~class, "(1.) result of XOR must be same class as receiver")
  self~assertSame(7, resultObj~items, "array with 3 distinct items XOR table with 4 different distinct items" || nl ||                    "should result in array with 7 items")

  self~assertTrue(resultObj~hasItem("Elm"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item Elm")
  self~assertTrue(resultObj~hasItem("Cadillac"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item Cadillac")
  self~assertTrue(resultObj~hasItem("Morning Glory"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item Morning Glory")
  self~assertTrue(resultObj~hasItem("School"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item School")
  self~assertTrue(resultObj~hasItem("Office"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item Office")
  self~assertTrue(resultObj~hasItem("Church"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item Church")
  self~assertTrue(resultObj~hasItem("Apartment"), "Result of Elm Cadillac 'Morning Glory' XOR School Office Church Apartment" || nl ||                    "must have item Apartment")


  receiverArray = .array~new
  receiverArray[10] = "Elm"
  receiverArray[13] = "Cadillac"
  receiverArray[15] = "Desert"

  argumentTable = .table~new
  argumentTable["a"] = "Elm"
  argumentTable["b"] = "Cadillac"
  argumentTable["c"] = "Desert"

  resultObj = receiverArray~XOR(argumentTable)
  self~assertSame(.array, resultObj~class, "(2.) result of XOR must be same class as receiver")
  self~assertSame(0, resultObj~items, "Result of Elm Cadillac Desert XOR Elm Cadillac Desert Desert" || nl ||                    "should be empty")


/* ================= additional, ARRAY specific methods =============== */

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
