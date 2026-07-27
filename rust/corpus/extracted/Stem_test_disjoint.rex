/* extracted from Stem::test_disjoint */
::routine main public
  emptystem = .stem~new
  stem1 = .stem~new
  stem1[1] = "a"
  stem1[2] = "b"
  stem1[3] = "c"

  stem2 = stem1~copy
  -- same indexes, different values
  stem3 = .stem~new
  stem1[1] = "x"
  stem1[2] = "y"
  stem1[3] = "z"

  -- empty stems of two different sizes...still equivalent


  self~assertTrue(emptystem~disjoint(.stem~new))
  -- empty vs. non-empty, two ways
  self~assertTrue(stem1~disjoint(emptystem))
  self~assertTrue(emptystem~disjoint(stem1))
  -- simple overlap test
  self~assertFalse(stem1~disjoint(stem2))
  -- same indices, different values
  self~assertTrue(stem1~disjoint(stem3))
  -- same values, different indexes
  stem3~empty
  stem3[4] = "a"
  stem3[5] = "b"
  stem3[6] = "c"
  self~assertTrue(stem1~disjoint(stem3))
  -- different number of items
  stem2~remove(3)
  self~assertFalse(stem1~disjoint(stem2))



/* --------------------- Stem specific methods -------------------- */

/* test the default value */

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
