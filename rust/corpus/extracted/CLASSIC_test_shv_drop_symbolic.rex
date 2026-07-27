/* extracted from CLASSIC::test_shv_drop_symbolic */
::routine main public
  null = ""
  range = xrange()
  n = "tail"
  large = "X"~copies(4 * 1024 + 1)
  stem.n = "stem value"
  var250____________________________________________________________________________________________________________________________________________________________________________________________________________________________________________________ = .250
  array = .Array~new(0)

  do name over "null", "Range", "stem.n", "n", "large", "var250"~left(250, "_")
    self~assertSame(.Shv~OK, TestFVariablePool(.Shv~new("d", name)), name)
    self~assertSame(.Shv~NEWV, TestFVariablePool(.Shv~new("d", name)), name)
    self~assertFalse(var(name), name)
  end

  -- dropping an unassigned variable returns NEWV
  do name over "NOTset", "stem.n", "_!?", "rc"
    self~assertSame(.Shv~NEWV, TestFVariablePool(.Shv~new("d", name)), name)
  end

  -- invalid variable names return BADN
  do name over "v"~copies(256), "", "*", 0, "2invalid", "with blank", '00'x
    self~assertSame(.Shv~BADN, TestFVariablePool(.Shv~new("d", name)), name)
  end

  n = 0.123
  -- a chained request returns a composite OR'ed return code
  shv = .Shv~new("d", "rc"), .Shv~new("d", "n"), .Shv~new("d", "*")
  self~assertSame(.Shv~NEWV + .Shv~OK + .Shv~BADN, TestFVariablePool(shv))
  self~assertSame(.Shv~NEWV,  shv[1]~shvret)
  self~assertSame(.Shv~OK,    shv[2]~shvret)
  self~assertSame(.Shv~BADN,  shv[3]~shvret)
  self~assertFalse(var("N"))

-- test SHVBLOCK S/SET, direct interface
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
