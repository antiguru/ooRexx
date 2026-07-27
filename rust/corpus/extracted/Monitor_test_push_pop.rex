/* extracted from Monitor::test_push_pop */
::routine main public
  d1 = .Object~new
  d2 = .Destination~new("other")
  m = .Monitor~new
  self~assertNull(m~current)
  -- destination(d) returns the current 'd', not the previous one
  -- same for destination(), which pops, and then returns current
  self~assertSame(d1, m~destination(d1)) -- set d1 destination
  self~assertSame(d1, m~current)
  self~assertSame(d2, m~destination(d2)) -- set d2 destination
  self~assertSame(d2, m~current)
  self~assertSame(d1, m~destination) -- set to previous destination d1
  self~assertSame(d1, m~current)
  self~assertNull(m~destination)
  self~assertNull(m~current)

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
