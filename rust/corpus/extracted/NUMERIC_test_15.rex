/* extracted from NUMERIC::test_15 */
::routine main public
   Numeric Digits 12
   Numeric Fuzz    2
   Numeric Form Engineering
   a1=digits() fuzz() form()
   Call X
   a4=digits() fuzz() form()
   Pull a2
   Pull a3
   self~assertSame(a1, '12 2 ENGINEERING')
   self~assertSame(a2, '12 2 ENGINEERING')
   self~assertSame(a3, '9 0 SCIENTIFIC')
   self~assertSame(a4, '12 2 ENGINEERING')
   return

   X: Procedure
   a2=digits() fuzz() form()
   Numeric Digits  9
   Numeric Fuzz    0
   Numeric Form
   a3=digits() fuzz() form()
   Queue a2
   Queue a3
   Return

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
