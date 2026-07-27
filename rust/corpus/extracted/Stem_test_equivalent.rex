/* extracted from Stem::test_equivalent */
::routine main public
  emptystem = .stem~new
  stem1 = .stem~new
  stem1[1] = "a"
  stem1[2] = "b"
  stem1[3] = "c"

  stem2 = stem1~copy

  -- empty stems of two different sizes...still equivalent


  self~assertTrue(emptystem~equivalent(.stem~new))
  -- empty vs. non-empty, two ways
  self~assertFalse(stem1~equivalent(emptystem))
  self~assertFalse(emptystem~equivalent(stem1))
  -- simple true test
  self~assertTrue(stem1~equivalent(stem2))
  stem2[2] = 'z'
  -- same number of items, different values
  self~assertFalse(stem1~equivalent(stem2))
  -- same values, different indexes
  stem3 = stem1~copy
  stem3~remove(2)
  stem3[4] = "b"
  self~assertFalse(stem1~equivalent(stem3))
  -- different number of items
  stem3~remove(4)
  self~assertFalse(stem1~equivalent(stem3))
  self~assertFalse(stem3~equivalent(stem1))

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
