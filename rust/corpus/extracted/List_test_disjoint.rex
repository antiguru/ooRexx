/* extracted from List::test_disjoint */
::routine main public
  -- empty lists are disjoint because they have no elements in common.
  self~assertTrue(.list~new~disjoint(.list~new))
  -- empty vs. non-empty, two ways.  Both disjoint
  self~assertTrue(.list~of(1,2,3)~disjoint(.list~new))
  self~assertTrue(.list~new~disjoint(.list~of(1,2,3)))
  -- simple true test
  self~assertTrue(.list~of(1,2,3)~disjoint(.list~of(4,5,6)))
  -- equal errays
  self~assertFalse(.list~of(1,2,3)~disjoint(.list~of(1,2,3)))
  -- single element overlap
  self~assertFalse(.list~of(1,2,3)~disjoint(.list~of(3,4,5)))



/* ================= additional, List specific methods =============== */

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
