/* extracted from Comparator::TestCaselessColumnComparator */
::routine main public
  c = .CaselessColumnComparator~new(4, 3)
  self~assertSame(0, c~compare("xxxabcxxx", "xxxabcxxx"))
  self~assertSame(1, c~compare("xxxdbcxxx", "xxxabcxxx"))
  self~assertSame(-1, c~compare("xxxabcxxx", "xxxdbcxxx"))
  self~assertSame(-1, c~compare("xxxab", "xxxabcxxx"))
  self~assertSame(1, c~compare("xxxabcxxx", "xxxab"))

  self~assertSame(0, c~compare("xxxABCxxx", "XXXabcXXX"))
  self~assertSame(1, c~compare("xxxDBCxxx", "XXXabcXXX"))
  self~assertSame(-1, c~compare("xxxABCxxx", "XXXdbcXXX"))
  self~assertSame(-1, c~compare("XXXAB", "xxxabcxxx"))
  self~assertSame(1, c~compare("xxxABCxxx", "XXXab"))

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
