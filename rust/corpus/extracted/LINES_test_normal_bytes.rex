/* extracted from LINES::test_normal_bytes */
::routine main public

      -- Create a temp file with some lines.
      fileName = createFile(.array~of('dog', 'cat', 'lion', 'tiger'), "delMe_BIF_LinesBytes")
      self~assertFalse(fileName == "", 'temp file must be created')

      -- See how many bytes in size it is
      bytes = stream(fileName, "C", "QUERY SIZE")

      -- If we read in all the bytes, normal should return 0
      discard = charin(fileName, bytes)
      self~assertSame(0, lines(fileName), 'Read all bytes, 0 lines should remain')

      -- Delete the file and create another one for the next test.
      junk = lineout(fileName)
      junk = deleteFile(fileName)

      -- If we read all the bytes but the last new line chars, lines should
      -- return 1.
      fileName = createFile(.array~of('tom', 'frank', 'john', 'harry'), "delMe_BIF_LinesBytes2")
      self~assertFalse(fileName == "", 'temp file must be created')
      bytes = stream(fileName, "C", "QUERY SIZE")

      -- On Windows the last two chars are the new line chars, on unix-like,
      -- just 1 char.  On MAC it has always been only 1 char, although a
      -- different char on pre MAC OS X.
      if .ooRexxUnit.OSName~abbrev("WIN") then readBytes = bytes - 2
      else readBytes = bytes - 1

      discard = charin(fileName, readBytes)
      self~assertSame(1, lines(fileName), 'Read all bytes but last newline, 1 line should remain')

      -- Delete the file
      junk = lineout(fileName)
      junk = deleteFile(fileName)

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
