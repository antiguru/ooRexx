/* extracted from format::test787 */
::routine main public
    numeric digits 3
    self~assertSame('    1.2E+05', 123456.78~format(5,1,2,4))
    self~assertSame('    1.2E+05', 123456.78~format(5,1,2,5))
    self~assertSame('    1.23E+05', 123456.78~format(5,2,2,4))
    self~assertSame('    1.23E+05', 123456.78~format(5,2,2,5))
    self~assertSame('    1.230E+05', 123456.78~format(5,3,2,4))
    self~assertSame('    1.230E+05', 123456.78~format(5,3,2,5))
    self~assertSame('     1.2E+05', 123456.78~format(6,1,2,4))
    self~assertSame('     1.2E+05', 123456.78~format(6,1,2,5))
    self~assertSame('123000.0', 123456.78~format(6,1,2,6))
    self~assertSame('     1.23E+05', 123456.78~format(6,2,2,4))
    self~assertSame('     1.23E+05', 123456.78~format(6,2,2,5))
    self~assertSame('123000.00', 123456.78~format(6,2,2,6))
    self~assertSame('     1.230E+05', 123456.78~format(6,3,2,4))
    self~assertSame('     1.230E+05', 123456.78~format(6,3,2,5))
    self~assertSame('123000.000', 123456.78~format(6,3,2,6))
    self~assertSame('      1.2E+05', 123456.78~format(7,1,2,4))
    self~assertSame('      1.2E+05', 123456.78~format(7,1,2,5))
    self~assertSame(' 123000.0', 123456.78~format(7,1,2,6))
    self~assertSame('      1.23E+05', 123456.78~format(7,2,2,4))
    self~assertSame('      1.23E+05', 123456.78~format(7,2,2,5))
    self~assertSame(' 123000.00', 123456.78~format(7,2,2,6))
    self~assertSame('      1.230E+05', 123456.78~format(7,3,2,4))
    self~assertSame('      1.230E+05', 123456.78~format(7,3,2,5))
    self~assertSame(' 123000.000', 123456.78~format(7,3,2,6))

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
