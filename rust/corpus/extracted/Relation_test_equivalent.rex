/* extracted from Relation::test_equivalent */
::routine main public
  emptyrelation = .relation~new
  relation1 = .relation~new
  relation1[1] = "a"
  relation1[2] = "b"
  relation1[3] = "c"

  relation2 = relation1~copy

  -- empty relations of two different sizes...still equivalent


  self~assertTrue(emptyrelation~equivalent(.relation~new))
  -- empty vs. non-empty, two ways
  self~assertFalse(relation1~equivalent(emptyrelation))
  self~assertFalse(emptyrelation~equivalent(relation1))
  -- simple true test
  self~assertTrue(relation1~equivalent(relation2))
  relation2[2] = 'z'
  -- same number of items, different values
  self~assertFalse(relation1~equivalent(relation2))
  -- same values, different indexes
  relation3 = relation1~copy
  relation3~remove(2)
  relation3[4] = "b"
  self~assertFalse(relation1~equivalent(relation3))
  -- different number of items
  relation3~remove(4)
  self~assertFalse(relation1~equivalent(relation3))
  self~assertFalse(relation3~equivalent(relation1))

  relation1 = .relation~new
  relation1[1] = "a"
  relation1[2] = "b"
  relation1[3] = "c"
  relation1[3] = "d"

  relation2 = .relation~new
  relation2[1] = "a"
  relation2[2] = "b"
  relation2[3] = "d"     -- duplicates created in a different order
  relation2[3] = "c"
  self~assertTrue(relation1~equivalent(relation2))

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
