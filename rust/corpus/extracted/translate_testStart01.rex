/* extracted from translate::testStart01 */
::routine main public
    self~assertSame(('aBC'), 'abc'~translate(,,,2))
    self~assertSame(('aBc'), 'abc'~translate(,,,2,1))
    self~assertSame(('Abc'), 'abc'~translate(,,,,1))
    self~assertSame(('a4.def'), 'abcdef'~translate('123456','aaabbbcc','.', 2, 3))
    self~assertSame('a$$def ', 'abcdef '~translate(,,'$',2,2))
    self~assertSame('a$$$$$$', 'abcdef '~translate(,,'$',2))
    self~assertSame('abcdef ', 'abcdef '~translate(,,'$',2,0))
    self~assertSame('abcdef ', 'abcdef '~translate(,,'$',8,6))

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
