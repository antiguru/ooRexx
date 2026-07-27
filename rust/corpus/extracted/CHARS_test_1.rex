/* extracted from CHARS::test_1 */
::routine main public

  l = "12345678901234567890"           -- 20 chars
  len = l~length + .endOfLine~length   -- end of line character count

  src = .array~of(l)
  fileName = createFile(src, "delMe.test_chars_test1")
  self~assertTrue(fileName \== "")

  self~assertSame(len, chars(fileName))

  -- See if we get the correct answer twice in a row.
  self~assertSame(len, chars(fileName))

  -- Close the file and delete it
  ret = lineout(fileName)
  self~assertSame(0, ret)
  j = deleteFile(fileName)

  self~assertSame(0, chars(fileName))

-- Tests chars() for an existing, 0-length file.
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
