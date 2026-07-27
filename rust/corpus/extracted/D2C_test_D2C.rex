/* extracted from D2C::test_D2C */
::routine main public
    self~assertSame('A', D2C(65))
    self~assertSame('A', D2C(65,1))
    self~assertSame(right('A', 2, "00"x), D2C(65,2))
    self~assertSame(right('A', 5, "00"x), D2C(65,5))
    self~assertSame('m', D2C(109)) /* '6D'x is an ASCII 'm' */
    self~assertSame('93'x, D2C(-109,1)) /* '93'x is an ASCII '�' */
    self~assertSame(right('L', 2, "00"x), D2C(76,2)) /* '4C'x is an ASCII ' L' */
    self~assertSame(right('L', 2, "FF"x), D2C(-180,2))

   -- new tests
    self~assertSame("", D2C(-180,0))


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
