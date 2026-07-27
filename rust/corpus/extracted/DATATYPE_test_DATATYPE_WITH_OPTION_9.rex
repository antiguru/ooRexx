/* extracted from DATATYPE::test_DATATYPE_WITH_OPTION_9 */
::routine main public
    a=digits()          -- get digits

    numeric digits 2    -- make sure that numeric digits is not set to 9
    self~assertTrue(DATATYPE('0'    ,'9')) -- whole number under 9 digits (?)
    self~assertTrue(DATATYPE('1'    ,'9'))
    self~assertTrue(DATATYPE('-1'   ,'9'))
    self~assertTrue(DATATYPE('12345' ,'9')) -- ?
    self~assertTrue(DATATYPE('1E3'   ,'9')) -- ?
    if .ooRexxUnit.architecture == 32 then
        self~assertFalse(DATATYPE('1E9'  ,'9')) -- ?
    else
        self~assertFalse(DATATYPE('1E18'  ,'9')) -- ?
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
