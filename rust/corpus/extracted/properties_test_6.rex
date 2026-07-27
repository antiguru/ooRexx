/* extracted from properties::test_6 */
::routine main public
   prop = .properties~new()
   prop['true'] = 1
   prop['false'] = 0
   self~assertEquals(1, prop~getLogical('true'))
   self~assertEquals(0, prop~getLogical('false'))
   prop['true'] = 'true'
   prop['false'] = 'false'
   self~assertEquals(1, prop~getLogical('true'))
   self~assertEquals(0, prop~getLogical('false'))
   self~assertEquals(1, prop~getLogical('xxx', 1))
   self~assertEquals(0, prop~getLogical('yyy', 0))
   prop['true'] = 6
   self~expectSyntax(34.904)
   self~assertEquals(1, prop~getLogical('true'))

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
