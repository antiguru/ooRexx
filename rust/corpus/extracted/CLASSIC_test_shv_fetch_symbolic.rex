/* extracted from CLASSIC::test_shv_fetch_symbolic */
::routine main public
  null = ""
  range = xrange()
  n = "tail"
  large = "X"~copies(4 * 1024 + 1)
  stem.n = "stem value"
  var250____________________________________________________________________________________________________________________________________________________________________________________________________________________________________________________ = .250
  array = .Array~new(0)

  do name over "null", "Range", "n", "large", "stem.n", "var250"~left(250, "_")
    shv = .Shv~new("f", name)
    self~assertSame(.Shv~OK, TestFVariablePool(shv), name)
    self~assertSame(value(name), shv~shvvalue)
  end

  -- SHV is strictly string-only
  shv = .Shv~new("f", "array")
  self~assertSame(.Shv~OK, TestFVariablePool(shv))
  self~assertSame("an Array", shv~shvvalue)

  -- variables not set return NEWV
  do name over "NOTset", "stem.large", "_!?", "rc"
    self~assertSame(.Shv~NEWV, TestFVariablePool(.Shv~new("f", name)), name)
  end

  -- variables are not accessible from within a procedure
  shv = .Shv~new("f", "n")
  self~assertSame(.Shv~NEWV, procedureVariablePool(shv))

  -- variable value truncation returns TRUNC
  do name over "range", "large"
    shv = .Shv~new("f", name, , 100) -- 100 is too small for range and large
    self~assertSame(.Shv~TRUNC, TestFVariablePool(shv), name)
    self~assertSame(value(name)~left(100), shv~shvvalue)
  end

  -- invalid variable names return BADN
  do name over "v"~copies(256), "", "*", 0, "2invalid", "with blank", '00'x
    self~assertSame(.Shv~BADN, TestFVariablePool(.Shv~new("f", name)), name)
  end

  -- a chained request returns a composite OR'ed return code
  shv = .Shv~new("f", "rc"), .Shv~new("f", "large", , 100), .Shv~new("f", "n"), .Shv~new("f", "*")
  self~assertSame(.Shv~NEWV + .Shv~TRUNC + .Shv~OK + .Shv~BADN, TestFVariablePool(shv))
  self~assertSame(.Shv~NEWV,  shv[1]~shvret)
  self~assertSame(.Shv~TRUNC, shv[2]~shvret)
  self~assertSame(.Shv~OK,    shv[3]~shvret)
  self~assertSame(n,          shv[3]~shvvalue)
  self~assertSame(.Shv~BADN,  shv[4]~shvret)

  return

  procedureVariablePool: procedure
  use strict arg shv
  return TestFVariablePool(shv)

-- test SHVBLOCK D/DROP, direct interface
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
