/* extracted from CircularQueue::test_disjoint */
::routine main public
  -- empty circularqueues are disjoint because they have no elements in common.
  self~assertTrue(.circularqueue~new(5)~disjoint(.circularqueue~new(4)))
  -- empty vs. non-empty, two ways.  Both disjoint
  self~assertTrue(.circularqueue~of(1,2,3)~disjoint(.circularqueue~new(10)))
  self~assertTrue(.circularqueue~new(2)~disjoint(.circularqueue~of(1,2,3)))
  -- simple true test
  self~assertTrue(.circularqueue~of(1,2,3)~disjoint(.circularqueue~of(4,5,6)))
  -- equal errays
  self~assertFalse(.circularqueue~of(1,2,3)~disjoint(.circularqueue~of(1,2,3)))
  -- single element overlap
  self~assertFalse(.circularqueue~of(1,2,3)~disjoint(.circularqueue~of(3,4,5)))

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
