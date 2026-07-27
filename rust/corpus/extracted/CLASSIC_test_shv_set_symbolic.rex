/* extracted from CLASSIC::test_shv_set_symbolic */
::routine main public

  vars = .StringTable~of( -
   ("null", ""), ("range", xrange()), ("n", "tail"), ("large", "x"~copies(4 * 1024 + 1)), -
   ("stem.n", "stem value"), ("var250"~left(250, "_"), .250))
  -- S will return NEWV first, OK on second try
  do with index name item value over vars
    self~assertSame(.Shv~NEWV, TestFVariablePool(.Shv~new("s", name, value)), name)
    self~assertSame(value, value(name))
    self~assertSame(.Shv~OK, TestFVariablePool(.Shv~new("s", name, value)), name)
    self~assertSame(value, value(name))
  end

  -- invalid variable names return BADN
  do name over "v"~copies(256), "", "*", 0, "2invalid", "with blank", '00'x
    self~assertSame(.Shv~BADN, TestFVariablePool(.Shv~new("s", name, "")), name)
  end

  -- a chained request returns a composite OR'ed return code
  shv = .Shv~new("s", "RC", -1), .Shv~new("s", "N", "other"), .Shv~new("s", "*")
  self~assertSame(.Shv~NEWV + .Shv~OK + .Shv~BADN, TestFVariablePool(shv))
  self~assertSame(.Shv~NEWV,  shv[1]~shvret)
  self~assertSame(.Shv~OK,    shv[2]~shvret)
  self~assertSame(.Shv~BADN,  shv[3]~shvret)
  self~assertSame(-1, rc)
  self~assertSame("other", n)

-- test SHVBLOCK P/PRIV, direct interface
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
