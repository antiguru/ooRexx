/* extracted from insert::test_INSERT */
::routine main public

    self~assertSame('abc def', 'abcdef'~INSERT(' ',3))
    self~assertSame('abc  123   ', 'abc'~INSERT('123',5,6))
    self~assertSame('abc++123+++', 'abc'~INSERT('123',5,6,'+'))
    self~assertSame('123abc', 'abc'~INSERT('123'))
    self~assertSame('123--abc', 'abc'~INSERT('123', ,5,'-'))

    self~assertSame('abcdef', 'abcdef'~INSERT('',3))
    self~assertSame('abcdef', 'abcdef'~INSERT(' ',3, 0))
    self~assertSame('', ''~INSERT(''))
    self~assertSame('   ', ''~INSERT('', 3, 0))
    self~assertSame('    ', ''~INSERT('', 3, 1))


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
