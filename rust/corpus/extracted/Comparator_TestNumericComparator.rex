/* extracted from Comparator::TestNumericComparator */
::routine main public
  c = .NumericComparator~new
  -- a whole series of equivalent numeric values
  self~assertSame(0, c~compare(1, 1e0))
  self~assertSame(0, c~compare(1.00000, 10e-1))
  -- these should compare equal under numeric digits 9
  self~assertSame(0, c~compare(1.000e+0, 1.000000001))
  -- now some greater options
  self~assertSame(1, c~compare("1", "-0.0"))
  self~assertSame(1, c~compare("-4e2", "-401"))
  self~assertSame(1, c~compare("+6", "5.99999999"))
  -- and some less than tests...we'll just invert the previous tests
  self~assertSame(-1, c~compare("-0.0", "1"))
  self~assertSame(-1, c~compare("-401", "-4e2"))
  self~assertSame(-1, c~compare("5.99999999", "+6"))

  c2 = .NumericComparator~new(12)
  -- some examples that would compare equal under default digits
  self~assertSame(0, c~compare(123456789000, 123456789001))
  self~assertSame(-1, c2~compare(123456789000, 123456789001))


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
