/* extracted from caselessChangestr::test_ChangeStr_caseless */
::routine main public

   self~assertSame('aABBcC', "aAbBcC"~caselessChangeStr('b',"B"))
   self~assertSame('aAbbcC', "aAbBcC"~caselessChangeStr('B',"b"))
   self~assertSame('aA99cC', "aAbBcC"~caselessChangeStr('B',"9"))

   -- new tests: optional, trailing "count" argument
    self~assertSame('ha-lo', 'hallo'~caselessChangeStr('L','-',1))
    self~assertSame('ha-lo', 'haLlo'~caselessChangeStr('L','-',1))
    self~assertSame('ha-Lo', 'haLLo'~caselessChangeStr('L','-',1))
    self~assertSame('ha-lo', 'haLlo'~caselessChangeStr('l','-',1))
    self~assertSame('ha-Lo', 'haLLo'~caselessChangeStr('l','-',1))

    self~assertSame('ha--o', 'hallo'~caselessChangeStr('l','-',2))
    self~assertSame('ha--o', 'haLlo'~caselessChangeStr('L','-',2))
    self~assertSame('ha--o', 'haLLo'~caselessChangeStr('L','-',2))
    self~assertSame('ha--o', 'haLlo'~caselessChangeStr('l','-',2))
    self~assertSame('ha--o', 'haLLo'~caselessChangeStr('l','-',2))

    self~assertSame('ha--o', 'hallo'~caselessChangestr('l','-',3))
    self~assertSame('ha--o', 'haLlo'~caselessChangeStr('L','-',3))
    self~assertSame('ha--o', 'haLLo'~caselessChangeStr('L','-',3))
    self~assertSame('ha--o', 'haLlo'~caselessChangeStr('l','-',3))
    self~assertSame('ha--o', 'haLLo'~caselessChangeStr('l','-',3))

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
