/* extracted from Array::test_disjoint */
::routine main public
  -- empty arrays are disjoint because they have no elements in common.
  self~assertTrue(.array~new(10)~disjoint(.array~new(20)))
  -- empty vs. non-empty, two ways.  Both disjoint
  self~assertTrue(.array~of(1,2,3)~disjoint(.array~new))
  self~assertTrue(.array~new~disjoint(.array~of(1,2,3)))
  -- simple true test
  self~assertTrue(.array~of(1,2,3)~disjoint(.array~of(4,5,6)))
  -- equal errays
  self~assertFalse(.array~of(1,2,3)~disjoint(.array~of(1,2,3)))
  -- single element overlap
  self~assertFalse(.array~of(1,2,3)~disjoint(.array~of(3,4,5)))
  -- disjointness of sparse arrays
  self~assertTrue(.array~of(1,,3)~disjoint(.array~of(4,,5)))
  self~assertFalse(.array~of(1,,3)~disjoint(.array~of(3,,1)))

  a1 = .array~new(3, 3)
  a1[1,3] = "a"
  a1[2,2] = "b"
  a1[3,1] = "c"

  a2 = .array~new(3, 3)
  a2[1,3] = "x"
  a2[2,2] = "y"
  a2[3,1] = "z"
  -- pair of multi-dimension arrays
  self~assertTrue(a1~disjoint(a2))
  -- add a commont item
  a2[1,2] = "a"
  self~assertFalse(a1~disjoint(a2))
  self~assertFalse(a2~disjoint(a1))

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
