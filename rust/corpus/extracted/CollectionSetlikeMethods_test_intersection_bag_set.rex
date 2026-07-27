/* extracted from CollectionSetlikeMethods::test_intersection_bag_set */
::routine main public

  expose inputBag1 inputBag2 differenceBag1 differenceBag2 intersectionBag unionBag xorBag emptyBag -
         inputSet1 inputSet2 differenceSet1 differenceSet2 intersectionSet unionSet xorSet emptySet

  self~assertTrue(sameContent(emptyBag, emptyBag~interSection(emptyBag)), "subtest1: sameContent(emptyBag, emptyBag~interSection(emptyBag))")
  self~assertTrue(sameContent(emptySet, emptySet~interSection(emptySet)), "subtest2: sameContent(emptySet, emptySet~interSection(emptySet))")

  self~assertTrue(sameContent(emptyBag, inputBag1~interSection(emptyBag)), "subtest3: sameContent(emptyBag, inputBag1~interSection(emptyBag))")
  self~assertTrue(sameContent(emptySet, inputSet1~interSection(emptySet)), "subtest4: sameContent(emptySet, inputSet1~interSection(emptySet))")

  self~assertTrue(sameContent(interSectionBag, inputBag1~interSection(inputBag2)), "subtest5: sameContent(interSectionBag, inputBag1~interSection(inputBag2))")
  self~assertTrue(sameContent(interSectionSet, inputSet1~interSection(inputSet2)), "subtest6: sameContent(interSectionSet, inputSet1~interSection(inputSet2))")

  self~assertTrue(sameContent(interSectionBag, inputBag2~interSection(inputBag1)), "subtest7: sameContent(interSectionBag, inputBag2~interSection(inputBag1))")
  self~assertTrue(sameContent(interSectionSet, inputSet2~interSection(inputSet1)), "subtest8: sameContent(interSectionSet, inputSet2~interSection(inputSet1))")



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
