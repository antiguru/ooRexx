/* extracted from Array::test_equivalent */
::routine main public
  -- empty arrays of two different sizes...still equivalent
  self~assertTrue(.array~new(10)~equivalent(.array~new(20)))
  -- empty vs. non-empty, two ways
  self~assertFalse(.array~of(1,2,3)~equivalent(.array~new))
  self~assertFalse(.array~new~equivalent(.array~of(1,2,3)))
  -- simple true test
  self~assertTrue(.array~of(1,2,3)~equivalent(.array~of(1,2,3)))
  -- same number of items, different values
  self~assertFalse(.array~of(1,2,3)~equivalent(.array~of(1,2,4)))
  -- mismatch on number of arguments, tested both ways
  self~assertFalse(.array~of(1,2,3)~equivalent(.array~of(1,2,3,4)))
  self~assertFalse(.array~of(1,2,3,4)~equivalent(.array~of(1,2,3)))
  -- equivalance of sparse arrays
  self~assertTrue(.array~of(1,,3)~equivalent(.array~of(1,,3)))

  a1 = .array~new(3, 3)
  a1[1,3] = "a"
  a1[2,2] = "b"
  a1[3,1] = "c"

  a2 = .array~new(3, 3)
  a2[1,3] = "a"
  a2[2,2] = "b"
  a2[3,1] = "c"
  -- pair of multi-dimention arrays
  self~assertTrue(a1~equivalent(a2))
  -- now non-equivalent ones
  a1[1,2] = "z"
  self~assertFalse(a1~equivalent(a2))
  self~assertFalse(a2~equivalent(a1))

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
