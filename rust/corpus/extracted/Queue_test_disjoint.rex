/* extracted from Queue::test_disjoint */
::routine main public
  -- empty queues are disjoint because they have no elements in common.
  self~assertTrue(.queue~new~disjoint(.queue~new))
  -- empty vs. non-empty, two ways.  Both disjoint
  self~assertTrue(.queue~of(1,2,3)~disjoint(.queue~new))
  self~assertTrue(.queue~new~disjoint(.queue~of(1,2,3)))
  -- simple true test
  self~assertTrue(.queue~of(1,2,3)~disjoint(.queue~of(4,5,6)))
  -- equal errays
  self~assertFalse(.queue~of(1,2,3)~disjoint(.queue~of(1,2,3)))
  -- single element overlap
  self~assertFalse(.queue~of(1,2,3)~disjoint(.queue~of(3,4,5)))

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
