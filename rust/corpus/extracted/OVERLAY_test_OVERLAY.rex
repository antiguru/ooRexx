/* extracted from OVERLAY::test_OVERLAY */
::routine main public
    self~assertSame('ab def', OVERLAY(' ','abcdef',3))
    self~assertSame('ab. ef', OVERLAY('.','abcdef',3,2))
    self~assertSame('qqcd', OVERLAY('qq','abcd'))
    self~assertSame('abcqq', OVERLAY('qq','abcd',4))
    self~assertSame('abc+123+++', OVERLAY('123','abc',5,6,'+'))

   -- new tests
    self~assertSame('abc', OVERLAY('','abc',3))
    self~assertSame('abc', OVERLAY('','abc',3, 0))
    self~assertSame('ab ', OVERLAY('','abc',3, 1))
    self~assertSame('abc   ', OVERLAY('','abc',6, 1))

    self~assertSame('abc', OVERLAY('  ','abc',3, 0))
    self~assertSame('ab ', OVERLAY('  ','abc',3, 1))
    self~assertSame('abc   ', OVERLAY('  ','abc',6, 1))

    self~assertSame('abc', OVERLAY('12','abc',3, 0))
    self~assertSame('ab3', OVERLAY('34','abc',3, 1))
    self~assertSame('abc  5', OVERLAY('56','abc',6, 1))


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
