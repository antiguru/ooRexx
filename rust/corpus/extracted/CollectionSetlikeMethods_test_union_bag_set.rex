/* extracted from CollectionSetlikeMethods::test_union_bag_set */
::routine main public

  expose inputBag1 inputBag2 differenceBag1 differenceBag2 intersectionBag unionBag xorBag emptyBag -
         inputSet1 inputSet2 differenceSet1 differenceSet2 intersectionSet unionSet xorSet emptySet


  self~assertTrue(sameContent(emptyBag, emptyBag~union(emptyBag)), "subtest1: sameContent(emptyBag, emptyBag~union(emptyBag))")
  self~assertTrue(sameContent(emptySet, emptySet~union(emptySet)), "subtest2: sameContent(emptySet, emptySet~union(emptySet))")

  self~assertTrue(sameContent(inputBag1, inputBag1~union(emptyBag)), "subtest3: sameContent(inputBag1, inputBag1~union(emptyBag))")
  self~assertTrue(sameContent(inputSet1, inputSet1~union(emptySet)), "subtest4: sameContent(inputSet1, inputSet1~union(emptySet))")

  self~assertTrue(sameContent(unionBag, inputBag1~union(inputBag2)), "subtest5: sameContent(unionBag, inputBag1~union(inputBag2))")
  self~assertTrue(sameContent(unionSet, inputSet1~union(inputSet2)), "subtest6: sameContent(unionSet, inputSet1~union(inputSet2))")

  self~assertTrue(sameContent(unionBag, inputBag2~union(inputBag1)), "subtest7: sameContent(unionBag, inputBag2~union(inputBag1))")
  self~assertTrue(sameContent(unionSet, inputSet2~union(inputSet1)), "subtest8: sameContent(unionSet, inputSet2~union(inputSet1))")


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
