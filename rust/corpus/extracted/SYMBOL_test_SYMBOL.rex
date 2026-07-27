/* extracted from SYMBOL::test_SYMBOL */
::routine main public
    /* following: Drop A.3; J=3 */
    drop a.3
    j=3

    self~assertEquals('VAR', SYMBOL('J'))
    self~assertEquals('LIT' /* has tested "3" */, SYMBOL(J))
    self~assertEquals('LIT' /* has tested A.3 */, SYMBOL('a.j'))
    self~assertEquals('LIT' /* a constant symbol */, SYMBOL(2))
    self~assertEquals('BAD' /* not a valid symbol */, SYMBOL('*'))

   -- new tests
    self~assertEquals('LIT' /* not a valid symbol */, SYMBOL('.'))
    self~assertEquals('LIT' /* not a valid symbol */, SYMBOL('.a'))
    self~assertEquals('BAD' /* not a valid symbol */, SYMBOL(''))
    self~assertEquals('BAD' /* not a valid symbol */, SYMBOL('  '))


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
