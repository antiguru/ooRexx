/* extracted from LINES::test_stdin_count */
::routine main public
    self~assertSame(0, lines(, "Count"), 'stdin empty "Count" is valid')
    self~assertSame(0, lines(, "count"), 'stdin empty "count" is valid')
    self~assertSame(0, lines(, "COUNT"), 'stdin empty "COUNT" is valid')
    self~assertSame(0, lines(, "cOuNt"), 'stdin empty "cOuNt" is valid')

    self~assertSame(0, lines(, "c"), 'stdin empty "c" is valid')
    self~assertSame(0, lines(, "C"), 'stdin empty "C" is valid')
    self~assertSame(0, lines(, "comic"), 'stdin empty "comic" is valid')

    opt = 'c'
    self~assertSame(0, lines(, opt), 'stdin empty use of a variable for "Count" is valid')

    opt = 'C' || "comedian"~copies(1000)
    self~assertSame(0, lines(, opt), 'stdin empty use of a long string for "Count" is valid')
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
