/* extracted from FORMAT::test842 */
::routine main public
    numeric digits 3
    numeric form Engineering
    self~assertSame('   12.3E+18', format(123456.78E+14,5,1,2,4))
    self~assertSame('   12.3E+18', format(123456.78E+14,5,1,2,5))
    self~assertSame('   12.3E+18', format(123456.78E+14,5,1,2,6))
    self~assertSame('   12.30E+18', format(123456.78E+14,5,2,2,4))
    self~assertSame('   12.30E+18', format(123456.78E+14,5,2,2,5))
    self~assertSame('   12.30E+18', format(123456.78E+14,5,2,2,6))
    self~assertSame('   12.300E+18', format(123456.78E+14,5,3,2,4))
    self~assertSame('   12.300E+18', format(123456.78E+14,5,3,2,5))
    self~assertSame('   12.300E+18', format(123456.78E+14,5,3,2,6))
    self~assertSame('    12.3E+18', format(123456.78E+14,6,1,2,4))
    self~assertSame('    12.3E+18', format(123456.78E+14,6,1,2,5))
    self~assertSame('    12.3E+18', format(123456.78E+14,6,1,2,6))
    self~assertSame('    12.30E+18', format(123456.78E+14,6,2,2,4))
    self~assertSame('    12.30E+18', format(123456.78E+14,6,2,2,5))
    self~assertSame('    12.30E+18', format(123456.78E+14,6,2,2,6))
    self~assertSame('    12.300E+18', format(123456.78E+14,6,3,2,4))
    self~assertSame('    12.300E+18', format(123456.78E+14,6,3,2,5))
    self~assertSame('    12.300E+18', format(123456.78E+14,6,3,2,6))
    self~assertSame('     12.3E+18', format(123456.78E+14,7,1,2,4))
    self~assertSame('     12.3E+18', format(123456.78E+14,7,1,2,5))
    self~assertSame('     12.3E+18', format(123456.78E+14,7,1,2,6))
    self~assertSame('     12.30E+18', format(123456.78E+14,7,2,2,4))
    self~assertSame('     12.30E+18', format(123456.78E+14,7,2,2,5))
    self~assertSame('     12.30E+18', format(123456.78E+14,7,2,2,6))
    self~assertSame('     12.300E+18', format(123456.78E+14,7,3,2,4))
    self~assertSame('     12.300E+18', format(123456.78E+14,7,3,2,5))
    self~assertSame('     12.300E+18', format(123456.78E+14,7,3,2,6))
    self~assertSame('   12.3E-12', format(123456.78E-16,5,1,2,0))
    self~assertSame('   12.3E-012', format(123456.78E-16,5,1,3,0))
    self~assertSame('   12.3E-0012', format(123456.78E-16,5,1,4,0))
    self~assertSame('   12.30E-12', format(123456.78E-16,5,2,2,0))
    self~assertSame('   12.30E-012', format(123456.78E-16,5,2,3,0))
    self~assertSame('   12.30E-0012', format(123456.78E-16,5,2,4,0))
    self~assertSame('   12.300E-12', format(123456.78E-16,5,3,2,0))
    self~assertSame('   12.300E-012', format(123456.78E-16,5,3,3,0))
    self~assertSame('   12.300E-0012', format(123456.78E-16,5,3,4,0))
    self~assertSame('    12.3E-12', format(123456.78E-16,6,1,2,0))
    self~assertSame('    12.3E-012', format(123456.78E-16,6,1,3,0))
    self~assertSame('    12.3E-0012', format(123456.78E-16,6,1,4,0))
    self~assertSame('    12.30E-12', format(123456.78E-16,6,2,2,0))
    self~assertSame('    12.30E-012', format(123456.78E-16,6,2,3,0))
    self~assertSame('    12.30E-0012', format(123456.78E-16,6,2,4,0))
    self~assertSame('    12.300E-12', format(123456.78E-16,6,3,2,0))
    self~assertSame('    12.300E-012', format(123456.78E-16,6,3,3,0))
    self~assertSame('    12.300E-0012', format(123456.78E-16,6,3,4,0))
    self~assertSame('     12.3E-12', format(123456.78E-16,7,1,2,0))
    self~assertSame('     12.3E-012', format(123456.78E-16,7,1,3,0))
    self~assertSame('     12.3E-0012', format(123456.78E-16,7,1,4,0))
    self~assertSame('     12.30E-12', format(123456.78E-16,7,2,2,0))
    self~assertSame('     12.30E-012', format(123456.78E-16,7,2,3,0))
    self~assertSame('     12.30E-0012', format(123456.78E-16,7,2,4,0))
    self~assertSame('     12.300E-12', format(123456.78E-16,7,3,2,0))
    self~assertSame('     12.300E-012', format(123456.78E-16,7,3,3,0))
    self~assertSame('     12.300E-0012', format(123456.78E-16,7,3,4,0))

-- from bif.testgroup
   -- test the BIF, using examples from the documentation
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
