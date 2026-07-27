/* extracted from PARSE::test_PARSE_upper_lower_caseless */
::routine main public
   string="AnToN-BeRtA-CaEsAr"
   PARSE UPPER VAR string "c" rest
   self~assertSame('', rest)

   PARSE UPPER VAR string "C" rest
   self~assertSame('AESAR', rest)

   PARSE LOWER VAR string "C" rest
   self~assertSame('', rest)

   PARSE LOWER VAR string "c" rest
   self~assertSame('aesar', rest)

   PARSE CASELESS VAR string "c" rest
   self~assertSame('aEsAr', rest)

   PARSE CASELESS VAR string "C" rest
   self~assertSame('aEsAr', rest)




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
