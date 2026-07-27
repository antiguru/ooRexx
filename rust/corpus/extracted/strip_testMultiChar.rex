/* extracted from strip::testMultiChar */
::routine main public
  self~assertSame("a-+b", strip("+-+-a-+b-+-+",,"-+"))
  self~assertSame("a-+b", strip("+-+-a-+b-+-+",'B',"-+"))
  self~assertSame("a-+b-+-+", strip("+-+-a-+b-+-+",'L',"-+"))
  self~assertSame("+-+-a-+b", strip("+-+-a-+b-+-+",'T',"-+"))
  self~assertSame("abc", strip("abc",,""))
  self~assertSame("", strip("+-+--+-+-+",,"-+"))
  self~assertSame("abc", strip('0001'x||"abc"||'0100'x,,'0001'x))
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
