/* extracted from subWord::test_SUBWORD */
::routine main public
    self~assertSame('is the', .mutablebuffer~new('Now is the time')~subword(2,2))
    self~assertSame('the time', .mutablebuffer~new('Now is the time')~subword(3))
    self~assertSame("", .mutablebuffer~new('Now is the time')~subword(5))

   -- new tests
    self~assertSame('', .mutablebuffer~new('Now is the time')~subword(2,0))
    self~assertSame('the  time', .mutablebuffer~new('Now  is  the  time  ')~subword(3))
    self~assertSame("time", .mutablebuffer~new('Now  is  the  time  ')~subword(4))
    self~assertSame("", .mutablebuffer~new('Now  is  the  time  ')~subword(5))



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
