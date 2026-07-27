/* extracted from SPACE::test_SPACE */
::routine main public
    self~assertSame('abc def', SPACE('abc def '))
    self~assertSame('abc   def', SPACE('  abc def',3))
    self~assertSame('abc def', SPACE('abc  def ',1))
    self~assertSame('abcdef', SPACE('abc  def ',0))
    self~assertSame('abc++def', SPACE('abc  def ',2,'+'))

   -- new tests
    self~assertSame('', SPACE('     ',0))
    self~assertSame('', SPACE('     ',1))
    self~assertSame('', SPACE('     ',2))
    self~assertSame('', SPACE('     ',3))
    self~assertSame('', SPACE(''     ,0))
    self~assertSame('', SPACE(''     ,1))
    self~assertSame('', SPACE(''     ,2))
    self~assertSame('', SPACE(''     ,3))


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
