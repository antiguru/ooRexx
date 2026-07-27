/* extracted from Properties::test_getLogical */
::routine main public
  p = .Properties~new
  p[".false"] = .false
  p[".true"] = .true
  p["false"] = "false" -- will be interpreted as .false
  p["true"] = "true" -- will be interpreted as .true
  self~assertFalse(p~getLogical("none", 0)) -- doesn't exist, uses default
  self~assertTrue(p~getLogical("none", 1)) -- doesn't exist, uses default
  self~assertFalse(p~getLogical(".false"))
  self~assertFalse(p~getLogical(".false", 0))
  self~assertFalse(p~getLogical(".false", 1))
  self~assertTrue(p~getLogical(".true"))
  self~assertFalse(p~getLogical("false"))
  self~assertTrue(p~getLogical("true"))


-- getWhole

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
