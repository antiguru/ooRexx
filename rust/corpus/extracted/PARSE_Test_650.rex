/* extracted from PARSE::Test_650 */
::routine main public
   xsb=''
   do i=1 to length(xsa)
      xsb=xsb||substr(xsa,i,1)||' '
      End
   Parse Var xsb a.1  a.2  a.3  a.4  a.5  a.6  a.7  a.8  a.9  a.10,
                 a.11 a.12 a.13 a.14 a.15 a.16 a.17 a.18 a.19 a.20,
                 a.21 a.22 a.23 a.24 a.25 a.26 a.27 a.28 a.29 a.30,
                 a.31 a.32 a.33 a.34 a.35 a.36 a.37 a.38 a.39 a.40,
                 a.41 a.42 a.43 a.44 a.45 a.46 a.47 a.48 a.49 a.50 .
   Do i=1 To 50
      self~assertSame(a.i, "WORD"(xsb,i))
      End

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
