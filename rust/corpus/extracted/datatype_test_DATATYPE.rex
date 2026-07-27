/* extracted from datatype::test_DATATYPE */
::routine main public
    self~assertEquals('NUM', ' 12 '~datatype())
    self~assertEquals('CHAR', ""~datatype())
    self~assertEquals('CHAR', '123*'~datatype())

    self~assertTrue('12.3'~datatype('N'))
    self~assertFalse('12.3'~datatype('W'))
    self~assertTrue('Fred'~datatype('M'))
    self~assertFalse('Fred'~datatype('U')) -- changed, syntax error in documentation !
    self~assertFalse('Fred'~datatype('L'))
    self~assertTrue('?20K'~datatype('s'))
    self~assertTrue('BCd3'~datatype('X'))
    self~assertTrue('BC d3'~datatype('X'))

   -- new tests
    self~assertTrue(''~datatype(     'X'))

    self~assertTrue('BCd3' ~datatype('A')) -- alphanumeric
    self~assertFalse('BC-d3'~datatype('A'))

    self~assertTrue('a1'   ~datatype('s')) -- symbol
    self~assertTrue('.a1'  ~datatype('s'))
    self~assertTrue('_'    ~datatype('s'))
    self~assertTrue('!'    ~datatype('s'))
    self~assertTrue('?'    ~datatype('s'))
    self~assertTrue('.'    ~datatype('s'))
    self~assertTrue('1'    ~datatype('s'))
    self~assertTrue('1b_!?'~datatype('s'))
    self~assertFalse('. .'  ~datatype('s'))

    self~assertTrue('abc'  ~datatype('v')) -- variable
    self~assertTrue('?'    ~datatype('v'))
    self~assertTrue('_'    ~datatype('v'))
    self~assertTrue('!'    ~datatype('v'))
    self~assertTrue('a1!_?'~datatype('v'))
    self~assertFalse('.'    ~datatype('v'))
    self~assertFalse('.a'   ~datatype('v'))
    self~assertFalse('1'    ~datatype('v'))


    a=digits()          -- get digits
    numeric digits 9
    self~assertTrue('0'    ~datatype('W')) -- whole number
    self~assertTrue('1'    ~datatype('W'))
    self~assertTrue('-1'   ~datatype('W'))
    self~assertTrue('12345'~datatype('W'))
    self~assertTrue('1E3'  ~datatype('W'))
    self~assertFalse('1E9'  ~datatype('W'))

    numeric digits a

    self~assertFalse('z'  ~datatype('X'))    -- heX-digits
    self~assertTrue(''  ~datatype('X'))

    self~assertFalse('z'  ~datatype('B'))   -- binary digits
    self~assertTrue(''  ~datatype('B'))
    self~assertTrue('01'  ~datatype('B'))
    self~assertTrue('01101001'  ~datatype('B'))
    self~assertTrue('0110 1001'  ~datatype('B'))
    self~assertFalse('011 01001'  ~datatype('B'))


    numeric digits 9
    self~assertEquals('NUM', ' 1e3       '~datatype())
    self~assertEquals('NUM', ' 123456789 '~datatype())
    self~assertEquals('NUM', ' 1234567891 '~datatype())
    self~assertEquals('CHAR', ""~datatype())
    self~assertEquals('CHAR', "a"~datatype())
    self~assertEquals('CHAR', "abc"~datatype())
    self~assertEquals('CHAR', "1A0"~datatype())

    self~assertTrue("123456789012345"~datatype( "N" )) -- Numeric
    self~assertTrue("0"              ~datatype( "N" )) -- Numeric
    self~assertTrue("1234567890.1234"~datatype( "N" )) -- Numeric
    self~assertFalse(""               ~datatype( "N" )) -- Numeric
    numeric digits a

    self~assertTrue(0~datatype(  "O"))  -- Logical (Boolean)
    self~assertTrue(1~datatype(  "O"))  -- Logical (Boolean)
    self~assertFalse(2~datatype(  "O"))  -- Logical (Boolean)
    self~assertFalse((-1)~datatype( "O"))  -- Logical (Boolean)
    self~assertFalse(""~datatype( "O"))  -- Logical (Boolean)


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
