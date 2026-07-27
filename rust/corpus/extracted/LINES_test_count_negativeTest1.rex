/* extracted from LINES::test_count_negativeTest1 */
::routine main public

      self~assertSame(0, lines('bogusZ2ls', 'count'), "Non-existent stream, 'count' 2nd arg should return 0")
      self~assertSame(0, lines('b0sokdogusZ2ls', 'C'), "Non-existent stream, 'C' 2nd arg should return 0")
      self~assertSame(0, lines('bklwerj4ogusZ2ls', "cOUn"), "Non-existent stream, 'cOUn' 2nd arg should return 0")

/* disable assertion, as it fails if there's anything in the type-ahead buffer
      self~assertSame(0, lines('', "CCC"), "Non-existent stream, 'CCC' 2nd arg should return 0")
*/

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
