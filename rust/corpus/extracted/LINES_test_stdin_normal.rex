/* extracted from LINES::test_stdin_normal */
::routine main public

    -- Default stream is stdin, default for omitted second arg should be Normal
    self~assertSame(0, lines(), "Should be no lines available in stdin")

    self~assertSame(0, lines( , "Normal"), 'stdin empty "Normal" is valid')
    self~assertSame(0, lines( , "normal"), 'stdin empty "normal" is valid')
    self~assertSame(0, lines( , "nORmAl"), 'stdin empty "nORmAl" is valid')
    self~assertSame(0, lines( , "NORMAL"), 'stdin empty "NORMAL" is valid')
    self~assertSame(0, lines( , "NORMALLYLONGERTHANNEEDEDSHOULDBEVALID"), 'stdin empty "NORMALLYLONGERTHANNEEDEDSHOULDBEVALID" is valid')
    self~assertSame(0, lines( , "n"), 'stdin empty "n" is valid')
    self~assertSame(0, lines( , "N"), 'stdin empty "N" is valid')
    self~assertSame(0, lines( , "Nor"), 'stdin empty "Nor" is valid')
    self~assertSame(0, lines( , "NIGHTSHADE"), 'stdin empty "NIGHTSHADE" is valid')

    opt = 'n'
    self~assertSame(0, lines( , opt), 'stdin empty use a variable for the option is valid')

    opt = 'N' || 'uncool'~copies(5000)
    self~assertSame(0, lines( , opt), 'stdin empty extremely long string is valid')
*/

/* disable test, as it fails if there's anything in the type-ahead buffer
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
