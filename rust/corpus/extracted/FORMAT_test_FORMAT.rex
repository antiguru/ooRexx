/* extracted from FORMAT::test_FORMAT */
::routine main public
    self~assertSame('   3', FORMAT('3',4))
    self~assertSame('   2', FORMAT('1.73',4,0))
    self~assertSame('   1.730', FORMAT('1.73',4,3))
    self~assertSame('  -0.8', FORMAT('-.76',4,1))
    self~assertSame('   3.03', FORMAT('3.03',4))
    self~assertSame('-12.7300', FORMAT(' - 12.73', ,4))
    self~assertSame('-12.73', FORMAT(' - 12.73'))
    self~assertSame('0', FORMAT('0.000'))

    self~assertSame('1.234573E+04', FORMAT('12345.73', , ,2,2)) --
    self~assertSame('1.235E+4', FORMAT('12345.73', ,3, ,0))
    self~assertSame('1.235', FORMAT('1.234573', ,3, ,0))
    self~assertSame('1.235    ', FORMAT('1.234573', ,3,2,0))
    self~assertSame('12345.73', FORMAT('12345.73', , ,3,6))
    self~assertSame('123456700000.000', FORMAT('1234567e5', ,3,0))

-- tests for the NumberString case digitsCount = -numberExponent
-- with unspecified digits width
-- e. g. 0.1~format(, 1) should be 0.1, not .1
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
