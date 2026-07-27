/* extracted from Relation::test_disjoint */
::routine main public
  emptyrelation = .relation~new
  relation1 = .relation~new
  relation1[1] = "a"
  relation1[2] = "b"
  relation1[3] = "c"

  relation2 = relation1~copy
  -- same indexes, different values
  relation3 = .relation~new
  relation1[1] = "x"
  relation1[2] = "y"
  relation1[3] = "z"

  -- empty relations of two different sizes...still equivalent


  self~assertTrue(emptyrelation~disjoint(.relation~new))
  -- empty vs. non-empty, two ways
  self~assertTrue(relation1~disjoint(emptyrelation))
  self~assertTrue(emptyrelation~disjoint(relation1))
  -- simple overlap test
  self~assertFalse(relation1~disjoint(relation2))
  -- same indices, different values
  self~assertTrue(relation1~disjoint(relation3))
  -- same values, different indexes
  relation3~empty
  relation3[4] = "a"
  relation3[5] = "b"
  relation3[6] = "c"
  self~assertTrue(relation1~disjoint(relation3))
  -- different number of items
  relation2~remove(3)
  self~assertFalse(relation1~disjoint(relation2))


/* add test cases for of and do over for 12.10.2014 Walter Pachl */

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
