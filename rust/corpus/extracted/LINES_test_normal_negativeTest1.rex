/* extracted from LINES::test_normal_negativeTest1 */
::routine main public

      self~assertSame(0, lines('bogusZ2ls'), "Non-existent stream, default 2nd arg should return 0")
      self~assertSame(0, lines('b0sokdogusZ2ls', 'N'), "Non-existent stream, 'N' 2nd arg should return 0")
      self~assertSame(0, lines('bklwerj4ogusZ2ls', "NORMAL"), "Non-existent stream, 'NORMAL' 2nd arg should return 0")

/* disable assertion, as it fails if there's anything in the type-ahead buffer
      self~assertSame(0, lines('', "NORMAL"), "Non-existent stream, 'NORMAL' 2nd arg should return 0")
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
