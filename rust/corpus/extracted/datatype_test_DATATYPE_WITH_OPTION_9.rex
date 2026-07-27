/* extracted from datatype::test_DATATYPE_WITH_OPTION_9 */
::routine main public
    a=digits()          -- get digits

    numeric digits 2    -- make sure that numeric digits is not set to 9
    self~assertTrue('0'    ~datatype('9')) -- whole number under 9 digits (?)
    self~assertTrue('1'    ~datatype('9'))
    self~assertTrue('-1'   ~datatype('9'))
    self~assertTrue('12345' ~datatype('9')) -- ?
    self~assertTrue('1E3'   ~datatype('9')) -- ?
    if .ooRexxUnit.architecture == 32 then
        self~assertFalse('1E9'  ~datatype('9')) -- ?
    else
        self~assertFalse('1E18'  ~datatype('9')) -- ?
    numeric digits a

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
