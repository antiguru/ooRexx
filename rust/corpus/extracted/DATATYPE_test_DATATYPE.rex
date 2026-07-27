/* extracted from DATATYPE::test_DATATYPE */
::routine main public
    self~assertEquals('NUM', DATATYPE(' 12 '))
    self~assertEquals('CHAR', DATATYPE(""))
    self~assertEquals('CHAR', DATATYPE('123*'))

    self~assertTrue(DATATYPE('12.3','N'))
    self~assertFalse(DATATYPE('12.3','W'))
    self~assertTrue(DATATYPE('Fred','M'))
    self~assertFalse(DATATYPE('Fred','U')) -- changed, syntax error in documentation !
    self~assertFalse(DATATYPE('Fred','L'))
    self~assertTrue(DATATYPE('?20K','s'))
    self~assertTrue(DATATYPE('BCd3','X'))
    self~assertTrue(DATATYPE('BC d3','X'))

   -- new tests
    self~assertTrue(DATATYPE('',     'X'))

    self~assertTrue(DATATYPE('BCd3' ,'A')) -- alphanumeric
    self~assertFalse(DATATYPE('BC-d3','A'))

    self~assertTrue(DATATYPE('a1'   ,'s')) -- symbol
    self~assertTrue(DATATYPE('.a1'  ,'s'))
    self~assertTrue(DATATYPE('_'    ,'s'))
    self~assertTrue(DATATYPE('!'    ,'s'))
    self~assertTrue(DATATYPE('?'    ,'s'))
    self~assertTrue(DATATYPE('.'    ,'s'))
    self~assertTrue(DATATYPE('1'    ,'s'))
    self~assertTrue(DATATYPE('1b_!?','s'))
    self~assertFalse(DATATYPE('. .'  ,'s'))

    self~assertTrue(DATATYPE('abc'  ,'v')) -- variable
    self~assertTrue(DATATYPE('?'    ,'v'))
    self~assertTrue(DATATYPE('_'    ,'v'))
    self~assertTrue(DATATYPE('!'    ,'v'))
    self~assertTrue(DATATYPE('a1!_?','v'))
    self~assertFalse(DATATYPE('.'    ,'v'))
    self~assertFalse(DATATYPE('.a'   ,'v'))
    self~assertFalse(DATATYPE('1'    ,'v'))


    a=digits()          -- get digits
    numeric digits 9
    self~assertTrue(DATATYPE('0'    ,'W')) -- whole number
    self~assertTrue(DATATYPE('1'    ,'W'))
    self~assertTrue(DATATYPE('-1'   ,'W'))
    self~assertTrue(DATATYPE('12345','W'))
    self~assertTrue(DATATYPE('1E3'  ,'W'))
    self~assertFalse(DATATYPE('1E9'  ,'W'))

    numeric digits a

    self~assertFalse(DATATYPE('z'  ,'X'))    -- heX-digits
    self~assertTrue(DATATYPE(''  ,'X'))

    self~assertFalse(DATATYPE('z'  ,'B'))   -- binary digits
    self~assertTrue(DATATYPE(''  ,'B'))
    self~assertTrue(DATATYPE('01'  ,'B'))
    self~assertTrue(DATATYPE('01101001'  ,'B'))
    self~assertTrue(DATATYPE('0110 1001'  ,'B'))
    self~assertFalse(DATATYPE('011 01001'  ,'B'))

    numeric digits 9
    self~assertEquals('NUM', DATATYPE(' 1e3       '))
    self~assertEquals('NUM', DATATYPE(' 123456789 '))
    self~assertEquals('NUM', DATATYPE(' 1234567891 '))
    self~assertEquals('CHAR', DATATYPE(""))
    self~assertEquals('CHAR', DATATYPE("a"))
    self~assertEquals('CHAR', DATATYPE("abc"))
    self~assertEquals('CHAR', DATATYPE("1A0"))

    self~assertTrue(DATATYPE("123456789012345", "N" )) -- Numeric
    self~assertTrue(DATATYPE("0"              , "N" )) -- Numeric
    self~assertTrue(DATATYPE("1234567890.1234", "N" )) -- Numeric
    self~assertFalse(DATATYPE(""               , "N" )) -- Numeric
    numeric digits a

    self~assertTrue(DATATYPE(0,  "O"))  -- Logical (Boolean)
    self~assertTrue(DATATYPE(1,  "O"))  -- Logical (Boolean)
    self~assertFalse(DATATYPE(2,  "O"))  -- Logical (Boolean)
    self~assertFalse(DATATYPE(-1, "O"))  -- Logical (Boolean)
    self~assertFalse(DATATYPE("", "O"))  -- Logical (Boolean)


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
