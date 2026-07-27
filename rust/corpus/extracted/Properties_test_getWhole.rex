/* extracted from Properties::test_getWhole */
::routine main public
  p = .Properties~new
  p["whole"] = 42
  p["whole.zero"] = 555.0
  p["minus"] = -1
  self~assertSame(99, p~getWhole("none", 99)) -- doesn't exist
  self~assertSame(42, p~getWhole("whole"))
  self~assertSame(42, p~getWhole("whole", 0))
  self~assertSame(42, p~getWhole("whole", ""))
  self~assertSame(555.0, p~getWhole("whole.zero"))
  self~assertSame(555.0, p~getWhole("whole.zero", 0))
  self~assertSame(555.0, p~getWhole("whole.zero", ""))
  self~assertSame(-1, p~getWhole("minus"))
  self~assertSame(-1, p~getWhole("minus", 0))
  self~assertSame(-1, p~getWhole("minus", ""))


-- getProperty, []

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
