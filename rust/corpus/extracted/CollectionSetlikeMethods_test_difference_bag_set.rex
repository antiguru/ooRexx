/* extracted from CollectionSetlikeMethods::test_difference_bag_set */
::routine main public

  expose inputBag1 inputBag2 differenceBag1 differenceBag2 intersectionBag unionBag xorBag emptyBag -
         inputSet1 inputSet2 differenceSet1 differenceSet2 intersectionSet unionSet xorSet emptySet

  self~assertTrue(sameContent(emptyBag, emptyBag~difference(emptyBag)), "subtest1: sameContent(emptyBag, emptyBag~difference(emptyBag))")
  self~assertTrue(sameContent(emptySet, emptySet~difference(emptySet)), "subtest2: sameContent(emptySet, emptySet~difference(emptySet))")

  self~assertTrue(sameContent(inputBag1, inputBag1~difference(emptyBag)), "subtest3: sameContent(inputBag1, inputBag1~difference(emptyBag))")
  self~assertTrue(sameContent(inputSet1, inputSet1~difference(emptySet)), "subtest4: sameContent(inputSet1, inputSet1~difference(emptySet))")

  self~assertTrue(sameContent(differenceBag1, inputBag1~difference(inputBag2)), "subtest5: sameContent(differenceBag1, inputBag1~difference(inputBag2))")
  self~assertTrue(sameContent(differenceSet1, inputSet1~difference(inputSet2)), "subtest6: sameContent(differenceSet1, inputSet1~difference(inputSet2))")

  self~assertTrue(sameContent(differenceBag2, inputBag2~difference(inputBag1)), "subtest7: sameContent(differenceBag2, inputBag2~difference(inputBag1))")
  self~assertTrue(sameContent(differenceSet2, inputSet2~difference(inputSet1)), "subtest8: sameContent(differenceSet2, inputSet2~difference(inputSet1))")


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
