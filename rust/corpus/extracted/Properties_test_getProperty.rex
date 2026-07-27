/* extracted from Properties::test_getProperty */
::routine main public
  p = .Properties~new
  p[""] = "nullstring"
  p[".false"] = .false -- will become "false" String
  p[".true"] = .true -- will become "true" String
  p["f"] = "false"
  p["t"] = "true"
  p["Repunit"] = 1~copies(20)
  p["spaced"] = " - "
  self~assertSame(.nil, p~getProperty("doesnt-exist"))
  self~assertSame(.nil, p~getProperty("doesnt-exist", .nil))
  self~assertSame("", p~getProperty("doesnt-exist", ""))
  self~assertSame(555, p~getProperty("doesnt-exist", 555))
  self~assertSame("nullstring", p~getProperty(""))
  self~assertSame("nullstring", p~getProperty("", 333))
  self~assertSame(0, p~getProperty(".false"))
  self~assertSame(1, p~getProperty(".true"))
  self~assertSame("false", p~getProperty("f")) -- "false" String
  self~assertSame("true", p~getProperty("t")) -- "true" String
  self~assertSame(.nil, p["repunit"]) -- names are case-sensitive
  self~assertSame(1~copies(20), p["Repunit"])
  self~assertSame("", p~getProperty("REPUNIT", "")) -- names are case-sensitive
  self~assertSame(" - ", p~getProperty("spaced"))


-- setLogical

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
