/* extracted from XRANGE::test_XRANGE */
::routine main public
    self~assertSame('abcdef', XRANGE('a','f'))
    self~assertSame('0304050607'x, XRANGE('03'x,'07'x))
    self~assertSame('0001020304'x, XRANGE(,'04'x))
    self~assertSame('FEFF000102'x, XRANGE('FE'x,'02'x))
    self~assertSame('ij' /* ASCII */, XRANGE('i','j'))

   -- new tests
    chars=""
    do i=0 to 255
       chars=chars||d2c(i)
    end
    self~assertSame(chars, XRANGE())
    self~assertSame(chars, XRANGE("00"x, "ff"x))


-- tests for RFE 639 xrange() to support more than one range

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
